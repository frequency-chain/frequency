import { ApiRx, WsProvider } from "@polkadot/api";
import { Keyring } from "@polkadot/api";

import assert from "assert";

import { AVRO_GRAPH_CHANGE } from "./scaffolding/fixtures/schemaTypes";
import { AVRO } from "./scaffolding/fixtures/modelTypes";
import { ON_CHAIN } from "./scaffolding/fixtures/payloadLocation";
import { filter, firstValueFrom, map, mergeMap, Observable, pipe } from "rxjs";
import { Event } from "@polkadot/types/interfaces";
import { ISubmittableResult } from "@polkadot/types/types";

describe("#createSchema", () => {
    let apiObservable: Observable<ApiRx>;
    let keys: any;

    beforeEach(() => {
        const provider = new WsProvider("ws://127.0.0.1:9944");
        apiObservable = ApiRx.create({ provider });
        const keyring = new Keyring({ type: "sr25519" });
        keys = keyring.addFromUri("//Alice");
    })

    it.only("should successfully create an Avro GraphChange schema", async () => {
        const chainEvents = await firstValueFrom(
            apiObservable.pipe(
                mergeMap((api) => api.tx.schemas.createSchema(JSON.stringify(AVRO_GRAPH_CHANGE), AVRO, ON_CHAIN).signAndSend(keys).pipe(
                filter(({status}) => status.isInBlock),
                groupEventsByKey()
            ))))
        assert.equal(chainEvents["system.ExtrinsicFailed"], undefined);
        assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined);
        assert.notEqual(chainEvents["schemas.SchemaCreated"], undefined);
    }).timeout(10000);
})


/************* Private Helper Functions *************/
function groupEventsByKey() {
    return pipe(
        map((result: ISubmittableResult) => result.events.reduce((acc, {event}) => { acc[eventKey(event)] = event.data; return acc}, {})),
    )
}

function eventKey(event:Event) {
    return `${event.section}.${event.method}`;
}