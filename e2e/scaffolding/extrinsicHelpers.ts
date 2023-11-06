import { ApiPromise, ApiRx } from "@polkadot/api";
import { ApiTypes, AugmentedEvent, SubmittableExtrinsic } from "@polkadot/api/types";
import { KeyringPair } from "@polkadot/keyring/types";
import { Compact, u128, u16, u32, u64, Vec, Option, Bool } from "@polkadot/types";
import { FrameSystemAccountInfo, SpRuntimeDispatchError } from "@polkadot/types/lookup";
import { AnyJson, AnyNumber, AnyTuple, Codec, IEvent, ISubmittableResult } from "@polkadot/types/types";
import { firstValueFrom, filter, map, pipe, tap } from "rxjs";
import { getBlockNumber, getExistentialDeposit, log, Sr25519Signature } from "./helpers";
import autoNonce, { AutoNonce } from "./autoNonce";
import { connect, connectPromise } from "./apiConnection";
import { DispatchError, Event, SignedBlock } from "@polkadot/types/interfaces";
import { IsEvent } from "@polkadot/types/metadata/decorate/types";
import { HandleResponse, ItemizedStoragePageResponse, MessageSourceId, PaginatedStorageResponse, PresumptiveSuffixesResponse, SchemaResponse } from "@frequency-chain/api-augment/interfaces";
import { u8aToHex } from "@polkadot/util/u8a/toHex";
import { u8aWrapBytes } from "@polkadot/util";
import type { Call } from '@polkadot/types/interfaces/runtime';
import { hasRelayChain } from "./env";

export type ReleaseSchedule = {
    start: number;
    period: number;
    periodCount: number;
    perPeriod: bigint;
};

export type AddKeyData = { msaId?: u64; expiration?: any; newPublicKey?: any; }
export type AddProviderPayload = { authorizedMsaId?: u64; schemaIds?: u16[], expiration?: any; }
export type ItemizedSignaturePayload = { msaId?: u64; schemaId?: u16, targetHash?: u32, expiration?: any; actions?: any; }
export type ItemizedSignaturePayloadV2 = { schemaId?: u16, targetHash?: u32, expiration?: any; actions?: any; }
export type PaginatedUpsertSignaturePayload = { msaId?: u64; schemaId?: u16, pageId?: u16, targetHash?: u32, expiration?: any; payload?: any; }
export type PaginatedUpsertSignaturePayloadV2 = { schemaId?: u16, pageId?: u16, targetHash?: u32, expiration?: any; payload?: any; }
export type PaginatedDeleteSignaturePayload = { msaId?: u64; schemaId?: u16, pageId?: u16, targetHash?: u32, expiration?: any; }
export type PaginatedDeleteSignaturePayloadV2 = { schemaId?: u16, pageId?: u16, targetHash?: u32, expiration?: any; }

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
        this.message = msg ?? "Call Error";
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
export type ParsedEventResult<C extends Codec[] = Codec[], N = unknown> = {
    target?: ParsedEvent<C, N>;
    eventMap: EventMap;
};


export class Extrinsic<N = unknown, T extends ISubmittableResult = ISubmittableResult, C extends Codec[] = Codec[]> {

    private event?: IsEvent<C, N>;
    private extrinsic: () => SubmittableExtrinsic<"rxjs", T>;
    private keys: KeyringPair;
    public api: ApiRx;

    constructor(extrinsic: () => SubmittableExtrinsic<"rxjs", T>, keys: KeyringPair, targetEvent?: IsEvent<C, N>) {
        this.extrinsic = extrinsic;
        this.keys = keys;
        this.event = targetEvent;
        this.api = ExtrinsicHelper.api;
    }

    // This uses automatic nonce management by default.
    public async signAndSend(inputNonce?: AutoNonce) {
        const nonce = await autoNonce.auto(this.keys, inputNonce);

        try {
            const op = this.extrinsic();
            return await firstValueFrom(op.signAndSend(this.keys, { nonce }).pipe(
                tap((result) => {
                    // If we learn a transaction has an error status (this does NOT include RPC errors)
                    // Then throw an error
                    if (result.isError) {
                        throw new CallError(result, `Failed Transaction for ${this.event?.meta.name || "unknown"}`);
                    }
                }),
                filter(({ status }) => status.isInBlock || status.isFinalized),
                this.parseResult(this.event),
            ));
        } catch (e) {
            if ((e as any).name === "RpcError" && inputNonce === 'auto') {
                console.error("WARNING: Unexpected RPC Error! If it is expected, use 'current' for the nonce.");
            }
            throw e;
        }
    }

    public async sudoSignAndSend() {
        const nonce = await autoNonce.auto(this.keys);
        return await firstValueFrom(this.api.tx.sudo.sudo(this.extrinsic()).signAndSend(this.keys, { nonce }).pipe(
            filter(({ status }) => status.isInBlock || status.isFinalized),
            this.parseResult(this.event),
        ))
    }

    public async payWithCapacity(inputNonce?: AutoNonce) {
        const nonce = await autoNonce.auto(this.keys, inputNonce);
        return await firstValueFrom(this.api.tx.frequencyTxPayment.payWithCapacity(this.extrinsic()).signAndSend(this.keys, { nonce }).pipe(
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

    async fundOperation(source: KeyringPair) {
        const [amount, accountInfo] = await Promise.all([this.getEstimatedTxFee(), ExtrinsicHelper.getAccountInfo(this.keys.address)]);
        const freeBalance = BigInt(accountInfo.data.free.toString()) - (await getExistentialDeposit());
        if (amount > freeBalance) {
            await ExtrinsicHelper.transferFunds(source, this.keys, amount).signAndSend();
        }
    }

    public async fundAndSend(source: KeyringPair) {
        await this.fundOperation(source);
        log("Fund and Send", `Fund Source: ${source.address}`);
        return this.signAndSend();
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
            map((result: ISubmittableResult) => {
                const eventMap = result.events.reduce((acc, { event }) => {
                    acc[eventKey(event)] = event;
                    if (this.api.events.sudo.Sudid.is(event)) {
                        let { data: [result] } = event;
                        if (result.isErr) {
                            let err = new EventError(result.asErr);
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
            }),
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
        return ExtrinsicHelper.apiPromise.rpc.chain.getBlock();
    }

    /** Query Extrinsics */
    public static getAccountInfo(address: string): Promise<FrameSystemAccountInfo> {
        return ExtrinsicHelper.apiPromise.query.system.account(address);
    }

    public static getSchemaMaxBytes() {
        return ExtrinsicHelper.apiPromise.query.schemas.governanceSchemaModelMaxBytes();
    }

    /** Balance Extrinsics */
    public static transferFunds(source: KeyringPair, dest: KeyringPair, amount: Compact<u128> | AnyNumber) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.balances.transferKeepAlive(dest.address, amount), source, ExtrinsicHelper.api.events.balances.Transfer);
    }

    public static emptyAccount(source: KeyringPair, dest: KeyringPair["address"]) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.balances.transferAll(dest, false), source, ExtrinsicHelper.api.events.balances.Transfer);
    }

    /** Schema Extrinsics */
    public static createSchema(keys: KeyringPair, model: any, modelType: "AvroBinary" | "Parquet", payloadLocation: "OnChain" | "IPFS" | "Itemized" | "Paginated") {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.schemas.createSchema(JSON.stringify(model), modelType, payloadLocation), keys, ExtrinsicHelper.api.events.schemas.SchemaCreated);
    }

    /** Schema v2 Extrinsics */
    public static createSchemaV2(keys: KeyringPair, model: any, modelType: "AvroBinary" | "Parquet", payloadLocation: "OnChain" | "IPFS" | "Itemized" | "Paginated", grant: ("AppendOnly" | "SignatureRequired")[]) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.schemas.createSchemaV2(JSON.stringify(model), modelType, payloadLocation, grant), keys, ExtrinsicHelper.api.events.schemas.SchemaCreated);
    }

    /** Generic Schema Extrinsics */
    public static createSchemaWithSettingsGov(keys: KeyringPair, model: any, modelType: "AvroBinary" | "Parquet", payloadLocation: "OnChain" | "IPFS" | "Itemized" | "Paginated", grant: "AppendOnly" | "SignatureRequired") {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.schemas.createSchemaViaGovernance(keys.publicKey, JSON.stringify(model), modelType, payloadLocation, [grant]), keys, ExtrinsicHelper.api.events.schemas.SchemaCreated);
    }

    /** Get Schema RPC */
    public static getSchema(schemaId: u16): Promise<Option<SchemaResponse>> {
        return ExtrinsicHelper.apiPromise.rpc.schemas.getBySchemaId(schemaId);
    }

    /** MSA Extrinsics */
    public static createMsa(keys: KeyringPair) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.create(), keys, ExtrinsicHelper.api.events.msa.MsaCreated);
    }

    public static addPublicKeyToMsa(keys: KeyringPair, ownerSignature: Sr25519Signature, newSignature: Sr25519Signature, payload: AddKeyData) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.addPublicKeyToMsa(keys.publicKey, ownerSignature, newSignature, payload), keys, ExtrinsicHelper.api.events.msa.PublicKeyAdded);
    }

    public static deletePublicKey(keys: KeyringPair, publicKey: Uint8Array) {
        ExtrinsicHelper.api.query.msa
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.deleteMsaPublicKey(publicKey), keys, ExtrinsicHelper.api.events.msa.PublicKeyDeleted);
    }

    public static retireMsa(keys: KeyringPair) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.retireMsa(), keys, ExtrinsicHelper.api.events.msa.MsaRetired);
    }

    public static createProvider(keys: KeyringPair, providerName: string) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.createProvider(providerName), keys, ExtrinsicHelper.api.events.msa.ProviderCreated);
    }

    public static createSponsoredAccountWithDelegation(delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: AddProviderPayload) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.createSponsoredAccountWithDelegation(delegatorKeys.publicKey, signature, payload), providerKeys, ExtrinsicHelper.api.events.msa.MsaCreated);
    }

    public static grantDelegation(delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: AddProviderPayload) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.grantDelegation(delegatorKeys.publicKey, signature, payload), providerKeys, ExtrinsicHelper.api.events.msa.DelegationGranted);
    }

    public static grantSchemaPermissions(delegatorKeys: KeyringPair, providerMsaId: any, schemaIds: any) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.grantSchemaPermissions(providerMsaId, schemaIds), delegatorKeys, ExtrinsicHelper.api.events.msa.DelegationUpdated);
    }

    public static revokeSchemaPermissions(delegatorKeys: KeyringPair, providerMsaId: any, schemaIds: any) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.revokeSchemaPermissions(providerMsaId, schemaIds), delegatorKeys, ExtrinsicHelper.api.events.msa.DelegationUpdated);
    }

    public static revokeDelegationByDelegator(keys: KeyringPair, providerMsaId: any) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.revokeDelegationByDelegator(providerMsaId), keys, ExtrinsicHelper.api.events.msa.DelegationRevoked);
    }

    public static revokeDelegationByProvider(delegatorMsaId: u64, providerKeys: KeyringPair) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.revokeDelegationByProvider(delegatorMsaId), providerKeys, ExtrinsicHelper.api.events.msa.DelegationRevoked);
    }

    /** Messages Extrinsics */
    public static addIPFSMessage(keys: KeyringPair, schemaId: any, cid: string, payload_length: number) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.messages.addIpfsMessage(schemaId, cid, payload_length), keys, ExtrinsicHelper.api.events.messages.MessagesStored);
    }

    /** Stateful Storage Extrinsics */
    public static applyItemActions(keys: KeyringPair, schemaId: any, msa_id: MessageSourceId, actions: any, target_hash: any) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.statefulStorage.applyItemActions(msa_id, schemaId, target_hash, actions), keys, ExtrinsicHelper.api.events.statefulStorage.ItemizedPageUpdated);
    }

    public static removePage(keys: KeyringPair, schemaId: any, msa_id: MessageSourceId, page_id: any, target_hash: any) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.statefulStorage.deletePage(msa_id, schemaId, page_id, target_hash), keys, ExtrinsicHelper.api.events.statefulStorage.PaginatedPageDeleted);
    }

    public static upsertPage(keys: KeyringPair, schemaId: any, msa_id: MessageSourceId, page_id: any, payload: any, target_hash: any) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.statefulStorage.upsertPage(msa_id, schemaId, page_id, target_hash, payload), keys, ExtrinsicHelper.api.events.statefulStorage.PaginatedPageUpdated);
    }

    public static applyItemActionsWithSignature(delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: ItemizedSignaturePayload) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.statefulStorage.applyItemActionsWithSignature(delegatorKeys.publicKey, signature, payload), providerKeys, ExtrinsicHelper.api.events.statefulStorage.ItemizedPageUpdated);
    }

    public static applyItemActionsWithSignatureV2(delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: ItemizedSignaturePayloadV2) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.statefulStorage.applyItemActionsWithSignatureV2(delegatorKeys.publicKey, signature, payload), providerKeys, ExtrinsicHelper.api.events.statefulStorage.ItemizedPageUpdated);
    }

    public static deletePageWithSignature(delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: PaginatedDeleteSignaturePayload) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.statefulStorage.deletePageWithSignature(delegatorKeys.publicKey, signature, payload), providerKeys, ExtrinsicHelper.api.events.statefulStorage.PaginatedPageDeleted);
    }

    public static deletePageWithSignatureV2(delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: PaginatedDeleteSignaturePayloadV2) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.statefulStorage.deletePageWithSignatureV2(delegatorKeys.publicKey, signature, payload), providerKeys, ExtrinsicHelper.api.events.statefulStorage.PaginatedPageDeleted);
    }

    public static upsertPageWithSignature(delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: PaginatedUpsertSignaturePayload) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.statefulStorage.upsertPageWithSignature(delegatorKeys.publicKey, signature, payload), providerKeys, ExtrinsicHelper.api.events.statefulStorage.PaginatedPageUpdated);
    }

    public static upsertPageWithSignatureV2(delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: PaginatedUpsertSignaturePayloadV2) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.statefulStorage.upsertPageWithSignatureV2(delegatorKeys.publicKey, signature, payload), providerKeys, ExtrinsicHelper.api.events.statefulStorage.PaginatedPageUpdated);
    }

    public static getItemizedStorage(msa_id: MessageSourceId, schemaId: any): Promise<ItemizedStoragePageResponse> {
        return ExtrinsicHelper.apiPromise.rpc.statefulStorage.getItemizedStorage(msa_id, schemaId);
    }

    public static getPaginatedStorage(msa_id: MessageSourceId, schemaId: any): Promise<Vec<PaginatedStorageResponse>> {
        return ExtrinsicHelper.apiPromise.rpc.statefulStorage.getPaginatedStorage(msa_id, schemaId);
    }

    public static timeReleaseTransfer(keys: KeyringPair, who: KeyringPair, schedule: ReleaseSchedule) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.timeRelease.transfer(who.address, schedule), keys, ExtrinsicHelper.api.events.timeRelease.ReleaseScheduleAdded);
    }

    public static claimHandle(delegatorKeys: KeyringPair, payload: any) {
        const proof = { Sr25519: u8aToHex(delegatorKeys.sign(u8aWrapBytes(payload.toU8a()))) }
        return new Extrinsic(() => ExtrinsicHelper.api.tx.handles.claimHandle(delegatorKeys.publicKey, proof, payload), delegatorKeys, ExtrinsicHelper.api.events.handles.HandleClaimed);
    }

    public static retireHandle(delegatorKeys: KeyringPair) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.handles.retireHandle(), delegatorKeys, ExtrinsicHelper.api.events.handles.HandleRetired);
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

    public static addOnChainMessage(keys: KeyringPair, schemaId: any, payload: string) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.messages.addOnchainMessage(null, schemaId, payload), keys, ExtrinsicHelper.api.events.messages.MessagesStored);
    }

    /** Capacity Extrinsics **/
    public static setEpochLength(keys: KeyringPair, epoch_length: any) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.capacity.setEpochLength(epoch_length), keys, ExtrinsicHelper.api.events.capacity.EpochLengthUpdated);
    }
    public static stake(keys: KeyringPair, target: any, amount: any) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.capacity.stake(target, amount), keys, ExtrinsicHelper.api.events.capacity.Staked);
    }

    public static unstake(keys: KeyringPair, target: any, amount: any) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.capacity.unstake(target, amount), keys, ExtrinsicHelper.api.events.capacity.UnStaked);
    }

    public static withdrawUnstaked(keys: KeyringPair) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.capacity.withdrawUnstaked(), keys, ExtrinsicHelper.api.events.capacity.StakeWithdrawn);
    }

    public static payWithCapacityBatchAll(keys: KeyringPair, calls: any) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.frequencyTxPayment.payWithCapacityBatchAll(calls), keys, ExtrinsicHelper.api.events.utility.BatchCompleted);
    }

    public static executeUtilityBatchAll(keys: KeyringPair, calls: any) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.utility.batchAll(calls), keys, ExtrinsicHelper.api.events.utility.BatchCompleted);
    }

    public static executeUtilityBatch(keys: KeyringPair, calls: any) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.utility.batch(calls), keys, ExtrinsicHelper.api.events.utility.BatchCompleted);
    }

    public static executeUtilityForceBatch(keys: KeyringPair, calls: any) {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.utility.forceBatch(calls), keys, ExtrinsicHelper.api.events.utility.BatchCompleted);
    }

    public static async runToBlock(blockNumber: number) {
      let currentBlock = await getBlockNumber();
      while (currentBlock < blockNumber) {
        // In Rococo, just wait
        if (hasRelayChain()) {
            await new Promise((r) => setTimeout(r, 4_000));
        } else {
            await ExtrinsicHelper.apiPromise.rpc.engine.createBlock(true, true);
        }
        currentBlock = await getBlockNumber();
      }
    }
}
