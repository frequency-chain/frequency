import { Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { u16, u32, u64 } from "@polkadot/types";
import { Codec } from "@polkadot/types/types";
import { u8aToHex, u8aWrapBytes } from "@polkadot/util";
import { mnemonicGenerate } from '@polkadot/util-crypto';
import { env } from "./env";
import {
  AddKeyData,
  AddProviderPayload,
  ExtrinsicHelper,
  ItemizedSignaturePayload, PaginatedDeleteSignaturePayload,
  PaginatedUpsertSignaturePayload
} from "./extrinsicHelpers";
import { EXISTENTIAL_DEPOSIT } from "./rootHooks";
import {MessageSourceId, PageHash} from "@frequency-chain/api-augment/interfaces";
import assert from "assert";

export interface DevAccount {
    uri: string,
    keys: KeyringPair,
}

export let devAccounts: DevAccount[] = [];


export type Sr25519Signature = { Sr25519: `0x${string}` }

export function signPayloadSr25519(keys: KeyringPair, data: Codec): Sr25519Signature {
    return { Sr25519: u8aToHex(keys.sign(u8aWrapBytes(data.toU8a()))) }
}

export async function generateDelegationPayload(payloadInputs: AddProviderPayload, expirationOffset?: number): Promise<AddProviderPayload> {
    let { expiration, ...payload } = payloadInputs;
    if (!expiration) {
        expiration = (await getBlockNumber()) + (expirationOffset || 5);
    }

    return {
        expiration,
        ...payload,
    }
}

export async function getBlockNumber(): Promise<number> {
    return (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber()
}

export async function generateAddKeyPayload(payloadInputs: AddKeyData, expirationOffset: number = 5, blockNumber?: number): Promise<AddKeyData> {
    let { expiration, ...payload } = payloadInputs;
    if (!expiration) {
        expiration = (blockNumber || (await getBlockNumber())) + expirationOffset;
    }

    return {
        expiration,
        ...payload,
    }
}

export async function generateItemizedSignaturePayload(payloadInputs: ItemizedSignaturePayload, expirationOffset?: number): Promise<ItemizedSignaturePayload> {
  let { expiration, ...payload } = payloadInputs;
  if (!expiration) {
    expiration = (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber() + (expirationOffset || 5);
  }

  return {
    expiration,
    ...payload,
  }
}

export async function generatePaginatedUpsertSignaturePayload(payloadInputs: PaginatedUpsertSignaturePayload, expirationOffset?: number): Promise<PaginatedUpsertSignaturePayload> {
  let { expiration, ...payload } = payloadInputs;
  if (!expiration) {
    expiration = (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber() + (expirationOffset || 5);
  }

  return {
    expiration,
    ...payload,
  }
}

export async function generatePaginatedDeleteSignaturePayload(payloadInputs: PaginatedDeleteSignaturePayload, expirationOffset?: number): Promise<PaginatedDeleteSignaturePayload> {
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

export async function fundKeypair(source: KeyringPair, dest: KeyringPair, amount: bigint, nonce?: number): Promise<void> {
    await ExtrinsicHelper.transferFunds(source, dest, amount).signAndSend(nonce);
}

export async function createAndFundKeypair(amount = EXISTENTIAL_DEPOSIT, keyName?: string, devAccount?: KeyringPair, nonce?: number): Promise<KeyringPair> {
    const keypair = createKeys(keyName);

    // Transfer funds from source (usually pre-funded dev account) to new account
    await fundKeypair((devAccount || devAccounts[0].keys), keypair, amount, nonce);

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

export async function createDelegator(): Promise<[KeyringPair, u64]> {
  let keys = await createAndFundKeypair();
  let delegator_msa_id = new u64(ExtrinsicHelper.api.registry, 0);
  const createMsa = ExtrinsicHelper.createMsa(keys);
  await createMsa.fundOperation();
  const [msaCreatedEvent, _] = await createMsa.signAndSend();

  if (msaCreatedEvent && ExtrinsicHelper.api.events.msa.MsaCreated.is(msaCreatedEvent)) {
    delegator_msa_id = msaCreatedEvent.data.msaId;
  }

  return [keys, delegator_msa_id];
}

export async function createDelegatorAndDelegation(schemaId: u16, providerId: u64, providerKeys: KeyringPair): Promise<[KeyringPair, u64]> {
    // Create a  delegator msa
    const [keys, delegator_msa_id] = await createDelegator();

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

export async function getCurrentItemizedHash(msa_id: MessageSourceId, schemaId: u16): Promise<PageHash> {
  const result = await ExtrinsicHelper.getItemizedStorage(msa_id, schemaId);
  return result.content_hash;
}

export async function getCurrentPaginatedHash(msa_id: MessageSourceId, schemaId: u16, page_id: number): Promise<u32> {
  const result = await ExtrinsicHelper.getPaginatedStorage(msa_id, schemaId);
  const page_response = result.filter((page) => page.page_id.toNumber() === page_id);
  if (page_response.length <= 0) {
    return new u32(ExtrinsicHelper.api.registry, 0);
  }

  return page_response[0].content_hash;
}

// Creates an MSA and a provider for the given keys
// Returns the MSA Id of the provider
export async function createMsaAndProvider(keys: KeyringPair, providerName: string, amount = EXISTENTIAL_DEPOSIT):
    Promise<u64>
{
    // Create and fund a keypair with stakeAmount
    // Use this keypair for stake operations
    await fundKeypair(devAccounts[0].keys, keys, amount);
    const createMsaOp = ExtrinsicHelper.createMsa(keys);
    const [MsaCreatedEvent] = await createMsaOp.fundAndSend();
    assert.notEqual(MsaCreatedEvent, undefined, 'createMsaAndProvider: should have returned MsaCreated event');

    const createProviderOp = ExtrinsicHelper.createProvider(keys, providerName);
    const [ProviderCreatedEvent] = await createProviderOp.fundAndSend();
    assert.notEqual(ProviderCreatedEvent, undefined, 'createMsaAndProvider: should have returned ProviderCreated event');

    if (ProviderCreatedEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(ProviderCreatedEvent)) {
        return ProviderCreatedEvent.data.providerId;
    }
    return Promise.reject('createMsaAndProvider: ProviderCreatedEvent should be ExtrinsicHelper.api.events.msa.ProviderCreated');
}

// Stakes the given amount of tokens from the given keys to the given provider
export async function stakeToProvider(keys: KeyringPair, providerId: u64, amount: bigint): Promise<void> {
    const stakeOp = ExtrinsicHelper.stake(keys, providerId, amount);
    const [stakeEvent] = await stakeOp.fundAndSend();
    assert.notEqual(stakeEvent, undefined, 'stakeToProvider: should have returned Stake event');

    if (stakeEvent && ExtrinsicHelper.api.events.capacity.Staked.is(stakeEvent)) {
        let stakedCapacity = stakeEvent.data.capacity;
        assert.equal(stakedCapacity, amount, 'stakeToProvider: staked capacity should be equal to amount: ' + amount);
    }
    else {
        return Promise.reject('stakeToProvider: stakeEvent should be ExtrinsicHelper.api.events.capacity.Staked');
    }
}
