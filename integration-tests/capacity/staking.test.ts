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

describe("Capacity Staking Tests", function () {
    const CENTS = 1000000n;
    const DOLLARS = 100n * CENTS;
    const accountBalance: bigint = 200n * DOLLARS;
    const tokenMinStake: bigint = 1n * CENTS;
    let capacityMin: bigint = tokenMinStake / 50n;

    before(async function () {
        // this isn't working now?
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

            await assert.doesNotReject(stakeToProvider(stakeKeys, stakeProviderId, tokenMinStake));

            // Confirm that the tokens were locked in the stakeKeys account using the query API
            const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
            assert.equal(stakedAcctInfo.data.miscFrozen, tokenMinStake, `expected 50 CENTs miscFrozen balance, got ${stakedAcctInfo.data.miscFrozen}`);
            assert.equal(stakedAcctInfo.data.feeFrozen, tokenMinStake, `expected 50 CENTs feeFrozen balance, got ${stakedAcctInfo.data.feeFrozen}`);

            // Confirm that the capacity was added to the stakeProviderId using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            assert.equal(capacityStaked.remainingCapacity, capacityMin, `expected capacityLedger.remainingCapacity = 1CENT, got ${capacityStaked.remainingCapacity}`);
            assert.equal(capacityStaked.totalTokensStaked, tokenMinStake, `expected capacityLedger.totalTokensStaked = 1CENT, got ${capacityStaked.totalTokensStaked}`);
            assert.equal(capacityStaked.totalCapacityIssued, capacityMin, `expected capacityLedger.totalCapacityIssued = 1CENT, got ${capacityStaked.totalCapacityIssued}`);
        });

        it("successfully unstakes the minimum amount", async function () {
            const stakeObj = ExtrinsicHelper.unstake(stakeKeys, stakeProviderId, tokenMinStake);
            const [unStakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(unStakeEvent, undefined, "should return an UnStaked event");

            if (unStakeEvent && ExtrinsicHelper.api.events.capacity.UnStaked.is(unStakeEvent)) {
                let unstakedCapacity = unStakeEvent.data.capacity;
                assert.equal(unstakedCapacity, capacityMin, "should return an UnStaked event with 1 CENT reduced capacity");
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
            await ExtrinsicHelper.run_to_block(newEpochBlock + TEST_EPOCH_LENGTH + 1);

            const withdrawObj = ExtrinsicHelper.withdrawUnstaked(stakeKeys);
            const [withdrawEvent] = await withdrawObj.fundAndSend();
            assert.notEqual(withdrawEvent, undefined, "should return a StakeWithdrawn event");

            if (withdrawEvent && ExtrinsicHelper.api.events.capacity.StakeWithdrawn.is(withdrawEvent)) {
                let amount = withdrawEvent.data.amount;
                assert.equal(amount, 1n * CENTS, "should return a StakeWithdrawn event with 1M amount");
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
                  // get the current account info
                   let oldStakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);

                    await assert.doesNotReject(stakeToProvider(stakeKeys, stakeProviderId, 1n*CENTS));

                    const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
                    assert.equal(capacityStaked.remainingCapacity, capacityMin, "should return a capacityLedger with  remainingCapacity");
                    assert.equal(capacityStaked.totalTokensStaked, tokenMinStake, "should return a capacityLedger with 50M total staked");
                    assert.equal(capacityStaked.totalCapacityIssued, capacityMin, "should return a capacityLedger with 1M capacity issued");

                    // Confirm that the tokens were staked in the stakeKeys account using the query API
                    const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);

                    let increasedMiscFrozen: bigint = stakedAcctInfo.data.miscFrozen.toBigInt() - oldStakedAcctInfo.data.miscFrozen.toBigInt();
                    let increasedFeeFrozen: bigint = stakedAcctInfo.data.feeFrozen.toBigInt() - oldStakedAcctInfo.data.feeFrozen.toBigInt();

                    assert.equal(increasedMiscFrozen, tokenMinStake, `expected miscFrozen=${tokenMinStake}, got ${increasedMiscFrozen}`);
                    assert.equal(increasedFeeFrozen, tokenMinStake, `expected feeFrozen=${tokenMinStake}, got ${increasedFeeFrozen}`);

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
                        await assert.doesNotReject(stakeToProvider(stakeKeys, otherProviderId, 1n * CENTS));
                        const expectedCapacity = CENTS/50n;


                        // Confirm that the staked capacity of the original stakeProviderId account is unchanged
                        // stakeProviderId should still have 1M from first test case in this describe.
                        // otherProvider should now have 1M
                        const origStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
                        assert.equal(origStaked.remainingCapacity, expectedCapacity, `expected 1/50 CENT remaining capacity, got ${origStaked.remainingCapacity}`);
                        assert.equal(origStaked.totalTokensStaked, 1n * CENTS, `expected 1 CENT staked, got ${origStaked.totalTokensStaked}`);
                        assert.equal(origStaked.totalCapacityIssued, expectedCapacity, `expected 1/50 CENT capacity issued, got ${origStaked.totalCapacityIssued}`);

                        // Confirm that the staked capacity was added to the otherProviderId account using the query API
                        const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(otherProviderId))).unwrap();
                        assert.equal(capacityStaked.remainingCapacity, expectedCapacity, "should return a capacityLedger with 1/50M remainingCapacity");
                        assert.equal(capacityStaked.totalTokensStaked, 1n * CENTS, "should return a capacityLedger with 1M total tokens staked");
                        assert.equal(capacityStaked.totalCapacityIssued, expectedCapacity, "should return a capacityLedger with 1/50M capacity issued");
                    });
                });
            });
        });
    });

    describe("when staking and targeting an InvalidTarget", async function () {
        it("fails to stake", async function () {
            let stakeKeys = createKeys("StakeKeys");
            let stakeAmount = 100n * DOLLARS;
            await fundKeypair(devAccounts[0].keys, stakeKeys, stakeAmount)

            const failStakeObj = ExtrinsicHelper.stake(stakeKeys, 99, stakeAmount);
            await assert.rejects(failStakeObj.fundAndSend(), { name: "InvalidTarget" });
        });
    });

    describe("when attempting to stake below the minimum staking requirements", async function () {
        it("should fail to stake for InsufficientStakingAmount", async function () {
            let stakingKeys = createKeys("stakingKeys");
            let providerId = await createMsaAndProvider(stakingKeys, "stakingKeys", accountBalance);
            let stakeAmount = 1500n;

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
            let accountBalance: bigint = 2n * CENTS;
            let stakingKeys = createKeys("stakingKeys");
            let providerId = await createMsaAndProvider(stakingKeys, "stakingKeys", accountBalance);
            let stakingAmount = 1n * DOLLARS;

            const failStakeObj = ExtrinsicHelper.stake(stakingKeys, providerId, stakingAmount);
            await assert.rejects(failStakeObj.fundAndSend(), { name: "BalanceTooLowtoStake" });
        });
    });


    describe("unstake()", function () {
        let unstakeKeys: KeyringPair;
        let providerId: u64;

        before(async function () {
            let accountBalance: bigint = 100n * CENTS;
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

    describe("withdraw_unstaked()", async function () {
        describe("when attempting to call withdrawUnstake before first calling unstake", async function () {
            it("errors with NoUnstakedTokensAvailable", async function () {
                let stakingKeys: KeyringPair = createKeys("stakingKeys");
                let providerId: u64 = await createMsaAndProvider(stakingKeys, "stakingKeys", accountBalance);

                const stakeObj = ExtrinsicHelper.stake(stakingKeys, providerId, tokenMinStake);
                const [stakeEvent] = await stakeObj.fundAndSend();
                assert.notEqual(stakeEvent, undefined, "should return a Stake event");

                const withdrawObj = ExtrinsicHelper.withdrawUnstaked(stakingKeys);
                await assert.rejects(withdrawObj.fundAndSend(), { name: "NoUnstakedTokensAvailable" });
            });
        })
    });
})
