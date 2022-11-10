import assert from "assert";

import ApiWrapper from "./scaffolding/api/apiWrapper";
import { getFrequencyAPI, getSignerAccountKeys} from "./scaffolding/api/apiConnection";

describe("#setMaxSchemaModelBytes", () => {
    let api: ApiWrapper;

    beforeEach(async () => {
        api = new ApiWrapper(await getFrequencyAPI(), getSignerAccountKeys());
    });

    it("should fail to set the schema size because of lack of root authority", async () => {
        await api.setMaxSchemaSize(api._keys, 1000000);
        
        const schemaEvent = api.getEvent("schemas.SchemaMaxSizeChanged");
        const successEvent = api.getEvent("system.ExtrinsicSuccess");
        const failureEvent = api.getEvent("system.ExtrinsicFailed");

        assert.equal(true, typeof(schemaEvent) == "undefined" && typeof(successEvent) == "undefined")
        assert.equal(true, typeof(failureEvent) !== "undefined");

    }).timeout(1000000);
});
