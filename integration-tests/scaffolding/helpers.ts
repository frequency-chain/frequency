import { Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { u16, u32, u64, Option, u128} from "@polkadot/types";
import { Codec } from "@polkadot/types/types";
import { u8aToHex, u8aWrapBytes } from "@polkadot/util";
import { mnemonicGenerate } from '@polkadot/util-crypto';
import { env } from "./env";
import {
  AddKeyData,
  AddProviderPayload,
  EventMap,
  ExtrinsicHelper,
  ItemizedSignaturePayload, ItemizedSignaturePayloadV2, PaginatedDeleteSignaturePayload,
  PaginatedDeleteSignaturePayloadV2, PaginatedUpsertSignaturePayload, PaginatedUpsertSignaturePayloadV2
} from "./extrinsicHelpers";
import { EXISTENTIAL_DEPOSIT } from "./rootHooks";
import {HandleResponse, MessageSourceId, PageHash, SchemaGrantResponse} from "@frequency-chain/api-augment/interfaces";
import assert from "assert";
import { firstValueFrom } from "rxjs";
import { AVRO_GRAPH_CHANGE } from "../schemas/fixtures/avroGraphChangeSchemaType";
import { PARQUET_BROADCAST } from "../schemas/fixtures/parquetBroadcastSchemaType";
import { AVRO_CHAT_MESSAGE } from "../stateful-pallet-storage/fixtures/itemizedSchemaType";

export interface Account {
  uri: string,
  keys: KeyringPair,
}

export let devAccounts: Account[] = [];
export let rococoAccounts: Account[] = [];

export type Sr25519Signature = { Sr25519: `0x${string}` }

export const TEST_EPOCH_LENGTH = 10;
export const CENTS = 1000000n;
export const DOLLARS = 100n * CENTS;
export const STARTING_BALANCE = 6n * CENTS + DOLLARS;
export const CHAIN_ENVIRONMENT = {
  DEVELOPMENT: "dev",
  ROCOCO_TESTNET: "rococo-testnet",
  ROCOCO_LOCAL: "rococo-local",
}

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

export function getDefaultFundingSource() {
  return process.env.CHAIN_ENVIRONMENT === CHAIN_ENVIRONMENT.ROCOCO_TESTNET ? rococoAccounts[0] : devAccounts[0];
}

export function hasRelayChain() {
  return process.env.CHAIN_ENVIRONMENT === CHAIN_ENVIRONMENT.ROCOCO_LOCAL
    || process.env.CHAIN_ENVIRONMENT === CHAIN_ENVIRONMENT.ROCOCO_TESTNET
}

export async function fundKeypair(source: KeyringPair, dest: KeyringPair, amount: bigint, nonce?: number): Promise<void> {
  await ExtrinsicHelper.transferFunds(source, dest, amount).signAndSend(nonce);
}

export async function createAndFundKeypair(amount = EXISTENTIAL_DEPOSIT, keyName?: string, source?: KeyringPair, nonce?: number): Promise<KeyringPair> {
  const default_funding_source = getDefaultFundingSource();
  const keypair = createKeys(keyName);

  // Transfer funds from source (usually pre-funded dev account) to new account
  await fundKeypair((source || default_funding_source.keys), keypair, amount, nonce);

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
  const [msaCreatedEvent, _] = await createMsa.fundAndSend();

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
  await grantDelegationOp.fundAndSend();

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
export async function createMsaAndProvider(keys: KeyringPair, providerName: string, amount = EXISTENTIAL_DEPOSIT):
  Promise<u64> {
  // Create and fund a keypair with stakeAmount
  // Use this keypair for stake operations
  const defaultFundingSource = getDefaultFundingSource();
  await fundKeypair(defaultFundingSource.keys, keys, amount);
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
export async function stakeToProvider(keys: KeyringPair, providerId: u64, tokensToStake: bigint): Promise<void> {
  const stakeOp = ExtrinsicHelper.stake(keys, providerId, tokensToStake);
  const [stakeEvent] = await stakeOp.fundAndSend();
  assert.notEqual(stakeEvent, undefined, 'stakeToProvider: should have returned Stake event');

  if (stakeEvent && ExtrinsicHelper.api.events.capacity.Staked.is(stakeEvent)) {
    let stakedCapacity = stakeEvent.data.capacity;

    // let capacityCost: bigint = ExtrinsicHelper.api.consts.capacity.capacityPerToken.toBigInt();
    let expectedCapacity = tokensToStake/TokenPerCapacity;

    assert.equal(stakedCapacity, expectedCapacity, `stakeToProvider: expected ${expectedCapacity}, got ${stakedCapacity}`);
  }
  else {
    return Promise.reject('stakeToProvider: stakeEvent should be ExtrinsicHelper.api.events.capacity.Staked');
  }
}
export async function boostProvider(keys: KeyringPair, providerId: u64, tokensToStake: bigint): Promise<void> {
  const stakeOp = ExtrinsicHelper.providerBoost(keys, providerId, tokensToStake);
  const [stakeEvent] = await stakeOp.fundAndSend();
  assert.notEqual(stakeEvent, undefined, 'stakeToProvider: should have returned Stake event');

  if (stakeEvent && ExtrinsicHelper.api.events.capacity.ProviderBoosted.is(stakeEvent)) {
    let stakedCapacity = stakeEvent.data.capacity;

    let expectedCapacity = tokensToStake/TokenPerCapacity/BoostAdjustment;

    assert.equal(stakedCapacity, expectedCapacity, `stakeToProvider: expected ${expectedCapacity}, got ${stakedCapacity}`);
  }
  else {
    return Promise.reject('stakeToProvider: stakeEvent should be ExtrinsicHelper.api.events.capacity.ProviderBoosted');
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

export async function getOrCreateGraphChangeSchema(): Promise<u16> {
  if (process.env.CHAIN_ENVIRONMENT === CHAIN_ENVIRONMENT.ROCOCO_TESTNET) {
    const ROCOCO_GRAPH_CHANGE_SCHEMA_ID: u16 = new u16(ExtrinsicHelper.api.registry, 53);
    return ROCOCO_GRAPH_CHANGE_SCHEMA_ID;
  } else {
    const [createSchemaEvent, eventMap] = await ExtrinsicHelper
      .createSchema(devAccounts[0].keys, AVRO_GRAPH_CHANGE, "AvroBinary", "OnChain")
      .fundAndSend();
    assertExtrinsicSuccess(eventMap);
    if (createSchemaEvent && ExtrinsicHelper.api.events.schemas.SchemaCreated.is(createSchemaEvent)) {
      return createSchemaEvent.data.schemaId;
    } else {
      assert.fail("failed to create a schema")
    }
  }
}

export async function getOrCreateParquetBroadcastSchema(): Promise<u16> {
  if (process.env.CHAIN_ENVIRONMENT === CHAIN_ENVIRONMENT.ROCOCO_TESTNET) {
    const ROCOCO_PARQUET_BROADCAST_SCHEMA_ID: u16 = new u16(ExtrinsicHelper.api.registry, 51);
    return ROCOCO_PARQUET_BROADCAST_SCHEMA_ID;
  } else {
    const createSchema = ExtrinsicHelper.createSchema(devAccounts[0].keys, PARQUET_BROADCAST, "Parquet", "IPFS");
    let [event] = await createSchema.fundAndSend();
    if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
      return event.data.schemaId;
    } else {
      assert.fail("failed to create a schema")
    }
  }
}

export async function getOrCreateDummySchema(): Promise<u16> {
  if (process.env.CHAIN_ENVIRONMENT === CHAIN_ENVIRONMENT.ROCOCO_TESTNET) {
    const ROCOCO_DUMMY_SCHEMA_ID: u16 = new u16(ExtrinsicHelper.api.registry, 52);
    return ROCOCO_DUMMY_SCHEMA_ID;
  } else {
    const createDummySchema = ExtrinsicHelper.createSchema(
      devAccounts[0].keys,
      { type: "record", name: "Dummy on-chain schema", fields: [] },
      "AvroBinary",
      "OnChain"
    );
    const [dummySchemaEvent] = await createDummySchema.fundAndSend();
    if (dummySchemaEvent && createDummySchema.api.events.schemas.SchemaCreated.is(dummySchemaEvent)) {
      return dummySchemaEvent.data.schemaId;
    } else {
      assert.fail("failed to create a schema")
    }
  }
}

export async function getOrCreateAvroChatMessagePaginatedSchema(): Promise<u16> {
  if (process.env.CHAIN_ENVIRONMENT === CHAIN_ENVIRONMENT.ROCOCO_TESTNET) {
    const ROCOCO_AVRO_CHAT_MESSAGE_PAGINATED: u16 = new u16(ExtrinsicHelper.api.registry, 55);
    return ROCOCO_AVRO_CHAT_MESSAGE_PAGINATED;
  } else {
    let schemaId: u16;
    // Create a schema for Paginated PayloadLocation
    const createSchema = ExtrinsicHelper.createSchema(devAccounts[0].keys, AVRO_CHAT_MESSAGE, "AvroBinary", "Paginated");
    const [event] = await createSchema.fundAndSend();
    if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
      return event.data.schemaId;
    } else {
      assert.fail("failed to create a schema")
    }
  }
}

export async function getOrCreateAvroChatMessageItemizedSchema(): Promise<u16> {
  if (process.env.CHAIN_ENVIRONMENT === CHAIN_ENVIRONMENT.ROCOCO_TESTNET) {
    const ROCOCO_AVRO_CHAT_MESSAGE_ITEMIZED: u16 = new u16(ExtrinsicHelper.api.registry, 54);
    return ROCOCO_AVRO_CHAT_MESSAGE_ITEMIZED;
  } else {
    let schemaId: u16;
    // Create a schema for Paginated PayloadLocation
    const createSchema = ExtrinsicHelper.createSchema(devAccounts[0].keys, AVRO_CHAT_MESSAGE, "AvroBinary", "Itemized");
    const [event] = await createSchema.fundAndSend();
    if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
      return event.data.schemaId;
    } else {
      assert.fail("failed to create a schema")
    }
  }
}

export const TokenPerCapacity = 50n;
export const BoostAdjustment = 20n;  // divide by 20 or 5% of Maximum Capacity

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
