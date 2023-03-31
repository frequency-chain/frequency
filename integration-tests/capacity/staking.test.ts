import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { u64, } from "@polkadot/types";
import assert from "assert";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import {
    devAccounts, createKeys, createMsaAndProvider,
    stakeToProvider, fundKeypair,
    getNextEpochBlock, TEST_EPOCH_LENGTH, setEpochLength
}
    from "../scaffolding/helpers";
import { firstValueFrom } from "rxjs";

describe("Capacity Transaction Tests", function () {
    let cents = 1000000n
    let dollars = 100n * cents;
    let accountBalance: bigint = 200n * dollars;

    before(async function () {
        await setEpochLength(devAccounts[0].keys, TEST_EPOCH_LENGTH);
    });

    describe("scenario: user staking, unstaking, and withdraw-unstaked", function () {
        let stakeKeys: KeyringPair;
        let stakeProviderId: u64;
        before(async function () {
            stakeKeys = createKeys("StakeKeys");
            stakeProviderId = await createMsaAndProvider(stakeKeys, "StakeProvider", accountBalance);
        });

        it("successfully stakes the minimum amount", async function () {
            await assert.doesNotReject(stakeToProvider(stakeKeys, stakeProviderId, 1n * cents));

            // Confirm that the tokens were locked in the stakeKeys account using the query API
            const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
            assert.equal(stakedAcctInfo.data.miscFrozen, 1n * cents, "should return an account with 1M miscFrozen balance");
            assert.equal(stakedAcctInfo.data.feeFrozen, 1n * cents, "should return an account with 1M feeFrozen balance");

            // Confirm that the capacity was added to the stakeProviderId using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            assert.equal(capacityStaked.remainingCapacity, 1n * cents, "should return a capacityLedger with 1M remainingCapacity");
            assert.equal(capacityStaked.totalTokensStaked, 1n * cents, "should return a capacityLedger with 1M total tokens staked");
            assert.equal(capacityStaked.totalCapacityIssued, 1n * cents, "should return a capacityLedger with 1M issued capacity");
        });

        it("successfully unstakes the minimum amount", async function () {
            const stakeObj = ExtrinsicHelper.unstake(stakeKeys, stakeProviderId, 1n * cents);
            const [unStakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(unStakeEvent, undefined, "should return an UnStaked event");

            if (unStakeEvent && ExtrinsicHelper.api.events.capacity.UnStaked.is(unStakeEvent)) {
                let unstakedCapacity = unStakeEvent.data.capacity;
                assert.equal(unstakedCapacity, 1n * cents, "should return an UnStaked event with 1M reduced capacity");
            }
            else {
                assert.fail("should return an capacity.UnStaked.is(unStakeEvent) event");
            }
            // Confirm that the tokens were unstaked in the stakeProviderId account using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            assert.equal(capacityStaked.remainingCapacity, 0, "should return a capacityLedger with 0 remainingCapacity");
            assert.equal(capacityStaked.totalTokensStaked, 0, "should return a capacityLedger with 0 total tokens staked");
            assert.equal(capacityStaked.totalCapacityIssued, 0, "should return a capacityLedger with 0 capacity issued");
        });

        it("successfully withdraws the unstaked amount", async function () {
            // Mine enough blocks to pass the unstake period = CapacityUnstakingThawPeriod = 2 epochs
            let newEpochBlock = await getNextEpochBlock();
            await ExtrinsicHelper.run_to_block(newEpochBlock + TEST_EPOCH_LENGTH);

            const withdrawObj = ExtrinsicHelper.withdrawUnstaked(stakeKeys);
            const [withdrawEvent] = await withdrawObj.fundAndSend();
            assert.notEqual(withdrawEvent, undefined, "should return a StakeWithdrawn event");

            if (withdrawEvent && ExtrinsicHelper.api.events.capacity.StakeWithdrawn.is(withdrawEvent)) {
                let amount = withdrawEvent.data.amount;
                assert.equal(amount, 1n * cents, "should return a StakeWithdrawn event with 1M amount");
            }

            // Confirm that the tokens were unstaked in the stakeKeys account using the query API
            const unStakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
            assert.equal(unStakedAcctInfo.data.miscFrozen, 0, "should return an account with 0 miscFrozen balance");
            assert.equal(unStakedAcctInfo.data.feeFrozen, 0, "should return an account with 0 feeFrozen balance");

            // Confirm that the staked capacity was removed from the stakeProviderId account using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            assert.equal(capacityStaked.remainingCapacity, 0, "should return a capacityLedger with 0 remainingCapacity");
            assert.equal(capacityStaked.totalTokensStaked, 0, "should return a capacityLedger with 0 total tokens staked");
            assert.equal(capacityStaked.totalCapacityIssued, 0, "should return a capacityLedger with 0 capacity issued");
        });

        describe("when staking to multiple times", async function () {
            describe("and targeting same provider", async function () {
                it("successfully increases the amount that was targeted to provider", async function () {
                    // Now starting from zero again with stakeProviderId, Stake 1M capacity
                    await assert.doesNotReject(stakeToProvider(stakeKeys, stakeProviderId, 1n * cents));

                    // Confirm that the tokens were staked in the stakeKeys account using the query API
                    const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
                    assert.equal(stakedAcctInfo.data.miscFrozen, 1n * cents, "should return an account with 1M miscFrozen balance");
                    assert.equal(stakedAcctInfo.data.feeFrozen, 1n * cents, "should return an account with 1M feeFrozen balance");

                    const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
                    assert.equal(capacityStaked.remainingCapacity, 1n * cents, "should return a capacityLedger with 1M remainingCapacity");
                    assert.equal(capacityStaked.totalTokensStaked, 1n * cents, "should return a capacityLedger with 1M total staked");
                    assert.equal(capacityStaked.totalCapacityIssued, 1n * cents, "should return a capacityLedger with 1M capacity issued");
                });

                describe("and targeting different provider", async function () {
                    let otherProviderKeys: KeyringPair;
                    let otherProviderId: u64;

                    beforeEach(async function () {
                        otherProviderKeys = createKeys("OtherProviderKeys");
                        otherProviderId = await createMsaAndProvider(otherProviderKeys, "OtherProvider", accountBalance);
                    });

                    it("does not change other targets amounts", async function () {
                        // Increase stake by 1 cent to a different target.
                        await assert.doesNotReject(stakeToProvider(stakeKeys, otherProviderId, 1n * cents));


                        // Confirm that the staked capacity of the original stakeProviderId account is unchanged
                        // stakeProviderId should still have 1M from first test case in this describe.
                        // otherProvider should now have 1M
                        const origStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
                        assert.equal(origStaked.remainingCapacity, 1n * cents, "should return a capacityLedger with 1M remainingCapacity");
                        assert.equal(origStaked.totalTokensStaked, 1n * cents, "should return a capacityLedger with 1M total tokens staked");
                        assert.equal(origStaked.totalCapacityIssued, 1n * cents, "should return a capacityLedger with 1M capacity issued");

                        // Confirm that the staked capacity was added to the otherProviderId account using the query API
                        const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(otherProviderId))).unwrap();
                        assert.equal(capacityStaked.remainingCapacity, 1n * cents, "should return a capacityLedger with 1M remainingCapacity");
                        assert.equal(capacityStaked.totalTokensStaked, 1n * cents, "should return a capacityLedger with 1M total tokens staked");
                        assert.equal(capacityStaked.totalCapacityIssued, 1n * cents, "should return a capacityLedger with 1M capacity issued");
                    });
                });
            });
        });
    });

    describe("when staking and targeting an InvalidTarget", async function () {
        it("fails to stake", async function () {
            let stakeKeys = createKeys("StakeKeys");
            let stakeAmount = 100n * dollars;
            await fundKeypair(devAccounts[0].keys, stakeKeys, stakeAmount)

            const failStakeObj = ExtrinsicHelper.stake(stakeKeys, 99, stakeAmount);
            await assert.rejects(failStakeObj.fundAndSend(), { name: "InvalidTarget" });
        });
    });

    describe("when attempting to stake below the minimum staking requirements", async function () {
        it("should fail to stake for InsufficientStakingAmount", async function () {
            let stakingKeys = createKeys("stakingKeys");
            let providerId = await createMsaAndProvider(stakingKeys, "stakingKeys", accountBalance);
            let stakeAmount = cents / 2n;

            const failStakeObj = ExtrinsicHelper.stake(stakingKeys, providerId, stakeAmount);
            await assert.rejects(failStakeObj.fundAndSend(), { name: "InsufficientStakingAmount" });
        });
    });

    describe("when attempting to stake a zero amount", async function () {
        it("fails to stake and errors ZeroAmountNotAllowed", async function () {
            let stakingKeys = createKeys("stakingKeys");
            let providerId = await createMsaAndProvider(stakingKeys, "stakingKeys", accountBalance);

            const failStakeObj = ExtrinsicHelper.stake(stakingKeys, providerId, 0);
            await assert.rejects(failStakeObj.fundAndSend(), { name: "ZeroAmountNotAllowed" });
        });
    });

    describe("when staking an amount and account balanace is too low", async function () {
        it("fails to stake and errors BalanceTooLowtoStake", async function () {
            let accountBalance: bigint = 2n * cents;
            let stakingKeys = createKeys("stakingKeys");
            let providerId = await createMsaAndProvider(stakingKeys, "stakingKeys", accountBalance);
            let stakingAmount = 1n * dollars;

            const failStakeObj = ExtrinsicHelper.stake(stakingKeys, providerId, stakingAmount);
            await assert.rejects(failStakeObj.fundAndSend(), { name: "BalanceTooLowtoStake" });
        });
    });


    describe("unstake()", function () {
        let unstakeKeys: KeyringPair;
        let providerId: u64;
        let stakingAmount = 1n * dollars;

        before(async function () {
            let accountBalance: bigint = 2n * cents;
            unstakeKeys = createKeys("stakingKeys");
            providerId = await createMsaAndProvider(unstakeKeys, "stakingKeys", accountBalance);
        });

        describe("when attempting to unstake a Zero amount", async function () {
            it("errors with UnstakedAmountIsZero", async function () {
                const failUnstakeObj = ExtrinsicHelper.unstake(unstakeKeys, providerId, 0);
                await assert.rejects(failUnstakeObj.fundAndSend(), { name: "UnstakedAmountIsZero" });
            });
        })

        describe("when account has not staked", async function () {
            it("errors with StakingAccountNotFound", async function () {
                const failUnstakeObj = ExtrinsicHelper.unstake(unstakeKeys, providerId, 1000000);
                await assert.rejects(failUnstakeObj.fundAndSend(), { name: "StakingAccountNotFound" });
            });
        });
    });

    describe("#withdraw_unstaked", async function () {
        describe("when attempting to call #withdrawUnstake before first calling #unstake", async function () {
            it("errors with NoUnstakedTokensAvailable", async function () {
                let stakingAmount = 1n * cents;
                let stakingKeys: KeyringPair = createKeys("stakingKeys");
                let providerId: u64 = await createMsaAndProvider(stakingKeys, "stakingKeys", accountBalance);

                const stakeObj = ExtrinsicHelper.stake(stakingKeys, providerId, stakingAmount);
                const [stakeEvent] = await stakeObj.fundAndSend();
                assert.notEqual(stakeEvent, undefined, "should return a Stake event");

                const withdrawObj = ExtrinsicHelper.withdrawUnstaked(stakingKeys);
                await assert.rejects(withdrawObj.fundAndSend(), { name: "NoUnstakedTokensAvailable" });
            });
        })
    });
})
