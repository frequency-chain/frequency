// E2E tests for pallets/stateful-pallet-storage/handleItemizedWithSignature.ts
import "@frequency-chain/api-augment";
import assert from "assert";
import {
  createDelegator,
  createProviderKeysAndId,
  generateItemizedSignaturePayload,
  generateItemizedSignaturePayloadV2,
  generatePaginatedDeleteSignaturePayload, generatePaginatedDeleteSignaturePayloadV2,
  generatePaginatedUpsertSignaturePayload, generatePaginatedUpsertSignaturePayloadV2,
  getCurrentItemizedHash,
  getCurrentPaginatedHash,
  signPayloadSr25519
} from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { AVRO_CHAT_MESSAGE } from "../stateful-pallet-storage/fixtures/itemizedSchemaType";
import { MessageSourceId, SchemaId } from "@frequency-chain/api-augment/interfaces";
import {Bytes, u16} from "@polkadot/types";

describe("📗 Stateful Pallet Storage Signature Required", () => {
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
        const createSchema = ExtrinsicHelper.createSchema(providerKeys, AVRO_CHAT_MESSAGE, "AvroBinary", "Itemized");
        const [event] = await createSchema.fundAndSend();
        if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
            itemizedSchemaId = event.data.schemaId;
        }
        assert.notEqual(itemizedSchemaId, undefined, "setup should populate schemaId");
      // Create a schema for Paginated PayloadLocation
        const createSchema2 = ExtrinsicHelper.createSchema(providerKeys, AVRO_CHAT_MESSAGE, "AvroBinary", "Paginated");
        const [event2] = await createSchema2.fundAndSend();
        assert.notEqual(event2, undefined, "setup should return a SchemaCreated event");
        if (event2 && createSchema2.api.events.schemas.SchemaCreated.is(event2)) {
            paginatedSchemaId = event2.data.schemaId;
            assert.notEqual(paginatedSchemaId, undefined, "setup should populate schemaId");
        }

        // Create a MSA for the delegator
        [delegatorKeys, msa_id] = await createDelegator();
        assert.notEqual(delegatorKeys, undefined, "setup should populate delegator_key");
        assert.notEqual(msa_id, undefined, "setup should populate msa_id");
    });

    describe("Itemized With Signature Storage Tests", () => {

      it("should be able to call applyItemizedActionWithSignature and apply actions", async function () {

        // Add and update actions
        let payload_1 = new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency");

        const add_action = {
          "Add": payload_1
        }

        let payload_2 = new Bytes(ExtrinsicHelper.api.registry, "Hello World Again From Frequency");

        const update_action = {
          "Add": payload_2
        }

        const target_hash = await getCurrentItemizedHash(msa_id, itemizedSchemaId);

        let add_actions = [add_action, update_action];
        const payload = await generateItemizedSignaturePayload({
          msaId: msa_id,
          targetHash: target_hash,
          schemaId: itemizedSchemaId,
          actions: add_actions,
        });
        const itemizedPayloadData = ExtrinsicHelper.api.registry.createType("PalletStatefulStorageItemizedSignaturePayload", payload);
        let itemized_add_result_1 = ExtrinsicHelper.applyItemActionsWithSignature(delegatorKeys, providerKeys, signPayloadSr25519(delegatorKeys, itemizedPayloadData), payload);
        const [pageUpdateEvent1, chainEvents] = await itemized_add_result_1.fundAndSend();
        assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
        assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
        assert.notEqual(pageUpdateEvent1, undefined, "should have returned a PalletStatefulStorageItemizedActionApplied event");
      }).timeout(10000);

      it("should be able to call applyItemizedActionWithSignatureV2 and apply actions", async function () {

        // Add and update actions
        let payload_1 = new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency");

        const add_action = {
          "Add": payload_1
        }

        let payload_2 = new Bytes(ExtrinsicHelper.api.registry, "Hello World Again From Frequency");

        const update_action = {
          "Add": payload_2
        }

        const target_hash = await getCurrentItemizedHash(msa_id, itemizedSchemaId);

        let add_actions = [add_action, update_action];
        const payload = await generateItemizedSignaturePayloadV2({
          targetHash: target_hash,
          schemaId: itemizedSchemaId,
          actions: add_actions,
        });
        const itemizedPayloadData = ExtrinsicHelper.api.registry.createType("PalletStatefulStorageItemizedSignaturePayloadV2", payload);
        let itemized_add_result_1 = ExtrinsicHelper.applyItemActionsWithSignatureV2(delegatorKeys, providerKeys, signPayloadSr25519(delegatorKeys, itemizedPayloadData), payload);
        const [pageUpdateEvent1, chainEvents] = await itemized_add_result_1.fundAndSend();
        assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
        assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
        assert.notEqual(pageUpdateEvent1, undefined, "should have returned a PalletStatefulStorageItemizedActionApplied event");
      }).timeout(10000);
    });

  describe("Paginated With Signature Storage Tests", () => {

    it("should be able to call upsert a page and delete it successfully", async function () {
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
      let upsert_result = ExtrinsicHelper.upsertPageWithSignature(delegatorKeys, providerKeys, signPayloadSr25519(delegatorKeys, upsertPayloadData), upsertPayload);
      const [pageUpdateEvent, chainEvents1] = await upsert_result.fundAndSend();
      assert.notEqual(chainEvents1["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
      assert.notEqual(chainEvents1["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
      assert.notEqual(pageUpdateEvent, undefined, "should have returned a PalletStatefulStoragePaginatedPageUpdate event");

      // Remove the page
      target_hash = await getCurrentPaginatedHash(msa_id, paginatedSchemaId, page_id.toNumber());
      const deletePayload = await generatePaginatedDeleteSignaturePayload({
        msaId: msa_id,
        targetHash: target_hash,
        schemaId: paginatedSchemaId,
        pageId: page_id,
      });
      const deletePayloadData = ExtrinsicHelper.api.registry.createType("PalletStatefulStoragePaginatedDeleteSignaturePayload", deletePayload);
      let remove_result = ExtrinsicHelper.deletePageWithSignature(delegatorKeys, providerKeys, signPayloadSr25519(delegatorKeys, deletePayloadData), deletePayload);
      const [pageRemove, chainEvents2] = await remove_result.fundAndSend();
      assert.notEqual(chainEvents2["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
      assert.notEqual(chainEvents2["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
      assert.notEqual(pageRemove, undefined, "should have returned a event");

      // no pages should exist
      const result = await ExtrinsicHelper.getPaginatedStorage(msa_id, paginatedSchemaId);
      assert.notEqual(result, undefined, "should have returned a valid response");
      assert.equal(result.length, 0, "should returned no paginated pages");
    }).timeout(10000);

    it("should be able to call upsertPageWithSignatureV2 a page and deletePageWithSignatureV2 it successfully", async function () {
      let page_id = new u16(ExtrinsicHelper.api.registry, 1);

      // Add and update actions
      let target_hash = await getCurrentPaginatedHash(msa_id, paginatedSchemaId, page_id.toNumber());
      const upsertPayload = await generatePaginatedUpsertSignaturePayloadV2({
        targetHash: target_hash,
        schemaId: paginatedSchemaId,
        pageId: page_id,
        payload: new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency"),
      });
      const upsertPayloadData = ExtrinsicHelper.api.registry.createType("PalletStatefulStoragePaginatedUpsertSignaturePayloadV2", upsertPayload);
      let upsert_result = ExtrinsicHelper.upsertPageWithSignatureV2(delegatorKeys, providerKeys, signPayloadSr25519(delegatorKeys, upsertPayloadData), upsertPayload);
      const [pageUpdateEvent, chainEvents1] = await upsert_result.fundAndSend();
      assert.notEqual(chainEvents1["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
      assert.notEqual(chainEvents1["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
      assert.notEqual(pageUpdateEvent, undefined, "should have returned a PalletStatefulStoragePaginatedPageUpdate event");

      // Remove the page
      target_hash = await getCurrentPaginatedHash(msa_id, paginatedSchemaId, page_id.toNumber());
      const deletePayload = await generatePaginatedDeleteSignaturePayloadV2({
        targetHash: target_hash,
        schemaId: paginatedSchemaId,
        pageId: page_id,
      });
      const deletePayloadData = ExtrinsicHelper.api.registry.createType("PalletStatefulStoragePaginatedDeleteSignaturePayloadV2", deletePayload);
      let remove_result = ExtrinsicHelper.deletePageWithSignatureV2(delegatorKeys, providerKeys, signPayloadSr25519(delegatorKeys, deletePayloadData), deletePayload);
      const [pageRemove, chainEvents2] = await remove_result.fundAndSend();
      assert.notEqual(chainEvents2["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
      assert.notEqual(chainEvents2["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
      assert.notEqual(pageRemove, undefined, "should have returned a event");

      // no pages should exist
      const result = await ExtrinsicHelper.getPaginatedStorage(msa_id, paginatedSchemaId);
      assert.notEqual(result, undefined, "should have returned a valid response");
      assert.equal(result.length, 0, "should returned no paginated pages");
    }).timeout(10000);
  });
});
