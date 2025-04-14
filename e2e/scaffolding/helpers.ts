import { Keyring } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
import { u16, u32, u64, Option, Bytes } from '@polkadot/types';
import type { FrameSystemAccountInfo, PalletCapacityCapacityDetails } from '@polkadot/types/lookup';
import { Codec } from '@polkadot/types/types';
import { u8aToHex, u8aWrapBytes } from '@polkadot/util';
import { mnemonicGenerate } from '@polkadot/util-crypto';
import {
  verbose,
  getGraphChangeSchema,
  getBroadcastSchema,
  getDummySchema,
  getAvroChatMessageItemizedSchema,
  getAvroChatMessagePaginatedSchema,
} from './env';
import {
  AddKeyData,
  AddProviderPayload,
  EventMap,
  ExtrinsicHelper,
  ItemizedSignaturePayloadV2,
  PaginatedDeleteSignaturePayloadV2,
  PaginatedUpsertSignaturePayloadV2,
} from './extrinsicHelpers';
import {
  BlockPaginationResponseMessage,
  HandleResponse,
  MessageResponse,
  MessageSourceId,
  PageHash,
  SchemaId,
} from '@frequency-chain/api-augment/interfaces';
import assert from 'assert';
import { AVRO_GRAPH_CHANGE } from '../schemas/fixtures/avroGraphChangeSchemaType';
import { PARQUET_BROADCAST } from '../schemas/fixtures/parquetBroadcastSchemaType';
import { AVRO_CHAT_MESSAGE } from '../stateful-pallet-storage/fixtures/itemizedSchemaType';
import { getUnifiedAddress } from './ethereum';
import { KeypairType } from '@polkadot/util-crypto/types';

export interface Account {
  uri: string;
  keys: KeyringPair;
}

export interface Sr25519Signature {
  Sr25519: `0x${string}`;
}

export interface Ed25519Signature {
  Ed25519: `0x${string}`;
}

export interface EcdsaSignature {
  Ecdsa: `0x${string}`;
}

export type MultiSignatureType = Sr25519Signature | Ed25519Signature | EcdsaSignature;

export interface Address20MultiAddress {
  Address20: number[];
}

export const TEST_EPOCH_LENGTH = 50;
export const CENTS = 1000000n;
export const DOLLARS = 100n * CENTS;
export const BOOST_ADJUSTMENT = 2n; // divide by 2 or 50% of Maximum Capacity

export function getTokenPerCapacity(): bigint {
  // Perbil
  return 1_000_000_000n / ExtrinsicHelper.api.consts.capacity.capacityPerToken.toBigInt();
}

export function getTestHandle(prefix = 'test-') {
  return prefix + Math.random().toFixed(10).toString().replaceAll('0.', '');
}

export function signPayloadSr25519(keys: KeyringPair, data: Codec): Sr25519Signature {
  return { Sr25519: u8aToHex(keys.sign(u8aWrapBytes(data.toU8a()))) };
}

export function signPayload(keys: KeyringPair, data: Codec): MultiSignatureType {
  switch (keys.type) {
    case 'ecdsa':
      throw new Error('Ecdsa key type is not supported and it should be replaced with ethereum ones!');
    case 'sr25519':
      return { Sr25519: u8aToHex(keys.sign(u8aWrapBytes(data.toU8a()))) };
    case 'ed25519':
      return { Ed25519: u8aToHex(keys.sign(u8aWrapBytes(data.toU8a()))) };
    case 'ethereum':
      return { Ecdsa: u8aToHex(keys.sign(data.toU8a())) };
  }
}

export async function generateDelegationPayload(
  payloadInputs: AddProviderPayload,
  expirationOffset: number = 100,
  blockNumber?: number
): Promise<AddProviderPayload> {
  const { expiration, ...payload } = payloadInputs;

  return {
    expiration: expiration || (blockNumber || (await getBlockNumber())) + expirationOffset,
    ...payload,
  };
}

export async function getFinalizedBlockNumber(): Promise<number> {
  return (await ExtrinsicHelper.getLastFinalizedBlock()).block.header.number.toNumber();
}

export async function getBlockNumber(): Promise<number> {
  return (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber();
}

let cacheED: null | bigint = null;

export async function getExistentialDeposit(): Promise<bigint> {
  if (cacheED !== null) return cacheED;
  return (cacheED = ExtrinsicHelper.api.consts.balances.existentialDeposit.toBigInt());
}

export async function generateAddKeyPayload(
  payloadInputs: AddKeyData,
  expirationOffset: number = 100,
  blockNumber?: number
): Promise<AddKeyData> {
  const { expiration, ...payload } = payloadInputs;

  return {
    expiration: expiration || (blockNumber || (await getBlockNumber())) + expirationOffset,
    ...payload,
  };
}

export async function generateItemizedSignaturePayload(
  payloadInputs: ItemizedSignaturePayloadV2,
  expirationOffset: number = 100,
  blockNumber?: number
): Promise<ItemizedSignaturePayloadV2> {
  const { expiration, ...payload } = payloadInputs;

  return {
    expiration: expiration || (blockNumber || (await getBlockNumber())) + expirationOffset,
    ...payload,
  };
}

export function generateItemizedActions(items: { action: 'Add' | 'Update'; value: string }[]) {
  return items.map(({ action, value }) => {
    const actionObj = {};
    actionObj[action] = new Bytes(ExtrinsicHelper.api.registry, value);
    return actionObj;
  });
}

export async function generateItemizedActionsPayloadAndSignature(
  payloadInput: ItemizedSignaturePayloadV2,
  payloadType: 'PalletStatefulStorageItemizedSignaturePayloadV2',
  signingKeys: KeyringPair
) {
  const payloadData = await generateItemizedSignaturePayload(payloadInput);
  const payload = ExtrinsicHelper.api.registry.createType(payloadType, payloadData);
  const signature = signPayload(signingKeys, payload);

  return { payload: payloadData, signature };
}

export async function generateItemizedActionsSignedPayloadV2(
  actions: any[],
  schemaId: SchemaId,
  signingKeys: KeyringPair,
  msaId: MessageSourceId
) {
  const payloadInput: ItemizedSignaturePayloadV2 = {
    targetHash: await getCurrentItemizedHash(msaId, schemaId),
    schemaId,
    actions,
  };

  return generateItemizedActionsPayloadAndSignature(
    payloadInput,
    'PalletStatefulStorageItemizedSignaturePayloadV2',
    signingKeys
  );
}

export async function generatePaginatedUpsertSignaturePayloadV2(
  payloadInputs: PaginatedUpsertSignaturePayloadV2,
  expirationOffset: number = 100,
  blockNumber?: number
): Promise<PaginatedUpsertSignaturePayloadV2> {
  const { expiration, ...payload } = payloadInputs;

  return {
    expiration: expiration || (blockNumber || (await getBlockNumber())) + expirationOffset,
    ...payload,
  };
}

export async function generatePaginatedDeleteSignaturePayloadV2(
  payloadInputs: PaginatedDeleteSignaturePayloadV2,
  expirationOffset: number = 100,
  blockNumber?: number
): Promise<PaginatedDeleteSignaturePayloadV2> {
  const { expiration, ...payload } = payloadInputs;

  return {
    expiration: expiration || (blockNumber || (await getBlockNumber())) + expirationOffset,
    ...payload,
  };
}

// Keep track of all the funded keys so that we can drain them at the end of the test
const createdKeys = new Map<string, KeyringPair>();

export function drainFundedKeys(dest: KeyringPair) {
  return drainKeys([...createdKeys.values()], dest);
}

export function createKeys(name: string = 'first pair', keyType: KeypairType = 'sr25519'): KeyringPair {
  const mnemonic = mnemonicGenerate();
  // create & add the pair to the keyring with the type and some additional
  // metadata specified
  const keyring = new Keyring({ type: keyType });
  const keypair = keyring.addFromUri(mnemonic, { name }, keyType);

  createdKeys.set(getUnifiedAddress(keypair), keypair);
  return keypair;
}

function canDrainAccount(info: FrameSystemAccountInfo): boolean {
  return (
    !info.isEmpty &&
    info.data.free.toNumber() > 1_500_000 && // ~Cost to do the transfer
    info.data.reserved.toNumber() < 1 &&
    info.data.frozen.toNumber() < 1
  );
}

export async function drainKeys(keyPairs: KeyringPair[], dest: KeyringPair) {
  try {
    await Promise.all(
      keyPairs.map(async (keypair) => {
        const info = await ExtrinsicHelper.getAccountInfo(keypair);
        // Only drain keys that can be
        if (canDrainAccount(info)) await ExtrinsicHelper.emptyAccount(keypair, dest).signAndSend();
      })
    );
  } catch (e) {
    console.log('Error draining accounts: ', e);
  }
}

export async function fundKeypair(
  source: KeyringPair,
  dest: KeyringPair,
  amount: bigint,
  nonce?: number
): Promise<void> {
  await ExtrinsicHelper.transferFunds(source, dest, amount).signAndSend(nonce);
}

// create and Fund keys with existential deposit amount or the value provided.
export async function createAndFundKeypair(
  source: KeyringPair,
  amount?: bigint,
  keyName?: string,
  nonce?: number,
  keyType: KeypairType = 'sr25519'
): Promise<KeyringPair> {
  const keypair = createKeys(keyName, keyType);

  await fundKeypair(source, keypair, amount || (await getExistentialDeposit()), nonce);
  log('Funded', `Name: ${keyName || 'None provided'}`, `Address: ${getUnifiedAddress(keypair)}`);

  return keypair;
}

export async function createAndFundKeypairs(
  source: KeyringPair,
  keyNames: string[],
  amountOverExDep: bigint = 100_000_000n,
  keyType: KeypairType = 'sr25519'
): Promise<KeyringPair[]> {
  const nonce = await getNonce(source);
  const existentialDeposit = await getExistentialDeposit();

  const wait: Promise<KeyringPair>[] = keyNames.map((keyName, i) => {
    const keypair = createKeys(keyName + ` ${i}th`, keyType);

    return fundKeypair(source, keypair, existentialDeposit + amountOverExDep, nonce + i).then(() => keypair);
  });
  return Promise.all(wait);
}

export function log(...args: any[]) {
  if (verbose) {
    console.log(...args);
  }
}

export async function createProviderKeysAndId(
  source: KeyringPair,
  amount: bigint = 1n * DOLLARS
): Promise<[KeyringPair, u64]> {
  const providerKeys = await createAndFundKeypair(source, amount);
  await ExtrinsicHelper.createMsa(providerKeys).fundAndSend(source);
  const createProviderOp = ExtrinsicHelper.createProvider(providerKeys, 'PrivateProvider');
  const { target: providerEvent } = await createProviderOp.fundAndSend(source);
  const providerId = providerEvent?.data.providerId || new u64(ExtrinsicHelper.api.registry, 0);
  return [providerKeys, providerId];
}

export async function createDelegatorAndDelegation(
  source: KeyringPair,
  schemaId: u16 | u16[],
  providerId: u64,
  providerKeys: KeyringPair,
  keyType: KeypairType = 'sr25519'
): Promise<[KeyringPair, u64]> {
  // Create a  delegator msa + keys.
  const [delegatorMsaId, delegatorKeys] = await createMsa(source, undefined, keyType);
  // Grant delegation to the provider
  const payload = await generateDelegationPayload({
    authorizedMsaId: providerId,
    schemaIds: Array.isArray(schemaId) ? schemaId : [schemaId],
  });
  const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

  const grantDelegationOp = ExtrinsicHelper.grantDelegation(
    delegatorKeys,
    providerKeys,
    signPayload(delegatorKeys, addProviderData),
    payload
  );
  await grantDelegationOp.fundAndSend(source);

  return [delegatorKeys, delegatorMsaId];
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

// 1. Creates and funds a key pair.
// 2. Key pair used to directly create its own MSA Id
// 3. returns MSA ID and the keys.
export async function createMsa(
  source: KeyringPair,
  amount?: bigint,
  keyType: KeypairType = 'sr25519'
): Promise<[u64, KeyringPair]> {
  const keys = await createAndFundKeypair(source, amount, undefined, undefined, keyType);
  const createMsaOp = ExtrinsicHelper.createMsa(keys);
  const { target } = await createMsaOp.fundAndSend(source);
  assert.notEqual(target, undefined, 'createMsa: should have returned MsaCreated event');

  return [target!.data.msaId, keys];
}

// Creates an MSA and a provider for the given keys
// Returns the MSA Id of the provider
export async function createMsaAndProvider(
  source: KeyringPair,
  keys: KeyringPair,
  providerName: string,
  amount: bigint | undefined = undefined
): Promise<u64> {
  const createMsaOp = ExtrinsicHelper.createMsa(keys);
  const createProviderOp = ExtrinsicHelper.createProvider(keys, providerName);
  const minimumFund = (
    await Promise.all([getExistentialDeposit(), createMsaOp.getEstimatedTxFee(), createProviderOp.getEstimatedTxFee()])
  ).reduce((i, j) => i + j, 100_000n);
  // Create and fund a keypair with stakeAmount
  // Use this keypair for stake operations
  await fundKeypair(source, keys, amount || minimumFund);

  const { eventMap } = await ExtrinsicHelper.executeUtilityBatchAll(keys, [
    createMsaOp.extrinsic(),
    createProviderOp.extrinsic(),
  ]).signAndSend();

  const providerCreatedEvent = eventMap['msa.ProviderCreated'];
  if (ExtrinsicHelper.api.events.msa.ProviderCreated.is(providerCreatedEvent)) {
    return providerCreatedEvent.data.providerId;
  }
  return Promise.reject('Did not create provider with msa.ProviderCreated event');
}

// Stakes the given amount of tokens from the given keys to the given provider
export async function stakeToProvider(
  source: KeyringPair,
  keys: KeyringPair,
  providerId: u64,
  tokensToStake: bigint
): Promise<void> {
  const stakeOp = ExtrinsicHelper.stake(keys, providerId, tokensToStake);
  const { target: stakeEvent } = await stakeOp.fundAndSend(source);
  assert.notEqual(stakeEvent, undefined, 'stakeToProvider: should have returned Stake event');

  if (stakeEvent) {
    const stakedCapacity = stakeEvent.data.capacity;

    const expectedCapacity = tokensToStake / getTokenPerCapacity();

    assert.equal(
      stakedCapacity,
      expectedCapacity,
      `stakeToProvider: expected ${expectedCapacity}, got ${stakedCapacity}`
    );
  } else {
    return Promise.reject('stakeToProvider: stakeEvent should be capacity.Staked event');
  }
}

export async function boostProvider(
  source: KeyringPair,
  keys: KeyringPair,
  providerId: u64,
  tokensToStake: bigint
): Promise<void> {
  const stakeOp = ExtrinsicHelper.providerBoost(keys, providerId, tokensToStake);
  const { target: stakeEvent } = await stakeOp.fundAndSend(source);
  assert.notEqual(stakeEvent, undefined, 'stakeToProvider: should have returned Stake event');
  if (stakeEvent) {
    const stakedCapacity = stakeEvent.data.capacity;

    const expectedCapacity = tokensToStake / getTokenPerCapacity() / BOOST_ADJUSTMENT;

    assert.equal(
      stakedCapacity,
      expectedCapacity,
      `stakeToProvider: expected ${expectedCapacity}, got ${stakedCapacity}`
    );
  } else {
    return Promise.reject('stakeToProvider: stakeEvent should be capacity.Staked event');
  }
}

export async function getNextEpochBlock() {
  const epochInfo = await ExtrinsicHelper.apiPromise.query.capacity.currentEpochInfo();
  const actualEpochLength = await ExtrinsicHelper.apiPromise.query.capacity.epochLength();
  return actualEpochLength.toNumber() + epochInfo.epochStart.toNumber() + 1;
}

export async function setEpochLength(keys: KeyringPair, epochLength: number): Promise<void> {
  const setEpochLengthOp = ExtrinsicHelper.setEpochLength(keys, epochLength);
  const { target: setEpochLengthEvent } = await setEpochLengthOp.sudoSignAndSend();
  if (setEpochLengthEvent) {
    const epochLength = setEpochLengthEvent.data.blocks;
    assert.equal(epochLength.toNumber(), TEST_EPOCH_LENGTH, 'should set epoch length to TEST_EPOCH_LENGTH blocks');
    const actualEpochLength = await ExtrinsicHelper.apiPromise.query.capacity.epochLength();
    assert.equal(
      actualEpochLength,
      TEST_EPOCH_LENGTH,
      `should have set epoch length to TEST_EPOCH_LENGTH blocks, but it's ${actualEpochLength}`
    );
  } else {
    assert.fail('should return an EpochLengthUpdated event');
  }
}

export async function getNextRewardEraBlock(): Promise<number> {
  const eraInfo = await ExtrinsicHelper.apiPromise.query.capacity.currentEraInfo();
  const actualEraLength: number = ExtrinsicHelper.api.consts.capacity.eraLength.toNumber();
  return actualEraLength + eraInfo.startedAt.toNumber() + 1;
}

export async function getOrCreateGraphChangeSchema(source: KeyringPair): Promise<u16> {
  const existingSchemaId = getGraphChangeSchema();
  if (existingSchemaId) {
    return new u16(ExtrinsicHelper.api.registry, existingSchemaId);
  } else {
    const op = await ExtrinsicHelper.createSchemaV3(
      source,
      AVRO_GRAPH_CHANGE,
      'AvroBinary',
      'OnChain',
      [],
      'test.graphChangeSchema'
    );
    const { target: createSchemaEvent, eventMap } = await op.fundAndSend(source);
    assertExtrinsicSuccess(eventMap);
    if (createSchemaEvent) {
      return createSchemaEvent.data.schemaId;
    } else {
      assert.fail('failed to create a schema');
    }
  }
}

export async function getOrCreateParquetBroadcastSchema(source: KeyringPair): Promise<u16> {
  const existingSchemaId = getBroadcastSchema();
  if (existingSchemaId) {
    return new u16(ExtrinsicHelper.api.registry, existingSchemaId);
  } else {
    const createSchema = ExtrinsicHelper.createSchemaV3(
      source,
      PARQUET_BROADCAST,
      'Parquet',
      'IPFS',
      [],
      'test.parquetBroadcast'
    );
    const { target: event } = await createSchema.fundAndSend(source);
    if (event) {
      return event.data.schemaId;
    } else {
      assert.fail('failed to create a schema');
    }
  }
}

export async function getOrCreateDummySchema(source: KeyringPair): Promise<u16> {
  const existingSchemaId = getDummySchema();
  if (existingSchemaId) {
    return new u16(ExtrinsicHelper.api.registry, existingSchemaId);
  } else {
    const createDummySchema = ExtrinsicHelper.createSchemaV3(
      source,
      { type: 'record', name: 'Dummy on-chain schema', fields: [] },
      'AvroBinary',
      'OnChain',
      [],
      'test.dummySchema'
    );
    const { target: dummySchemaEvent } = await createDummySchema.fundAndSend(source);
    if (dummySchemaEvent) {
      return dummySchemaEvent.data.schemaId;
    } else {
      assert.fail('failed to create a schema');
    }
  }
}

export async function getOrCreateAvroChatMessagePaginatedSchema(source: KeyringPair): Promise<u16> {
  const existingSchemaId = getAvroChatMessagePaginatedSchema();
  if (existingSchemaId) {
    return new u16(ExtrinsicHelper.api.registry, existingSchemaId);
  } else {
    // Create a schema for Paginated PayloadLocation
    const createSchema = ExtrinsicHelper.createSchemaV3(
      source,
      AVRO_CHAT_MESSAGE,
      'AvroBinary',
      'Paginated',
      [],
      'test.AvroChatMessagePaginated'
    );
    const { target: event } = await createSchema.fundAndSend(source);
    if (event) {
      return event.data.schemaId;
    } else {
      assert.fail('failed to create a schema');
    }
  }
}

export async function getOrCreateAvroChatMessageItemizedSchema(source: KeyringPair): Promise<u16> {
  const existingSchemaId = getAvroChatMessageItemizedSchema();
  if (existingSchemaId) {
    return new u16(ExtrinsicHelper.api.registry, existingSchemaId);
  } else {
    // Create a schema for Paginated PayloadLocation
    const createSchema = ExtrinsicHelper.createSchemaV3(
      source,
      AVRO_CHAT_MESSAGE,
      'AvroBinary',
      'Itemized',
      [],
      'test.AvroChatMessageItemized'
    );
    const { target: event } = await createSchema.fundAndSend(source);
    if (event) {
      return event.data.schemaId;
    } else {
      assert.fail('failed to create a schema');
    }
  }
}

export async function getCapacity(providerId: u64): Promise<PalletCapacityCapacityDetails> {
  return (await ExtrinsicHelper.apiPromise.query.capacity.capacityLedger(providerId)).unwrap();
}

export async function getNonce(keys: KeyringPair): Promise<number> {
  const nonce = await ExtrinsicHelper.apiPromise.call.accountNonceApi.accountNonce(getUnifiedAddress(keys));
  return nonce.toNumber();
}

export function assertEvent(events: EventMap, eventName: string) {
  assert(Object.hasOwn(events, eventName), `Could not find expected event: ${eventName}`);
}

export function assertExtrinsicSuccess(eventMap: EventMap) {
  assert.notEqual(eventMap['system.ExtrinsicSuccess'], undefined);
}

export function assertHasMessage(response: BlockPaginationResponseMessage, testFn: (x: MessageResponse) => boolean) {
  const messages = response.content;
  assert(messages.length > 0, 'Expected some messages, but found none.');

  const found = messages.find(testFn);

  if (found) {
    assert.notEqual(found, undefined);
  } else {
    const allPayloads = messages.map((x) => x.payload.toString());
    assert.fail(`Unable to find message in response (length: ${messages.length}, Payloads: ${allPayloads.join(', ')})`);
  }
}

export async function assertAddNewKey(
  capacityKeys: KeyringPair,
  addKeyPayload: AddKeyData,
  newControlKeypair: KeyringPair
) {
  const addKeyPayloadCodec: Codec = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', addKeyPayload);
  const ownerSig: MultiSignatureType = signPayload(capacityKeys, addKeyPayloadCodec);
  const newSig: MultiSignatureType = signPayload(newControlKeypair, addKeyPayloadCodec);
  const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(capacityKeys, ownerSig, newSig, addKeyPayload);
  const { eventMap } = await addPublicKeyOp.signAndSend();
  assertEvent(eventMap, 'system.ExtrinsicSuccess');
  assertEvent(eventMap, 'msa.PublicKeyAdded');
}

export function generateSchemaPartialName(length: number): string {
  let result = '';
  const characters = 'abcdefghijklmnopqrstuvwxyz-';
  const charactersLength = characters.length;
  let counter = 0;
  while (counter < length) {
    const randomChar = characters.charAt(Math.floor(Math.random() * charactersLength));
    if ((counter == 0 || counter == length - 1) && randomChar === '-') {
      // avoid creating invalid name
      continue;
    }
    result += randomChar;
    counter += 1;
  }
  return result;
}

export const base64UrlToUint8Array = (base64: string): Uint8Array => new Uint8Array(Buffer.from(base64, 'base64url'));

export async function getFreeBalance(source: KeyringPair): Promise<bigint> {
  const accountInfo = await ExtrinsicHelper.getAccountInfo(source);
  return BigInt(accountInfo.data.free.toString()) - (await getExistentialDeposit());
}

export async function assertExtrinsicSucceededAndFeesPaid(chainEvents: any) {
  assert.notEqual(chainEvents['system.ExtrinsicSuccess'], undefined, 'should have returned an ExtrinsicSuccess event');
  assert.notEqual(chainEvents['balances.Withdraw'], undefined, 'should have returned a balances.Withdraw event');
}
