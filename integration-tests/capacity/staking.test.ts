import "@frequency-chain/api-augment";
import { MessageSourceId } from "@frequency-chain/api-augment/interfaces";
import { KeyringPair } from "@polkadot/keyring/types";
import { u64, u128 } from "@polkadot/types";
import { AccountId32 } from "@polkadot/types/interfaces";
import assert from "assert";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { devAccounts, createAndFundKeypair, log } from "../scaffolding/helpers";
import { EXISTENTIAL_DEPOSIT } from "../scaffolding/rootHooks";
import { ApiPromise, WsProvider } from '@polkadot/api';
import { firstValueFrom} from "rxjs";

// REMOVE: .only This is only for debugging
describe.only("Capacity Scenario Tests", function () {
    const TEST_EPOCH_LENGTH = 25;
    let otherProviderKeys: KeyringPair;
    let otherProviderId: u64;
    let stakeKeys: KeyringPair;
    let stakeProviderId: u64;
    let unstakeKeys: KeyringPair;
    let unstakeProviderId: u64;
    let withdrawKeys: KeyringPair;
    let withdrawProviderId: u64;

    let stakeAmount: bigint = 6000000n;

    before(async function () {
        // Set the Maximum Epoch Length to TEST_EPOCH_LENGTH blocks
        // This will allow us to test the epoch transition logic
        // without having to wait for 100 blocks
        let setEpochLengthOp = ExtrinsicHelper.setEpochLength(devAccounts[0].keys, TEST_EPOCH_LENGTH);
        let [setEpochLengthEvent] = await setEpochLengthOp.sudoSignAndSend();
        if (setEpochLengthEvent && 
            ExtrinsicHelper.api.events.capacity.EpochLengthUpdated.is(setEpochLengthEvent)) {
            let epochLength = setEpochLengthEvent.data.blocks;
            assert.equal(epochLength.toNumber(), TEST_EPOCH_LENGTH, "should set epoch length to TEST_EPOCH_LENGTH blocks");
        }
        else {
            assert.fail("should return an EpochLengthUpdated event");
        }

        // Create and fund a keypair with stakeAmount
        // Use this keypair for stake operations
        stakeKeys = await createAndFundKeypair(stakeAmount);
        let createStakeProviderMsaOp = ExtrinsicHelper.createMsa(stakeKeys);
        await createStakeProviderMsaOp.fundAndSend();
        let createStakeProviderOp = ExtrinsicHelper.createProvider(stakeKeys, "TestProvider");
        let [stakeProviderEvent] = await createStakeProviderOp.fundAndSend();
        assert.notEqual(stakeProviderEvent, undefined, "should return a ProviderCreated event");
        if (stakeProviderEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(stakeProviderEvent)) {
            stakeProviderId = stakeProviderEvent.data.providerId;
        }
        assert.notEqual(stakeProviderId, undefined, "should populate stakeProviderId");

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for unstake operations
        unstakeKeys = await createAndFundKeypair();
        let createUnstakeProviderMsaOp = ExtrinsicHelper.createMsa(unstakeKeys);
        await createUnstakeProviderMsaOp.fundAndSend();
        let createUnstakeProviderOp = ExtrinsicHelper.createProvider(unstakeKeys, "TestProvider");
        let [unstakeProviderEvent] = await createUnstakeProviderOp.fundAndSend();
        assert.notEqual(unstakeProviderEvent, undefined, "should return a ProviderCreated event");
        if (unstakeProviderEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(unstakeProviderEvent)) {
            unstakeProviderId = unstakeProviderEvent.data.providerId;
        }
        assert.notEqual(unstakeProviderId, undefined, "should populate unstakeProviderId");

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for withdraw operations
        withdrawKeys = await createAndFundKeypair();
        let createWithdrawProviderMsaOp = ExtrinsicHelper.createMsa(withdrawKeys);
        await createWithdrawProviderMsaOp.fundAndSend();
        let createWithdrawProviderOp = ExtrinsicHelper.createProvider(withdrawKeys, "TestProvider");
        let [withdrawProviderEvent] = await createWithdrawProviderOp.fundAndSend();
        assert.notEqual(withdrawProviderEvent, undefined, "should return a ProviderCreated event");
        if (withdrawProviderEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(withdrawProviderEvent)) {
            withdrawProviderId = withdrawProviderEvent.data.providerId;
        }
        assert.notEqual(withdrawProviderId, undefined, "should populate withdrawProviderId");

        otherProviderKeys = await createAndFundKeypair();
        let createProviderMsaOp = ExtrinsicHelper.createMsa(otherProviderKeys);
        await createProviderMsaOp.fundAndSend();
        let createProviderOp = ExtrinsicHelper.createProvider(otherProviderKeys, "TestProvider");
        let [providerEvent] = await createProviderOp.fundAndSend();
        assert.notEqual(providerEvent, undefined, "should return a ProviderCreated event");
        if (providerEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(providerEvent)) {
            otherProviderId = providerEvent.data.providerId;
        }
        assert.notEqual(otherProviderId, undefined, "should populate providerId");
    });

    describe("stake-unstake-withdraw_unstaked testing", function () {

        it("should successfully stake the minimum amount for Staked event", async function () {
            const stakeObj = ExtrinsicHelper.stake(stakeKeys, stakeProviderId, 1000000);
            const [stakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(stakeEvent, undefined, "should return a Stake event");

            if (stakeEvent && ExtrinsicHelper.api.events.capacity.Staked.is(stakeEvent)) {   
                let stakedCapacity = stakeEvent.data.capacity;
                assert.equal(stakedCapacity, 1000000, "should return a Stake event with 1000000 capacity");
            }
            else {
                assert.fail("should return a capacity.Staked.is(stakeEvent) event");
            }

            // Confirm that the tokens were locked in the stakeKeys account using the query API
            // REMOVE: 
            // log("DBG: 1000000 staked");
            // log("DBG:stakedAcct: " + stakeKeys.address);
            const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
            // REMOVE: 
            // log("DBG:stakedAcctInfo: " + stakedAcctInfo);
            assert.equal(stakedAcctInfo.data.miscFrozen, 1000000, "should return a StakeWithdrawn event with 0 miscFrozen balance");
            assert.equal(stakedAcctInfo.data.feeFrozen,  1000000, "should return a StakeWithdrawn event with 0 feeFrozen balance");

            // Confirm that the capacity was added to the stake provider using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            // REMOVE:
            // log("DBG:capacityStaked: " + capacityStaked + "\n\n");
            // log("DBG");
            assert.equal(capacityStaked.remaining,         1000000, "should return a Stake event with 1000000 remaining capacity");
            assert.equal(capacityStaked.totalTokensStaked, 1000000, "should return a Stake event with 1000000 total tokens capacity");
            assert.equal(capacityStaked.totalAvailable,    1000000, "should return a Stake event with 1000000 available capacity");
        });

        it("should successfully unstake the minimum amount", async function () {
            const stakeObj = ExtrinsicHelper.unstake(stakeKeys, stakeProviderId, 1000000);
            const [unStakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(unStakeEvent, undefined, "should return an UnStaked event");

            if (unStakeEvent && ExtrinsicHelper.api.events.capacity.UnStaked.is(unStakeEvent)) {
                let unstakedCapacity = unStakeEvent.data.capacity;
                assert.equal(unstakedCapacity, 1000000, "should return an UnStaked event with 1000000 reduced capacity");
            }
            else {
                assert.fail("should return an capacity.UnStaked.is(unStakeEvent) event");
            }
            // Confirm that the tokens were unstaked in the stakeProviderId account using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            // REMOVE:
            // log("DBG: 1000000 unstaked");
            // log("DBG:stakedAcct: " + stakeKeys.address);
            // const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
            // REMOVE: 
            // log("DBG:stakedAcctInfo: " + stakedAcctInfo);
            // log("DBG:capacityStaked: " + capacityStaked);
            // log("DBG");
            assert.equal(capacityStaked.remaining,         1000000, "should return a Stake event with 1000000 remaining capacity");
            assert.equal(capacityStaked.totalTokensStaked, 0,       "should return a Stake event with 0 total tokens capacity");
            assert.equal(capacityStaked.totalAvailable,    0,       "should return a Stake event with 1000000 available capacity");
        });

        it("should successfully withdraw the unstaked amount", async function () {
            // Mine enough blocks to pass the unstake period
            for (let index = 0; index < 2*TEST_EPOCH_LENGTH; index++) {
                await ExtrinsicHelper.createBlock();
            }
            // let capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            // REMOVE:
            // log("DBG:capacityStaked: " + capacityStaked);

            const withdrawObj = ExtrinsicHelper.withdrawUnstaked(stakeKeys);
            const [withdrawEvent] = await withdrawObj.fundAndSend();
            assert.notEqual(withdrawEvent, undefined, "should return a StakeWithdrawn event");

            // REMOVE: 
            // log("DBG: 1000000 withdrawn");
            // log("DBG:stakedAcct: " + stakeKeys.address);
            // const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
            // REMOVE: 
            // log("DBG:stakedAcctInfo: " + stakedAcctInfo);

            if (withdrawEvent && ExtrinsicHelper.api.events.capacity.StakeWithdrawn.is(withdrawEvent)) {
                let amount = withdrawEvent.data.amount;
                assert.equal(amount, 1000000, "should return a StakeWithdrawn event with 1000000 amount");
            }
            // Confirm that the tokens were unstaked in the stakeProviderId account using the query API
            const unStakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
            //REMOVE:
            // log("DBG:unStakedAcctInfo: " + unStakedAcctInfo);
            assert.equal(unStakedAcctInfo.data.miscFrozen, 0, "should return a StakeWithdrawn event with 0 miscFrozen balance");
            assert.equal(unStakedAcctInfo.data.feeFrozen,  0, "should return a StakeWithdrawn event with 0 feeFrozen balance");

            // Confirm that the staked capacity was removed from the stakeProviderId account using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            // REMOVE:
            // log("DBG:capacityStaked: " + capacityStaked);
            // log("DBG");
            assert.equal(capacityStaked.remaining,         1000000, "should return a Stake event with 1000000 remaining capacity");
            assert.equal(capacityStaked.totalTokensStaked, 0,       "should return a Stake event with 0 total tokens capacity");
            assert.equal(capacityStaked.totalAvailable,    0,       "should return a Stake event with 1000000 available capacity");
        });
    });

    describe("increase stake testing", function () {
        it("should successfully increase stake", async function () {
            // Stake 1000000 capacity
            const stakeObj = ExtrinsicHelper.stake(stakeKeys, stakeProviderId, 1000000);
            const [stakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(stakeEvent, undefined, "should return a Staked event");

            if (stakeEvent && ExtrinsicHelper.api.events.capacity.Staked.is(stakeEvent)) {
                let stakedCapacity = stakeEvent.data.capacity;
                assert.equal(stakedCapacity, 1000000, "should return a Staked event with 1000000 capacity");
            }
            
            // Increase stake by 2000000 capacity
            const increaseStakeObj = ExtrinsicHelper.stake(stakeKeys, stakeProviderId, 2000000);
            const [increaseStakeEvent] = await increaseStakeObj.fundAndSend();
            assert.notEqual(increaseStakeEvent, undefined, "should return a Staked event");

            if (increaseStakeEvent && ExtrinsicHelper.api.events.capacity.Staked.is(increaseStakeEvent)) {
                // let stakedAccount = increaseStakeEvent.data.account;
                // let stakedTarget = increaseStakeEvent.data.target;
                let stakedAmount = increaseStakeEvent.data.amount;
                let stakedCapacity = increaseStakeEvent.data.capacity;
                // // REMOVE: 
                // log("stakedAccount: " + stakedAccount);
                // log("stakedTarget: " + stakedTarget);
                // log("DBG:stakedAmount: " + stakedAmount);
                // log("DBG:stakedCapacity: " + stakedCapacity);
                assert.equal(stakedAmount,   2000000, "should return a Staked event with 2000000 Amount");
                assert.equal(stakedCapacity, 2000000, "should return a Staked event with 2000000 capacity");
            }

            // Confirm that the tokens were staked in the stakeProviderId account using the query API
            const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
            // REMOVE:
            // log("DBG:stakedAcctInfo: " + stakedAcctInfo);
            assert.equal(stakedAcctInfo.data.miscFrozen, 3000000, "should return a Staked event with 3000000 miscFrozen balance");
            assert.equal(stakedAcctInfo.data.feeFrozen,  3000000, "should return a Staked event with 3000000 feeFrozen balance");

            // Confirm that the staked capacity was added to the stakeProviderId account using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            // REMOVE:
            // log("DBG:capacityStaked: " + capacityStaked);
            // log("DBG");
            // Capacity calculation:
            // 1000000 (initial test case) + 1000000 + 2000000 (this test case) = 4000000
            assert.equal(capacityStaked.remaining,         4000000, "should return a Staked event with 4000000 remaining capacity");
            assert.equal(capacityStaked.totalTokensStaked, 3000000, "should return a Staked event with 3000000 total tokens capacity");
            assert.equal(capacityStaked.totalAvailable,    3000000, "should return a Staked event with 3000000 available capacity");
        });

        it("should successfully increase stake to a different provider", async function () {
            // Compare these values with the next transaction for a different provider
            let stakedAccount: AccountId32;
            let stakedTarget: u64;
            let stakedCapacity: u128;

            // Stake 1000000 capacity
            const stakeObj = ExtrinsicHelper.stake(stakeKeys, stakeProviderId, 1000000);
            const [stakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(stakeEvent, undefined, "should return a Staked event");

            if (stakeEvent && ExtrinsicHelper.api.events.capacity.Staked.is(stakeEvent)) {
                stakedAccount = stakeEvent.data.account;
                stakedTarget = stakeEvent.data.target;
                stakedCapacity = stakeEvent.data.capacity;
                assert.equal(stakedCapacity, 1000000, "should return a Staked event with 1000000 capacity");
            }
            else {
                assert.fail("should return a capacity.Staked.is(stakeEvent) event");
            }

            // Increase stake by 1000000 capacity
            const increaseStakeObj = ExtrinsicHelper.stake(stakeKeys, otherProviderId, 1000000);
            const [increaseStakeEvent] = await increaseStakeObj.fundAndSend();
            assert.notEqual(increaseStakeEvent, undefined, "should return a Staked event");

            if (increaseStakeEvent && ExtrinsicHelper.api.events.capacity.Staked.is(increaseStakeEvent)) {
                let increaseStakedAccount = increaseStakeEvent.data.account;
                let increaseStakedTarget = increaseStakeEvent.data.target;
                let increaseStakedAmount = increaseStakeEvent.data.amount;
                let increaseStakedCapacity = increaseStakeEvent.data.capacity;
                assert.equal(increaseStakedAmount,   1000000, "should return a (increased) Staked event with 1000000 Amount");
                assert.equal(increaseStakedCapacity, 1000000, "should return a (increased) Staked event with 1000000 capacity");
                // Confirm that the provider for the increased stake is different from the original stake: same account
                assert.deepEqual(stakedAccount, increaseStakedAccount, "should have increased stake for the same account");
                assert.notEqual( stakedTarget,  increaseStakedTarget, "should have increased stake for a different provider");
            }
            // Confirm that the staked capacity was added to the otherProviderId account using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(otherProviderId))).unwrap();
            assert.equal(capacityStaked.remaining,         1000000, "should return a Staked event with 1000000 remaining capacity");
            assert.equal(capacityStaked.totalTokensStaked, 1000000, "should return a Staked event with 1000000 total tokens capacity");
            assert.equal(capacityStaked.totalAvailable,    1000000, "should return a Staked event with 1000000 available capacity");
        });
    });

    describe("stake testing-invalid paths", function () {
        it("should fail to stake for InvalidTarget", async function () {
            const failStakeObj = ExtrinsicHelper.stake(stakeKeys, 99, 1000000);
            await assert.rejects(failStakeObj.fundAndSend(), { name: "InvalidTarget" });
        });

        // Need to use a different account that is not already staked
        it("should fail to stake for InsufficientStakingAmount", async function () {
            const failStakeObj = ExtrinsicHelper.stake(otherProviderKeys, otherProviderId, 1000);
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
            assert.notEqual(stakeEvent, undefined, "should return a Stake event");

            const withdrawObj = ExtrinsicHelper.withdrawUnstaked(withdrawKeys);
            assert.rejects(withdrawObj.fundAndSend(), { name: "NoUnstakedTokensAvailable" });
        });
    });
})
