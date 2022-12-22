import { ApiRx } from "@polkadot/api";
import { SubmittableExtrinsic } from "@polkadot/api/types";
import { KeyringPair } from "@polkadot/keyring/types";
import RpcError from "@polkadot/rpc-provider/coder/error";
import { Compact, u128 } from "@polkadot/types";
import { FrameSystemAccountInfo } from "@polkadot/types/lookup";
import { AnyNumber, ISubmittableResult } from "@polkadot/types/types";
import { firstValueFrom, filter, catchError, Observable } from "rxjs";
import { EventMap, parseResult, Sr25519Signature } from "./helpers";

type AddKeyData = { msaId?: any; expiration?: any; newPublicKey?: any; }
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


/** Generic wrapper **/
export async function signAndSend<T extends ISubmittableResult>(
    f: () => SubmittableExtrinsic<"rxjs", T>,
    keys: KeyringPair,
    customErrorHandler: (err: RpcError, caught: Observable<T>) => Observable<T> = (err, caught) => caught): Promise<EventMap> {
    return firstValueFrom(f().signAndSend(keys).pipe(
        filter(({ status }) => status.isInBlock || status.isFinalized),
        catchError((err: RpcError, caught) => {
            console.log(`code: ${err.code}, name: ${err.name}, message: ${err.message}`);
            if (customErrorHandler) {
                return customErrorHandler(err, caught);
            }
            throw err;
        }),
        parseResult(),
    ));
}

/** Query Extrinsics */
export async function getAccountInfo(api: ApiRx, address: string): Promise<FrameSystemAccountInfo> {
    return firstValueFrom(api.query.system.account(address));
}

/** Balance Extrinsics */
export async function transferFunds(api: ApiRx, keys: KeyringPair, dest: KeyringPair, amount: Compact<u128> | AnyNumber): Promise<EventMap> {
    return signAndSend(() => api.tx.balances.transfer(dest.address, amount), keys);
}

/** Schema Extrinsics */
export async function createSchema(api: ApiRx, keys: KeyringPair, model: any, modelType: "AvroBinary" | "Parquet", payloadLocation: "OnChain" | "IPFS"): Promise<EventMap> {
    return signAndSend(() => api.tx.schemas.createSchema(JSON.stringify(model), modelType, payloadLocation), keys);
}

/** MSA Extrinsics */
export async function createMsa(api: ApiRx, keys: KeyringPair, ignore_already_exist = true): Promise<EventMap> {
    return signAndSend(() => api.tx.msa.create(), keys, (err, caught) => {
        if (!ignore_already_exist || err.name !== 'KeyAlreadyRegistered') {
            throw err;
        }

        return caught;
    });
}

export function addPublicKeyToMsa(api: ApiRx, keys: KeyringPair, ownerSignature: Sr25519Signature, newSignature: Sr25519Signature, payload: AddKeyData): Promise<EventMap> {
    return signAndSend(() => api.tx.msa.addPublicKeyToMsa(keys.publicKey, ownerSignature, newSignature, payload), keys);
}

export function deletePublicKey(api: ApiRx, keys: KeyringPair, publicKey: Uint8Array): Promise<EventMap> {
    return signAndSend(() => api.tx.msa.deleteMsaPublicKey(publicKey), keys);
}

export function createProvider(api: ApiRx, keys: KeyringPair, providerName: string): Promise<EventMap> {
    return signAndSend(() => api.tx.msa.createProvider(providerName), keys);
}

export function grantDelegation(api: ApiRx, delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: AddProviderPayload): Promise<EventMap> {
    return signAndSend(() => api.tx.msa.grantDelegation(delegatorKeys.publicKey, signature, payload), providerKeys);
}

export function grantSchemaPermissions(api: ApiRx, delegatorKeys: KeyringPair, providerMsaId: any, schemaIds: any): Promise<EventMap> {
    return signAndSend(() => api.tx.msa.grantSchemaPermissions(providerMsaId, schemaIds), delegatorKeys);
}

export function revokeDelegationByDelegator(api: ApiRx, keys: KeyringPair, providerMsaId: any): Promise<EventMap> {
    return signAndSend(() => api.tx.msa.revokeDelegationByDelegator(providerMsaId), keys);
}

/** Messages Extrinsics */
export async function addIPFSMessage(api: ApiRx, keys: KeyringPair, schemaId: any, cid: string, payload_length: number): Promise<EventMap> {
    return signAndSend(() => api.tx.messages.addIpfsMessage(schemaId, cid, payload_length), keys);
}
