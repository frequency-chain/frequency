import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { u16, u64 } from "@polkadot/types"
import assert from "assert";
import { Extrinsic, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import {
  createKeys,
  createMsaAndProvider,
  stakeToProvider,
  fundKeypair,
  getNextEpochBlock,
  getOrCreateGraphChangeSchema,
  CENTS,
  DOLLARS,
  TokenPerCapacity,
  assertEvent,
  getRemainingCapacity,
  getNonce,
} from "../scaffolding/helpers";
import { getFundingSource } from "../scaffolding/funding";
import { isTestnet } from "../scaffolding/env";

describe("Capacity Replenishment Testing: ", function () {
  let schemaId: u16;
  const fundingSource = getFundingSource("capacity-replenishment");


  async function createAndStakeProvider(name: string, stakingAmount: bigint): Promise<[KeyringPair, u64]> {
    const stakeKeys = createKeys(name);
    const stakeProviderId = await createMsaAndProvider(fundingSource, stakeKeys, "ReplProv", 50n * DOLLARS);
    assert.notEqual(stakeProviderId, 0, "stakeProviderId should not be zero");
    await stakeToProvider(fundingSource, stakeKeys, stakeProviderId, stakingAmount);
    return [stakeKeys, stakeProviderId];
  }


  before(async function () {
    // Replenishment requires the epoch length to be shorter than testnet (set in globalHooks)
    if (isTestnet()) this.skip();

    schemaId = await getOrCreateGraphChangeSchema(fundingSource);
  });

  describe("Capacity is replenished", function () {
    it("after new epoch", async function () {
      const totalStaked = 3n * DOLLARS;
      let expectedCapacity = totalStaked / TokenPerCapacity;
      const [stakeKeys, stakeProviderId] = await createAndStakeProvider("ReplFirst", totalStaked);
      const payload = JSON.stringify({ changeType: 1, fromId: 1, objectId: 2 })
      const call = ExtrinsicHelper.addOnChainMessage(stakeKeys, schemaId, payload);

      // confirm that we start with a full tank
      await ExtrinsicHelper.runToBlock(await getNextEpochBlock());
      let remainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
      assert.equal(expectedCapacity, remainingCapacity, "Our expected capacity from staking is wrong");

      await call.payWithCapacity(-1);
      remainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
      assert(expectedCapacity > remainingCapacity, "Our remaining capacity is much higher than expected.");
      const capacityPerCall = expectedCapacity - remainingCapacity;
      assert(remainingCapacity > capacityPerCall, "We don't have enough to make a second call");

      // one more txn to deplete capacity more so this current remaining is different from when
      // we submitted the first message.
      await call.payWithCapacity(-1);
      await ExtrinsicHelper.runToBlock(await getNextEpochBlock());

      // this should cause capacity to be refilled and then deducted by the cost of one message.
      await call.payWithCapacity(-1);
      let newRemainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();

      // this should be the same as after sending the first message, since it is the first message after
      // the epoch.
      assert.equal(remainingCapacity, newRemainingCapacity);
    });
  });

  function assert_capacity_call_fails_with_balance_too_low(call: Extrinsic) {
    return assert.rejects(
      call.payWithCapacity(-1), { name: "RpcError", message: /1010.+account balance too low/ });
  }

  describe("Capacity is not replenished", function () {
    it("if out of capacity and last_replenished_at is <= current epoch", async function () {
      let [stakeKeys, stakeProviderId] = await createAndStakeProvider("NoSend", 150n * CENTS);
      let payload = JSON.stringify({ changeType: 1, fromId: 1, objectId: 2 })
      let call = ExtrinsicHelper.addOnChainMessage(stakeKeys, schemaId, payload);

      // run until we can't afford to send another message.
      await drainCapacity(call, stakeProviderId, stakeKeys);

      await assert_capacity_call_fails_with_balance_too_low(call);
    });
  });

  describe("Regression test: when user attempts to stake tiny amounts before provider's first message of an epoch,", function () {
    it("provider is still replenished and can send a message", async function () {
      const providerStakeAmt = 3n * DOLLARS;
      const userStakeAmt = 100n * CENTS;
      const userIncrementAmt = 1n * CENTS;

      const [stakeKeys, stakeProviderId] = await createAndStakeProvider("TinyStake", providerStakeAmt);
      // new user/msa stakes to provider
      const userKeys = createKeys("userKeys");
      await fundKeypair(fundingSource, userKeys, 5n * DOLLARS);
      let [_, events] = await ExtrinsicHelper.stake(userKeys, stakeProviderId, userStakeAmt).fundAndSend(fundingSource);
      assertEvent(events, 'system.ExtrinsicSuccess');

      const payload = JSON.stringify({ changeType: 1, fromId: 1, objectId: 2 })
      const call = ExtrinsicHelper.addOnChainMessage(stakeKeys, schemaId, payload);

      let expectedCapacity = (providerStakeAmt + userStakeAmt) / TokenPerCapacity;
      const totalCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
      assert.equal(expectedCapacity, totalCapacity, `expected ${expectedCapacity} capacity, got ${totalCapacity}`);

      const callCapacityCost = await drainCapacity(call, stakeProviderId, stakeKeys);

      // ensure provider can't send a message; they are out of capacity
      await assert_capacity_call_fails_with_balance_too_low(call);

      // go to next epoch
      let nextEpochBlock = await getNextEpochBlock();
      await ExtrinsicHelper.runToBlock(nextEpochBlock);

      let remainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
      // double check we still do not have enough to send another message
      assert(remainingCapacity < callCapacityCost);

      // user stakes tiny additional amount
      [_, events] = await ExtrinsicHelper.stake(userKeys, stakeProviderId, userIncrementAmt).fundAndSend(fundingSource);
      assertEvent(events, 'capacity.Staked');

      // provider can now send a message
      [_, events] = await call.payWithCapacity(-1);
      assertEvent(events, 'capacity.CapacityWithdrawn');

      remainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
      // show that capacity was replenished and then fee deducted.
      let approxExpected = providerStakeAmt + userStakeAmt + userIncrementAmt - callCapacityCost;
      assert(remainingCapacity <= approxExpected, `remainingCapacity = ${remainingCapacity.toString()}`);
    });
  });
});

async function drainCapacity(call, stakeProviderId: u64, stakeKeys: KeyringPair): Promise<bigint> {
  const totalCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
  // Figure out the cost per call in Capacity
  await call.payWithCapacity(-1);
  let remainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
  const callCapacityCost = totalCapacity - remainingCapacity;

  // Run them out of funds, but don't flake just because it landed near an epoch boundary.
  await ExtrinsicHelper.runToBlock(await getNextEpochBlock());
  const callsBeforeEmpty = Math.floor(Number(totalCapacity) / Number(callCapacityCost));
  const nonce = await getNonce(stakeKeys);
  await Promise.all(Array.from({ length: callsBeforeEmpty }, (_, k) => call.payWithCapacity(nonce + k)));
  return callCapacityCost;
}
