import "@frequency-chain/api-augment";
import { ApiRx } from "@polkadot/api";
import assert from "assert";
import { connect, createKeys } from "../scaffolding/apiConnection";
import { signAndSend } from "../scaffolding/extrinsicHelpers";
import { DevAccounts, EventError, showTotalCost } from "../scaffolding/helpers";

describe("#setMaxSchemaModelBytes", function () {
    this.timeout(15000);

    const context = this.title;
    let api: ApiRx;
    let keys: any;

    before(async function () {
        let connectApi = await connect(process.env.WS_PROVIDER_URL);
        api = connectApi
        keys = createKeys(DevAccounts.Alice)
    })

    after(async function () {
        await showTotalCost(api, context);
        await api.disconnect()
    })

    it("should fail to set the schema size because of lack of root authority", async function () {
        await assert.rejects(signAndSend(() => api.tx.schemas.setMaxSchemaModelBytes(1000000), keys), EventError);
    });

    // NOTE: We need a governance account or a sudo call to test the positive case
});
