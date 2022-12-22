import "@frequency-chain/api-augment";
import { ApiRx } from "@polkadot/api";
import { connect } from "../scaffolding/apiConnection"

import assert from "assert";

import { AVRO_GRAPH_CHANGE } from "./fixtures/avroGraphChangeSchemaType";
import { KeyringPair } from "@polkadot/keyring/types";
import { createSchema } from "../scaffolding/extrinsicHelpers";
import { AccountFundingInputs, createAndFundAccount, generateFundingInputs, txAccountingHook } from "../scaffolding/helpers";

describe("#createSchema", function () {
    this.timeout(15000);

    let fundingInputs: AccountFundingInputs;

    let api: ApiRx;
    let keys: KeyringPair;

    before(async function () {
        let connectApi = await connect(process.env.WS_PROVIDER_URL);
        api = connectApi
        fundingInputs = generateFundingInputs(api, this.title);
        keys = (await createAndFundAccount(fundingInputs)).newAccount;
    })

    after(async function () {
        await txAccountingHook(api, fundingInputs.context);
        await api.disconnect()
    });

    it("should successfully create an Avro GraphChange schema", async function () {
        const chainEvents = await createSchema(api, keys, AVRO_GRAPH_CHANGE, "AvroBinary", "OnChain")

        assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined);
        assert.notEqual(chainEvents["schemas.SchemaCreated"], undefined);
    });
})
