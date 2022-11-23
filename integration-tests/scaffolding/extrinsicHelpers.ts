import { ApiRx } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { Event } from "@polkadot/types/interfaces";
import { firstValueFrom, filter } from "rxjs";
import { EventMap, groupEventsByKey, Sr25519Signature } from "./helpers";

type AddKeyData = {
    msaId?: any;
    expiration?: any;
    newPublicKey?: any;
}

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