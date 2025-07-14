import '@frequency-chain/api-augment';
import { KeyringPair } from '@polkadot/keyring/types';
import { u64 } from '@polkadot/types';
import assert from 'assert';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { createKeys, createMsaAndProvider, CENTS, DOLLARS } from '../scaffolding/helpers';
import { getFundingSource } from '../scaffolding/funding';

const accountBalance: bigint = 2n * DOLLARS;
const tokenMinStake: bigint = 1n * CENTS;
let fundingSource: KeyringPair;

describe('Capacity Unstaking Tests', function () {
  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);
  });

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
