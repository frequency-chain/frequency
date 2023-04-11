import '@frequency-chain/api-augment';
import assert from 'assert';
import { Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import {
  createKeys,
  createMsaAndProvider,
  stakeToProvider,
  CENTS,
  DOLLARS,
  createAndFundKeypair,
  createProviderKeysAndId,
  boostProvider,
  getNextEpochBlock,
  getNextRewardEraBlock,
} from '../scaffolding/helpers';
import { isTestnet } from '../scaffolding/env';
import { KeyringPair } from '@polkadot/keyring/types';
import { UnclaimedRewardInfo } from '@frequency-chain/api-augment/interfaces';
import { Vec } from '@polkadot/types';

const fundingSource = getFundingSource('capacity-replenishment');

describe('Capacity: list_unclaimed_rewards', function() {
  const providerBalance = 2n * DOLLARS;

  const setUpForBoosting = async (boosterName: string, providerName: string): Promise<[number, KeyringPair]> => {
    const booster = await createAndFundKeypair(fundingSource, 5n * DOLLARS, boosterName);
    const providerKeys = createKeys(providerName);
    const provider = await createMsaAndProvider(fundingSource, providerKeys, providerName, providerBalance);
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));

    return [provider.toNumber(), booster];
  };

  it('can be called', async function() {
    const [_provider, booster] = await setUpForBoosting("booster1", "provider1");
    const result = ExtrinsicHelper.api.rpc.state.call(
      'CapacityRuntimeApi_list_unclaimed_rewards', booster.address);
    let count = 0;
    const subscription = result.subscribe((x) => {
      count++;
    });
    //  Failing to do this results in "helpful" error:
    //  `Bad input data provided to list_unclaimed_rewards: Input buffer has still data left after decoding!`
    subscription.unsubscribe();
    assert(count === 0, `should have been empty but had ${count} items`);
  });

  it('returns correct rewards after enough eras have passed', async function() {
    if (isTestnet()) {
      this.skip();
    }
    const [_provider, booster] = await setUpForBoosting("booster2", "provider2");
    console.debug(`Booster pubkey: ${booster.address}`);

    await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());
    await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());
    await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());

    const encodedAddr = ExtrinsicHelper.api.registry.createType('AccountId32', booster.address);

    const result = await ExtrinsicHelper.apiPromise.rpc.state.call(
      'CapacityRuntimeApi_list_unclaimed_rewards', encodedAddr);

    const decResult: Vec<UnclaimedRewardInfo> = ExtrinsicHelper.api.registry.createType('Vec<UnclaimedRewardInfo>', result);
    let count = 0;
    assert(decResult.every(item => {
      count++;
      return item.staked_amount.toHuman() === '1.0000 UNIT';
    }));
    assert.equal(count, 3);

    assert.equal(decResult[0].eligible_amount.toHuman(), '1.0000 UNIT');
    assert.equal(decResult[0].earned_amount.toHuman(), '3.8000 mUNIT');

    assert.equal(decResult[1].eligible_amount.toHuman(), '1.0000 UNIT');
    assert.equal(decResult[1].earned_amount.toHuman(), '3.8000 mUNIT');

    assert.equal(decResult[2].eligible_amount.toHuman(), '1.0000 UNIT');
    assert.equal(decResult[2].earned_amount.toHuman(), '3.8000 mUNIT');
  });
});
