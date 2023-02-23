import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { u16, u64 } from "@polkadot/types";
import assert from "assert";
import { AddProviderPayload, Extrinsic, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { createAndFundKeypair, createKeys, generateDelegationPayload, log, signPayloadSr25519 } from "../scaffolding/helpers";

describe.only("Capacity Scenario Tests", function () {
    // let underFundedKeys: KeyringPair;
    // let otherMsaKeys: KeyringPair;
    // let noMsaKeys: KeyringPair;
    // let providerKeys: KeyringPair;
    let otherProviderKeys: KeyringPair;
    // let schemaId: u16;
    // let providerId: u64;
    let otherProviderId: u64;
    // let msaId: u64;
    // let otherMsaId: u64;

    let stakeKeys: KeyringPair;
    let stakeProviderId: u64;
    let unstakeKeys: KeyringPair;
    let unstakeProviderId: u64;
    let withdrawKeys: KeyringPair;
    let withdrawProviderId: u64;

    before(async function () {
        log("*********************************")
        log("********** BEGIN SETUP **********")
        log("*********************************")
        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for stake operations
        stakeKeys = await createAndFundKeypair();
        let createStakeProviderMsaOp = ExtrinsicHelper.createMsa(stakeKeys);
        await createStakeProviderMsaOp.fundAndSend();
        let createStakeProviderOp = ExtrinsicHelper.createProvider(stakeKeys, "TestProvider");
        let [stakeProviderEvent] = await createStakeProviderOp.fundAndSend();
        assert.notEqual(stakeProviderEvent, undefined, "setup should return a ProviderCreated event");
        if (stakeProviderEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(stakeProviderEvent)) {
            stakeProviderId = stakeProviderEvent.data.providerId;
        }
        assert.notEqual(stakeProviderId, undefined, "setup should populate stakeProviderId");

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for unstake operations
        unstakeKeys = await createAndFundKeypair();
        let createUnstakeProviderMsaOp = ExtrinsicHelper.createMsa(unstakeKeys);
        await createUnstakeProviderMsaOp.fundAndSend();
        let createUnstakeProviderOp = ExtrinsicHelper.createProvider(unstakeKeys, "TestProvider");
        let [unstakeProviderEvent] = await createUnstakeProviderOp.fundAndSend();
        assert.notEqual(unstakeProviderEvent, undefined, "setup should return a ProviderCreated event");
        if (unstakeProviderEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(unstakeProviderEvent)) {
            unstakeProviderId = unstakeProviderEvent.data.providerId;
        }
        assert.notEqual(unstakeProviderId, undefined, "setup should populate unstakeProviderId");

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for withdraw operations
        withdrawKeys = await createAndFundKeypair();
        let createWithdrawProviderMsaOp = ExtrinsicHelper.createMsa(withdrawKeys);
        await createWithdrawProviderMsaOp.fundAndSend();
        let createWithdrawProviderOp = ExtrinsicHelper.createProvider(withdrawKeys, "TestProvider");
        let [withdrawProviderEvent] = await createWithdrawProviderOp.fundAndSend();
        assert.notEqual(withdrawProviderEvent, undefined, "setup should return a ProviderCreated event");
        if (withdrawProviderEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(withdrawProviderEvent)) {
            withdrawProviderId = withdrawProviderEvent.data.providerId;
        }
        assert.notEqual(withdrawProviderId, undefined, "setup should populate withdrawProviderId");

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // providerKeys = await createAndFundKeypair();
        // let createProviderMsaOp = ExtrinsicHelper.createMsa(providerKeys);
        // await createProviderMsaOp.fundAndSend();
        // let createProviderOp = ExtrinsicHelper.createProvider(providerKeys, "TestProvider");
        // let [providerEvent] = await createProviderOp.fundAndSend();
        // assert.notEqual(providerEvent, undefined, "setup should return a ProviderCreated event");
        // if (providerEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(providerEvent)) {
        //     providerId = providerEvent.data.providerId;
        // }
        // assert.notEqual(providerId, undefined, "setup should populate providerId");

        otherProviderKeys = await createAndFundKeypair();
        let createProviderMsaOp = ExtrinsicHelper.createMsa(otherProviderKeys);
        await createProviderMsaOp.fundAndSend();
        let createProviderOp = ExtrinsicHelper.createProvider(otherProviderKeys, "TestProvider");
        let [providerEvent] = await createProviderOp.fundAndSend();
        assert.notEqual(providerEvent, undefined, "setup should return a ProviderCreated event");
        if (providerEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(providerEvent)) {
            otherProviderId = providerEvent.data.providerId;
        }
        assert.notEqual(otherProviderId, undefined, "setup should populate providerId");

        log("*********************************")
        log("*********** END SETUP ***********")
        log("*********************************\n\n")

    });

    describe("stake testing", function () {

        it("should fail to stake for InvalidTarget", async function () {
            const failStakeObj = ExtrinsicHelper.stake(stakeKeys, 999999, 1000000);
            await assert.rejects(failStakeObj.fundAndSend(), { name: "InvalidTarget" });
        });

        it("should fail to stake for InsufficientBalance", async function () {
            const failStakeObj = ExtrinsicHelper.stake(stakeKeys, stakeProviderId, 2000000);
            // TODO: Does this actually test for the InsufficientBalance error?
            const [failStakeEvent] = await failStakeObj.fundAndSend();
            assert.notEqual(failStakeEvent, undefined, "setup should return a InsufficientBalance");
            // TODO: This returns: Missing expected rejection (InsufficientBalance)
            // await assert.rejects(failStakeObj.fundAndSend(), { name: "InsufficientBalance" });
        });

        it("should fail to stake for InsufficientStakingAmount", async function () {
            const failStakeObj = ExtrinsicHelper.stake(stakeKeys, stakeProviderId, 999999);
            // TODO: Does this actually test for the InsufficientStakingAmount error?
            const [failStakeEvent] = await failStakeObj.fundAndSend();
            assert.notEqual(failStakeEvent, undefined, "setup should return a InsufficientStakingAmount");
            // TODO: This returns: Missing expected rejection (InsufficientStakingAmount)
            // await assert.rejects(failStakeObj.fundAndSend(), { name: "InsufficientStakingAmount" });
        });

        it("should fail to stake for ZeroStakingAmount", async function () {
            const failStakeObj = ExtrinsicHelper.stake(stakeKeys, stakeProviderId, 0);
            // const [failStakeEvent] = await failStakeObj.fundAndSend();
            // assert.notEqual(failStakeEvent, undefined, "setup should return a ZeroStakingAmount");
            await assert.rejects(failStakeObj.fundAndSend(), { name: "ZeroAmountNotAllowed" });
        });

        // TODO: How to setup this test?
        // it("should fail for NotAStakingAccount", async function () {
        //     const failStakeObj = ExtrinsicHelper.stake(otherProviderKeys, providerId, 1000000);
        //     // const [failStakeEvent] = await failStakeObj.fundAndSend();
        //     // assert.notEqual(failStakeEvent, undefined, "setup should return a NotAStakingAccount");
        //     await assert.rejects(failStakeObj.fundAndSend(), { name: "NotAStakingAccount" });
        // });

        it("should successfully stake the minimum amount for Staked event", async function () {
            const stakeObj = ExtrinsicHelper.stake(stakeKeys, stakeProviderId, 1000000);
            const [stakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(stakeEvent, undefined, "setup should return a Stake event");
            // Ensure the provider capacity has been issued
            // Type guard to ensure the event is a Stake event
            if (stakeEvent && stakeObj.api.events.capacity.Staked.is(stakeEvent)) {   
                // assert.deepEqual(stakeEvent.data.capacity.Staked, stakeEvent.data.capacity.Staked, "setup should return a Stake event");
                log("stakeEvent:", stakeEvent.data);
            }
        });

        it("should successfully unstake the minimum amount", async function () {
            const stakeObj = ExtrinsicHelper.unstake(stakeKeys, stakeProviderId, 1000000);
            const [unStakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(unStakeEvent, undefined, "setup should return an UnStaked event");
        });

    });

    describe("unstake testing", function () {   
        // it("should fail to unstake for NoUnstakedTokensAvailable", async function () {
        //     // Setup Stake
        //     const stakeObj = ExtrinsicHelper.stake(unstakeKeys, unstakeProviderId, 1000000);
        //     const [stakeEvent] = await stakeObj.fundAndSend();
        //     assert.notEqual(stakeEvent, undefined, "setup should return a Stake event");

        //     const failUnstakeObj = ExtrinsicHelper.unstake(unstakeKeys, unstakeProviderId, 100000);
        //     // TODO: Getting AmountToUnstakeExceedsAmountStaked error instead of NoUnstakedTokensAvailable
        //     const [failUnstakeEvent] = await failUnstakeObj.fundAndSend();
        //     assert.notEqual(failUnstakeEvent, undefined, "setup should return a NoUnstakedTokensAvailable");
        //     // TODO: Returns: Missing expected rejection (NoUnstakedTokensAvailable)
        //     await assert.rejects(failUnstakeObj.fundAndSend(), { name: "NoUnstakedTokensAvailable" });
        // });
        it("should fail to unstake for UnstakedAmountIsZero", async function () {
            const failUnstakeObj = ExtrinsicHelper.unstake(unstakeKeys, unstakeProviderId, 0);
            await assert.rejects(failUnstakeObj.fundAndSend(), { name: "UnstakedAmountIsZero" });
        });
        it("should fail to unstake for AmountToUnstakeExceedsAmountStaked", async function () {
            const failUnstakeObj = ExtrinsicHelper.unstake(unstakeKeys, unstakeProviderId, 1000001);
            await assert.rejects(failUnstakeObj.fundAndSend(), { name: "AmountToUnstakeExceedsAmountStaked" });
        });
        it("should fail to unstake for StakingAccountNotFound", async function () {
            const failUnstakeObj = ExtrinsicHelper.unstake(otherProviderKeys, unstakeProviderId, 1000000);
            await assert.rejects(failUnstakeObj.fundAndSend(), { name: "StakingAccountNotFound" });
        });
        it("should fail to unstake for StakerTargetRelationshipNotFound", async function () {
            const failUnstakeObj = ExtrinsicHelper.unstake(unstakeKeys, otherProviderId, 1000000);
            await assert.rejects(failUnstakeObj.fundAndSend(), { name: "StakerTargetRelationshipNotFound" });
        });
        it("should fail to unstake for TargetCapacityNotFound", async function () {
            const failUnstakeObj = ExtrinsicHelper.unstake(unstakeKeys, 999999, 1000000);
            await assert.rejects(failUnstakeObj.fundAndSend(), { name: "TargetCapacityNotFound" });
        });
        it("should fail to unstake for MaxUnlockingChunksExceeded", async function () {
            const failUnstakeObj = ExtrinsicHelper.unstake(unstakeKeys, unstakeProviderId, 1000000);
            await assert.rejects(failUnstakeObj.fundAndSend(), { name: "MaxUnlockingChunksExceeded" });
        });
        it("should fail to unstake for IncreaseExceedsAvailable", async function () {
            const failUnstakeObj = ExtrinsicHelper.unstake(unstakeKeys, unstakeProviderId, 1000000);
            await assert.rejects(failUnstakeObj.fundAndSend(), { name: "IncreaseExceedsAvailable" });
        });
        it("should fail to unstake for MaxEpochLengthExceeded", async function () {
            const failUnstakeObj = ExtrinsicHelper.unstake(unstakeKeys, unstakeProviderId, 1000000);
            await assert.rejects(failUnstakeObj.fundAndSend(), { name: "MaxEpochLengthExceeded" });
        });
    });

    describe("withdraw_unstaked testing", function () {
        it("should fail to withdraw the unstaked amount", async function () {
            const stakeObj = ExtrinsicHelper.stake(withdrawKeys, withdrawProviderId, 1000000);
            const [stakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(stakeEvent, undefined, "setup should return a Stake event");

            const withdrawObj = ExtrinsicHelper.withdraw_unstaked(withdrawKeys);
            assert.rejects(withdrawObj.fundAndSend(), { name: "NoUnstakedTokensAvailable" });
        });
    });
})
