import "@frequency-chain/api-augment";
import { ApiRx } from "@polkadot/api";
import assert from "assert";
import { filter, firstValueFrom } from "rxjs";
import { connect } from "./scaffolding/apiConnection";
import { groupEventsByKey } from "./scaffolding/helpers";

describe("#setMaxSchemaModelBytes", () => {
    let api: ApiRx;
    let keys: any;

    before(async () => {
        let {api: connectApi, keys: connectKeys} = await connect(process.env.WS_PROVIDER_URL);
        api = connectApi
        keys = connectKeys
    })

    after(() => {
        api.disconnect()
    })

    it("should fail to set the schema size because of lack of root authority", async () => {
        const chainEvents = await firstValueFrom(api.tx.schemas.setMaxSchemaModelBytes(1000000).signAndSend(keys).pipe(
                filter(({status}) => status.isInBlock || status.isFinalized),
                groupEventsByKey()))

        assert.notEqual(chainEvents["system.ExtrinsicFailed"], undefined);
        assert.equal(chainEvents["system.ExtrinsicSuccess"], undefined);

    }).timeout(15000);

    // NOTE: We need a governance account or a sudo call to test the positive case
});
