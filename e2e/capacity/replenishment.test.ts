import '@frequency-chain/api-augment';
import { KeyringPair } from '@polkadot/keyring/types';
import { u16, u64 } from '@polkadot/types';
import assert from 'assert';
import { Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  createKeys,
  createMsaAndProvider,
  stakeToProvider,
  fundKeypair,
  getNextEpochBlock,
  getOrCreateGraphChangeSchema,
  CENTS,
  DOLLARS,
  getTokenPerCapacity,
  assertEvent,
  getCapacity,
  getNonce,
} from '../scaffolding/helpers';
import { getFundingSource } from '../scaffolding/funding';
import { isTestnet } from '../scaffolding/env';

let fundingSource: KeyringPair;

describe('Capacity Replenishment Testing: ', function () {
  async function createAndStakeProvider(name: string, stakingAmount: bigint): Promise<[KeyringPair, u64]> {
    const stakeKeys = createKeys(name);
    const stakeProviderId = await createMsaAndProvider(fundingSource, stakeKeys, 'ReplProv', 50n * DOLLARS);
    assert.notEqual(stakeProviderId, 0, 'stakeProviderId should not be zero');
    await stakeToProvider(fundingSource, stakeKeys, stakeProviderId, stakingAmount);
    return [stakeKeys, stakeProviderId];
  }

  before(async function () {
    // Replenishment requires the epoch length to be shorter than testnet (set in globalHooks)
    if (isTestnet()) this.skip();
    fundingSource = await getFundingSource(import.meta.url);
  });

  describe('Capacity is replenished', function () {
    it('after new epoch', async function () {
      const schemaId = await getOrCreateGraphChangeSchema(fundingSource);
      const totalStaked = 3n * DOLLARS;
      const expectedCapacity = totalStaked / getTokenPerCapacity();
      const [stakeKeys, stakeProviderId] = await createAndStakeProvider('ReplFirst', totalStaked);
      const payload = JSON.stringify({ changeType: 1, fromId: 1, objectId: 2 });
      const call = ExtrinsicHelper.addOnChainMessage(stakeKeys, schemaId, payload);
      let nonce = await getNonce(stakeKeys);

      // confirm that we start with a full tank
      await ExtrinsicHelper.runToBlock(await getNextEpochBlock());
      let remainingCapacity = (await getCapacity(stakeProviderId)).remainingCapacity.toBigInt();
      assert.equal(expectedCapacity, remainingCapacity, 'Our expected capacity from staking is wrong');

      await call.payWithCapacity(nonce++);
      remainingCapacity = (await getCapacity(stakeProviderId)).remainingCapacity.toBigInt();
      assert(expectedCapacity > remainingCapacity, 'Our remaining capacity is much higher than expected.');
      const capacityPerCall = expectedCapacity - remainingCapacity;
      assert(
        remainingCapacity > capacityPerCall,
        `Not enough capacity! needed: ${capacityPerCall}, remaining: ${remainingCapacity}`
      );

      // one more txn to deplete capacity more so this current remaining is different from when
      // we submitted the first message.
      await call.payWithCapacity(nonce++);
      await ExtrinsicHelper.runToBlock(await getNextEpochBlock());

      // this should cause capacity to be refilled and then deducted by the cost of one message.
      await call.payWithCapacity(nonce++);
      const newRemainingCapacity = (await getCapacity(stakeProviderId)).remainingCapacity.toBigInt();

      // this should be the same as after sending the first message, since it is the first message after
      // the epoch.
      assert.equal(remainingCapacity, newRemainingCapacity);
    });
  });

  function assert_capacity_call_fails_with_balance_too_low(call: Extrinsic) {
    return assert.rejects(call.payWithCapacity('current'), {
      name: 'RpcError',
      message: /1010.+account balance too low/,
    });
  }

  describe('Capacity is not replenished', function () {
    it('if out of capacity and last_replenished_at is <= current epoch', async function () {
      const schemaId = await getOrCreateGraphChangeSchema(fundingSource);
      const [stakeKeys, stakeProviderId] = await createAndStakeProvider('NoSend', 150n * CENTS);
      const payload = JSON.stringify({ changeType: 1, fromId: 1, objectId: 2 });
      const call = ExtrinsicHelper.addOnChainMessage(stakeKeys, schemaId, payload);

      // run until we can't afford to send another message.
      await drainCapacity(call, stakeProviderId);

      await assert_capacity_call_fails_with_balance_too_low(call);
    });
  });

  describe("Regression test: when user attempts to stake tiny amounts before provider's first message of an epoch,", function () {
    it('provider is still replenished and can send a message', async function () {
      const providerStakeAmt = 3n * DOLLARS;
      const userStakeAmt = 100n * CENTS;
      const userIncrementAmt = 1n * CENTS;

      const [stakeKeys, stakeProviderId] = await createAndStakeProvider('TinyStake', providerStakeAmt);
      // new user/msa stakes to provider
      const userKeys = createKeys('userKeys');
      await fundKeypair(fundingSource, userKeys, 5n * DOLLARS);
      const { eventMap } = await ExtrinsicHelper.stake(userKeys, stakeProviderId, userStakeAmt).signAndSend();
      assertEvent(eventMap, 'system.ExtrinsicSuccess');

      const schemaId = await getOrCreateGraphChangeSchema(fundingSource);
      const payload = JSON.stringify({ changeType: 1, fromId: 1, objectId: 2 });
      const call = ExtrinsicHelper.addOnChainMessage(stakeKeys, schemaId, payload);

      const expectedCapacity = (providerStakeAmt + userStakeAmt) / getTokenPerCapacity();
      const totalCapacity = (await getCapacity(stakeProviderId)).totalCapacityIssued.toBigInt();
      assert.equal(expectedCapacity, totalCapacity, `expected ${expectedCapacity} capacity, got ${totalCapacity}`);

      const callCapacityCost = await drainCapacity(call, stakeProviderId);

      // ensure provider can't send a message; they are out of capacity
      await assert_capacity_call_fails_with_balance_too_low(call);

      // go to next epoch
      const nextEpochBlock = await getNextEpochBlock();
      await ExtrinsicHelper.runToBlock(nextEpochBlock);

      let remainingCapacity = (await getCapacity(stakeProviderId)).remainingCapacity.toBigInt();
      // double check we still do not have enough to send another message
      assert(remainingCapacity < callCapacityCost);

      // user stakes tiny additional amount
      const { eventMap: hasStaked } = await ExtrinsicHelper.stake(
        userKeys,
        stakeProviderId,
        userIncrementAmt
      ).signAndSend();
      assertEvent(hasStaked, 'capacity.Staked');

      // provider can now send a message
      const { eventMap: hasCapacityWithdrawn } = await call.payWithCapacity();
      assertEvent(hasCapacityWithdrawn, 'capacity.CapacityWithdrawn');

      remainingCapacity = (await getCapacity(stakeProviderId)).remainingCapacity.toBigInt();
      // show that capacity was replenished and then fee deducted.
      const approxExpected = providerStakeAmt + userStakeAmt + userIncrementAmt - callCapacityCost;
      assert(remainingCapacity <= approxExpected, `remainingCapacity = ${remainingCapacity.toString()}`);
    });
  });
});

async function drainCapacity(call, stakeProviderId: u64): Promise<bigint> {
  const totalCapacity = (await getCapacity(stakeProviderId)).totalCapacityIssued.toBigInt();
  let nonce = await getNonce(call.keys);
  // Figure out the cost per call in Capacity
  const { eventMap } = await call.payWithCapacity(nonce++);

  const callCapacityCost = eventMap['capacity.CapacityWithdrawn'].data.amount.toBigInt();

  // // Run them out of funds, but don't flake just because it landed near an epoch boundary.
  await ExtrinsicHelper.runToBlock(await getNextEpochBlock());
  const callsBeforeEmpty = Math.floor(Number(totalCapacity) / Number(callCapacityCost));
  await Promise.all(Array.from({ length: callsBeforeEmpty }, (_, k) => call.payWithCapacity(nonce + k)));
  return callCapacityCost;
}
