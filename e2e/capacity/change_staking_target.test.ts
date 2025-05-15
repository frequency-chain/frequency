import '@frequency-chain/api-augment';
import assert from 'assert';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import {
  createKeys,
  createMsaAndProvider,
  stakeToProvider,
  CENTS,
  DOLLARS,
  createProviderKeysAndId,
} from '../scaffolding/helpers';

const fundingSource = getFundingSource(import.meta.url);

describe('Capacity: change_staking_target', function () {
  const tokenMinStake: bigint = 1n * CENTS;

  it('successfully stake tokens to a provider', async function () {
    const providerBalance = 2n * DOLLARS;
    const stakeKeys = createKeys('staker');

    // Setup
    const [oldProvider, [_bar, newProvider]] = await Promise.all([
      createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance),
      createProviderKeysAndId(fundingSource, providerBalance),
    ]);

    // Stake
    await assert.doesNotReject(stakeToProvider(fundingSource, stakeKeys, oldProvider, tokenMinStake * 3n));

    // Change Stake
    const call = ExtrinsicHelper.changeStakingTarget(stakeKeys, oldProvider, newProvider, tokenMinStake);
    const events = await call.signAndSend();
    assert.notEqual(events, undefined);
  });

  // not intended to be exhaustive, just check one error case
  it("fails if 'to' is not a Provider", async function () {
    const providerBalance = 2n * DOLLARS;
    const stakeKeys = createKeys('staker');
    const oldProvider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider2', providerBalance);

    await assert.doesNotReject(stakeToProvider(fundingSource, stakeKeys, oldProvider, tokenMinStake * 6n));
    const notAProvider = 9999;
    const call = ExtrinsicHelper.changeStakingTarget(stakeKeys, oldProvider, notAProvider, tokenMinStake * 2n);
    await assert.rejects(call.signAndSend(), { name: 'InvalidTarget' });
  });
});
