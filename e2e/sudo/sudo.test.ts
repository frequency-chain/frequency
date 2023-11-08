// All the sudo required tests must be in one test for parallelization needs

import "@frequency-chain/api-augment";
import { MessageSourceId, SchemaId } from "@frequency-chain/api-augment/interfaces";
import { KeyringPair } from "@polkadot/keyring/types";
import assert from "assert";
import { Extrinsic, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { isTestnet } from "../scaffolding/env";
import { getSudo, getFundingSource } from "../scaffolding/funding";
import { AVRO_GRAPH_CHANGE } from "../schemas/fixtures/avroGraphChangeSchemaType";
import { Bytes, u16 } from "@polkadot/types";
import {
  DOLLARS,
  createDelegatorAndDelegation,
  createProviderKeysAndId,
  getCurrentItemizedHash,
} from "../scaffolding/helpers";
import { AVRO_CHAT_MESSAGE } from "../stateful-pallet-storage/fixtures/itemizedSchemaType";

describe("Sudo required", function () {
  let sudoKey: KeyringPair;
  let fundingSource: KeyringPair;

  before(function () {
    if (isTestnet()) this.skip();
    sudoKey = getSudo().keys;
    fundingSource = getFundingSource("sudo-transactions");
  });

  describe("schema#setMaxSchemaModelBytes", function () {
    it("should fail to set the schema size because of lack of root authority (BadOrigin)", async function () {
      const operation = new Extrinsic(() => ExtrinsicHelper.api.tx.schemas.setMaxSchemaModelBytes(1000000), sudoKey);
      await assert.rejects(operation.signAndSend(), { name: 'BadOrigin' });
    });

    it("should fail to set max schema size > hard-coded limit (SchemaModelMaxBytesBoundedVecLimit)", async function () {
      const limit = ExtrinsicHelper.api.consts.schemas.schemaModelMaxBytesBoundedVecLimit.toBigInt();
      const op = new Extrinsic(() => ExtrinsicHelper.api.tx.schemas.setMaxSchemaModelBytes(limit + 1n), sudoKey);
      await assert.rejects(op.sudoSignAndSend(), { name: 'ExceedsMaxSchemaModelBytes' });
    })

    it("should successfully set the max schema size", async function () {
      const size = (await ExtrinsicHelper.apiPromise.query.schemas.governanceSchemaModelMaxBytes()).toBigInt();
      const op = new Extrinsic(() => ExtrinsicHelper.api.tx.schemas.setMaxSchemaModelBytes(size + 1n), sudoKey);
      await op.sudoSignAndSend();
      assert.equal(true, true, 'operation should not have thrown error');
      const newSize = (await ExtrinsicHelper.apiPromise.query.schemas.governanceSchemaModelMaxBytes()).toBigInt();
      assert.equal(size + 1n, newSize, 'new size should have been set');
    });
  });

  describe("stateful-pallet-storage", function () {

    it("should fail to create non itemized schema with AppendOnly settings", async function () {
      if (isTestnet()) this.skip();

      const ex = ExtrinsicHelper.createSchemaWithSettingsGov(sudoKey, AVRO_GRAPH_CHANGE, "AvroBinary", "Paginated", "AppendOnly");
      await assert.rejects(ex.sudoSignAndSend(), {
        name: 'InvalidSetting'
      });
    });

    it("should not fail to create itemized schema with AppendOnly settings", async function () {
      if (isTestnet()) this.skip();

      const createSchema = ExtrinsicHelper.createSchemaWithSettingsGov(sudoKey, AVRO_GRAPH_CHANGE, "AvroBinary", "Itemized", "AppendOnly");
      const { target: event } = await createSchema.sudoSignAndSend();
      assert.notEqual(event, undefined);
      const itemizedSchemaId: u16 = event?.data.schemaId || new u16(ExtrinsicHelper.api.registry, 0);
      assert.notEqual(itemizedSchemaId.toNumber(), 0);
      let schema_response = await ExtrinsicHelper.getSchema(itemizedSchemaId);
      assert(schema_response.isSome);
      let schema_response_value = schema_response.unwrap();
      let schema_settings = schema_response_value.settings;
      assert.notEqual(schema_settings.length, 0);
    });


    describe("📗 Stateful Pallet Storage AppendOnly Schemas", function () {
      // This requires schema creation abilities

      let itemizedSchemaId: SchemaId;
      let msa_id: MessageSourceId;
      let providerId: MessageSourceId;
      let providerKeys: KeyringPair;

      before(async function () {
        // Create a provider for the MSA, the provider will be used to grant delegation
        [providerKeys, providerId] = await createProviderKeysAndId(fundingSource, 2n * DOLLARS);
        assert.notEqual(providerId, undefined, "setup should populate providerId");
        assert.notEqual(providerKeys, undefined, "setup should populate providerKeys");

        // Create a schema for Itemized PayloadLocation
        const createSchema = ExtrinsicHelper.createSchemaWithSettingsGov(sudoKey, AVRO_CHAT_MESSAGE, "AvroBinary", "Itemized", "AppendOnly");
        const { target: event } = await createSchema.sudoSignAndSend();
        itemizedSchemaId = event!.data.schemaId;

        // Create a MSA for the delegator and delegate to the provider for the itemized schema
        [, msa_id] = await createDelegatorAndDelegation(fundingSource, itemizedSchemaId, providerId, providerKeys);
        assert.notEqual(msa_id, undefined, "setup should populate msa_id");
      });

      describe("Itemized With AppendOnly Storage Tests", function () {

        it("should not be able to call delete action", async function () {

          // Add and update actions
          let payload_1 = new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency");

          const add_action = {
            "Add": payload_1
          }

          let payload_2 = new Bytes(ExtrinsicHelper.api.registry, "Hello World Again From Frequency");

          const update_action = {
            "Add": payload_2
          }

          const idx_1: u16 = new u16(ExtrinsicHelper.api.registry, 1)

          const delete_action = {
            "Delete": idx_1
          }
          const target_hash = await getCurrentItemizedHash(msa_id, itemizedSchemaId);

          let add_actions = [add_action, update_action, delete_action];

          let itemized_add_result_1 = ExtrinsicHelper.applyItemActions(providerKeys, itemizedSchemaId, msa_id, add_actions, target_hash);
          await assert.rejects(itemized_add_result_1.fundAndSend(fundingSource), {
            name: 'UnsupportedOperationForSchema',
            section: 'statefulStorage',
          });
        });
      });
    });
  });

});
