import assert from "assert";
import { KeyringPair } from "@polkadot/keyring/types";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { createAndFundKeypair } from "../scaffolding/helpers";
import { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";

describe("Utility Batch Filtering", function () {
    let sender: KeyringPair;
    let recipient: KeyringPair;

    before(async function () {
        sender = await createAndFundKeypair();
        recipient = await createAndFundKeypair();
    });

    it("should successfully execute ✅ batch with allowed calls", async function () {
        // good batch: with only allowed calls
        const goodBatch: SubmittableExtrinsic<ApiTypes>[] = [];
        goodBatch.push(ExtrinsicHelper.api.tx.balances.transfer(recipient.address, 1000))
        goodBatch.push(ExtrinsicHelper.api.tx.system.remark("Hello From Batch"))
        goodBatch.push(ExtrinsicHelper.api.tx.msa.create())
        const batch = ExtrinsicHelper.executeUtilityBatchAll(sender, goodBatch);
        const [event, eventMap] = await batch.fundAndSend();
        assert.notEqual(event, undefined, "should return an event");
        assert.notEqual(eventMap, undefined, "should return an eventMap");
    });

    it("should fail to execute ❌ batch with disallowed calls", async function () {
        // bad batch: with a mix of allowed and disallowed calls
        const badBatch: SubmittableExtrinsic<ApiTypes>[] = [];
        badBatch.push(ExtrinsicHelper.api.tx.balances.transfer(recipient.address, 1000))
        badBatch.push(ExtrinsicHelper.api.tx.system.remark("Hello From Batch"))
        badBatch.push(ExtrinsicHelper.api.tx.msa.retireMsa())
        badBatch.push(ExtrinsicHelper.api.tx.handles.retireHandle())
        const batch = ExtrinsicHelper.executeUtilityBatchAll(sender, badBatch);
        try {
            await batch.fundAndSend();
        } catch (error) {
            assert.notEqual(error, undefined, "should return an error");
        }
    });    
});
