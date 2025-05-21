import '@frequency-chain/api-augment';
import assert from 'assert';
import { ApiPromise, ApiRx } from '@polkadot/api';
import { ApiTypes, AugmentedEvent, SubmittableExtrinsic, SignerOptions } from '@polkadot/api/types';
import { KeyringPair } from '@polkadot/keyring/types';
import { Compact, u128, u16, u32, u64, Vec, Option, Bool } from '@polkadot/types';
import { FrameSystemAccountInfo, SpRuntimeDispatchError } from '@polkadot/types/lookup';
import { AnyJson, AnyNumber, AnyTuple, Codec, IEvent, ISubmittableResult } from '@polkadot/types/types';
import { firstValueFrom, filter, map, pipe, tap } from 'rxjs';
import { getBlockNumber, getExistentialDeposit, getFinalizedBlockNumber, log, MultiSignatureType } from './helpers';
import autoNonce, { AutoNonce } from './autoNonce';
import { connect, connectPromise } from './apiConnection';
import { DispatchError, Event, Index, SignedBlock } from '@polkadot/types/interfaces';
import { IsEvent } from '@polkadot/types/metadata/decorate/types';
import {
  HandleResponse,
  ItemizedStoragePageResponse,
  MessageSourceId,
  PaginatedStorageResponse,
  PresumptiveSuffixesResponse,
  RpcEvent,
  SchemaResponse,
} from '@frequency-chain/api-augment/interfaces';
import { u8aToHex } from '@polkadot/util/u8a/toHex';
import { u8aWrapBytes } from '@polkadot/util';
import type { AccountId32, Call, H256 } from '@polkadot/types/interfaces/runtime';
import { hasRelayChain } from './env';
import { getUnifiedAddress, getUnifiedPublicKey } from './ethereum';
import { RpcErrorInterface } from '@polkadot/rpc-provider/types';

export interface ReleaseSchedule {
  start: number;
  period: number;
  periodCount: number;
  perPeriod: bigint;
}

export interface AddKeyData {
  msaId?: u64;
  expiration?: any;
  newPublicKey?: any;
}
export interface AuthorizedKeyData {
  msaId: u64;
  expiration?: AnyNumber;
  authorizedPublicKey: KeyringPair['publicKey'];
}
export interface AddProviderPayload {
  authorizedMsaId?: u64;
  schemaIds?: u16[];
  expiration?: any;
}
export interface ItemizedSignaturePayload {
  msaId?: u64;
  schemaId?: u16;
  targetHash?: u32;
  expiration?: any;
  actions?: any;
}
export interface ItemizedSignaturePayloadV2 {
  schemaId?: u16;
  targetHash?: u32;
  expiration?: any;
  actions?: any;
}
export interface PaginatedUpsertSignaturePayload {
  msaId?: u64;
  schemaId?: u16;
  pageId?: u16;
  targetHash?: u32;
  expiration?: any;
  payload?: any;
}
export interface PaginatedUpsertSignaturePayloadV2 {
  schemaId?: u16;
  pageId?: u16;
  targetHash?: u32;
  expiration?: any;
  payload?: any;
}
export interface PaginatedDeleteSignaturePayload {
  msaId?: u64;
  schemaId?: u16;
  pageId?: u16;
  targetHash?: u32;
  expiration?: any;
}
export interface PaginatedDeleteSignaturePayloadV2 {
  schemaId?: u16;
  pageId?: u16;
  targetHash?: u32;
  expiration?: any;
}

export function isRpcError<T = string>(e: any): e is RpcErrorInterface<T> {
  return e?.name === 'RpcError';
}

export class EventError extends Error {
  name: string = '';
  message: string = '';
  stack?: string = '';
  section?: string = '';
  rawError: DispatchError | SpRuntimeDispatchError;

  constructor(source: DispatchError | SpRuntimeDispatchError) {
    super();

    if (source.isModule) {
      const decoded = source.registry.findMetaError(source.asModule);
      this.name = decoded.name;
      this.message = decoded.docs.join(' ');
      this.section = decoded.section;
    } else {
      this.name = source.type;
      this.message = source.type;
      this.section = '';
    }
    this.rawError = source;
  }

  public toString() {
    return `${this.section}.${this.name}: ${this.message}`;
  }
}

class CallError extends Error {
  message: string;
  result: AnyJson;

  constructor(submittable: ISubmittableResult, msg?: string) {
    super();
    this.result = submittable.toHuman();
    this.message = msg ?? 'Call Error';
  }
}

export type EventMap = Record<string, Event>;

function eventKey(event: Event): string {
  return `${event.section}.${event.method}`;
}

/**
 * These helpers return a map of events, some of which contain useful data, some of which don't.
 * Extrinsics that "create" records typically contain an ID of the entity they created, and this
 * would be a useful value to return. However, this data seems to be nested inside an array of arrays.
 *
 * Ex: schemaId = events["schemas.SchemaCreated"][<arbitrary_index>]
 *
 * To get the value associated with an event key, we would need to query inside that nested array with
 * a set of arbitrary indices. Should an object at any level of that querying be undefined, the helper
 * will throw an unchecked exception.
 *
 * To get type checking and cast a returned event as a specific event type, you can utilize TypeScripts
 * type guard functionality like so:
 *
 *      const msaCreatedEvent = events.defaultEvent;
 *      if (ExtrinsicHelper.api.events.msa.MsaCreated.is(msaCreatedEvent)) {
 *          msaId = msaCreatedEvent.data.msaId;
 *      }
 *
 * Normally, I'd say the best experience is for the helper to return both the ID of the created entity
 * along with a map of emitted events. But in this case, returning that value will increase the complexity
 * of each helper, since each would have to check for undefined values at every lookup. So, this may be
 * a rare case when it is best to simply return the map of emitted events and trust the user to look them
 * up in the test.
 */

type ParsedEvent<C extends Codec[] = Codec[], N = unknown> = IEvent<C, N>;
export interface ParsedEventResult<C extends Codec[] = Codec[], N = unknown> {
  target?: ParsedEvent<C, N>;
  eventMap: EventMap;
}

export class Extrinsic<N = unknown, T extends ISubmittableResult = ISubmittableResult, C extends Codec[] = Codec[]> {
  private event?: IsEvent<C, N>;
  public readonly extrinsic: () => SubmittableExtrinsic<'rxjs', T>;
  private keys: KeyringPair;
  public api: ApiRx;

  constructor(extrinsic: () => SubmittableExtrinsic<'rxjs', T>, keys: KeyringPair, targetEvent?: IsEvent<C, N>) {
    this.extrinsic = extrinsic;
    this.keys = keys;
    this.event = targetEvent;
    this.api = ExtrinsicHelper.api;
  }

  // This uses automatic nonce management by default.
  public async signAndSend(inputNonce?: AutoNonce, options: Partial<SignerOptions> = {}, waitForInBlock = true) {
    const nonce = await autoNonce.auto(this.keys, inputNonce);

    try {
      const op = this.extrinsic();
      // Era is 0 for tests due to issues with BirthBlock
      // eslint-disable-next-line no-restricted-syntax
      return await firstValueFrom(
        op.signAndSend(this.keys, { nonce, era: 0, ...options }).pipe(
          tap((result) => {
            // If we learn a transaction has an error status (this does NOT include RPC errors)
            // Then throw an error
            if (result.status.isInvalid) {
              console.error('SEND ALERT: INVALID FOUND', op.method.toHuman(), 'txHash', result.txHash.toHex());
            }
            if (result.isError) {
              throw new CallError(
                result,
                `Failed Transaction for ${this.event?.meta.name || 'unknown'}, status: ${result.status}`
              );
            }
          }),
          filter(({ status }) => (waitForInBlock && status.isInBlock) || status.isFinalized),
          this.parseResult(this.event)
        )
      );
    } catch (e) {
      if (isRpcError(e)) {
        if (inputNonce === 'auto') {
          console.error("WARNING: Unexpected RPC Error! If it is expected, use 'current' for the nonce.");
        }
        log(`RpcError:`, { code: e.code, data: e.data });
      }
      throw e;
    }
  }

  public async sudoSignAndSend(waitForInBlock = true) {
    const nonce = await autoNonce.auto(this.keys);
    // Era is 0 for tests due to issues with BirthBlock
    return await firstValueFrom(
      this.api.tx.sudo
        .sudo(this.extrinsic())
        .signAndSend(this.keys, { nonce, era: 0 })
        .pipe(
          filter(({ status }) => (waitForInBlock && status.isInBlock) || status.isFinalized),
          this.parseResult(this.event)
        )
    );
  }

  public async payWithCapacity(inputNonce?: AutoNonce, waitForInBlock = true) {
    const nonce = await autoNonce.auto(this.keys, inputNonce);
    // Era is 0 for tests due to issues with BirthBlock
    return await firstValueFrom(
      this.api.tx.frequencyTxPayment
        .payWithCapacity(this.extrinsic())
        .signAndSend(this.keys, { nonce, era: 0 })
        .pipe(
          tap((result) => {
            if (result.status.isInvalid) {
              console.error(
                'CAPACITY ALERT: INVALID FOUND',
                this.extrinsic().method.toHuman(),
                'txHash',
                result.txHash
              );
            }
            if (result.isError) {
              throw new CallError(
                result,
                `Failed Transaction for ${this.event?.meta.name || 'unknown'}, status is ${result.status}`
              );
            }
          }),
          // Can comment out filter to help debug hangs
          filter(({ status }) => (waitForInBlock && status.isInBlock) || status.isFinalized),
          this.parseResult(this.event)
        )
    );
  }

  // check transaction cost difference between local+upgrade and testnet
  public getEstimatedTxFee(): Promise<bigint> {
    return firstValueFrom(
      this.extrinsic()
        .paymentInfo(getUnifiedAddress(this.keys))
        .pipe(map((info) => info.partialFee.toBigInt()))
    );
  }

  public getCall(): Call {
    const call = ExtrinsicHelper.api.createType('Call', this.extrinsic.call);
    return call;
  }

  async fundOperation(source: KeyringPair) {
    const [amount, accountInfo] = await Promise.all([
      this.getEstimatedTxFee(),
      ExtrinsicHelper.getAccountInfo(this.keys),
    ]);
    const freeBalance = BigInt(accountInfo.data.free.toString()) - (await getExistentialDeposit());
    if (amount > freeBalance) {
      await assert.doesNotReject(
        ExtrinsicHelper.transferFunds(source, this.keys, amount).signAndSend(undefined, undefined, false)
      );
    }
  }

  public async fundAndSend(source: KeyringPair, waitForInBlock = true) {
    await this.fundOperation(source);
    log('Fund and Send', `${this.extrinsic().method.method} Fund Source: ${getUnifiedAddress(source)}`);
    return this.signAndSend(undefined, undefined, waitForInBlock);
  }

  public async fundAndSendUnsigned(source: KeyringPair, willError = false) {
    await this.fundOperation(source);
    log('Fund and Send', `Fund Source: ${getUnifiedAddress(source)}`);
    return this.sendUnsigned(willError);
  }

  public async sendUnsigned(willError = false) {
    const op = this.extrinsic();
    try {
      return await firstValueFrom(
        op.send().pipe(
          tap((result) => {
            // If we learn a transaction has an error status (this does NOT include RPC errors)
            // Then throw an error
            if (result.status.isInvalid) {
              console.error('UNSIGNED ALERT: INVALID FOUND', op.method.toHuman(), 'txHash', result.txHash);
            }
            if (result.isError) {
              throw new CallError(result, `Failed Transaction for ${this.event?.meta.name || 'unknown'}`);
            }
          }),
          filter(({ status }) => status.isInBlock || status.isFinalized),
          this.parseResult(this.event)
        )
      );
    } catch (e) {
      if ((e as any).name === 'RpcError' && !willError) {
        console.error("WARNING: Unexpected RPC Error! If it is expected, use 'current' for the nonce.");
      }
      throw e;
    }
  }

  private parseResult<ApiType extends ApiTypes = 'rxjs', T extends AnyTuple = AnyTuple, N = unknown>(
    targetEvent?: AugmentedEvent<ApiType, T, N>
  ) {
    return pipe(
      tap((result: ISubmittableResult) => {
        if (result.dispatchError) {
          const err = new EventError(result.dispatchError);
          log(err.toString());
          throw err;
        }
      }),
      map((result: ISubmittableResult) => {
        const eventMap = result.events.reduce((acc, { event }) => {
          acc[eventKey(event)] = event;
          if (this.api.events.sudo.Sudid.is(event)) {
            const {
              data: [result],
            } = event;
            if (result.isErr) {
              const err = new EventError(result.asErr);
              log(err.toString());
              throw err;
            }
          }
          return acc;
        }, {} as EventMap);

        const target = targetEvent && result.events.find(({ event }) => targetEvent.is(event))?.event;
        // Required for Typescript to be happy
        if (target && targetEvent.is(target)) {
          return { target, eventMap };
        }

        return { eventMap };
      }),
      tap(({ eventMap }) => {
        Object.entries(eventMap).map(([k, v]) => log(k, v.toHuman()));
      })
    );
  }
}

export class ExtrinsicHelper {
  public static api: ApiRx;
  public static apiPromise: ApiPromise;

  public static async initialize(providerUrl: string | string[]) {
    ExtrinsicHelper.api = await connect(providerUrl);
    // For single state queries (api.query), ApiPromise is better
    ExtrinsicHelper.apiPromise = await connectPromise(providerUrl);
  }

  public static async getLastFinalizedBlock(): Promise<SignedBlock> {
    const finalized = await ExtrinsicHelper.apiPromise.rpc.chain.getFinalizedHead();
    return ExtrinsicHelper.apiPromise.rpc.chain.getBlock(finalized);
  }

  public static getLastBlock(): Promise<SignedBlock> {
    return ExtrinsicHelper.apiPromise.rpc.chain.getBlock();
  }

  /** Query Extrinsics */
  public static getAccountInfo(keyPair: KeyringPair): Promise<FrameSystemAccountInfo> {
    return ExtrinsicHelper.apiPromise.query.system.account(getUnifiedAddress(keyPair));
  }

  public static getSchemaMaxBytes() {
    return ExtrinsicHelper.apiPromise.query.schemas.governanceSchemaModelMaxBytes();
  }

  /** Balance Extrinsics */
  public static transferFunds(source: KeyringPair, dest: KeyringPair, amount: Compact<u128> | AnyNumber) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedAddress(dest), amount),
      source,
      ExtrinsicHelper.api.events.balances.Transfer
    );
  }

  public static emptyAccount(source: KeyringPair, dest: KeyringPair) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.balances.transferAll(getUnifiedAddress(dest), false),
      source,
      ExtrinsicHelper.api.events.balances.Transfer
    );
  }

  /** Schema Extrinsics */
  public static async getOrCreateSchemaV3(
    keys: KeyringPair,
    model: any,
    modelType: 'AvroBinary' | 'Parquet',
    payloadLocation: 'OnChain' | 'IPFS' | 'Itemized' | 'Paginated',
    grant: ('AppendOnly' | 'SignatureRequired')[],
    schemaNme: string
  ): Promise<u16> {
    // Check to see if the schema name already exists
    const [group, name] = schemaNme.toLowerCase().split('.');
    const { ids } = await ExtrinsicHelper.apiPromise.query.schemas.schemaNameToIds(group, name);
    if (ids.length > 0) {
      return ids[ids.length - 1];
    }
    // Not found? Create it!
    const { target: event } = await ExtrinsicHelper.createSchemaV3(
      keys,
      model,
      modelType,
      payloadLocation,
      grant,
      schemaNme
    ).signAndSend(undefined, undefined, false);
    if (event?.data.schemaId) {
      return event.data.schemaId;
    }
    throw new Error(`Tried to create a schema for ${schemaNme}, but it failed!`);
  }

  /** Schema v3 Extrinsics */
  public static createSchemaV3(
    keys: KeyringPair,
    model: any,
    modelType: 'AvroBinary' | 'Parquet',
    payloadLocation: 'OnChain' | 'IPFS' | 'Itemized' | 'Paginated',
    grant: ('AppendOnly' | 'SignatureRequired')[],
    schemaNme: string | null
  ) {
    return new Extrinsic(
      () =>
        ExtrinsicHelper.api.tx.schemas.createSchemaV3(
          JSON.stringify(model),
          modelType,
          payloadLocation,
          grant,
          schemaNme
        ),
      keys,
      ExtrinsicHelper.api.events.schemas.SchemaCreated
    );
  }

  /** Generic Schema Extrinsics v2 */
  public static createSchemaWithSettingsGovV2(
    keys: KeyringPair,
    model: any,
    modelType: 'AvroBinary' | 'Parquet',
    payloadLocation: 'OnChain' | 'IPFS' | 'Itemized' | 'Paginated',
    grant: 'AppendOnly' | 'SignatureRequired',
    schemaName: string | null
  ) {
    return new Extrinsic(
      () =>
        ExtrinsicHelper.api.tx.schemas.createSchemaViaGovernanceV2(
          getUnifiedPublicKey(keys),
          JSON.stringify(model),
          modelType,
          payloadLocation,
          [grant],
          schemaName
        ),
      keys,
      ExtrinsicHelper.api.events.schemas.SchemaCreated
    );
  }

  /** Get Schema RPC */
  public static getSchema(schemaId: u16): Promise<Option<SchemaResponse>> {
    return ExtrinsicHelper.apiPromise.rpc.schemas.getBySchemaId(schemaId);
  }

  /** MSA Extrinsics */
  public static createMsa(keys: KeyringPair) {
    return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.create(), keys, ExtrinsicHelper.api.events.msa.MsaCreated);
  }

  public static addPublicKeyToMsa(
    keys: KeyringPair,
    ownerSignature: MultiSignatureType,
    newSignature: MultiSignatureType,
    payload: AddKeyData
  ) {
    return new Extrinsic(
      () =>
        ExtrinsicHelper.api.tx.msa.addPublicKeyToMsa(getUnifiedPublicKey(keys), ownerSignature, newSignature, payload),
      keys,
      ExtrinsicHelper.api.events.msa.PublicKeyAdded
    );
  }

  public static deletePublicKey(keys: KeyringPair, publicKey: Uint8Array) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.msa.deleteMsaPublicKey(publicKey),
      keys,
      ExtrinsicHelper.api.events.msa.PublicKeyDeleted
    );
  }

  public static retireMsa(keys: KeyringPair) {
    return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.retireMsa(), keys, ExtrinsicHelper.api.events.msa.MsaRetired);
  }

  public static createProvider(keys: KeyringPair, providerName: string) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.msa.createProvider(providerName),
      keys,
      ExtrinsicHelper.api.events.msa.ProviderCreated
    );
  }

  public static createSponsoredAccountWithDelegation(
    delegatorKeys: KeyringPair,
    providerKeys: KeyringPair,
    signature: MultiSignatureType,
    payload: AddProviderPayload
  ) {
    return new Extrinsic(
      () =>
        ExtrinsicHelper.api.tx.msa.createSponsoredAccountWithDelegation(
          getUnifiedPublicKey(delegatorKeys),
          signature,
          payload
        ),
      providerKeys,
      ExtrinsicHelper.api.events.msa.MsaCreated
    );
  }

  public static grantDelegation(
    delegatorKeys: KeyringPair,
    providerKeys: KeyringPair,
    signature: MultiSignatureType,
    payload: AddProviderPayload
  ) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.msa.grantDelegation(getUnifiedPublicKey(delegatorKeys), signature, payload),
      providerKeys,
      ExtrinsicHelper.api.events.msa.DelegationGranted
    );
  }

  public static grantSchemaPermissions(delegatorKeys: KeyringPair, providerMsaId: any, schemaIds: any) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.msa.grantSchemaPermissions(providerMsaId, schemaIds),
      delegatorKeys,
      ExtrinsicHelper.api.events.msa.DelegationUpdated
    );
  }

  public static revokeDelegationByDelegator(keys: KeyringPair, providerMsaId: any) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.msa.revokeDelegationByDelegator(providerMsaId),
      keys,
      ExtrinsicHelper.api.events.msa.DelegationRevoked
    );
  }

  public static revokeDelegationByProvider(delegatorMsaId: u64, providerKeys: KeyringPair) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.msa.revokeDelegationByProvider(delegatorMsaId),
      providerKeys,
      ExtrinsicHelper.api.events.msa.DelegationRevoked
    );
  }

  /** Messages Extrinsics */
  public static addIPFSMessage(keys: KeyringPair, schemaId: any, cid: string, payload_length: number) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.messages.addIpfsMessage(schemaId, cid, payload_length),
      keys,
      ExtrinsicHelper.api.events.messages.MessagesInBlock
    );
  }

  /** Stateful Storage Extrinsics */
  public static applyItemActions(
    keys: KeyringPair,
    schemaId: any,
    msa_id: MessageSourceId,
    actions: any,
    target_hash: any
  ) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.statefulStorage.applyItemActions(msa_id, schemaId, target_hash, actions),
      keys,
      ExtrinsicHelper.api.events.statefulStorage.ItemizedPageUpdated
    );
  }

  public static removePage(keys: KeyringPair, schemaId: any, msa_id: MessageSourceId, page_id: any, target_hash: any) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.statefulStorage.deletePage(msa_id, schemaId, page_id, target_hash),
      keys,
      ExtrinsicHelper.api.events.statefulStorage.PaginatedPageDeleted
    );
  }

  public static upsertPage(
    keys: KeyringPair,
    schemaId: any,
    msa_id: MessageSourceId,
    page_id: any,
    payload: any,
    target_hash: any
  ) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.statefulStorage.upsertPage(msa_id, schemaId, page_id, target_hash, payload),
      keys,
      ExtrinsicHelper.api.events.statefulStorage.PaginatedPageUpdated
    );
  }

  public static applyItemActionsWithSignatureV2(
    delegatorKeys: KeyringPair,
    providerKeys: KeyringPair,
    signature: MultiSignatureType,
    payload: ItemizedSignaturePayloadV2
  ) {
    return new Extrinsic(
      () =>
        ExtrinsicHelper.api.tx.statefulStorage.applyItemActionsWithSignatureV2(
          getUnifiedPublicKey(delegatorKeys),
          signature,
          payload
        ),
      providerKeys,
      ExtrinsicHelper.api.events.statefulStorage.ItemizedPageUpdated
    );
  }

  public static deletePageWithSignatureV2(
    delegatorKeys: KeyringPair,
    providerKeys: KeyringPair,
    signature: MultiSignatureType,
    payload: PaginatedDeleteSignaturePayloadV2
  ) {
    return new Extrinsic(
      () =>
        ExtrinsicHelper.api.tx.statefulStorage.deletePageWithSignatureV2(
          getUnifiedPublicKey(delegatorKeys),
          signature,
          payload
        ),
      providerKeys,
      ExtrinsicHelper.api.events.statefulStorage.PaginatedPageDeleted
    );
  }

  public static upsertPageWithSignatureV2(
    delegatorKeys: KeyringPair,
    providerKeys: KeyringPair,
    signature: MultiSignatureType,
    payload: PaginatedUpsertSignaturePayloadV2
  ) {
    return new Extrinsic(
      () =>
        ExtrinsicHelper.api.tx.statefulStorage.upsertPageWithSignatureV2(
          getUnifiedPublicKey(delegatorKeys),
          signature,
          payload
        ),
      providerKeys,
      ExtrinsicHelper.api.events.statefulStorage.PaginatedPageUpdated
    );
  }

  public static getItemizedStorage(msa_id: MessageSourceId, schemaId: any): Promise<ItemizedStoragePageResponse> {
    return ExtrinsicHelper.apiPromise.rpc.statefulStorage.getItemizedStorage(msa_id, schemaId);
  }

  public static getPaginatedStorage(msa_id: MessageSourceId, schemaId: any): Promise<Vec<PaginatedStorageResponse>> {
    return ExtrinsicHelper.apiPromise.rpc.statefulStorage.getPaginatedStorage(msa_id, schemaId);
  }

  public static timeReleaseTransfer(keys: KeyringPair, who: KeyringPair, schedule: ReleaseSchedule) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.timeRelease.transfer(getUnifiedAddress(who), schedule),
      keys,
      ExtrinsicHelper.api.events.timeRelease.ReleaseScheduleAdded
    );
  }

  public static timeReleaseScheduleNamedTransfer(
    keys: KeyringPair,
    id: Uint8Array,
    who: KeyringPair,
    schedule: ReleaseSchedule,
    when: number
  ) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.timeRelease.scheduleNamedTransfer(id, getUnifiedAddress(who), schedule, when),
      keys,
      ExtrinsicHelper.api.events.scheduler.Scheduled
    );
  }

  public static timeReleaseCancelScheduledNamedTransfer(keys: KeyringPair, id: Uint8Array) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.timeRelease.cancelScheduledNamedTransfer(id),
      keys,
      ExtrinsicHelper.api.events.scheduler.Canceled
    );
  }

  public static claimHandle(delegatorKeys: KeyringPair, payload: any) {
    const proof = { Sr25519: u8aToHex(delegatorKeys.sign(u8aWrapBytes(payload.toU8a()))) };
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.handles.claimHandle(getUnifiedPublicKey(delegatorKeys), proof, payload),
      delegatorKeys,
      ExtrinsicHelper.api.events.handles.HandleClaimed
    );
  }

  public static retireHandle(delegatorKeys: KeyringPair) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.handles.retireHandle(),
      delegatorKeys,
      ExtrinsicHelper.api.events.handles.HandleRetired
    );
  }

  public static getHandleForMSA(msa_id: MessageSourceId): Promise<Option<HandleResponse>> {
    return ExtrinsicHelper.apiPromise.rpc.handles.getHandleForMsa(msa_id);
  }

  public static getMsaForHandle(handle: string): Promise<Option<MessageSourceId>> {
    return ExtrinsicHelper.apiPromise.rpc.handles.getMsaForHandle(handle);
  }

  public static getNextSuffixesForHandle(base_handle: string, count: number): Promise<PresumptiveSuffixesResponse> {
    return ExtrinsicHelper.apiPromise.rpc.handles.getNextSuffixes(base_handle, count);
  }

  public static validateHandle(base_handle: string): Promise<Bool> {
    return ExtrinsicHelper.apiPromise.rpc.handles.validateHandle(base_handle);
  }

  public static getFrequencyEvents(at: H256 | string): Promise<Vec<RpcEvent>> {
    return ExtrinsicHelper.apiPromise.rpc.frequency.getEvents(at);
  }

  public static getMissingNonceValues(accountId: AccountId32 | string | Uint8Array): Promise<Vec<Index>> {
    return ExtrinsicHelper.apiPromise.rpc.frequency.getMissingNonceValues(accountId);
  }

  public static addOnChainMessage(keys: KeyringPair, schemaId: any, payload: string) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.messages.addOnchainMessage(null, schemaId, payload),
      keys,
      ExtrinsicHelper.api.events.messages.MessagesInBlock
    );
  }

  /** Capacity Extrinsics **/
  public static setEpochLength(keys: KeyringPair, epoch_length: any) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.capacity.setEpochLength(epoch_length),
      keys,
      ExtrinsicHelper.api.events.capacity.EpochLengthUpdated
    );
  }
  public static stake(keys: KeyringPair, target: any, amount: any) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.capacity.stake(target, amount),
      keys,
      ExtrinsicHelper.api.events.capacity.Staked
    );
  }

  public static unstake(keys: KeyringPair, target: any, amount: any) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.capacity.unstake(target, amount),
      keys,
      ExtrinsicHelper.api.events.capacity.UnStaked
    );
  }

  public static withdrawUnstaked(keys: KeyringPair) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.capacity.withdrawUnstaked(),
      keys,
      ExtrinsicHelper.api.events.capacity.StakeWithdrawn
    );
  }

  public static providerBoost(keys: KeyringPair, target: any, amount: any) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.capacity.providerBoost(target, amount),
      keys,
      ExtrinsicHelper.api.events.capacity.ProviderBoosted
    );
  }

  public static changeStakingTarget(keys: KeyringPair, from: any, to: any, amount: any) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.capacity.changeStakingTarget(from, to, amount),
      keys,
      ExtrinsicHelper.api.events.capacity.StakingTargetChanged
    );
  }

  public static payWithCapacityBatchAll(keys: KeyringPair, calls: any) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.frequencyTxPayment.payWithCapacityBatchAll(calls),
      keys,
      ExtrinsicHelper.api.events.utility.BatchCompleted
    );
  }

  public static executeUtilityBatchAll(keys: KeyringPair, calls: any) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.utility.batchAll(calls),
      keys,
      ExtrinsicHelper.api.events.utility.BatchCompleted
    );
  }

  public static executeUtilityBatch(keys: KeyringPair, calls: any) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.utility.batch(calls),
      keys,
      ExtrinsicHelper.api.events.utility.BatchCompleted
    );
  }

  public static executeUtilityForceBatch(keys: KeyringPair, calls: any) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.utility.forceBatch(calls),
      keys,
      ExtrinsicHelper.api.events.utility.BatchCompleted
    );
  }

  public static async waitForFinalization(ofBlockNumber?: number) {
    const start = Date.now();
    const blockNumber = ofBlockNumber || (await getBlockNumber());
    let currentBlock = await getFinalizedBlockNumber();
    while (currentBlock < blockNumber) {
      if (start + 120_000 < Date.now()) {
        throw new Error(
          `Waiting for Finalized Block took longer than 120s. Waiting for "${blockNumber.toString()}", Current: "${currentBlock.toString()}"`
        );
      }
      // In Testnet, just wait
      if (hasRelayChain()) {
        await new Promise((r) => setTimeout(r, 3_000));
      } else {
        await ExtrinsicHelper.apiPromise.rpc.engine.createBlock(true, true);
      }
      currentBlock = await getFinalizedBlockNumber();
    }
  }

  public static async runToBlock(blockNumber: number) {
    const start = Date.now();
    let currentBlock = await getBlockNumber();
    while (currentBlock < blockNumber) {
      if (start + 120_000 < Date.now()) {
        throw new Error(
          `Waiting to run to Block took longer than 120s. Waiting for "${blockNumber.toString()}", Current: "${currentBlock.toString()}"`
        );
      }
      // In Testnet, just wait
      if (hasRelayChain()) {
        await new Promise((r) => setTimeout(r, 3_000));
      } else {
        await ExtrinsicHelper.apiPromise.rpc.engine.createBlock(true, true);
      }
      currentBlock = await getBlockNumber();
    }
  }

  public static submitProposal(keys: KeyringPair, spendAmount: AnyNumber | Compact<u128>) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.treasury.proposeSpend(spendAmount, getUnifiedAddress(keys)),
      keys,
      ExtrinsicHelper.api.events.treasury.Proposed
    );
  }

  public static rejectProposal(keys: KeyringPair, proposalId: any) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.treasury.rejectProposal(proposalId),
      keys,
      ExtrinsicHelper.api.events.treasury.Rejected
    );
  }

  /** Passkey Extrinsics **/
  public static executePassKeyProxy(keys: KeyringPair, payload: any) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.passkey.proxy(payload),
      keys,
      ExtrinsicHelper.api.events.passkey.TransactionExecutionSuccess
    );
  }

  public static executePassKeyProxyV2(keys: KeyringPair, payload: any) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.passkey.proxyV2(payload),
      keys,
      ExtrinsicHelper.api.events.passkey.TransactionExecutionSuccess
    );
  }

  public static withdrawTokens(
    keys: KeyringPair,
    ownerKeys: KeyringPair,
    ownerSignature: MultiSignatureType,
    payload: AddKeyData
  ) {
    return new Extrinsic(
      () => ExtrinsicHelper.api.tx.msa.withdrawTokens(getUnifiedPublicKey(ownerKeys), ownerSignature, payload),
      keys
    );
  }
}
