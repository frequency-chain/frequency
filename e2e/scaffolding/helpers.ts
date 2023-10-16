import { Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { u16, u32, u64, Option, u128 } from "@polkadot/types";
import { Codec } from "@polkadot/types/types";
import { u8aToHex, u8aWrapBytes } from "@polkadot/util";
import { mnemonicGenerate } from '@polkadot/util-crypto';
import { env, isTestnet } from "./env";
import {
  AddKeyData,
  AddProviderPayload,
  EventMap,
  ExtrinsicHelper,
  ItemizedSignaturePayload, ItemizedSignaturePayloadV2, PaginatedDeleteSignaturePayload,
  PaginatedDeleteSignaturePayloadV2, PaginatedUpsertSignaturePayload, PaginatedUpsertSignaturePayloadV2
} from "./extrinsicHelpers";
import { HandleResponse, MessageSourceId, PageHash, SchemaGrantResponse } from "@frequency-chain/api-augment/interfaces";
import assert from "assert";
import { firstValueFrom } from "rxjs";
import { AVRO_GRAPH_CHANGE } from "../schemas/fixtures/avroGraphChangeSchemaType";
import { PARQUET_BROADCAST } from "../schemas/fixtures/parquetBroadcastSchemaType";
import { AVRO_CHAT_MESSAGE } from "../stateful-pallet-storage/fixtures/itemizedSchemaType";

export interface Account {
  uri: string,
  keys: KeyringPair,
}

export type Sr25519Signature = { Sr25519: `0x${string}` }

export const TEST_EPOCH_LENGTH = 50;
export const CENTS = 1000000n;
export const DOLLARS = 100n * CENTS;
export const STARTING_BALANCE = 6n * CENTS + DOLLARS;

export function signPayloadSr25519(keys: KeyringPair, data: Codec): Sr25519Signature {
  return { Sr25519: u8aToHex(keys.sign(u8aWrapBytes(data.toU8a()))) }
}

export async function generateDelegationPayload(payloadInputs: AddProviderPayload, expirationOffset: number = 100, blockNumber?: number): Promise<AddProviderPayload> {
  let { expiration, ...payload } = payloadInputs;
  if (!expiration) {
    expiration = (blockNumber || (await getBlockNumber())) + expirationOffset;
  }

  return {
    expiration,
    ...payload,
  }
}

export async function getBlockNumber(): Promise<number> {
  return (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber()
}


let cacheED: null | bigint = null;
export async function getExistentialDeposit(): Promise<bigint> {
  if (cacheED !== null) return cacheED;
  return cacheED = ExtrinsicHelper.api.consts.balances.existentialDeposit.toBigInt();
}

export async function generateAddKeyPayload(payloadInputs: AddKeyData, expirationOffset: number = 100, blockNumber?: number): Promise<AddKeyData> {
  let { expiration, ...payload } = payloadInputs;
  if (!expiration) {
    expiration = (blockNumber || (await getBlockNumber())) + expirationOffset;
  }

  return {
    expiration,
    ...payload,
  }
}

export async function generateItemizedSignaturePayload(payloadInputs: ItemizedSignaturePayload, expirationOffset: number = 100, blockNumber?: number): Promise<ItemizedSignaturePayload> {
  let { expiration, ...payload } = payloadInputs;
  if (!expiration) {
    expiration = (blockNumber || (await getBlockNumber())) + expirationOffset;
  }

  return {
    expiration,
    ...payload,
  }
}

export async function generateItemizedSignaturePayloadV2(payloadInputs: ItemizedSignaturePayloadV2, expirationOffset: number = 100, blockNumber?: number): Promise<ItemizedSignaturePayloadV2> {
  let { expiration, ...payload } = payloadInputs;
  if (!expiration) {
    expiration = (blockNumber || (await getBlockNumber())) + expirationOffset;
  }

  return {
    expiration,
    ...payload,
  }
}

export async function generatePaginatedUpsertSignaturePayload(payloadInputs: PaginatedUpsertSignaturePayload, expirationOffset: number = 100, blockNumber?: number): Promise<PaginatedUpsertSignaturePayload> {
  let { expiration, ...payload } = payloadInputs;
  if (!expiration) {
    expiration = (blockNumber || (await getBlockNumber())) + expirationOffset;
  }

  return {
    expiration,
    ...payload,
  }
}

export async function generatePaginatedUpsertSignaturePayloadV2(payloadInputs: PaginatedUpsertSignaturePayloadV2, expirationOffset: number = 100, blockNumber?: number): Promise<PaginatedUpsertSignaturePayloadV2> {
  let { expiration, ...payload } = payloadInputs;
  if (!expiration) {
    expiration = (blockNumber || (await getBlockNumber())) + expirationOffset;
  }

  return {
    expiration,
    ...payload,
  }
}

export async function generatePaginatedDeleteSignaturePayload(payloadInputs: PaginatedDeleteSignaturePayload, expirationOffset: number = 100, blockNumber?: number): Promise<PaginatedDeleteSignaturePayload> {
  let { expiration, ...payload } = payloadInputs;
  if (!expiration) {
    expiration = (blockNumber || (await getBlockNumber())) + expirationOffset;
  }

  return {
    expiration,
    ...payload,
  }
}

export async function generatePaginatedDeleteSignaturePayloadV2(payloadInputs: PaginatedDeleteSignaturePayloadV2, expirationOffset: number = 100, blockNumber?: number): Promise<PaginatedDeleteSignaturePayloadV2> {
  let { expiration, ...payload } = payloadInputs;
  if (!expiration) {
    expiration = (blockNumber || (await getBlockNumber())) + expirationOffset;
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

export async function createAndFundKeypair(source: KeyringPair, amount: bigint | undefined = undefined, keyName?: string, nonce?: number): Promise<KeyringPair> {
  const keypair = createKeys(keyName);

  await fundKeypair(source, keypair, amount || await getExistentialDeposit(), nonce);

  return keypair;
}

export function log(...args: any[]) {
  if (env.verbose) {
    console.log(...args);
  }
}

export async function createProviderKeysAndId(source: KeyringPair): Promise<[KeyringPair, u64]> {
  let providerKeys = await createAndFundKeypair(source);
  let createProviderMsaOp = ExtrinsicHelper.createMsa(providerKeys);
  let providerId = new u64(ExtrinsicHelper.api.registry, 0)
  await createProviderMsaOp.fundAndSend(source);
  let createProviderOp = ExtrinsicHelper.createProvider(providerKeys, "PrivateProvider");
  let [providerEvent] = await createProviderOp.fundAndSend(source);
  if (providerEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(providerEvent)) {
    providerId = providerEvent.data.providerId;
  }
  return [providerKeys, providerId];
}

export async function createDelegator(source: KeyringPair): Promise<[KeyringPair, u64]> {
  let keys = await createAndFundKeypair(source);
  let delegator_msa_id = new u64(ExtrinsicHelper.api.registry, 0);
  const createMsa = ExtrinsicHelper.createMsa(keys);
  const [msaCreatedEvent, _] = await createMsa.fundAndSend(source);

  if (msaCreatedEvent && ExtrinsicHelper.api.events.msa.MsaCreated.is(msaCreatedEvent)) {
    delegator_msa_id = msaCreatedEvent.data.msaId;
  }

  return [keys, delegator_msa_id];
}

export async function createDelegatorAndDelegation(source: KeyringPair, schemaId: u16, providerId: u64, providerKeys: KeyringPair): Promise<[KeyringPair, u64]> {
  // Create a  delegator msa
  const [keys, delegator_msa_id] = await createDelegator(source);

  // Grant delegation to the provider
  const payload = await generateDelegationPayload({
    authorizedMsaId: providerId,
    schemaIds: [schemaId],
  });
  const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

  const grantDelegationOp = ExtrinsicHelper.grantDelegation(keys, providerKeys, signPayloadSr25519(keys, addProviderData), payload);
  await grantDelegationOp.fundAndSend(source);

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

export async function getHandleForMsa(msa_id: MessageSourceId): Promise<Option<HandleResponse>> {
  const result = await ExtrinsicHelper.getHandleForMSA(msa_id);
  return result;
}

// Creates an MSA and a provider for the given keys
// Returns the MSA Id of the provider
export async function createMsaAndProvider(source: KeyringPair, keys: KeyringPair, providerName: string, amount: bigint | undefined = undefined):
  Promise<u64> {
  // Create and fund a keypair with stakeAmount
  // Use this keypair for stake operations
  await fundKeypair(source, keys, amount || (await getExistentialDeposit()));
  const createMsaOp = ExtrinsicHelper.createMsa(keys);
  const [MsaCreatedEvent] = await createMsaOp.fundAndSend(source);
  assert.notEqual(MsaCreatedEvent, undefined, 'createMsaAndProvider: should have returned MsaCreated event');

  const createProviderOp = ExtrinsicHelper.createProvider(keys, providerName);
  const [ProviderCreatedEvent] = await createProviderOp.fundAndSend(source);
  assert.notEqual(ProviderCreatedEvent, undefined, 'createMsaAndProvider: should have returned ProviderCreated event');

  if (ProviderCreatedEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(ProviderCreatedEvent)) {
    return ProviderCreatedEvent.data.providerId;
  }
  return Promise.reject('createMsaAndProvider: ProviderCreatedEvent should be ExtrinsicHelper.api.events.msa.ProviderCreated');
}

// Stakes the given amount of tokens from the given keys to the given provider
export async function stakeToProvider(source: KeyringPair, keys: KeyringPair, providerId: u64, tokensToStake: bigint): Promise<void> {
  const stakeOp = ExtrinsicHelper.stake(keys, providerId, tokensToStake);
  const [stakeEvent] = await stakeOp.fundAndSend(source);
  assert.notEqual(stakeEvent, undefined, 'stakeToProvider: should have returned Stake event');

  if (stakeEvent && ExtrinsicHelper.api.events.capacity.Staked.is(stakeEvent)) {
    let stakedCapacity = stakeEvent.data.capacity;

    // let capacityCost: bigint = ExtrinsicHelper.api.consts.capacity.capacityPerToken.toBigInt();
    let expectedCapacity = tokensToStake / TokenPerCapacity;

    assert.equal(stakedCapacity, expectedCapacity, `stakeToProvider: expected ${expectedCapacity}, got ${stakedCapacity}`);
  }
  else {
    return Promise.reject('stakeToProvider: stakeEvent should be ExtrinsicHelper.api.events.capacity.Staked');
  }
}

export async function getNextEpochBlock() {
  const epochInfo = await firstValueFrom(ExtrinsicHelper.api.query.capacity.currentEpochInfo());
  const actualEpochLength = await firstValueFrom(ExtrinsicHelper.api.query.capacity.epochLength());
  return actualEpochLength.toNumber() + epochInfo.epochStart.toNumber() + 1;
}

export async function setEpochLength(keys: KeyringPair, epochLength: number): Promise<void> {
  const setEpochLengthOp = ExtrinsicHelper.setEpochLength(keys, epochLength);
  const [setEpochLengthEvent] = await setEpochLengthOp.sudoSignAndSend();
  if (setEpochLengthEvent &&
    ExtrinsicHelper.api.events.capacity.EpochLengthUpdated.is(setEpochLengthEvent)) {
    const epochLength = setEpochLengthEvent.data.blocks;
    assert.equal(epochLength.toNumber(), TEST_EPOCH_LENGTH, "should set epoch length to TEST_EPOCH_LENGTH blocks");
    const actualEpochLength = await firstValueFrom(ExtrinsicHelper.api.query.capacity.epochLength());
    assert.equal(actualEpochLength, TEST_EPOCH_LENGTH, `should have set epoch length to TEST_EPOCH_LENGTH blocks, but it's ${actualEpochLength}`);
  }
  else {
    assert.fail("should return an EpochLengthUpdated event");
  }
}

export async function getOrCreateGraphChangeSchema(source: KeyringPair): Promise<u16> {
  if (isTestnet()) {
    const ROCOCO_GRAPH_CHANGE_SCHEMA_ID: u16 = new u16(ExtrinsicHelper.api.registry, 53);
    return ROCOCO_GRAPH_CHANGE_SCHEMA_ID;
  } else {
    const [createSchemaEvent, eventMap] = await ExtrinsicHelper
      .createSchema(source, AVRO_GRAPH_CHANGE, "AvroBinary", "OnChain")
      .fundAndSend(source);
    assertExtrinsicSuccess(eventMap);
    if (createSchemaEvent && ExtrinsicHelper.api.events.schemas.SchemaCreated.is(createSchemaEvent)) {
      return createSchemaEvent.data.schemaId;
    } else {
      assert.fail("failed to create a schema")
    }
  }
}

export async function getOrCreateParquetBroadcastSchema(source: KeyringPair): Promise<u16> {
  if (isTestnet()) {
    const ROCOCO_PARQUET_BROADCAST_SCHEMA_ID: u16 = new u16(ExtrinsicHelper.api.registry, 51);
    return ROCOCO_PARQUET_BROADCAST_SCHEMA_ID;
  } else {
    const createSchema = ExtrinsicHelper.createSchema(source, PARQUET_BROADCAST, "Parquet", "IPFS");
    let [event] = await createSchema.fundAndSend(source);
    if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
      return event.data.schemaId;
    } else {
      assert.fail("failed to create a schema")
    }
  }
}

export async function getOrCreateDummySchema(source: KeyringPair): Promise<u16> {
  if (isTestnet()) {
    const ROCOCO_DUMMY_SCHEMA_ID: u16 = new u16(ExtrinsicHelper.api.registry, 52);
    return ROCOCO_DUMMY_SCHEMA_ID;
  } else {
    const createDummySchema = ExtrinsicHelper.createSchema(
      source,
      { type: "record", name: "Dummy on-chain schema", fields: [] },
      "AvroBinary",
      "OnChain"
    );
    const [dummySchemaEvent] = await createDummySchema.fundAndSend(source);
    if (dummySchemaEvent && createDummySchema.api.events.schemas.SchemaCreated.is(dummySchemaEvent)) {
      return dummySchemaEvent.data.schemaId;
    } else {
      assert.fail("failed to create a schema")
    }
  }
}

export async function getOrCreateAvroChatMessagePaginatedSchema(source: KeyringPair): Promise<u16> {
  if (isTestnet()) {
    const ROCOCO_AVRO_CHAT_MESSAGE_PAGINATED: u16 = new u16(ExtrinsicHelper.api.registry, 55);
    return ROCOCO_AVRO_CHAT_MESSAGE_PAGINATED;
  } else {
    let schemaId: u16;
    // Create a schema for Paginated PayloadLocation
    const createSchema = ExtrinsicHelper.createSchema(source, AVRO_CHAT_MESSAGE, "AvroBinary", "Paginated");
    const [event] = await createSchema.fundAndSend(source);
    if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
      return event.data.schemaId;
    } else {
      assert.fail("failed to create a schema")
    }
  }
}

export async function getOrCreateAvroChatMessageItemizedSchema(source: KeyringPair): Promise<u16> {
  if (isTestnet()) {
    const ROCOCO_AVRO_CHAT_MESSAGE_ITEMIZED: u16 = new u16(ExtrinsicHelper.api.registry, 54);
    return ROCOCO_AVRO_CHAT_MESSAGE_ITEMIZED;
  } else {
    let schemaId: u16;
    // Create a schema for Paginated PayloadLocation
    const createSchema = ExtrinsicHelper.createSchema(source, AVRO_CHAT_MESSAGE, "AvroBinary", "Itemized");
    const [event] = await createSchema.fundAndSend(source);
    if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
      return event.data.schemaId;
    } else {
      assert.fail("failed to create a schema")
    }
  }
}

export const TokenPerCapacity = 50n;

export function assertEvent(events: EventMap, eventName: string) {
  assert(events.hasOwnProperty(eventName));
}
export async function getRemainingCapacity(providerId: u64): Promise<u128> {
  const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(providerId))).unwrap();
  return capacityStaked.remainingCapacity;
}

export async function getNonce(keys: KeyringPair): Promise<number> {
  const nonce = await firstValueFrom(ExtrinsicHelper.api.call.accountNonceApi.accountNonce(keys.address));
  return nonce.toNumber();
}

export function assertExtrinsicSuccess(eventMap: EventMap) {
  assert.notEqual(eventMap["system.ExtrinsicSuccess"], undefined);
}
