import '@frequency-chain/api-augment';
import assert from 'assert';
import { getFundingSource } from '../scaffolding/funding';
import {
  createKeys,
  createMsaAndProvider,
  CENTS,
  DOLLARS,
  createAndFundKeypair,
  boostProvider, stakeToProvider,
} from '../scaffolding/helpers';

const fundingSource = getFundingSource('capacity-provider-boost');
const tokenMinStake: bigint = 1n * CENTS;

describe('Capacity: provider_boost extrinsic', function () {
  const providerBalance = 2n * DOLLARS;

  it('An account can do a simple provider boost call', async function () {
    const stakeKeys = createKeys('booster');
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);
    const booster = await createAndFundKeypair(fundingSource, 5n * DOLLARS, 'booster');
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
  });

  it.only('fails when staker is a Maximized Capacity staker', async function () {
    const stakeKeys = createKeys('booster');
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);
    await assert.doesNotReject(stakeToProvider(fundingSource, stakeKeys, provider, tokenMinStake));
    await assert.rejects(boostProvider(fundingSource, stakeKeys, provider, tokenMinStake), {name: "CannotChangeStakingType"});
  });

  it("fails when staker doesn't have enough token", async function () {
    const stakeKeys = createKeys('booster');
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);
    const booster = await createAndFundKeypair(fundingSource, 1n * DOLLARS, 'booster');
    await assert.rejects(boostProvider(booster, booster, provider, 1n * DOLLARS), {name: "BalanceTooLowtoStake"});
  });

  it('staker can boost multiple times', async function () {
    const stakeKeys = createKeys('booster');
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);
    const booster = await createAndFundKeypair(fundingSource, 10n * DOLLARS, 'booster');
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
  });
});
