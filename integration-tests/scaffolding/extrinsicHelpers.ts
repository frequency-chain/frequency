import { ApiRx } from "@polkadot/api";
import { SubmittableExtrinsic } from "@polkadot/api/types";
import { KeyringPair } from "@polkadot/keyring/types";
import { Compact, u128 } from "@polkadot/types";
import { FrameSystemAccountInfo } from "@polkadot/types/lookup";
import { AnyNumber, ISubmittableResult } from "@polkadot/types/types";
import { firstValueFrom, filter, map, catchError } from "rxjs";
import { devAccounts, EventMap, parseResult, Sr25519Signature } from "./helpers";
import { connect } from "./apiConnection";
import { SignedBlock } from "@polkadot/types/interfaces";

export type AddKeyData = { msaId?: any; expiration?: any; newPublicKey?: any; }
type AddProviderPayload = { authorizedMsaId?: any; schemaIds?: any; expiration?: any; }

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
 * Normally, I'd say the best experience is for the helper to return both the ID of the created entity
 * along with a map of emitted events. But in this case, returning that value will increase the complexity
 * of each helper, since each would have to check for undefined values at every lookup. So, this may be
 * a rare case when it is best to simply return the map of emitted events and trust the user to look them
 * up in the test.
 */

export enum SubmissionType {
    Submit,
    GetEstimatedFee,
    FundOperation,
}

export class Extrinsic<T extends ISubmittableResult = ISubmittableResult> {

    private extrinsic: () => SubmittableExtrinsic<"rxjs", T>;
    private keys: KeyringPair;

    constructor(extrinsic: () => SubmittableExtrinsic<"rxjs", T>, keys: KeyringPair) {
        this.extrinsic = extrinsic;
        this.keys = keys;
    }

    public async signAndSend(): Promise<EventMap> {
        return firstValueFrom(this.extrinsic().signAndSend(this.keys).pipe(
            filter(({ status }) => status.isInBlock || status.isFinalized),
            parseResult(),
        ));
    }

    public async getEstimatedTxFee(): Promise<bigint> {
        return firstValueFrom(this.extrinsic().paymentInfo(this.keys).pipe(
            map((info) => info.partialFee.toBigInt())
        ));
    }

    private async fundOperation(source?: KeyringPair): Promise<void> {
        const amount = await this.getEstimatedTxFee();
        await ExtrinsicHelper.transferFunds(source || devAccounts[0].keys, this.keys, amount).signAndSend();
    }

    public async fundAndSend(source?: KeyringPair): Promise<EventMap> {
        await this.fundOperation(source);
        return this.signAndSend();
    }
}

export class ExtrinsicHelper {
    public static api: ApiRx;

    constructor() { }

    public static async initialize(providerUrl?: string | string[] | undefined) {
        ExtrinsicHelper.api = await connect(providerUrl);
    }

    public static getLastBlock(): Promise<SignedBlock> {
        return firstValueFrom(ExtrinsicHelper.api.rpc.chain.getBlock());
    }

    /** Query Extrinsics */
    public static getAccountInfo(address: string): Promise<FrameSystemAccountInfo> {
        return firstValueFrom(ExtrinsicHelper.api.query.system.account(address));
    }

    public static getSchemaMaxBytes() {
        return firstValueFrom(ExtrinsicHelper.api.query.schemas.governanceSchemaModelMaxBytes());
    }

    /** Balance Extrinsics */
    public static transferFunds(keys: KeyringPair, dest: KeyringPair, amount: Compact<u128> | AnyNumber): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.balances.transfer(dest.address, amount), keys);
    }

    /** Schema Extrinsics */
    public static createSchema(keys: KeyringPair, model: any, modelType: "AvroBinary" | "Parquet", payloadLocation: "OnChain" | "IPFS"): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.schemas.createSchema(JSON.stringify(model), modelType, payloadLocation), keys);
    }

    /** MSA Extrinsics */
    public static createMsa(keys: KeyringPair): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.create(), keys);
    }

    public static addPublicKeyToMsa(keys: KeyringPair, ownerSignature: Sr25519Signature, newSignature: Sr25519Signature, payload: AddKeyData): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.addPublicKeyToMsa(keys.publicKey, ownerSignature, newSignature, payload), keys);
    }

    public static deletePublicKey(keys: KeyringPair, publicKey: Uint8Array): Extrinsic {
        ExtrinsicHelper.api.query.msa
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.deleteMsaPublicKey(publicKey), keys);
    }

    public static retireMsa(keys: KeyringPair): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.retireMsa(), keys);
    }

    public static createProvider(keys: KeyringPair, providerName: string): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.createProvider(providerName), keys);
    }

    public static grantDelegation(delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: AddProviderPayload): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.grantDelegation(delegatorKeys.publicKey, signature, payload), providerKeys);
    }

    public static grantSchemaPermissions(delegatorKeys: KeyringPair, providerMsaId: any, schemaIds: any): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.grantSchemaPermissions(providerMsaId, schemaIds), delegatorKeys);
    }

    public static revokeDelegationByDelegator(keys: KeyringPair, providerMsaId: any): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.msa.revokeDelegationByDelegator(providerMsaId), keys);
    }

    /** Messages Extrinsics */
    public static addIPFSMessage(keys: KeyringPair, schemaId: any, cid: string, payload_length: number): Extrinsic {
        return new Extrinsic(() => ExtrinsicHelper.api.tx.messages.addIpfsMessage(schemaId, cid, payload_length), keys);
    }
}
