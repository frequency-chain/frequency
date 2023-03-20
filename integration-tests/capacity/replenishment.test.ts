import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import {Null, u16, u64, u128} from "@polkadot/types"
import assert from "assert";
import {EventMap, ExtrinsicHelper} from "../scaffolding/extrinsicHelpers";
import {
  devAccounts,
  createKeys,
  createMsaAndProvider,
  stakeToProvider,
  fundKeypair,
  getNextEpochBlock,
  TEST_EPOCH_LENGTH
} from "../scaffolding/helpers";
import { firstValueFrom} from "rxjs";
import {AVRO_GRAPH_CHANGE} from "../schemas/fixtures/avroGraphChangeSchemaType";


describe("Capacity Replenishment Testing: ", function () {
  const STARTING_BALANCE=6n* 1000n * 1000n + 100n*1000n*1000n;

  let schemaId: u16;

  async function createGraphChangeSchema() {
    const keys = createKeys('SchemaCreatorKeys');
    await fundKeypair(devAccounts[0].keys, keys, STARTING_BALANCE);
    const [createSchemaEvent, eventMap]  = await ExtrinsicHelper
      .createSchema(keys, AVRO_GRAPH_CHANGE, "AvroBinary", "OnChain")
      .fundAndSend();
    assert.notEqual(eventMap["system.ExtrinsicSuccess"], undefined);
    if (createSchemaEvent && ExtrinsicHelper.api.events.schemas.SchemaCreated.is(createSchemaEvent)) {
      schemaId = createSchemaEvent.data.schemaId;
    } else {
      assert.fail("failed to create a schema")
    }
  }

  async function getRemainingCapacity(providerId: u64): Promise<u128> {
    const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(providerId))).unwrap();
    return capacityStaked.remainingCapacity;
  }

  async function createAndStakeProvider(name: string, stakingAmount: bigint): Promise<[KeyringPair, u64]> {
    const stakeKeys = createKeys(name);
    const stakeProviderId = await createMsaAndProvider(stakeKeys, "ReplProv", STARTING_BALANCE);
    assert.notEqual(stakeProviderId, 0, "stakeProviderId should not be zero");
    await stakeToProvider(stakeKeys, stakeProviderId, stakingAmount);
    return [stakeKeys, stakeProviderId];
  }

  function assertEvent(events: EventMap, eventName: string) {
    assert(events.hasOwnProperty(eventName));
  }

  before(async function() {
    const setEpochLengthOp = ExtrinsicHelper.setEpochLength(devAccounts[0].keys, TEST_EPOCH_LENGTH);
    const [setEpochLengthEvent] = await setEpochLengthOp.sudoSignAndSend();
    const epoch_was_set = setEpochLengthEvent && ExtrinsicHelper.api.events.capacity.EpochLengthUpdated.is(setEpochLengthEvent);
    assert(epoch_was_set, "failed to set epoch");
    await createGraphChangeSchema();
  });

  describe("Capacity is replenished", function(){
    it("after new epoch", async function() {
      const totalCapacity = 3n*1500n*1000n;
      const [stakeKeys, stakeProviderId] = await createAndStakeProvider("ReplFirst", totalCapacity);
      const payload = JSON.stringify({ changeType: 1,  fromId: 1, objectId: 2 })
      // const call = ExtrinsicHelper.api.tx.messages.addOnchainMessage(null, schemaId, payload);
      const call = ExtrinsicHelper.addOnChainMessage(stakeKeys, schemaId, payload);

      // confirm that we start with a full tank
      let remainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
      assert.equal(totalCapacity, remainingCapacity);

      await call.payWithCapacity(-1);
      remainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
      assert(totalCapacity > remainingCapacity);

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

  describe("Capacity is not replenished", function() {
    it("if out of capacity and last_replenished_at is <= current epoch", async function() {
      let [stakeKeys, stakeProviderId] = await createAndStakeProvider("NoSend", 3n*1000n*1000n);
      let payload = JSON.stringify({ changeType: 1,  fromId: 1, objectId: 2 })
      let call = ExtrinsicHelper.addOnChainMessage(stakeKeys, schemaId, payload);

      // run until we can't afford to send another message.
      await call.payWithCapacity(-1);

      let nextEpochBlock = await getNextEpochBlock();

      // mine until just before epoch rolls over
      await ExtrinsicHelper.run_to_block(nextEpochBlock - 1);

      // TODO: it's weird that we can run one more capacity txn before running out.
      await call.payWithCapacity(-1)

      await assert.rejects(
        call.payWithCapacity(-1), {name: "RpcError", message: "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low"});
    });
  });

  describe("Regression test: when user attempts to stake tiny amounts before provider's first message of an epoch,", function() {
    it("provider is still replenished and can send a message", async function(){
      const providerStakeAmt = 2n*1000n*1000n;
      const userStakeAmt = 1n*1000n*1000n;
      const userIncrementAmt = 1000n;

      const [stakeKeys, stakeProviderId] = await createAndStakeProvider("TinyStake", 1n*1000n*1000n);
      // new user/msa stakes to provider
      const userKeys = createKeys("userKeys");
      await fundKeypair(devAccounts[0].keys, userKeys, 2n*1000n*1000n + 100n*1000n*1000n);
      await ExtrinsicHelper.createMsa(userKeys).fundAndSend();
      let [_, events] = await ExtrinsicHelper.stake(userKeys, stakeProviderId, userStakeAmt).fundAndSend();
      assertEvent(events, 'system.ExtrinsicSuccess');

      const payload = JSON.stringify({ changeType: 1,  fromId: 1, objectId: 2 })
      const call = ExtrinsicHelper.addOnChainMessage(stakeKeys, schemaId, payload);

      const totalCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt()
      assert.equal(providerStakeAmt, totalCapacity);

      await call.payWithCapacity(-1)

      // ensure provider can't send a message; they are out of capacity
      await assert.rejects(call.payWithCapacity(-1),
        {  name: "RpcError",
           message: "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low"
        });

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
