import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { u64 } from "@polkadot/types";
import assert from "assert";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { devAccounts, log, createKeys, createMsaAndProvider } from "../scaffolding/helpers";
import { firstValueFrom} from "rxjs";

describe("Capacity Transaction Tests", function () {
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
        const setEpochLengthOp = ExtrinsicHelper.setEpochLength(devAccounts[0].keys, TEST_EPOCH_LENGTH);
        const [setEpochLengthEvent] = await setEpochLengthOp.sudoSignAndSend();
        if (setEpochLengthEvent &&
            ExtrinsicHelper.api.events.capacity.EpochLengthUpdated.is(setEpochLengthEvent)) {
            const epochLength = setEpochLengthEvent.data.blocks;
            assert.equal(epochLength.toNumber(), TEST_EPOCH_LENGTH, "should set epoch length to TEST_EPOCH_LENGTH blocks");
        }
        else {
            assert.fail("should return an EpochLengthUpdated event");
        }

        // Create and fund a keypair with stakeAmount
        // Use this keypair for stake operations
        stakeKeys = createKeys("StakeKeys");
        stakeProviderId = await createMsaAndProvider(stakeKeys, "StakeProvider", stakeAmount);
        assert.equal(stakeProviderId, 1, "should populate stakeProviderId");

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for unstake operations
        unstakeKeys = createKeys("UnstakeKeys");
        unstakeProviderId = await createMsaAndProvider(unstakeKeys, "UnstakeProvider");
        assert.equal(unstakeProviderId, 2, "should populate unstakeProviderId");

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for withdraw operations
        withdrawKeys = createKeys("WithdrawKeys");
        withdrawProviderId = await createMsaAndProvider(withdrawKeys, "WithdrawProvider");
        assert.equal(withdrawProviderId, 3, "should populate withdrawProviderId");

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for other operations
        otherProviderKeys = createKeys("OtherProviderKeys");
        otherProviderId = await createMsaAndProvider(otherProviderKeys, "OtherProvider");
        assert.equal(otherProviderId, 4, "should populate otherProviderId");
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
            const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
            assert.equal(stakedAcctInfo.data.miscFrozen, 1000000, "should return an account with 1000000 miscFrozen balance");
            assert.equal(stakedAcctInfo.data.feeFrozen,  1000000, "should return an account with 1000000 feeFrozen balance");

            // Confirm that the capacity was added to the stakeProviderId using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            assert.equal(capacityStaked.remainingCapacity,   1000000, "should return a capacityLedger with 1000000 remainingCapacity");
            assert.equal(capacityStaked.totalTokensStaked,   1000000, "should return a capacityLedger with 1000000 total tokens staked");
            assert.equal(capacityStaked.totalCapacityIssued, 1000000, "should return a capacityLedger with 1000000 issued capacity");
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
            assert.equal(capacityStaked.remainingCapacity,   1000000, "should return a capacityLedger with 1000000 remainingCapacity");
            assert.equal(capacityStaked.totalTokensStaked,   0,       "should return a capacityLedger with 0 total tokens staked");
            assert.equal(capacityStaked.totalCapacityIssued, 0,       "should return a capacityLedger with 0 capacity issued");
        });

        it("should successfully withdraw the unstaked amount", async function () {
            // Mine enough blocks to pass the unstake period
            for (let index = 0; index < 2*TEST_EPOCH_LENGTH; index++) {
                await ExtrinsicHelper.createBlock();
            }

            const withdrawObj = ExtrinsicHelper.withdrawUnstaked(stakeKeys);
            const [withdrawEvent] = await withdrawObj.fundAndSend();
            assert.notEqual(withdrawEvent, undefined, "should return a StakeWithdrawn event");

            if (withdrawEvent && ExtrinsicHelper.api.events.capacity.StakeWithdrawn.is(withdrawEvent)) {
                let amount = withdrawEvent.data.amount;
                assert.equal(amount, 1000000, "should return a StakeWithdrawn event with 1000000 amount");
            }
            // Confirm that the tokens were unstaked in the stakeKeys account using the query API
            const unStakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
            assert.equal(unStakedAcctInfo.data.miscFrozen, 0, "should return an account with 0 miscFrozen balance");
            assert.equal(unStakedAcctInfo.data.feeFrozen,  0, "should return an account with 0 feeFrozen balance");

            // Confirm that the staked capacity was removed from the stakeProviderId account using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            assert.equal(capacityStaked.remainingCapacity,   1000000, "should return a capacityLedger with 1000000 remainingCapacity");
            assert.equal(capacityStaked.totalTokensStaked,   0,       "should return a capacityLedger with 0 total tokens staked");
            assert.equal(capacityStaked.totalCapacityIssued, 0,       "should return a capacityLedger with 0 capacity issued");
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
                let stakedAmount = increaseStakeEvent.data.amount;
                let stakedCapacity = increaseStakeEvent.data.capacity;
                assert.equal(stakedAmount,   2000000, "should return a Staked event with 2000000 Amount");
                assert.equal(stakedCapacity, 2000000, "should return a Staked event with 2000000 capacity");
            }

            // Confirm that the tokens were staked in the stakeKeys account using the query API
            const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
            assert.equal(stakedAcctInfo.data.miscFrozen, 3000000, "should return an account with 3000000 miscFrozen balance");
            assert.equal(stakedAcctInfo.data.feeFrozen,  3000000, "should return an account with 3000000 feeFrozen balance");

            // Confirm that the staked capacity was added to the stakeProviderId account using the query API
            // Capacity calculation:
            // 1000000 (initial test case) + 1000000 + 2000000 (this test case) = 4000000 capacity
            // - 1000000 (unstaked) = 3000000 staked tokens
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            assert.equal(capacityStaked.remainingCapacity,   4000000, "should return a capacityLedger with 4000000 remainingCapacity");
            assert.equal(capacityStaked.totalTokensStaked,   3000000, "should return a capacityLedger with 3000000 total staked");
            assert.equal(capacityStaked.totalCapacityIssued, 3000000, "should return a capacityLedger with 3000000 capacity issued");
        });

        it("should successfully increase stake to a different provider", async function () {
            // Increase stake by 1000000 capacity to a different provider
            const increaseStakeObj = ExtrinsicHelper.stake(stakeKeys, otherProviderId, 1000000);
            const [increaseStakeEvent] = await increaseStakeObj.fundAndSend();
            assert.notEqual(increaseStakeEvent, undefined, "should return a Staked event");

            if (increaseStakeEvent && ExtrinsicHelper.api.events.capacity.Staked.is(increaseStakeEvent)) {
                let increaseStakedAmount = increaseStakeEvent.data.amount;
                let increaseStakedCapacity = increaseStakeEvent.data.capacity;
                assert.equal(increaseStakedAmount,   1000000, "should return a (increased) Staked event with 1000000 Amount");
                assert.equal(increaseStakedCapacity, 1000000, "should return a (increased) Staked event with 1000000 capacity");
            }
            // Confirm that the staked capacity of the original stakeProviderId account is unchanged
            // Capacity calculation:
            // 1000000 (initial test case) + 1000000 + 2000000 (2nd test case) = 4000000 capacity
            // - 1000000 (unstaked) = 3000000 staked tokens
            const origStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            assert.equal(origStaked.remainingCapacity,   4000000, "should return a capacityLedger with 4000000 remainingCapacity");
            assert.equal(origStaked.totalTokensStaked,   3000000, "should return a capacityLedger with 3000000 total tokens staked");
            assert.equal(origStaked.totalCapacityIssued, 3000000, "should return a capacityLedger with 3000000 capacity issued");
            // Confirm that the staked capacity was added to the otherProviderId account using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(otherProviderId))).unwrap();
            assert.equal(capacityStaked.remainingCapacity,   1000000, "should return a capacityLedger with 1000000 remainingCapacity");
            assert.equal(capacityStaked.totalTokensStaked,   1000000, "should return a capacityLedger with 1000000 total tokens staked");
            assert.equal(capacityStaked.totalCapacityIssued, 1000000, "should return a capacityLedger with 1000000 capacity issued");
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
