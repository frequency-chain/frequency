import { ApiRx } from "@polkadot/api";
import { connect } from "./scaffolding/apiConnection"

import assert from "assert";

import { AVRO_GRAPH_CHANGE } from "./fixtures/avroGraphChangeSchemaType";
import { filter } from "rxjs";
import { groupEventsByKey } from "./scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";

describe("#createSchema", () => {
    let api: ApiRx;
    let keys: KeyringPair;

    before(async () => {
        let {api, keys} = await connect(process.env.WS_PROVIDER_URL);
        api = api
        keys = keys
    })

    after(() => {
        api.disconnect()
    })

    it("should successfully create an Avro GraphChange schema", async () => {
        const chainEvents = api.tx.schemas.createSchema(JSON.stringify(AVRO_GRAPH_CHANGE), "AvroBinary", "OnChain").signAndSend(keys).pipe(
                filter(({status}) => status.isInBlock),
                groupEventsByKey())

        assert.equal(chainEvents["system.ExtrinsicFailed"], undefined);
        assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined);
        assert.notEqual(chainEvents["schemas.SchemaCreated"], undefined);
    }).timeout(15000);
})
