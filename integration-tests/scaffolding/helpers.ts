import { KeyringPair } from "@polkadot/keyring/types";
import { Event } from "@polkadot/types/interfaces";
import { Codec, ISubmittableResult } from "@polkadot/types/types";
import { u8aToHex, u8aWrapBytes } from "@polkadot/util";
import { pipe, map } from "rxjs";

export type EventMap = {[key: string]: Event}
export type Sr25519Signature = {Sr25519: `0x${string}`}

export function groupEventsByKey() {
    return pipe(
        map((result: ISubmittableResult) => result.events.reduce((acc, {event}) => { acc[eventKey(event)] = event.data; return acc}, {})),
    )
}

export function eventKey(event:Event): string {
    return `${event.section}.${event.method}`;
}

export function signPayloadSr25519(keys: KeyringPair, data: Codec): Sr25519Signature {
    return {Sr25519: u8aToHex(keys.sign(u8aWrapBytes(data.toU8a())))}
}