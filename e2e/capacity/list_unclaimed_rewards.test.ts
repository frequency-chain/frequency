import '@frequency-chain/api-augment';
import assert from 'assert';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import {
  createKeys,
  createMsaAndProvider,
  DOLLARS,
  createAndFundKeypair,
  boostProvider,
  getNextRewardEraBlock,
} from '../scaffolding/helpers';
import { isTestnet } from '../scaffolding/env';
import { KeyringPair } from '@polkadot/keyring/types';
import { getUnifiedAddress } from '@frequency-chain/ethereum-utils';

const fundingSource = getFundingSource(import.meta.url);

describe('Capacity: list_unclaimed_rewards', function () {
  const providerBalance = 2n * DOLLARS;

  const setUpForBoosting = async (boosterName: string, providerName: string): Promise<[number, KeyringPair]> => {
    const booster = await createAndFundKeypair(fundingSource, 5n * DOLLARS, boosterName);
    const providerKeys = createKeys(providerName);
    const provider = await createMsaAndProvider(fundingSource, providerKeys, providerName, providerBalance);
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));

    return [provider.toNumber(), booster];
  };

  it('can be called', async function () {
    const [_provider, booster] = await setUpForBoosting('booster1', 'provider1');
    const result = await ExtrinsicHelper.apiPromise.call.capacityRuntimeApi.listUnclaimedRewards(
      getUnifiedAddress(booster)
    );
    assert.equal(result.length, 0, `result should have been empty but had ${result.length} items`);
  });

  it('returns correct rewards after enough eras have passed', async function () {
    // this will be too long if run against testnet
    if (isTestnet()) this.skip();

    const [_provider, booster] = await setUpForBoosting('booster2', 'provider2');

    // Move out of the era we boosted inside of
    await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());
    // Have three eras where we get rewards
    await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());
    await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());
    await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());

    const result = await ExtrinsicHelper.apiPromise.call.capacityRuntimeApi.listUnclaimedRewards(
      getUnifiedAddress(booster)
    );

    assert(result.length >= 4, `Length should be >= 4 but is ${result.length}`);

    // This is the era we first boosted in, shouldn't have any rewards
    assert.equal(result[0].stakedAmount.toHuman(), '100,000,000');
    assert.equal(result[0].eligibleAmount.toHuman(), '0');
    assert.equal(result[0].earnedAmount.toHuman(), '0');

    // Boosted entire eras, should have rewards
    assert.equal(result[1].stakedAmount.toHuman(), '100,000,000');
    assert.equal(result[1].eligibleAmount.toHuman(), '100,000,000');
    assert.equal(result[1].earnedAmount.toHuman(), '575,000');

    assert.equal(result[2].stakedAmount.toHuman(), '100,000,000');
    assert.equal(result[2].eligibleAmount.toHuman(), '100,000,000');
    assert.equal(result[2].earnedAmount.toHuman(), '575,000');

    assert.equal(result[3].stakedAmount.toHuman(), '100,000,000');
    assert.equal(result[3].eligibleAmount.toHuman(), '100,000,000');
    assert.equal(result[3].earnedAmount.toHuman(), '575,000');
  });
});
