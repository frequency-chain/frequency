import { Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { u16, u32, u64 } from "@polkadot/types";
import { Codec } from "@polkadot/types/types";
import { u8aToHex, u8aWrapBytes } from "@polkadot/util";
import { mnemonicGenerate } from '@polkadot/util-crypto';
import { env } from "./env";
import { AddKeyData, AddProviderPayload, ExtrinsicHelper } from "./extrinsicHelpers";
import { EXISTENTIAL_DEPOSIT } from "./rootHooks";

export interface DevAccount {
    uri: string,
    keys: KeyringPair,
}

export let devAccounts: DevAccount[] = [];


export type Sr25519Signature = { Sr25519: `0x${string}` }

export function signPayloadSr25519(keys: KeyringPair, data: Codec): Sr25519Signature {
    return { Sr25519: u8aToHex(keys.sign(u8aWrapBytes(data.toU8a()))) }
}

export async function generateDelegationPayload(payloadInputs: AddProviderPayload, expirationOffet?: number): Promise<AddProviderPayload> {
    let { expiration, ...payload } = payloadInputs;
    if (!expiration) {
        expiration = (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber() + (expirationOffet || 5);
    }

    return {
        expiration,
        ...payload,
    }
}

export async function generateAddKeyPayload(payloadInputs: AddKeyData, expirationOffset?: number): Promise<AddKeyData> {
    let { expiration, ...payload } = payloadInputs;
    if (!expiration) {
        expiration = (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber() + (expirationOffset || 5);
    }

    return {
        expiration,
        ...payload,
    }
}

export function createKeys(name: string = 'first pair'): KeyringPair {
    const mnemonic = mnemonicGenerate();
    // create & add the pair to the keyring with the type and some additional
    // metadata specified
    const keyring = new Keyring({ type: 'sr25519' });
    const keypair = keyring.addFromUri(mnemonic, { name }, 'sr25519');

    return keypair;
}

export async function fundKeypair(source: KeyringPair, dest: KeyringPair, amount: bigint): Promise<void> {
    await ExtrinsicHelper.transferFunds(source, dest, amount).signAndSend();
}

export async function createAndFundKeypair(amount = EXISTENTIAL_DEPOSIT, keyName?: string): Promise<KeyringPair> {
    const keypair = createKeys(keyName);

    // Transfer funds from source (usually pre-funded dev account) to new account
    await fundKeypair(devAccounts[0].keys, keypair, amount);

    return keypair;
}

export function log(...args: any[]) {
    if (env.verbose) {
        console.log(...args);
    }
}

export async function createProviderKeysAndId(): Promise<[KeyringPair, u64]> {
    let providerKeys = await createAndFundKeypair();
    let createProviderMsaOp = ExtrinsicHelper.createMsa(providerKeys);
    let providerId = new u64(ExtrinsicHelper.api.registry, 0)
    await createProviderMsaOp.fundAndSend();
    let createProviderOp = ExtrinsicHelper.createProvider(providerKeys, "PrivateProvider");
    let [providerEvent] = await createProviderOp.fundAndSend();
    if (providerEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(providerEvent)) {
        providerId = providerEvent.data.providerId;
    }
    return [providerKeys, providerId];
}

export async function createDelegatorAndDelegation(schemaId: u16, providerId: u64, providerKeys: KeyringPair): Promise<[KeyringPair, u64]> {
    // Create a  delegator msa
    let keys = await createAndFundKeypair();
    let delegator_msa_id = new u64(ExtrinsicHelper.api.registry, 0);
    const createMsa = ExtrinsicHelper.createMsa(keys);
    await createMsa.fundOperation();
    const [msaCreatedEvent, chainEvents] = await createMsa.signAndSend();
    if (msaCreatedEvent && ExtrinsicHelper.api.events.msa.MsaCreated.is(msaCreatedEvent)) {
        delegator_msa_id = msaCreatedEvent.data.msaId;
    }

    // Grant delegation to the provider
    const payload = await generateDelegationPayload({
        authorizedMsaId: providerId,
        schemaIds: [schemaId],
    });
    const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

    const grantDelegationOp = ExtrinsicHelper.grantDelegation(keys, providerKeys, signPayloadSr25519(keys, addProviderData), payload);
    await grantDelegationOp.fundOperation();
    await grantDelegationOp.signAndSend();

    return [keys, delegator_msa_id];
}