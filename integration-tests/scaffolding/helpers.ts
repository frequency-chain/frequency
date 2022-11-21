import { keys } from "@polkadot/api-derive/staking";
import { KeyringPair } from "@polkadot/keyring/types";
import { Event } from "@polkadot/types/interfaces";
import { Codec, ISubmittableResult } from "@polkadot/types/types";
import { u8aToHex, u8aWrapBytes } from "@polkadot/util";
import { pipe, map } from "rxjs";

export function groupEventsByKey() {
    return pipe(
        map((result: ISubmittableResult) => result.events.reduce((acc, {event}) => { acc[eventKey(event)] = event.data; return acc}, {})),
    )
}

export function eventKey(event:Event) {
    return `${event.section}.${event.method}`;
}

export function signPayloadSr25519(keys: KeyringPair, data: Codec) {
    return {Sr25519: u8aToHex(keys.sign(u8aWrapBytes(data.toU8a())))}
}