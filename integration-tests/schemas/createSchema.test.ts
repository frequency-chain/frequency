import "@frequency-chain/api-augment";
import { ApiRx } from "@polkadot/api";
import { connect, createKeys } from "../scaffolding/apiConnection"

import assert from "assert";

import { AVRO_GRAPH_CHANGE } from "./fixtures/avroGraphChangeSchemaType";
import { filter, firstValueFrom } from "rxjs";
import { groupEventsByKey } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";

describe("#createSchema", () => {
    let api: ApiRx;
    let keys: KeyringPair;

    before(async () => {
        let connectApi = await connect(process.env.WS_PROVIDER_URL);
        api = connectApi
    })

    beforeEach(() => {
        keys = createKeys("//Alice")
    })

    after(() => {
        api.disconnect()
    })

    it("should successfully create an Avro GraphChange schema", async () => {
        const chainEvents = await firstValueFrom(api.tx.schemas.createSchema(JSON.stringify(AVRO_GRAPH_CHANGE), "AvroBinary", "OnChain").signAndSend(keys).pipe(
                filter(({status}) => status.isInBlock || status.isFinalized),
                groupEventsByKey()))

        assert.equal(chainEvents["system.ExtrinsicFailed"], undefined);
        assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined);
        assert.notEqual(chainEvents["schemas.SchemaCreated"], undefined);
    }).timeout(15000);
})
