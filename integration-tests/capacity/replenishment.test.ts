import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import {Null, u16, u64, u128} from "@polkadot/types"
import assert from "assert";
import {Extrinsic, ExtrinsicHelper} from "../scaffolding/extrinsicHelpers";
import {
  devAccounts,
  log,
  createKeys,
  createMsaAndProvider,
  getBlockNumber,
  stakeToProvider, fundKeypair, assertDefined
} from "../scaffolding/helpers";
import { firstValueFrom} from "rxjs";
import {AVRO_GRAPH_CHANGE} from "../schemas/fixtures/avroGraphChangeSchemaType";
import type { } from '@polkadot/types-codec';
import {AugmentedSubmittable, SubmittableExtrinsic} from "@polkadot/api/types";


describe("Capacity Replenishment Testing: ", function () {
  const TEST_EPOCH_LENGTH=10;
  const STARTING_BALANCE=6n* 1000n * 1000n;

  let schemaId: u16;

  async function createGraphChangeSchema() {
    let keys = createKeys('SchemaCreatorKeys');
    await fundKeypair(devAccounts[0].keys, keys, STARTING_BALANCE);
    const f = ExtrinsicHelper.createSchema(keys, AVRO_GRAPH_CHANGE, "AvroBinary", "OnChain");
    const [createSchemaEvent, eventMap] = await f.fundAndSend();
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
    let stakeKeys = createKeys(name);
    let stakeProviderId = await createMsaAndProvider(stakeKeys, "ReplProv", STARTING_BALANCE);
    assert.notEqual(stakeProviderId, 0, "stakeProviderId should not be zero");
    await stakeToProvider(stakeKeys, stakeProviderId, stakingAmount);
    return [stakeKeys, stakeProviderId];
  }

  before(async function() {
    const setEpochLengthOp = ExtrinsicHelper.setEpochLength(devAccounts[0].keys, TEST_EPOCH_LENGTH);
    const [setEpochLengthEvent] = await setEpochLengthOp.sudoSignAndSend();
    let epoch_was_set = setEpochLengthEvent && ExtrinsicHelper.api.events.capacity.EpochLengthUpdated.is(setEpochLengthEvent);
    assert.equal(true, epoch_was_set, "failed to set epoch");
    await createGraphChangeSchema();
  });

  describe("Capacity is replenished", function(){
    it("when provider sends first message after new epoch", async function() {
      let totalCapacity = 3n*1500n*1000n;
      let [stakeKeys, stakeProviderId] = await createAndStakeProvider("ReplFirst", totalCapacity);
      let payload = JSON.stringify({ changeType: 1,  fromId: 1, objectId: 2 })
      let call = ExtrinsicHelper.api.tx.messages.addOnchainMessage(null, schemaId, payload);
      let remainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
      assert.equal(totalCapacity, remainingCapacity);

      await assert.doesNotReject(ExtrinsicHelper.payWithCapacity(stakeKeys, call).fundAndSend());
      remainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
      assert(totalCapacity > remainingCapacity);

      // one more txn to deplete capacity more, guaranteeing no funny business
      await assert.doesNotReject(ExtrinsicHelper.payWithCapacity(stakeKeys, call).fundAndSend());

      let epochInfo = await firstValueFrom(ExtrinsicHelper.api.query.capacity.currentEpochInfo())
      let newEpochBlock = epochInfo.epochStart.toNumber() + TEST_EPOCH_LENGTH + 1;
      await ExtrinsicHelper.run_to_block(newEpochBlock);

      // this should cause capacity to be refilled and then deducted by the cost of one message.
      await assert.doesNotReject(ExtrinsicHelper.payWithCapacity(stakeKeys, call).fundAndSend());
      let newRemainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();

      // this should be the same as after sending the first message, since it is the first message after
      // the epoch.
      assert.equal(remainingCapacity, newRemainingCapacity);
    });
    // it("and provider previously with no capacity can now pay for message with capacity", async function() {});
  });

  describe("Capacity is not replenished", function() {
    it("when provider sends message before new epoch", async function() {
      let [stakeKeys, stakeProviderId] = await createAndStakeProvider("NotRepl", 4n*1000n*1000n);

      let currentRemaining = await getRemainingCapacity(stakeProviderId);
      let payload = JSON.stringify({ changeType: 1,  fromId: 1,  objectId: 2 });
      let call: AugmentedSubmittable<any> = ExtrinsicHelper.api.tx.messages.addOnchainMessage(null, schemaId, payload);
      await assert.doesNotReject(ExtrinsicHelper.payWithCapacity(stakeKeys, call).fundAndSend());

      let remainingCapacity = await getRemainingCapacity(stakeProviderId);
      assert(currentRemaining.gt(remainingCapacity));
      currentRemaining = remainingCapacity;

      await assert.doesNotReject(ExtrinsicHelper.payWithCapacity(stakeKeys, call).fundAndSend());

      remainingCapacity = await getRemainingCapacity(stakeProviderId);
      assert(currentRemaining.gt(remainingCapacity));
    });
  });

  describe("Provider cannot send capacity message", function() {
    it("if out of capacity and last_replenished_at is <= current epoch", async function() {
      let [stakeKeys, stakeProviderId] = await createAndStakeProvider("NoSend", 3n*1000n*1000n);
      let payload = JSON.stringify({ changeType: 1,  fromId: 1, objectId: 2 })
      let call = ExtrinsicHelper.api.tx.messages.addOnchainMessage(null, schemaId, payload);
      const expectedApproxCost = 1891000n;

      // run until we can't afford to send another message
      for (let i=0; i< 5; i++) {
        await ExtrinsicHelper
          .payWithCapacity(stakeKeys, call)
          .fundAndSend();
        let remainingCapacity = (await getRemainingCapacity(stakeProviderId)).toBigInt();
        console.log("remaining capacity: ", remainingCapacity.toString())
        if (remainingCapacity < expectedApproxCost) {
          break;
        }
      }
      let epochInfo = await firstValueFrom(ExtrinsicHelper.api.query.capacity.currentEpochInfo())
      let blockBeforeNextEpoch = epochInfo.epochStart.toNumber() + TEST_EPOCH_LENGTH - 1

      // mine until just before epoch rolls over
      await ExtrinsicHelper.run_to_block(blockBeforeNextEpoch);

      // TODO: it's weird that we can run one more capacity txn before running out.
      await assert.doesNotReject(ExtrinsicHelper.payWithCapacity(stakeKeys, call).fundAndSend());

      await assert.rejects(
        ExtrinsicHelper.payWithCapacity(stakeKeys, call).fundAndSend(), {name: "RpcError", message: "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low"});
    });
  })

  // describe("DoS attack: when user attempts to stake tiny amounts before provider's first message of an epoch,", function() {
  //   it("provider is still replenished and can send a message", async function(){
  //
  //   })
  // })
});
