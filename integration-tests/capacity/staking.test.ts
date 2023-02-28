import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { u64 } from "@polkadot/types";
import assert from "assert";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { devAccounts, createAndFundKeypair, log } from "../scaffolding/helpers";

describe("Capacity Scenario Tests", function () {
    const TEST_EPOCH_LENGTH = 25;
    let otherProviderKeys: KeyringPair;
    let otherProviderId: u64;
    let stakeKeys: KeyringPair;
    let stakeProviderId: u64;
    let unstakeKeys: KeyringPair;
    let unstakeProviderId: u64;
    let withdrawKeys: KeyringPair;
    let withdrawProviderId: u64;

    before(async function () {
        // Set the Maximum Epoch Length to TEST_EPOCH_LENGTH blocks
        // This will allow us to test the epoch transition logic
        // without having to wait for 100 blocks
        let setEpochLengthOp = ExtrinsicHelper.setEpochLength(devAccounts[0].keys, TEST_EPOCH_LENGTH);
        let [setEpochLengthEvent] = await setEpochLengthOp.sudoSignAndSend();
        if (setEpochLengthEvent && 
            ExtrinsicHelper.api.events.capacity.EpochLengthUpdated.is(setEpochLengthEvent)) {
            let epochLength = setEpochLengthEvent.data.blocks;
            assert.equal(epochLength.toNumber(), TEST_EPOCH_LENGTH, "setup should set epoch length to TEST_EPOCH_LENGTH blocks");
        }
        else {
            assert.fail("setup should return an EpochLengthUpdated event");
        }

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
    });

    describe("stake-unstake-withdraw_unstaked testing-->happy path", function () {

        it("should successfully stake the minimum amount for Staked event", async function () {
            const stakeObj = ExtrinsicHelper.stake(stakeKeys, stakeProviderId, 1000000);
            const [stakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(stakeEvent, undefined, "setup should return a Stake event");

            if (stakeEvent && ExtrinsicHelper.api.events.capacity.Staked.is(stakeEvent)) {   
                let stakedCapacity = stakeEvent.data.capacity;
                assert.equal(stakedCapacity, 1000000, "setup should return a Stake event with 1000000 capacity");
            }
            else {
                assert.fail("setup should return a Stake event");
            }
        });

        it("should successfully unstake the minimum amount", async function () {
            const stakeObj = ExtrinsicHelper.unstake(stakeKeys, stakeProviderId, 1000000);
            const [unStakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(unStakeEvent, undefined, "setup should return an UnStaked event");

            if (unStakeEvent && ExtrinsicHelper.api.events.capacity.UnStaked.is(unStakeEvent)) {
                let unstakedCapacity = unStakeEvent.data.capacity;
                assert.equal(unstakedCapacity, 1000000, "setup should return an UnStaked event with 1000000 reduced capacity");
            }
            else {
                assert.fail("setup should return an UnStaked event");
            }

            // Mine enough blocks to pass the unstake period
            for (let index = 0; index < 2*TEST_EPOCH_LENGTH; index++) {
                await ExtrinsicHelper.createBlock();
            }
        });
        it("should successfully withdraw the unstaked amount", async function () {
            const withdrawObj = ExtrinsicHelper.withdrawUnstaked(stakeKeys);
            const [withdrawEvent] = await withdrawObj.fundAndSend();
            assert.notEqual(withdrawEvent, undefined, "setup should return a StakeWithdrawn event");

            if (withdrawEvent && ExtrinsicHelper.api.events.capacity.StakeWithdrawn.is(withdrawEvent)) {
                let amount = withdrawEvent.data.amount;
                assert.equal(amount, 1000000, "setup should return a StakeWithdrawn event with 1000000 amount");
            }
            else {
                assert.fail("setup should return a StakeWithdrawn event");
            }
        });
    });

    describe("stake testing-invalid paths", function () {
        it("should fail to stake for InvalidTarget", async function () {
            const failStakeObj = ExtrinsicHelper.stake(stakeKeys, 99, 1000000);
            await assert.rejects(failStakeObj.fundAndSend(), { name: "InvalidTarget" });
        });

        it("should fail to stake for InsufficientStakingAmount", async function () {
            const failStakeObj = ExtrinsicHelper.stake(stakeKeys, stakeProviderId, 999999);
            await assert.rejects(failStakeObj.fundAndSend(), { name: "InsufficientStakingAmount" });
        });

        it("should fail to stake for ZeroAmountNotAllowed", async function () {
            const failStakeObj = ExtrinsicHelper.stake(stakeKeys, stakeProviderId, 0);
            await assert.rejects(failStakeObj.fundAndSend(), { name: "ZeroAmountNotAllowed" });
        });
    });

    describe("unstake testing", function () {   
        it("should fail to unstake for UnstakedAmountIsZero", async function () {
            const failUnstakeObj = ExtrinsicHelper.unstake(unstakeKeys, unstakeProviderId, 0);
            await assert.rejects(failUnstakeObj.fundAndSend(), { name: "UnstakedAmountIsZero" });
        });
        it("should fail to unstake for StakingAccountNotFound", async function () {
            const failUnstakeObj = ExtrinsicHelper.unstake(otherProviderKeys, unstakeProviderId, 1000000);
            await assert.rejects(failUnstakeObj.fundAndSend(), { name: "StakingAccountNotFound" });
        });
    });

    describe("withdraw_unstaked testing", function () {
        it("should fail to withdraw the unstaked amount", async function () {
            const stakeObj = ExtrinsicHelper.stake(withdrawKeys, withdrawProviderId, 1000000);
            const [stakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(stakeEvent, undefined, "setup should return a Stake event");

            const withdrawObj = ExtrinsicHelper.withdrawUnstaked(withdrawKeys);
            assert.rejects(withdrawObj.fundAndSend(), { name: "NoUnstakedTokensAvailable" });
        });
    });
})
