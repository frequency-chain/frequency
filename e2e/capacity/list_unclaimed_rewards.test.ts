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

const fundingSource = getFundingSource('capacity-list-unclaimed-rewards');

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
    const result = await ExtrinsicHelper.apiPromise.call.capacityRuntimeApi.listUnclaimedRewards(booster.address);
    assert.equal(result.length, 0, `result should have been empty but had ${result.length} items`);
  });

  it('returns correct rewards after enough eras have passed', async function () {
    // this will be too long if run against testnet
    if (isTestnet()) this.skip();

    const [_provider, booster] = await setUpForBoosting('booster2', 'provider2');
    console.debug(`Booster pubkey: ${booster.address}`);

    // Move out of the era we boosted inside of
    await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());
    // Have three eras where we get rewards
    await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());
    await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());
    await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());

    const result = await ExtrinsicHelper.apiPromise.call.capacityRuntimeApi.listUnclaimedRewards(booster.address);

    assert(result.length >= 4, 'Might have more than 4 if other blocks formed additional eras');

    // This is the era we first boosted in, shouldn't have any rewards
    assert.equal(result[0].staked_amount.toHuman(), '1.0000 UNIT');
    assert.equal(result[0].eligible_amount.toHuman(), '0');
    assert.equal(result[0].earned_amount.toHuman(), '0');

    // Boosted entire eras, should have rewards
    assert.equal(result[1].staked_amount.toHuman(), '1.0000 UNIT');
    assert.equal(result[1].eligible_amount.toHuman(), '1.0000 UNIT');
    assert.equal(result[1].earned_amount.toHuman(), '3.8000 mUNIT');

    assert.equal(result[2].staked_amount.toHuman(), '1.0000 UNIT');
    assert.equal(result[2].eligible_amount.toHuman(), '1.0000 UNIT');
    assert.equal(result[2].earned_amount.toHuman(), '3.8000 mUNIT');

    assert.equal(result[3].staked_amount.toHuman(), '1.0000 UNIT');
    assert.equal(result[3].eligible_amount.toHuman(), '1.0000 UNIT');
    assert.equal(result[3].earned_amount.toHuman(), '3.8000 mUNIT');
  });
});
