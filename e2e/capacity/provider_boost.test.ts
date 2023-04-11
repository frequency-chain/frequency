import "@frequency-chain/api-augment";
import assert from "assert";
import { getFundingSource } from '../scaffolding/funding';
import {
  createKeys, createMsaAndProvider,
  CENTS, DOLLARS, createAndFundKeypair, boostProvider,
} from '../scaffolding/helpers';

const fundingSource = getFundingSource('capacity-replenishment');

describe("Capacity: provider_boost extrinsic", function() {
  const providerBalance = 2n * DOLLARS;

  it("happy path succeeds", async function () {
    const stakeKeys = createKeys("booster");
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, "Provider1", providerBalance);
    const booster = await createAndFundKeypair(fundingSource, 5n * DOLLARS, "booster");
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
  });

  it("fails when they are a Maximized Capacity staker", async function() {
    const stakeKeys = createKeys("booster");
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, "Provider1", providerBalance);
    await assert.rejects(boostProvider(fundingSource, stakeKeys, provider, 1n * DOLLARS));
  });

  it("fails when they don't have enough token", async function() {
    const stakeKeys = createKeys("booster");
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, "Provider1", providerBalance);
    const booster = await createAndFundKeypair(fundingSource, 1n * DOLLARS, "booster");
    await assert.rejects(boostProvider(booster, booster, provider, 1n * DOLLARS));
  });

  it("they can boost multiple times", async function () {
    const stakeKeys = createKeys("booster");
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, "Provider1", providerBalance);
    const booster = await createAndFundKeypair(fundingSource, 10n * DOLLARS, "booster");
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
  });
});
