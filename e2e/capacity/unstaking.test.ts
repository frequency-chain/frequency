import '@frequency-chain/api-augment';
import { KeyringPair } from '@polkadot/keyring/types';
import { u64 } from '@polkadot/types';
import assert from 'assert';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  createKeys,
  createMsaAndProvider,
  CENTS,
  DOLLARS,
  createAndFundKeypair,
  boostProvider
} from '../scaffolding/helpers';
import { getFundingSource } from '../scaffolding/funding';
import {getUnifiedAddress} from "../scaffolding/ethereum";

const accountBalance: bigint = 2n * DOLLARS;
const tokenMinStake: bigint = 1n * CENTS;
const fundingSource = getFundingSource(import.meta.url);

describe('Capacity Unstaking Tests', function () {
  describe('unstake()', function () {
    let unstakeKeys: KeyringPair;
    let providerId: u64;

    before(async function () {
      const accountBalance: bigint = 100n * CENTS;
      unstakeKeys = createKeys('stakingKeys');
      providerId = await createMsaAndProvider(fundingSource, unstakeKeys, 'stakingKeys', accountBalance);
    });

    describe('when attempting to unstake a Zero amount', function () {
      it('errors with UnstakedAmountIsZero', async function () {
        const failUnstakeObj = ExtrinsicHelper.unstake(unstakeKeys, providerId, 0);
        await assert.rejects(failUnstakeObj.signAndSend(), { name: 'UnstakedAmountIsZero' });
      });
    });

    describe('when account has not staked', function () {
      it('errors with StakingAccountNotFound', async function () {
        const failUnstakeObj = ExtrinsicHelper.unstake(unstakeKeys, providerId, tokenMinStake);
        await assert.rejects(failUnstakeObj.signAndSend(), { name: 'NotAStakingAccount' });
      });
    });

    describe("when account has staked", function () {
      it("succeeds when boosting a provider and no unclaimed rewards", async function () {
        const stakeKeys = createKeys('booster');
        const providerBalance = 2n * DOLLARS;
        const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);
        const booster = await createAndFundKeypair(fundingSource, 10n * DOLLARS, 'booster');
        await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 4n * DOLLARS));
        const boosterAddr = getUnifiedAddress(booster);

        let result = await ExtrinsicHelper.apiPromise.query.capacity.stakingAccountLedger(boosterAddr);
        const startingAmount = result.unwrap().active.toNumber();
        assert.equal(startingAmount, 4n * DOLLARS);

        // Unboost 1mil from provider—this succeeds
        const unstakeOp = ExtrinsicHelper.unstake(booster, provider, 1n * DOLLARS);
        assert.doesNotReject(unstakeOp.fundAndSend(fundingSource));
        result = await ExtrinsicHelper.apiPromise.query.capacity.stakingAccountLedger(boosterAddr);
        const afterUnstakeAmount = result.unwrap().active.toNumber();
        assert.equal(3n * DOLLARS, afterUnstakeAmount);
      });
    });
  });

  describe('withdraw_unstaked()', function () {
    describe('when attempting to call withdrawUnstake before first calling unstake', function () {
      it('errors with NoUnstakedTokensAvailable', async function () {
        const stakingKeys: KeyringPair = createKeys('stakingKeys');
        const providerId: u64 = await createMsaAndProvider(fundingSource, stakingKeys, 'stakingKeys', accountBalance);

        const stakeObj = ExtrinsicHelper.stake(stakingKeys, providerId, tokenMinStake);
        const { target: stakeEvent } = await stakeObj.signAndSend();
        assert.notEqual(stakeEvent, undefined, 'should return a Stake event');

        const withdrawObj = ExtrinsicHelper.withdrawUnstaked(stakingKeys);
        await assert.rejects(withdrawObj.signAndSend(), { name: 'NoUnstakedTokensAvailable' });
      });
    });
  });

});
