import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { u64, } from "@polkadot/types";
import assert from "assert";
import { ExtrinsicHelper, } from "../scaffolding/extrinsicHelpers";
import {
  devAccounts, createKeys, createMsaAndProvider,
  stakeToProvider, CHAIN_ENVIRONMENT,
  TEST_EPOCH_LENGTH, setEpochLength,
  CENTS, DOLLARS, createAndFundKeypair, createProviderKeysAndId
}
  from "../scaffolding/helpers";
import { firstValueFrom } from "rxjs";
import { MessageSourceId} from "@frequency-chain/api-augment/interfaces";

describe("change_staking_target tests", () => {
  const tokenMinStake: bigint = 1n * CENTS;
  const capacityMin: bigint = tokenMinStake / 50n;

  const unusedMsaId = async () => {
    const maxMsaId = (await ExtrinsicHelper.getCurrentMsaIdentifierMaximum()).toNumber();
    return maxMsaId + 99;
  }

  before(async () => {
    if (process.env.CHAIN_ENVIRONMENT === CHAIN_ENVIRONMENT.DEVELOPMENT) {
      await setEpochLength(devAccounts[0].keys, TEST_EPOCH_LENGTH);
    }
  });

  it("happy path succeeds", async () => {
      const providerBalance = 2n * DOLLARS;
      const stakeKeys = createKeys("staker");
      const oldProvider = await createMsaAndProvider(stakeKeys, "Provider1", providerBalance);
      const [_unused, newProvider] = await createProviderKeysAndId();

      await assert.doesNotReject(stakeToProvider(stakeKeys, oldProvider, tokenMinStake*3n));

      const call = ExtrinsicHelper.changeStakingTarget(stakeKeys, oldProvider, newProvider, tokenMinStake);
      const [events] = await call.signAndSend();
      assert.notEqual(events, undefined);
  });

  // not intended to be exhaustive, just check one error case
  it("fails if 'to' is not a Provider", async () => {
    const providerBalance = 2n * DOLLARS;
    const stakeKeys = createKeys("staker");
    const notAProvider = await unusedMsaId();
    const oldProvider = await createMsaAndProvider(stakeKeys, "Provider1", providerBalance);
    await assert.doesNotReject(stakeToProvider(stakeKeys, oldProvider, tokenMinStake*3n));
    const call = ExtrinsicHelper.changeStakingTarget(stakeKeys, oldProvider, notAProvider, tokenMinStake);
    await assert.rejects(call.signAndSend(), {name: "InvalidTarget"})
  });
});
