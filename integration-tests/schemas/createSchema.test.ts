import "@frequency-chain/api-augment";
import { ApiRx } from "@polkadot/api";
import { connect, createKeys } from "../scaffolding/apiConnection"

import assert from "assert";

import { AVRO_GRAPH_CHANGE } from "./fixtures/avroGraphChangeSchemaType";
import { KeyringPair } from "@polkadot/keyring/types";
import { createSchema } from "../scaffolding/extrinsicHelpers";

describe("#createSchema", () => {
    let api: ApiRx;
    let keys: KeyringPair;

    before(async () => {
        let connectApi = await connect(process.env.WS_PROVIDER_URL);
        keys = createKeys("//Alice")
        api = connectApi
    })

    after(() => {
        api.disconnect()
    })

    it("should successfully create an Avro GraphChange schema", async () => {
        const chainEvents = await createSchema(api, keys, AVRO_GRAPH_CHANGE, "AvroBinary", "OnChain")

        assert.equal(chainEvents["system.ExtrinsicFailed"], undefined);
        assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined);
        assert.notEqual(chainEvents["schemas.SchemaCreated"], undefined);
    }).timeout(15000);
})
