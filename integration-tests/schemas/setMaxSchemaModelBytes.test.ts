import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import assert from "assert";
import { Extrinsic, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { devAccounts, EventError } from "../scaffolding/helpers";

describe("#setMaxSchemaModelBytes", function () {
    let keys: KeyringPair;

    before(async function () {
        keys = devAccounts[0].keys;
    })

    it("should fail to set the schema size because of lack of root authority", async function () {
        const operation = new Extrinsic(() => ExtrinsicHelper.api.tx.schemas.setMaxSchemaModelBytes(1000000), keys);
        await assert.rejects(operation.signAndSend(), EventError);
    });

    // NOTE: We need a governance account or a sudo call to test the positive case
});
