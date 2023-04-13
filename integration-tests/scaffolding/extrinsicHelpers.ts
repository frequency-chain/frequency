import { ApiPromise, ApiRx } from "@polkadot/api";
import { ApiTypes, AugmentedEvent, SubmittableExtrinsic } from "@polkadot/api/types";
import { KeyringPair } from "@polkadot/keyring/types";
import { Compact, u128, u16, u32, u64, Vec, Option, Bytes } from "@polkadot/types";
import { FrameSystemAccountInfo, SpRuntimeDispatchError } from "@polkadot/types/lookup";
import { AnyNumber, AnyTuple, Codec, IEvent, ISubmittableResult } from "@polkadot/types/types";
import {firstValueFrom, filter, map, pipe, tap} from "rxjs";
import {devAccounts, getBlockNumber, log, Sr25519Signature} from "./helpers";
import { connect, connectPromise } from "./apiConnection";
import { CreatedBlock, DispatchError, Event, SignedBlock } from "@polkadot/types/interfaces";
import { IsEvent } from "@polkadot/types/metadata/decorate/types";
import { HandleResponse, ItemizedStoragePageResponse, MessageSourceId, PaginatedStorageResponse, PresumptiveSuffixesResponse } from "@frequency-chain/api-augment/interfaces";
import { u8aToHex } from "@polkadot/util/u8a/toHex";
import { u8aWrapBytes } from "@polkadot/util";
import type { Call } from '@polkadot/types/interfaces/runtime';

export type ReleaseSchedule = {
    start: number;
    period: number;
    periodCount: number;
    perPeriod: bigint;
};

export type AddKeyData = { msaId?: u64; expiration?: any; newPublicKey?: any; }
export type AddProviderPayload = { authorizedMsaId?: u64; schemaIds?: u16[], expiration?: any; }
export type ItemizedSignaturePayload = { msaId?: u64; schemaId?: u16, targetHash?: u32, expiration?: any; actions?: any; }
export type PaginatedUpsertSignaturePayload = { msaId?: u64; schemaId?: u16, pageId?: u16, targetHash?: u32, expiration?: any; payload?: any; }
export type PaginatedDeleteSignaturePayload = { msaId?: u64; schemaId?: u16, pageId?: u16, targetHash?: u32, expiration?: any; }

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

export type EventMap = { [key: string]: Event }

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
export type ParsedEventResult<C extends Codec[] = Codec[], N = unknown> = [ParsedEvent<C, N> | undefined, EventMap];


export class Extrinsic<T extends ISubmittableResult = ISubmittableResult, C extends Codec[] = Codec[], N = unknown> {

    private event?: IsEvent<C, N>;
    private extrinsic: () => SubmittableExtrinsic<"rxjs", T>;
    // private call: Call;
    private keys: KeyringPair;
    public api: ApiRx;

    constructor(extrinsic: () => SubmittableExtrinsic<"rxjs", T>, keys: KeyringPair, targetEvent?: IsEvent<C, N>) {
        this.extrinsic = extrinsic;
        this.keys = keys;
        this.event = targetEvent;
        this.api = ExtrinsicHelper.api;
    }

    public signAndSend(nonce?: number): Promise<ParsedEventResult> {
        return firstValueFrom(this.extrinsic().signAndSend(this.keys, {nonce: nonce}).pipe(
            filter(({ status }) => status.isInBlock || status.isFinalized),
            this.parseResult(this.event),
        ))
    }

    public sudoSignAndSend(): Promise<[ParsedEvent<C, N> | undefined, EventMap]> {
        return firstValueFrom(this.api.tx.sudo.sudo(this.extrinsic()).signAndSend(this.keys).pipe(
            filter(({ status }) => status.isInBlock || status.isFinalized),
            this.parseResult(this.event),
        ))
    }

    public payWithCapacity(nonce?: number): Promise<ParsedEventResult> {
        return firstValueFrom(this.api.tx.frequencyTxPayment.payWithCapacity(this.extrinsic()).signAndSend(this.keys, {nonce: nonce}).pipe(
            filter(({ status }) => status.isInBlock || status.isFinalized),
            this.parseResult(this.event),
        ))
    }

    public getEstimatedTxFee(): Promise<bigint> {
        return firstValueFrom(this.extrinsic().paymentInfo(this.keys).pipe(
            map((info) => info.partialFee.toBigInt())
        ));
    }

    public getCall(): Call {
        const call = ExtrinsicHelper.api.createType('Call', this.extrinsic.call);
        return call;
    }

    public async fundOperation(source?: KeyringPair, nonce?: number): Promise<void> {
        const amount = await this.getEstimatedTxFee();
        await ExtrinsicHelper.transferFunds(source || devAccounts[0].keys, this.keys, amount).signAndSend(nonce);
    }

    public async fundAndSend(source?: KeyringPair, nonce?: number): Promise<ParsedEventResult> {
        await this.fundOperation(source);
        return this.signAndSend(nonce);
    }

    private parseResult<ApiType extends ApiTypes = "rxjs", T extends AnyTuple = AnyTuple, N = unknown>(targetEvent?: AugmentedEvent<ApiType, T, N>) {
        return pipe(
            tap((result: ISubmittableResult) => {
                if (result.dispatchError) {
                    let err = new EventError(result.dispatchError);
                    log(err.toString());
                    throw err;
                }
            }),
            map((result: ISubmittableResult) => result.events.reduce((acc, { event }) => {
                acc[eventKey(event)] = event;
                if (targetEvent && targetEvent.is(event)) {
                    acc["defaultEvent"] = event;
                }
                if(this.api.events.sudo.Sudid.is(event)) {
                  let { data: [result] } = event;
                  if (result.isErr) {
                    let err = new EventError(result.asErr);
                    log(err.toString());
                    throw err;
                  }
                }
                return acc;
            }, {} as EventMap)),
            map((em) => {
                let result: ParsedEventResult<T, N> = [undefined, {}];
                if (targetEvent && targetEvent.is(em?.defaultEvent)) {
                    result[0] = em.defaultEvent;
                }
                result[1] = em;
                return result;
            }),
            tap((events) => log(events)),
        );
    }

}

export class ExtrinsicHelper {
    public static api: ApiRx;
    public static apiPromise: ApiPromise;

    constructor() { }

    public static async initialize(providerUrl?: string | string[] | undefined) {
        ExtrinsicHelper.api = await connect(providerUrl);
        // For single state queries (api.query), ApiPromise is better
        ExtrinsicHelper.apiPromise = await connectPromise(providerUrl);
    }

    public static getLastBlock(): Promise<SignedBlock> {
        return firstValueFrom(ExtrinsicHelper.api.rpc.chain.getBlock());
    }

    /** engine_createBlock **/
    public static createBlock(): Promise<CreatedBlock> {
        return firstValueFrom(ExtrinsicHelper.api.rpc.engine.createBlock(true, true));
    }

    /** Query Extrinsics */
    public static getAccountInfo(address: string): Promise<FrameSystemAccountInfo> {
        return ExtrinsicHelper.apiPromise.query.system.account(address);
    }

    public static getSchemaMaxBytes() {
        return ExtrinsicHelper.apiPromise.query.schemas.governanceSchemaModelMaxBytes();
    }

    /** Balance Extrinsics */
    public static transferFunds(keys: KeyringPair, dest: KeyringPair, amount: Compact<u128> | AnyNumber): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.balances.transfer(dest.address, amount), keys, ExtrinsicHelper.api.events.balances.Transfer);
    }

    /** Schema Extrinsics */
    public static createSchema(keys: KeyringPair, model: any, modelType: "AvroBinary" | "Parquet", payloadLocation: "OnChain" | "IPFS" | "Itemized" | "Paginated"): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.schemas.createSchema(JSON.stringify(model), modelType, payloadLocation), keys, ExtrinsicHelper.api.events.schemas.SchemaCreated);
    }

    /** Generic Schema Extrinsics */
    public static createSchemaWithSettingsGov(delegatorKeys: KeyringPair, keys: KeyringPair, model: any, modelType: "AvroBinary" | "Parquet", payloadLocation: "OnChain" | "IPFS"| "Itemized" | "Paginated", grant: "AppendOnly"| "SignatureRequired"): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.schemas.createSchemaViaGovernance(delegatorKeys.publicKey, JSON.stringify(model), modelType, payloadLocation, [grant]), keys, ExtrinsicHelper.api.events.schemas.SchemaCreated);
    }

    /** MSA Extrinsics */
    public static createMsa(keys: KeyringPair): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.create(), keys, ExtrinsicHelper.api.events.msa.MsaCreated);
    }

    public static addPublicKeyToMsa(keys: KeyringPair, ownerSignature: Sr25519Signature, newSignature: Sr25519Signature, payload: AddKeyData): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.addPublicKeyToMsa(keys.publicKey, ownerSignature, newSignature, payload), keys, ExtrinsicHelper.api.events.msa.PublicKeyAdded);
    }

    public static deletePublicKey(keys: KeyringPair, publicKey: Uint8Array): Extrinsic {
        ExtrinsicHelper.api.query.msa
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.deleteMsaPublicKey(publicKey), keys, ExtrinsicHelper.api.events.msa.PublicKeyDeleted);
    }

    public static retireMsa(keys: KeyringPair): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.retireMsa(), keys, ExtrinsicHelper.api.events.msa.MsaRetired);
    }

    public static createProvider(keys: KeyringPair, providerName: string): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.createProvider(providerName), keys, ExtrinsicHelper.api.events.msa.ProviderCreated);
    }

    public static createSponsoredAccountWithDelegation(delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: AddProviderPayload): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.createSponsoredAccountWithDelegation(delegatorKeys.publicKey, signature, payload), providerKeys, ExtrinsicHelper.api.events.msa.MsaCreated);
    }

    public static grantDelegation(delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: AddProviderPayload): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.grantDelegation(delegatorKeys.publicKey, signature, payload), providerKeys, ExtrinsicHelper.api.events.msa.DelegationGranted);
    }

    public static grantSchemaPermissions(delegatorKeys: KeyringPair, providerMsaId: any, schemaIds: any): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.grantSchemaPermissions(providerMsaId, schemaIds), delegatorKeys, ExtrinsicHelper.api.events.msa.DelegationUpdated);
    }

    public static revokeSchemaPermissions(delegatorKeys: KeyringPair, providerMsaId: any, schemaIds: any): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.revokeSchemaPermissions(providerMsaId, schemaIds), delegatorKeys, ExtrinsicHelper.api.events.msa.DelegationUpdated);
    }

    public static revokeDelegationByDelegator(keys: KeyringPair, providerMsaId: any): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.revokeDelegationByDelegator(providerMsaId), keys, ExtrinsicHelper.api.events.msa.DelegationRevoked);
    }

    public static revokeDelegationByProvider(delegatorMsaId: u64, providerKeys: KeyringPair): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.revokeDelegationByProvider(delegatorMsaId), providerKeys, ExtrinsicHelper.api.events.msa.DelegationRevoked);
    }

    /** Messages Extrinsics */
    public static addIPFSMessage(keys: KeyringPair, schemaId: any, cid: string, payload_length: number): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.messages.addIpfsMessage(schemaId, cid, payload_length), keys, ExtrinsicHelper.api.events.messages.MessagesStored);
    }

    /** Stateful Storage Extrinsics */
    public static applyItemActions(keys: KeyringPair, schemaId: any, msa_id: MessageSourceId, actions: any, target_hash: any): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.statefulStorage.applyItemActions(msa_id, schemaId, target_hash,actions), keys, ExtrinsicHelper.api.events.statefulStorage.ItemizedPageUpdated);
    }

    public static removePage(keys: KeyringPair, schemaId: any, msa_id: MessageSourceId, page_id: any, target_hash: any): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.statefulStorage.deletePage(msa_id, schemaId, page_id, target_hash), keys, ExtrinsicHelper.api.events.statefulStorage.PaginatedPageDeleted);
    }

    public static upsertPage(keys: KeyringPair, schemaId: any, msa_id: MessageSourceId, page_id: any, payload: any, target_hash: any): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.statefulStorage.upsertPage(msa_id, schemaId, page_id, target_hash, payload), keys, ExtrinsicHelper.api.events.statefulStorage.PaginatedPageUpdated);
    }

    public static applyItemActionsWithSignature(delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: ItemizedSignaturePayload): Extrinsic {
      return new Extrinsic(() => ExtrinsicHelper.api.tx.statefulStorage.applyItemActionsWithSignature(delegatorKeys.publicKey, signature, payload), providerKeys, ExtrinsicHelper.api.events.statefulStorage.ItemizedPageUpdated);
    }

    public static removePageWithSignature(delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: PaginatedDeleteSignaturePayload): Extrinsic {
      return new Extrinsic(() => ExtrinsicHelper.api.tx.statefulStorage.deletePageWithSignature(delegatorKeys.publicKey, signature, payload), providerKeys, ExtrinsicHelper.api.events.statefulStorage.PaginatedPageDeleted);
    }

    public static upsertPageWithSignature(delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: PaginatedUpsertSignaturePayload): Extrinsic {
      return new Extrinsic(() => ExtrinsicHelper.api.tx.statefulStorage.upsertPageWithSignature(delegatorKeys.publicKey, signature, payload), providerKeys, ExtrinsicHelper.api.events.statefulStorage.PaginatedPageUpdated);
    }

    public static getItemizedStorage(msa_id: MessageSourceId, schemaId: any): Promise<ItemizedStoragePageResponse> {
        return firstValueFrom(ExtrinsicHelper.api.rpc.statefulStorage.getItemizedStorage(msa_id, schemaId));
    }

    public static getPaginatedStorage(msa_id: MessageSourceId, schemaId: any): Promise<Vec<PaginatedStorageResponse>> {
        return firstValueFrom(ExtrinsicHelper.api.rpc.statefulStorage.getPaginatedStorage(msa_id, schemaId));
    }

    public static timeReleaseTransfer(keys: KeyringPair, who: KeyringPair, schedule: ReleaseSchedule): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.timeRelease.transfer(who.address, schedule), keys, ExtrinsicHelper.api.events.timeRelease.ReleaseScheduleAdded);
    }

    public static claimHandle(delegatorKeys: KeyringPair, payload: any): Extrinsic {
        const proof = { Sr25519: u8aToHex(delegatorKeys.sign(u8aWrapBytes(payload.toU8a()))) }
        return new Extrinsic(() => ExtrinsicHelper.api.tx.handles.claimHandle(delegatorKeys.publicKey, proof, payload), delegatorKeys, ExtrinsicHelper.api.events.handles.HandleClaimed);
    }

    public static retireHandle(delegatorKeys: KeyringPair): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.handles.retireHandle(), delegatorKeys, ExtrinsicHelper.api.events.handles.HandleRetired);
    }

    public static getHandleForMSA(msa_id: MessageSourceId): Promise<Option<HandleResponse>> {
        let handle_response = ExtrinsicHelper.api.rpc.handles.getHandleForMsa(msa_id);
        return firstValueFrom(handle_response);
    }

    public static getMsaForHandle(handle: string): Promise<Option<MessageSourceId>> {
        let msa_response = ExtrinsicHelper.api.rpc.handles.getMsaForHandle(handle);
        return firstValueFrom(msa_response);
    }

    public static getNextSuffixesForHandle(base_handle: string, count: number): Promise<PresumptiveSuffixesResponse> {
        let suffixes = ExtrinsicHelper.api.rpc.handles.getNextSuffixes(base_handle, count);
        return firstValueFrom(suffixes);
    }

    public static addOnChainMessage(keys: KeyringPair, schemaId: any, payload: string): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.messages.addOnchainMessage(null, schemaId, payload), keys, ExtrinsicHelper.api.events.messages.MessagesStored);
    }

    /** Capacity Extrinsics **/
    public static setEpochLength(keys: KeyringPair, epoch_length: any): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.capacity.setEpochLength(epoch_length), keys, ExtrinsicHelper.api.events.capacity.EpochLengthUpdated);
    }
    public static stake(keys: KeyringPair, target: any, amount: any): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.capacity.stake(target, amount), keys, ExtrinsicHelper.api.events.capacity.Staked);
    }

    public static unstake(keys: KeyringPair, target: any, amount: any): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.capacity.unstake(target, amount), keys, ExtrinsicHelper.api.events.capacity.UnStaked);
    }

    public static withdrawUnstaked(keys: KeyringPair): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.capacity.withdrawUnstaked(), keys, ExtrinsicHelper.api.events.capacity.StakeWithdrawn);
    }

    public static payWithCapacityBatchAll(keys: KeyringPair, calls: Vec<Call>): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.frequencyTxPayment.payWithCapacityBatchAll(calls), keys, ExtrinsicHelper.api.events.utility.BatchCompleted);
    }

    public static async mine() {
      let res: CreatedBlock = await firstValueFrom(ExtrinsicHelper.api.rpc.engine.createBlock(true, true));
      ExtrinsicHelper.api.rpc.engine.finalizeBlock(res.blockHash);
    }

    public static async run_to_block(blockNumber: number) {
      let currentBlock = await getBlockNumber();
      while (currentBlock < blockNumber) {
        await ExtrinsicHelper.mine();
        currentBlock = await getBlockNumber();
      }
    }
}
