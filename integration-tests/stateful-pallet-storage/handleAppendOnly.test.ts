// Integration tests for pallets/stateful-pallet-storage/handleItemizedWithSignature.ts
import "@frequency-chain/api-augment";
import assert from "assert";
import {
  createDelegator,
  createDelegatorAndDelegation,
  createProviderKeysAndId,
  generateItemizedSignaturePayload, generatePaginatedDeleteSignaturePayload, generatePaginatedUpsertSignaturePayload,
  getCurrentItemizedHash, getCurrentPaginatedHash,
  signPayloadSr25519
} from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { AVRO_CHAT_MESSAGE } from "./fixtures/itemizedSchemaType";
import { MessageSourceId, SchemaId } from "@frequency-chain/api-augment/interfaces";
import {Bytes, u16} from "@polkadot/types";

describe("📗 Stateful Pallet Storage AppendOnly Schemas", () => {
    let itemizedSchemaId: SchemaId;
    let paginatedSchemaId: SchemaId;
    let msa_id: MessageSourceId;
    let providerId: MessageSourceId;
    let providerKeys: KeyringPair;
    let delegatorKeys: KeyringPair;

    before(async function () {

        // Create a provider for the MSA, the provider will be used to grant delegation
        [providerKeys, providerId] = await createProviderKeysAndId();
        assert.notEqual(providerId, undefined, "setup should populate providerId");
        assert.notEqual(providerKeys, undefined, "setup should populate providerKeys");

        // Create a schema for Itemized PayloadLocation
        const createSchema = ExtrinsicHelper.createSchemaWithSettings(providerKeys, AVRO_CHAT_MESSAGE, "AvroBinary", "Itemized", "AppendOnly");
        const [event] = await createSchema.fundAndSend();
        if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
            itemizedSchemaId = event.data.schemaId;
        }
        assert.notEqual(itemizedSchemaId, undefined, "setup should populate schemaId");
        // Create a schema for Paginated PayloadLocation
        const createSchema2 = ExtrinsicHelper.createSchemaWithSettings(providerKeys, AVRO_CHAT_MESSAGE, "AvroBinary", "Paginated", "AppendOnly");
        const [event2] = await createSchema2.fundAndSend();
        assert.notEqual(event2, undefined, "setup should return a SchemaCreated event");
        if (event2 && createSchema2.api.events.schemas.SchemaCreated.is(event2)) {
            paginatedSchemaId = event2.data.schemaId;
            assert.notEqual(paginatedSchemaId, undefined, "setup should populate schemaId");
        }

        // Create a MSA for the delegator and delegate to the provider for the itemized schema
        [, msa_id] = await createDelegatorAndDelegation(itemizedSchemaId, providerId, providerKeys);
        assert.notEqual(msa_id, undefined, "setup should populate msa_id");

        // Create a MSA for the delegator and delegate to the provider for the paginated schema
        [, msa_id] = await createDelegatorAndDelegation(paginatedSchemaId, providerId, providerKeys);
        assert.notEqual(msa_id, undefined, "setup should populate msa_id");
    });

    describe("Itemized With AppendOnly Storage Tests", () => {

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
        await assert.rejects(async () => {
          await itemized_add_result_1.fundAndSend();
        }, {
          name: 'SchemaNotSupported',
          section: 'statefulStorage',
      });
    });
  });

  describe("Paginated With AppendOnly Storage Tests", () => {

    it("should not be able to call delete an AppendOnly page", async function () {
      let page_id = new u16(ExtrinsicHelper.api.registry, 1);

      // Add and update actions
      let target_hash = await getCurrentPaginatedHash(msa_id, paginatedSchemaId, page_id.toNumber());
      const upsertPayload = await generatePaginatedUpsertSignaturePayload({
        msaId: msa_id,
        targetHash: target_hash,
        schemaId: paginatedSchemaId,
        pageId: page_id,
        payload: new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency"),
      });
      const upsertPayloadData = ExtrinsicHelper.api.registry.createType("PalletStatefulStoragePaginatedUpsertSignaturePayload", upsertPayload);
      let paginated_add_result_1 = ExtrinsicHelper.upsertPage(providerKeys, paginatedSchemaId, msa_id, page_id, upsertPayloadData, target_hash);
      const [pageUpdateEvent1, chainEvents] = await paginated_add_result_1.fundAndSend();
      assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
      assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
      assert.notEqual(pageUpdateEvent1, undefined, "should have returned a PalletStatefulStoragepaginatedActionApplied event");

      // Remove the second page
      target_hash = await getCurrentPaginatedHash(msa_id, paginatedSchemaId, 1)
      let paginated_remove_result_1 = ExtrinsicHelper.removePage(providerKeys, paginatedSchemaId, msa_id, page_id, target_hash);
      await assert.rejects(async () => { 
        await paginated_remove_result_1.fundAndSend();
      }, {
        name: 'SchemaNotSupported',
        section: 'statefulStorage',
      });
      // pages should exist
      const result = await ExtrinsicHelper.getPaginatedStorages(msa_id, paginatedSchemaId);
      assert.notEqual(result, undefined, "should have returned a valid response");
      assert.notEqual(result.length, 0, "should returned no paginated pages");
    }).timeout(10000);
  });
});