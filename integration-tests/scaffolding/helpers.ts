import { Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
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
