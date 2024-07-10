import "@frequency-chain/api-augment";
import assert from "assert";
import { ExtrinsicHelper, } from "../scaffolding/extrinsicHelpers";
import { getFundingSource } from '../scaffolding/funding';
import {
  createKeys, createMsaAndProvider,
  stakeToProvider,
  CENTS, DOLLARS, createAndFundKeypair, createProviderKeysAndId
} from "../scaffolding/helpers";

const fundingSource = getFundingSource('capacity-replenishment');

describe("Capacity: change_staking_target", function() {
  const tokenMinStake: bigint = 1n * CENTS;
  const capacityMin: bigint = tokenMinStake / 50n;

  it("happy path succeeds", async function() {
      const providerBalance = 2n * DOLLARS;
      const stakeKeys = createKeys("staker");
      const oldProvider = await createMsaAndProvider(fundingSource, stakeKeys, "Provider1", providerBalance);
      const [_bar, newProvider] = await createProviderKeysAndId(fundingSource, providerBalance);

      await assert.doesNotReject(stakeToProvider(fundingSource, stakeKeys, oldProvider, tokenMinStake*3n));

      const call = ExtrinsicHelper.changeStakingTarget(stakeKeys, oldProvider, newProvider, tokenMinStake);
      const events = await call.signAndSend();
      assert.notEqual(events, undefined);
  });

  // not intended to be exhaustive, just check one error case
  it("fails if 'to' is not a Provider", async function() {
    const providerBalance = 2n * DOLLARS;
    const stakeKeys = createKeys("staker");
    const oldProvider = await createMsaAndProvider(fundingSource, stakeKeys, "Provider2", providerBalance);

    await assert.doesNotReject(stakeToProvider(fundingSource, stakeKeys, oldProvider, tokenMinStake*3n));
    // const notAProvider = 3;
    // const call = ExtrinsicHelper.changeStakingTarget(stakeKeys, oldProvider, notAProvider, tokenMinStake);
    // await assert.rejects(call.signAndSend(), {name: "InvalidTarget"})
  });
});
