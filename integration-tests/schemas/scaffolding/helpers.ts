import { Event } from "@polkadot/types/interfaces";
import { ISubmittableResult } from "@polkadot/types/types";
import { pipe, map } from "rxjs";

export function groupEventsByKey() {
    return pipe(
        map((result: ISubmittableResult) => result.events.reduce((acc, {event}) => { acc[eventKey(event)] = event.data; return acc}, {})),
    )
}

export function eventKey(event:Event) {
    return `${event.section}.${event.method}`;
}