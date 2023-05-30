import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { u16, u64 } from "@polkadot/types"
import assert from "assert";
import { Extrinsic, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import {
  devAccounts,
  createKeys,
  createMsaAndProvider,
  stakeToProvider,
  fundKeypair,
  getNextEpochBlock,
  TEST_EPOCH_LENGTH,
  setEpochLength,
  getOrCreateGraphChangeSchema,
  CENTS,
  DOLLARS,
  TokenPerCapacity,
  assertEvent,
  getRemainingCapacity
} from "../scaffolding/helpers";

describe("Capacity Replenishment Testing: ", function () {
  let schemaId: u16;


  async function createAndStakeProvider(name: string, stakingAmount: bigint): Promise<[KeyringPair, u64]> {
    const stakeKeys = createKeys(name);
    const stakeProviderId = await createMsaAndProvider(stakeKeys, "ReplProv", 50n * DOLLARS);
    assert.notEqual(stakeProviderId, 0, "stakeProviderId should not be zero");
    await stakeToProvider(stakeKeys, stakeProviderId, stakingAmount);
    return [stakeKeys, stakeProviderId];
  }


  before(async function () {
    await setEpochLength(devAccounts[0].keys, TEST_EPOCH_LENGTH);
    schemaId = await getOrCreateGraphChangeSchema();
  });

  describe("Capacity is replenished", function () {
    it("after new epoch", async function () {
      const totalStaked = 2n * DOLLARS;
      let expectedCapacity = totalStaked / TokenPerCapacity;
      const [stakeKeys, stakeProviderId] = await createAndStakeProvider("ReplFirst", totalStaked);
      const payload = JSON.stringify({ changeType: 1, fromId: 1, objectId: 2 })
      const call = ExtrinsicHelper.addOnChainMessage(stakeKeys, schemaId, payload);

      // confirm that we start with a full tank
      let remainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
      assert.equal(expectedCapacity, remainingCapacity);

      await call.payWithCapacity(-1);
      remainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
      assert(expectedCapacity > remainingCapacity);

      // one more txn to deplete capacity more so this current remaining is different from when
      // we submitted the first message.
      await call.payWithCapacity(-1);
      const newEpochBlock = await getNextEpochBlock();
      await ExtrinsicHelper.run_to_block(newEpochBlock);

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
      let [stakeKeys, _] = await createAndStakeProvider("NoSend", 150n * CENTS);
      let payload = JSON.stringify({ changeType: 1, fromId: 1, objectId: 2 })
      let call = ExtrinsicHelper.addOnChainMessage(stakeKeys, schemaId, payload);

      // run until we can't afford to send another message.
      await call.payWithCapacity(-1);

      let nextEpochBlock = await getNextEpochBlock();

      // mine until just before epoch rolls over
      await ExtrinsicHelper.run_to_block(nextEpochBlock - 1);

      // TODO: it's weird that we can run one more capacity txn before running out.
      await call.payWithCapacity(-1)

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
      await fundKeypair(devAccounts[0].keys, userKeys, 5n* DOLLARS);
      let [_, events] = await ExtrinsicHelper.stake(userKeys, stakeProviderId, userStakeAmt).fundAndSend();
      assertEvent(events, 'system.ExtrinsicSuccess');

      const payload = JSON.stringify({ changeType: 1, fromId: 1, objectId: 2 })
      const call = ExtrinsicHelper.addOnChainMessage(stakeKeys, schemaId, payload);

      let expected_capacity = (providerStakeAmt + userStakeAmt)/TokenPerCapacity;
      const totalCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt()
      assert.equal(expected_capacity, totalCapacity, `expected ${expected_capacity} capacity, got ${totalCapacity}`);

      // do the txn a few times to run them out of funds
      await call.payWithCapacity(-1)
      await call.payWithCapacity(-1)
      await call.payWithCapacity(-1)
      await call.payWithCapacity(-1)

      // ensure provider can't send a message; they are out of capacity
      await assert_capacity_call_fails_with_balance_too_low(call);

      // go to next epoch
      let nextEpochBlock = await getNextEpochBlock();
      await ExtrinsicHelper.run_to_block(nextEpochBlock);

      let remainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
      let callCapacityCost = totalCapacity - remainingCapacity;
      // double check we still do not have enough to send another message
      assert(remainingCapacity < callCapacityCost);

      // user stakes tiny additional amount
      [_, events] = await ExtrinsicHelper.stake(userKeys, stakeProviderId, userIncrementAmt).fundAndSend();
      assertEvent(events, 'capacity.Staked');

      // provider can now send a message
      [_, events] = await call.payWithCapacity(-1);
      assertEvent(events, 'capacity.CapacityWithdrawn');

      remainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
      // show that capacity was replenished and then fee deducted.
      let approxExpected = providerStakeAmt + userStakeAmt + userIncrementAmt - callCapacityCost;
      assert(remainingCapacity <= approxExpected, `remainingCapacity = ${remainingCapacity.toString()}`);
    })
  })
});
