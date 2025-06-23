import '@frequency-chain/api-augment';
import { KeyringPair } from '@polkadot/keyring/types';
import { u64 } from '@polkadot/types';
import assert from 'assert';
import { ExtrinsicHelper, ReleaseSchedule } from '../scaffolding/extrinsicHelpers';
import {
  createKeys,
  createMsaAndProvider,
  stakeToProvider,
  getNextEpochBlock,
  TEST_EPOCH_LENGTH,
  CENTS,
  DOLLARS,
  createAndFundKeypair,
  getCapacity,
  createMsa,
  calculateReleaseSchedule,
  getSpendableBalance,
} from '../scaffolding/helpers';
import { isDev } from '../scaffolding/env';
import { getFundingSource } from '../scaffolding/funding';
import { BigInt } from '@polkadot/x-bigint';

const accountBalance: bigint = 2n * DOLLARS;
const tokenMinStake: bigint = 1n * CENTS;
const capacityMin: bigint = tokenMinStake / 50n;
let fundingSource: KeyringPair;

describe('Capacity Staking Tests', function () {
  // The frozen balance is initialized and tracked throughout the staking end to end tests
  // to accommodate for the fact that withdrawing unstaked token tests are not executed
  // against a relay chain. Since the length of time to wait for an epoch period to roll over could
  // potentially be hours / days, that test is skipped. Therefore, we must track the frozen balance
  // so that tests against it can pass regardless if withdrawing (the frozen balance decreasing)
  // has happened or not.
  let trackedFrozenBalance: bigint = 0n;

  describe('scenario: user staking, unstaking, and withdraw-unstaked', function () {
    let stakeKeys: KeyringPair;
    let stakeProviderId: u64;

    before(async function () {
      fundingSource = await getFundingSource(import.meta.url);
      stakeKeys = createKeys('StakeKeys');
      stakeProviderId = await createMsaAndProvider(fundingSource, stakeKeys, 'StakeProvider', accountBalance);
    });

    it('successfully stakes the minimum amount', async function () {
      await assert.doesNotReject(stakeToProvider(fundingSource, stakeKeys, stakeProviderId, tokenMinStake));

      // Confirm that the tokens were locked in the stakeKeys account using the query API
      const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys);
      assert.equal(
        stakedAcctInfo.data.frozen,
        tokenMinStake,
        `expected ${tokenMinStake} frozen balance, got ${stakedAcctInfo.data.frozen}`
      );

      // Confirm that the capacity was added to the stakeProviderId using the query API
      const capacityStaked = await getCapacity(stakeProviderId);
      assert.equal(
        capacityStaked.remainingCapacity,
        capacityMin,
        `expected capacityLedger.remainingCapacity = 1CENT, got ${capacityStaked.remainingCapacity}`
      );
      assert.equal(
        capacityStaked.totalTokensStaked,
        tokenMinStake,
        `expected capacityLedger.totalTokensStaked = 1CENT, got ${capacityStaked.totalTokensStaked}`
      );
      assert.equal(
        capacityStaked.totalCapacityIssued,
        capacityMin,
        `expected capacityLedger.totalCapacityIssued = 1CENT, got ${capacityStaked.totalCapacityIssued}`
      );
      trackedFrozenBalance += tokenMinStake;
    });

    it('successfully unstakes the minimum amount', async function () {
      const stakeObj = ExtrinsicHelper.unstake(stakeKeys, stakeProviderId, tokenMinStake);
      const { target: unStakeEvent } = await stakeObj.signAndSend();
      assert.notEqual(unStakeEvent, undefined, 'should return an UnStaked event');
      assert.equal(
        unStakeEvent?.data.capacity,
        capacityMin,
        'should return an UnStaked event with 1 CENT reduced capacity'
      );

      // Confirm that the tokens were unstaked in the stakeProviderId account using the query API
      const capacityStaked = await getCapacity(stakeProviderId);
      assert.equal(capacityStaked.remainingCapacity, 0, 'should return a capacityLedger with 0 remainingCapacity');
      assert.equal(capacityStaked.totalTokensStaked, 0, 'should return a capacityLedger with 0 total tokens staked');
      assert.equal(capacityStaked.totalCapacityIssued, 0, 'should return a capacityLedger with 0 capacity issued');
    });

    it('successfully withdraws the unstaked amount', async function () {
      // Withdrawing unstaked token will only be executed against a Frequency
      // node built for development due to the long length of time it would
      // take to wait for an epoch period to roll over.
      if (!isDev()) this.skip();

      // Mine enough blocks to pass the unstake period = CapacityUnstakingThawPeriod = 2 epochs
      const newEpochBlock = await getNextEpochBlock();
      await ExtrinsicHelper.runToBlock(newEpochBlock + TEST_EPOCH_LENGTH + 1);

      const withdrawObj = ExtrinsicHelper.withdrawUnstaked(stakeKeys);
      const { target: withdrawEvent } = await withdrawObj.signAndSend();
      assert.notEqual(withdrawEvent, undefined, 'should return a StakeWithdrawn event');

      const amount = withdrawEvent!.data.amount;
      assert.equal(amount, tokenMinStake, 'should return a StakeWithdrawn event with 1M amount');

      // Confirm that the tokens were unstaked in the stakeKeys account using the query API
      const unStakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys);
      assert.equal(unStakedAcctInfo.data.frozen, 0, 'should return an account with 0 frozen balance');

      // Confirm that the staked capacity was removed from the stakeProviderId account using the query API
      const capacityStaked = await getCapacity(stakeProviderId);
      assert.equal(capacityStaked.remainingCapacity, 0, 'should return a capacityLedger with 0 remainingCapacity');
      assert.equal(capacityStaked.totalTokensStaked, 0, 'should return a capacityLedger with 0 total tokens staked');
      assert.equal(capacityStaked.totalCapacityIssued, 0, 'should return a capacityLedger with 0 capacity issued');

      trackedFrozenBalance -= tokenMinStake;
    });

    describe('when staking to multiple times', function () {
      describe('and targeting same provider', function () {
        it('successfully increases the amount that was targeted to provider', async function () {
          await assert.doesNotReject(stakeToProvider(fundingSource, stakeKeys, stakeProviderId, tokenMinStake));

          const capacityStaked = await getCapacity(stakeProviderId);
          assert.equal(
            capacityStaked.remainingCapacity,
            capacityMin,
            `expected capacityStaked.remainingCapacity = ${capacityMin}, got ${capacityStaked.remainingCapacity}`
          );
          assert.equal(
            capacityStaked.totalTokensStaked,
            tokenMinStake,
            `expected capacityStaked.totalTokensStaked = ${tokenMinStake}, got ${capacityStaked.totalTokensStaked}`
          );
          assert.equal(
            capacityStaked.totalCapacityIssued,
            capacityMin,
            `expected capacityStaked.totalCapacityIssued = ${capacityMin}, got ${capacityStaked.totalCapacityIssued}`
          );
          trackedFrozenBalance += tokenMinStake;

          // Confirm that the tokens were staked in the stakeKeys account using the query API
          const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys);

          const increasedFrozen: bigint = stakedAcctInfo.data.frozen.toBigInt();

          assert.equal(
            increasedFrozen,
            trackedFrozenBalance,
            `expected frozen=${tokenMinStake}, got ${increasedFrozen}`
          );
        });
      });

      describe('and targeting different provider', function () {
        let otherProviderKeys: KeyringPair;
        let otherProviderId: u64;

        before(async function () {
          otherProviderKeys = createKeys('OtherProviderKeys');
          otherProviderId = await createMsaAndProvider(fundingSource, otherProviderKeys, 'OtherProvider');
        });

        it('does not change other targets amounts', async function () {
          // Increase stake by 1 cent to a different target.
          await assert.doesNotReject(stakeToProvider(fundingSource, stakeKeys, otherProviderId, 1n * CENTS));
          const expectedCapacity = CENTS / 50n;

          // Confirm that the staked capacity of the original stakeProviderId account is unchanged
          // stakeProviderId should still have 1M from first test case in this describe.
          // otherProvider should now have 1M
          const origStaked = await getCapacity(stakeProviderId);
          assert.equal(
            origStaked.remainingCapacity,
            expectedCapacity,
            `expected 1/50 CENT remaining capacity, got ${origStaked.remainingCapacity}`
          );
          assert.equal(
            origStaked.totalTokensStaked,
            1n * CENTS,
            `expected 1 CENT staked, got ${origStaked.totalTokensStaked}`
          );
          assert.equal(
            origStaked.totalCapacityIssued,
            expectedCapacity,
            `expected 1/50 CENT capacity issued, got ${origStaked.totalCapacityIssued}`
          );

          // Confirm that the staked capacity was added to the otherProviderId account using the query API
          const capacityStaked = await getCapacity(otherProviderId);
          assert.equal(
            capacityStaked.remainingCapacity,
            expectedCapacity,
            'should return a capacityLedger with 1/50M remainingCapacity'
          );
          assert.equal(
            capacityStaked.totalTokensStaked,
            1n * CENTS,
            'should return a capacityLedger with 1M total tokens staked'
          );
          assert.equal(
            capacityStaked.totalCapacityIssued,
            expectedCapacity,
            'should return a capacityLedger with 1/50M capacity issued'
          );
        });

        it('successfully increases the amount that was targeted to provider from different accounts', async function () {
          // Create a new account
          const additionalKeys = await createAndFundKeypair(fundingSource, accountBalance);
          const currentStaked = await getCapacity(stakeProviderId);

          await assert.doesNotReject(stakeToProvider(fundingSource, additionalKeys, stakeProviderId, tokenMinStake));

          const capacityStaked = await getCapacity(stakeProviderId);
          assert.equal(
            capacityStaked.remainingCapacity,
            currentStaked.remainingCapacity.toBigInt() + capacityMin,
            `should return a capacityLedger with ${capacityMin} remaining, got ${capacityStaked.remainingCapacity}`
          );
          assert.equal(
            capacityStaked.totalTokensStaked,
            currentStaked.totalTokensStaked.toBigInt() + tokenMinStake,
            `should return a capacityLedger with ${tokenMinStake} total staked, got: ${capacityStaked.totalTokensStaked}`
          );
          assert.equal(
            capacityStaked.totalCapacityIssued,
            currentStaked.totalCapacityIssued.toBigInt() + capacityMin,
            `should return a capacityLedger with ${capacityMin} total issued, got ${capacityStaked.totalCapacityIssued}`
          );

          // Confirm that the tokens were not staked in the stakeKeys account using the query API
          const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(additionalKeys);

          const increasedFrozen: bigint = stakedAcctInfo.data.frozen.toBigInt();

          assert.equal(increasedFrozen, tokenMinStake, `expected frozen=${tokenMinStake}, got ${increasedFrozen}`);
        });
      });
    });
  });

  describe('when staking and targeting an InvalidTarget', function () {
    it('fails to stake', async function () {
      const stakeAmount = 10n * CENTS;
      const [notProviderMsaId, stakeKeys] = await createMsa(fundingSource, 10n * CENTS);

      const failStakeObj = ExtrinsicHelper.stake(stakeKeys, notProviderMsaId, stakeAmount);
      await assert.rejects(failStakeObj.signAndSend(), { name: 'InvalidTarget' });
    });
  });

  describe('when attempting to stake below the minimum staking requirements', function () {
    it('should fail to stake for StakingAmountBelowMinimum', async function () {
      const stakingKeys = createKeys('stakingKeys');
      const providerId = await createMsaAndProvider(fundingSource, stakingKeys, 'stakingKeys', 150n * CENTS);
      const stakeAmount = 1500n;

      const failStakeObj = ExtrinsicHelper.stake(stakingKeys, providerId, stakeAmount);
      await assert.rejects(failStakeObj.signAndSend(), { name: 'StakingAmountBelowMinimum' });
    });
  });

  describe('when attempting to stake a zero amount', function () {
    it('fails to stake and errors ZeroAmountNotAllowed', async function () {
      const stakingKeys = createKeys('stakingKeys');
      const providerId = await createMsaAndProvider(fundingSource, stakingKeys, 'stakingKeys', 10n * CENTS);

      const failStakeObj = ExtrinsicHelper.stake(stakingKeys, providerId, 0);
      await assert.rejects(failStakeObj.signAndSend(), { name: 'ZeroAmountNotAllowed' });
    });
  });

  describe('when staking an amount and account balance is too low', function () {
    it('fails to stake and errors BalanceTooLowtoStake', async function () {
      const stakingKeys = createKeys('stakingKeys');
      const providerId = await createMsaAndProvider(fundingSource, stakingKeys, 'stakingKeys', 1n * DOLLARS);
      const stakingAmount = 2n * DOLLARS;

      const failStakeObj = ExtrinsicHelper.stake(stakingKeys, providerId, stakingAmount);
      await assert.rejects(failStakeObj.signAndSend(), { name: 'BalanceTooLowtoStake' });
    });

    it('fails to stake when stake is >= than stakable_amount + minimum token balance', async function () {
      const stakingKeys = createKeys('stakingKeys');
      const providerId = await createMsaAndProvider(fundingSource, stakingKeys, 'stakingKeys', 1n * DOLLARS);
      const stakingAmount = 1n * DOLLARS;

      const failStakeObj = ExtrinsicHelper.stake(stakingKeys, providerId, stakingAmount);
      await assert.rejects(failStakeObj.signAndSend(), { name: 'BalanceTooLowtoStake' });
    });
  });

  describe('staking when there are other freezes on the balance', function () {
    let vesterKeys: KeyringPair;
    let providerKeys: KeyringPair;
    let providerId: u64;

    async function assertSpendable(keys: KeyringPair, amount: bigint) {
      const spendable = await getSpendableBalance(keys);
      assert.equal(spendable, amount, `Expected spendable ${amount}, got ${spendable}`);
    }

    async function assertFrozen(keys: KeyringPair, amount: bigint) {
      const accountInfo = await ExtrinsicHelper.getAccountInfo(keys);
      assert.equal(accountInfo.data.frozen, amount, `Expected frozen ${amount}, got ${accountInfo.data.frozen}`);
    }

    before(async function () {
      vesterKeys = await createAndFundKeypair(fundingSource, 50_000_000n);
      await assertSpendable(vesterKeys, 49n * BigInt(CENTS)); // less ED
      await assertFrozen(vesterKeys, 0n);
      providerKeys = await createAndFundKeypair(fundingSource, 10n * CENTS);
      providerId = await createMsaAndProvider(fundingSource, providerKeys, 'Provider Whale', 10n * DOLLARS);
    });

    it('succeeds when there is a time-release freeze', async function () {
      const vestingAmount = 100n * DOLLARS;
      const schedule: ReleaseSchedule = calculateReleaseSchedule(vestingAmount);

      const vestedTransferTx = ExtrinsicHelper.timeReleaseTransfer(fundingSource, vesterKeys, schedule);

      await assert.doesNotReject(vestedTransferTx.signAndSend(undefined, undefined, false));
      await assertFrozen(vesterKeys, 100n * DOLLARS);

      await assert.doesNotReject(
        ExtrinsicHelper.stake(vesterKeys, providerId, 80n * DOLLARS).signAndSend(undefined, undefined, false)
      );

      const spendable = await getSpendableBalance(vesterKeys);
      // after txn fees
      assert(spendable > 47n * CENTS, `Expected spendable > 47 CENTS, got ${spendable}`);
    });
  });
});
