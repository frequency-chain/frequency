import { ApiRx } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { firstValueFrom, filter } from "rxjs";
import { EventMap, groupEventsByKey, Sr25519Signature } from "./helpers";

type AddKeyData = { msaId?: any; expiration?: any; newPublicKey?: any;}
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

/** Schema Extrinsics */
export async function createSchema(api: ApiRx, keys: KeyringPair, model: any, modelType: "AvroBinary" | "Parquet", payloadLocation: "OnChain" | "IPFS"): Promise<EventMap> {
    return firstValueFrom(api.tx.schemas.createSchema(JSON.stringify(model), modelType, payloadLocation).signAndSend(keys).pipe(
        filter(({status}) => status.isInBlock || status.isFinalized),
        groupEventsByKey()))
}

/** MSA Extrinsics */
export async function createMsa(api: ApiRx, keys: KeyringPair): Promise<EventMap> {
    return firstValueFrom(api.tx.msa.create().signAndSend(keys).pipe(
        filter(({status}) => status.isInBlock || status.isFinalized),
        groupEventsByKey()))
}

export function addPublicKeyToMsa(api: ApiRx, keys: KeyringPair, ownerSignature: Sr25519Signature, newSignature: Sr25519Signature, payload: AddKeyData): Promise<EventMap> {
    return firstValueFrom(
        api.tx.msa.addPublicKeyToMsa(keys.publicKey, ownerSignature, newSignature, payload)
        .signAndSend(keys)
        .pipe(
            filter(({status}) => status.isInBlock || status.isFinalized),
            groupEventsByKey()
        ))
}

export function deletePublicKey(api: ApiRx, keys: KeyringPair, publicKey: Uint8Array): Promise<EventMap> {
    return firstValueFrom(
        api.tx.msa.deleteMsaPublicKey(publicKey)
        .signAndSend(keys)
        .pipe(
            filter(({status}) => status.isInBlock || status.isFinalized),
            groupEventsByKey()
        ))
}

export function createProvider(api: ApiRx, keys: KeyringPair, providerName: string): Promise<EventMap> {
    return firstValueFrom(api.tx.msa.createProvider(providerName).signAndSend(keys)
    .pipe(
        filter(({status}) => status.isInBlock || status.isFinalized),
        groupEventsByKey()
    ))
}

export function grantDelegation(api: ApiRx, delegatorKeys: KeyringPair, providerKeys: KeyringPair, signature: Sr25519Signature, payload: AddProviderPayload): Promise<EventMap> {
    return firstValueFrom(api.tx.msa.grantDelegation(delegatorKeys.publicKey, signature, payload).signAndSend(providerKeys)
        .pipe(
            filter(({status}) => status.isInBlock || status.isFinalized),
            groupEventsByKey()
        )
    )
}

export function grantSchemaPermissions(api: ApiRx, delegatorKeys: KeyringPair, providerMsaId: any, schemaIds: any): Promise<EventMap> {
    return firstValueFrom(
        api.tx.msa.grantSchemaPermissions(providerMsaId, schemaIds).signAndSend(delegatorKeys)
        .pipe(
            filter(({status}) => status.isInBlock || status.isFinalized),
            groupEventsByKey()
        )
    )
}

export function revokeDelegationByDelegator(api: ApiRx, keys: KeyringPair, providerMsaId: any): Promise<EventMap> {
    return firstValueFrom(api.tx.msa.revokeDelegationByDelegator(providerMsaId).signAndSend(keys)
        .pipe(
            filter(({status}) => status.isInBlock || status.isFinalized),
            groupEventsByKey()
        )
    )
}