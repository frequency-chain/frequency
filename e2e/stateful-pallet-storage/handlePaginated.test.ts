// E2E tests for pallets/stateful-pallet-storage/handlePaginated.ts
import "@frequency-chain/api-augment";
import assert from "assert";
import {createProviderKeysAndId, createDelegatorAndDelegation, getCurrentPaginatedHash, createMsa} from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { AVRO_CHAT_MESSAGE } from "./fixtures/itemizedSchemaType";
import { MessageSourceId, SchemaId } from "@frequency-chain/api-augment/interfaces";
import { Bytes, u16, u64 } from "@polkadot/types";
import { getFundingSource } from "../scaffolding/funding";

describe("📗 Stateful Pallet Storage", function () {
    const fundingSource = getFundingSource("stateful-storage-handle-paginated");

    let schemaId: SchemaId;
    let schemaId_unsupported: SchemaId;
    let msa_id: MessageSourceId;
    let providerId: MessageSourceId;
    let providerKeys: KeyringPair;
    let badMsaId: u64;

    before(async function () {
        // Create a provider for the MSA, the provider will be used to grant delegation
        [providerKeys, providerId] = await createProviderKeysAndId(fundingSource);
        assert.notEqual(providerId, undefined, "setup should populate providerId");
        assert.notEqual(providerKeys, undefined, "setup should populate providerKeys");

        // Create a schema for Paginated PayloadLocation
        const createSchema = ExtrinsicHelper.createSchema(providerKeys, AVRO_CHAT_MESSAGE, "AvroBinary", "Paginated");
        const { target: event } = await createSchema.fundAndSend(fundingSource);
        schemaId = event!.data.schemaId;

        // Create non supported schema
        const createSchema2 = ExtrinsicHelper.createSchema(providerKeys, AVRO_CHAT_MESSAGE, "AvroBinary", "OnChain");
        const { target: event2 } = await createSchema2.fundAndSend(fundingSource);
        assert.notEqual(event2, undefined, "setup should return a SchemaCreated event");
        schemaId_unsupported = event2!.data.schemaId;

        // Create a MSA for the delegator and delegate to the provider
        [, msa_id] = await createDelegatorAndDelegation(fundingSource, schemaId, providerId, providerKeys);
        assert.notEqual(msa_id, undefined, "setup should populate msa_id");

        // Create an MSA that is not a provider to be used for testing failure cases
        [badMsaId] = await createMsa(fundingSource);
    });

    describe("Paginated Storage Upsert/Remove Tests 😊/😥", function () {

        it("✅ should be able to call upsert page and add a page and remove a page via id", async function () {
            let page_id = 0;
            let target_hash = await getCurrentPaginatedHash(msa_id, schemaId, page_id)

            // Add and update actions
            let payload_1 = new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency");
            let paginated_add_result_1 = ExtrinsicHelper.upsertPage(providerKeys, schemaId, msa_id, page_id, payload_1, target_hash);
            const { target: pageUpdateEvent1, eventMap: chainEvents } = await paginated_add_result_1.fundAndSend(fundingSource);
            assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
            assert.notEqual(pageUpdateEvent1, undefined, "should have returned a PalletStatefulStoragepaginatedActionApplied event");

            // Add another page
            page_id = 1;
            target_hash = await getCurrentPaginatedHash(msa_id, schemaId, page_id)
            let paginated_add_result_2 = ExtrinsicHelper.upsertPage(providerKeys, schemaId, msa_id, page_id, payload_1, target_hash);
            const { target: pageUpdateEvent2, eventMap: chainEvents2 } = await paginated_add_result_2.fundAndSend(fundingSource);
            assert.notEqual(chainEvents2["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(chainEvents2["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
            assert.notEqual(pageUpdateEvent2, undefined, "should have returned a PalletStatefulStoragepaginatedActionApplied event");

            // Remove the second page
            target_hash = await getCurrentPaginatedHash(msa_id, schemaId, page_id)
            let paginated_remove_result_1 = ExtrinsicHelper.removePage(providerKeys, schemaId, msa_id, page_id, target_hash);
            const { target: pageRemove, eventMap: chainEvents3 } = await paginated_remove_result_1.fundAndSend(fundingSource);
            assert.notEqual(chainEvents3["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(chainEvents3["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
            assert.notEqual(pageRemove, undefined, "should have returned a event");
        });

        it("🛑 should fail call to upsert page with invalid schemaId", async function () {

            let page_id = 0;
            let target_hash = await getCurrentPaginatedHash(msa_id, schemaId, page_id)
            let payload_1 = new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency");
            let fake_schema_id = new u16(ExtrinsicHelper.api.registry, 999);
            let paginated_add_result_1 = ExtrinsicHelper.upsertPage(providerKeys, fake_schema_id, msa_id, page_id, payload_1, target_hash);
            await assert.rejects(paginated_add_result_1.fundAndSend(fundingSource), {
                name: 'InvalidSchemaId',
                section: 'statefulStorage',
            });
        });

        it("🛑 should fail call to upsert page with invalid schema location", async function () {

            let page_id = 0;
            let target_hash = await getCurrentPaginatedHash(msa_id, schemaId, page_id)
            let payload_1 = new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency");
            let paginated_add_result_1 = ExtrinsicHelper.upsertPage(providerKeys, schemaId_unsupported, msa_id, page_id, payload_1, target_hash);
            await assert.rejects(paginated_add_result_1.fundAndSend(fundingSource), {
                name: 'SchemaPayloadLocationMismatch',
                section: 'statefulStorage',
            });
        });

        it("🛑 should fail call to upsert page with for un-delegated attempts", async function () {

            let page_id = 0;
            let payload_1 = new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency");

            let target_hash = await getCurrentPaginatedHash(msa_id, schemaId, page_id)
            let paginated_add_result_1 = ExtrinsicHelper.upsertPage(providerKeys, schemaId, badMsaId, page_id, payload_1, target_hash);
            await assert.rejects(paginated_add_result_1.fundAndSend(fundingSource), {
                name: 'UnauthorizedDelegate',
                section: 'statefulStorage',
            });
        });

        it("🛑 should fail call to upsert page with stale target hash", async function () {

            let page_id = 0;
            let payload_1 = new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency");

            let paginated_add_result_1 = ExtrinsicHelper.upsertPage(providerKeys, schemaId, msa_id, page_id, payload_1, 0);
            await assert.rejects(paginated_add_result_1.fundAndSend(fundingSource), {
                name: 'StalePageState',
                section: 'statefulStorage',
            });
        });
    });

    describe("Paginated Storage Removal Negative Tests 😊/😥", function () {

        it("🛑 should fail call to remove page with invalid schemaId", async function () {
            let fake_schema_id = 999;
            let page_id = 0;
            let paginated_add_result_1 = ExtrinsicHelper.removePage(providerKeys, fake_schema_id, msa_id, page_id, 0);
            await assert.rejects(paginated_add_result_1.fundAndSend(fundingSource), {
                name: 'InvalidSchemaId',
                section: 'statefulStorage',
            });
        });

        it("🛑 should fail call to remove page with invalid schema location", async function () {
            let page_id = 0;
            let paginated_add_result_1 = ExtrinsicHelper.removePage(providerKeys, schemaId_unsupported, msa_id, page_id, 0);
            await assert.rejects(paginated_add_result_1.fundAndSend(fundingSource), {
                name: 'SchemaPayloadLocationMismatch',
                section: 'statefulStorage',
            });
        });

        it("🛑 should fail call to remove page for un-delegated attempts", async function () {
            let paginated_add_result_1 = ExtrinsicHelper.removePage(providerKeys, schemaId, badMsaId, 0, 0);
            await assert.rejects(paginated_add_result_1.fundAndSend(fundingSource), {
                name: 'UnauthorizedDelegate',
                section: 'statefulStorage',
            });
        });

        it("🛑 should fail call to remove page with stale target hash", async function () {
            let paginated_add_result_1 = ExtrinsicHelper.removePage(providerKeys, schemaId, msa_id, 0, 0);
            await assert.rejects(paginated_add_result_1.fundAndSend(fundingSource), {
                name: 'StalePageState',
                section: 'statefulStorage',
            });
        });
    });

    describe("Paginated Storage RPC Tests", function () {
        it("✅ should be able to call get_paginated_storage and get paginated data", async function () {
            const result = await ExtrinsicHelper.getPaginatedStorage(msa_id, schemaId);
            assert.notEqual(result, undefined, "should have returned a valid response");
            assert.notEqual(result.length, 0, "should have returned paginated responses");
            assert.notEqual(result[0].hash, undefined, "should have returned a valid page");
        });
    });
});
