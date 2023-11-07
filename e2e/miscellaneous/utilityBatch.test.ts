import assert from "assert";
import { KeyringPair } from "@polkadot/keyring/types";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { DOLLARS, createAndFundKeypair } from "../scaffolding/helpers";
import { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
import { getFundingSource } from "../scaffolding/funding";

describe("Utility Batch Filtering", function () {
    let sender: KeyringPair;
    let recipient: KeyringPair;

    const fundingSource = getFundingSource("misc-util-batch");

    beforeEach(async function () {
      sender = await createAndFundKeypair(fundingSource, 5n * DOLLARS, 'utility-sender');
      recipient = await createAndFundKeypair(fundingSource, 5n * DOLLARS, 'utility-recipient');
    });

    it("should successfully execute ✅ batch with allowed calls", async function () {
        // good batch: with only allowed calls
        const goodBatch: SubmittableExtrinsic<ApiTypes>[] = [];
        goodBatch.push(ExtrinsicHelper.api.tx.balances.transferAllowDeath(recipient.address, 1000))
        goodBatch.push(ExtrinsicHelper.api.tx.system.remark("Hello From Batch"))
        goodBatch.push(ExtrinsicHelper.api.tx.msa.create())
        const batch = ExtrinsicHelper.executeUtilityBatchAll(sender, goodBatch);
        const { target: event, eventMap } = await batch.fundAndSend(fundingSource);
        assert.notEqual(event, undefined, "should return an event");
        assert.notEqual(eventMap, undefined, "should return an eventMap");
    });

    it("should fail to execute ❌ batchAll with disallowed calls", async function () {
        // bad batch: with a mix of allowed and disallowed calls
        const badBatch: SubmittableExtrinsic<ApiTypes>[] = [];
        //allowed
        badBatch.push(ExtrinsicHelper.api.tx.balances.transferAllowDeath(recipient.address, 1000))
        badBatch.push(ExtrinsicHelper.api.tx.system.remark("Hello From Batch"))
        // not allowed
        badBatch.push(ExtrinsicHelper.api.tx.handles.retireHandle())
        badBatch.push(ExtrinsicHelper.api.tx.msa.retireMsa())
        let error: any;

        // batchAll
        const batchAll = ExtrinsicHelper.executeUtilityBatchAll(sender, badBatch);
        try {
            await batchAll.fundAndSend(fundingSource);
            assert.fail("batchAll should have caused an error");
        } catch (err) {
            assert.notEqual(err, undefined, " batchAll should return an error");
        }
    });

    it("should fail to execute ❌ batch with disallowed calls", async function () {
        // bad batch: with a mix of allowed and disallowed calls
        const badBatch: SubmittableExtrinsic<ApiTypes>[] = [];
        badBatch.push(ExtrinsicHelper.api.tx.balances.transferAllowDeath(recipient.address, 1000))
        badBatch.push(ExtrinsicHelper.api.tx.system.remark("Hello From Batch"))
        badBatch.push(ExtrinsicHelper.api.tx.handles.retireHandle())
        badBatch.push(ExtrinsicHelper.api.tx.msa.retireMsa())

        // batch
        const batch = ExtrinsicHelper.executeUtilityBatch(sender, badBatch);
        const { target: ok, eventMap } = await batch.fundAndSend(fundingSource);
        assert.equal(ok, undefined, "should not return an ok event");
        assert.equal(eventMap["utility.BatchCompleted"], undefined, "should not return a batch completed event");
        assert.notEqual(eventMap["utility.BatchInterrupted"], undefined, "should return a batch interrupted event");
    });

    it("should fail to execute ❌ forceBatch with disallowed calls", async function () {
        // bad batch: with a mix of allowed and disallowed calls
        const badBatch: SubmittableExtrinsic<ApiTypes>[] = [];
        badBatch.push(ExtrinsicHelper.api.tx.balances.transferAllowDeath(recipient.address, 1000))
        badBatch.push(ExtrinsicHelper.api.tx.system.remark("Hello From Batch"))
        badBatch.push(ExtrinsicHelper.api.tx.handles.retireHandle())
        badBatch.push(ExtrinsicHelper.api.tx.msa.retireMsa())

        // forceBatch
        const forceBatch = ExtrinsicHelper.executeUtilityForceBatch(sender, badBatch);
        const { target: ok, eventMap } = await forceBatch.fundAndSend(fundingSource);
        assert.equal(ok, undefined, "should not return an ok event");
        assert.equal(eventMap["utility.BatchCompleted"], undefined, "should not return a batch completed event");
        assert.notEqual(eventMap["utility.BatchCompletedWithErrors"], undefined, "should return a batch completed with error event");
    });

    it("should fail to execute ❌ batch  with `Pays::No` calls", async function () {
        // bad batch: with frequency related Pays::No call
        const badBatch: SubmittableExtrinsic<ApiTypes>[] = [];
        badBatch.push(ExtrinsicHelper.api.tx.msa.retireMsa())
        const batch = ExtrinsicHelper.executeUtilityBatchAll(sender, badBatch);
        try {
            await batch.fundAndSend(fundingSource);
            assert.fail("batch should have caused an error");
        } catch (err) {
            assert.notEqual(err, undefined, "should return an error");
        }
    });

    it("should fail to execute ❌ batch with `Pays::Yes` `create_provider`call blocked by Frequency", async function () {
        // bad batch: with frequency related Pays::Yes call
        const badBatch: SubmittableExtrinsic<ApiTypes>[] = [];
        badBatch.push(ExtrinsicHelper.api.tx.msa.createProvider("I am a ba(tch)d provider"))
        const batch = ExtrinsicHelper.executeUtilityBatchAll(sender, badBatch);
        try {
            await batch.fundAndSend(fundingSource);
            assert.fail("batch should have caused an error");
        } catch (err) {
            assert.notEqual(err, undefined, "should return an error");
        }
    });

    it("should fail to execute ❌ batch with `Pays::Yes` `create_schema` call blocked by Frequency", async function () {
        // bad batch: with frequency related Pays::Yes call
        const badBatch: SubmittableExtrinsic<ApiTypes>[] = [];
        badBatch.push(ExtrinsicHelper.api.tx.msa.createProvider("I am a ba(tch)d provider"))
        const batch = ExtrinsicHelper.executeUtilityBatchAll(sender, badBatch);
        try {
            await batch.fundAndSend(fundingSource);
            assert.fail("batch should have caused an error");
        } catch (err) {
            assert.notEqual(err, undefined, "should return an error");
        }
    });

    it("should fail to execute ❌ batch with nested batch", async function () {
        // batch with nested batch
        const nestedBatch: SubmittableExtrinsic<ApiTypes>[] = [];
        const innerBatch: SubmittableExtrinsic<ApiTypes>[] = [];
        innerBatch.push(ExtrinsicHelper.api.tx.balances.transferAllowDeath(recipient.address, 1000))
        innerBatch.push(ExtrinsicHelper.api.tx.system.remark("Hello From Batch"))
        nestedBatch.push(ExtrinsicHelper.api.tx.utility.batch(innerBatch))
        const batch = ExtrinsicHelper.executeUtilityBatchAll(sender, nestedBatch);
        try {
            await batch.fundAndSend(fundingSource);
            assert.fail("batch should have caused an error");
        } catch (err) {
            assert.notEqual(err, undefined, "should return an error");
        }
    });
});
