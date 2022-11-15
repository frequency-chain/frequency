import { ApiRx } from "@polkadot/api";
import assert from "assert";
import { filter } from "rxjs";
import { connect } from "./scaffolding/apiConnection";
import { groupEventsByKey } from "./scaffolding/helpers";

describe("#setMaxSchemaModelBytes", () => {
    let api: ApiRx;
    let keys: any;

    beforeEach(async () => {
        let { api, keys } = await connect();
        api = api;
        keys = keys
    })

    it("should fail to set the schema size because of lack of root authority", async () => {
        const chainEvents = api.tx.schemas.setMaxSchemaModelBytes(1000000).signAndSend(keys).pipe(
                filter(({status}) => status.isInBlock),
                groupEventsByKey()
            )

        assert.notEqual(chainEvents["system.ExtrinsicFailed"], undefined);
        assert.equal(chainEvents["system.ExtrinsicSuccess"], undefined);

    }).timeout(1000000);

    // NOTE: We need a root account to test the positive case
});
