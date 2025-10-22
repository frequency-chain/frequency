import '@frequency-chain/api-augment';
import assert from 'assert';
import type { KeyringPair } from '@polkadot/keyring/types';
import { getFundingSource } from '../scaffolding/funding';
import {
  createKeys,
  createMsaAndProvider,
  CENTS,
  DOLLARS,
  createAndFundKeypair,
  boostProvider,
  stakeToProvider,
  addProxy,
  getBlockNumber,
} from '../scaffolding/helpers';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
let fundingSource: KeyringPair;
const tokenMinStake: bigint = 1n * CENTS;

describe('Capacity: provider_boost extrinsic', function () {
  const providerBalance = 2n * DOLLARS;

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);
  });

  it('An account can do a simple provider boost call', async function () {
    const stakeKeys = createKeys('booster');
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);
    const booster = await createAndFundKeypair(fundingSource, 5n * DOLLARS, 'booster');
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
  });

  it('fails when staker is a Maximized Capacity staker', async function () {
    const stakeKeys = createKeys('booster');
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);
    await assert.doesNotReject(stakeToProvider(fundingSource, stakeKeys, provider, tokenMinStake));
    await assert.rejects(boostProvider(fundingSource, stakeKeys, provider, tokenMinStake), {
      name: 'CannotChangeStakingType',
    });
  });

  it("fails when staker doesn't have enough token", async function () {
    const stakeKeys = createKeys('booster');
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);
    const booster = await createAndFundKeypair(fundingSource, 1n * DOLLARS, 'booster');
    await assert.rejects(boostProvider(booster, booster, provider, 1n * DOLLARS), { name: 'BalanceTooLowtoStake' });
  });

  it('staker can boost multiple times', async function () {
    const stakeKeys = createKeys('booster');
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);
    const booster = await createAndFundKeypair(fundingSource, 10n * DOLLARS, 'booster');
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
  });

  it('can boost, claim, and unstake via proxy account', async function () {
    const alice = await createAndFundKeypair(fundingSource, 1000n * DOLLARS, 'alice');
    const bob = await createAndFundKeypair(fundingSource, 2000n * DOLLARS, 'bob');

    await addProxy(bob, alice, 'Staking');
    const innerBoost = ExtrinsicHelper.api.tx.capacity.providerBoost(1, 2n * DOLLARS);
    const expectedEvent = ExtrinsicHelper.api.events.capacity.ProviderBoosted;
    await ExtrinsicHelper.proxySignAndSend(innerBoost, alice, bob, expectedEvent);
    const block = await getBlockNumber();
    await ExtrinsicHelper.runToBlock(block + 41);

    const innerClaim = ExtrinsicHelper.api.tx.capacity.claimStakingRewards();
    const expectedClaimEvent = ExtrinsicHelper.api.events.capacity.ProviderBoostRewardClaimed;
    await ExtrinsicHelper.proxySignAndSend(innerClaim, alice, bob, expectedClaimEvent);

    const innerUnstake = ExtrinsicHelper.api.tx.capacity.unstake(1, 2n * DOLLARS);
    const expectedUnstakeEvent = ExtrinsicHelper.api.events.capacity.UnStaked;
    await ExtrinsicHelper.proxySignAndSend(innerUnstake, alice, bob, expectedUnstakeEvent);
  });
});
