// Integration tests for pallets/stateful-pallet-storage/handlepaginated.ts
import "@frequency-chain/api-augment";
import assert from "assert";
import { createAndFundKeypair, generateDelegationPayload, signPayloadSr25519 } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { AVRO_CHAT_MESSAGE } from "./fixtures/itemizedSchemaType";
import { MessageSourceId, SchemaId } from "@frequency-chain/api-augment/interfaces";
import { u16, u32, u64 } from "@polkadot/types";

describe("📗 Stateful Pallet Storage", () => {
    let schemaId: SchemaId;
    let schemaId_unsupported: SchemaId;
    let msa_id: MessageSourceId;
    let providerId: MessageSourceId;
    let providerKeys: KeyringPair;

    before(async function () {
        // Create a provider for the MSA, the provider will be used to grant delegation
        providerKeys = await createAndFundKeypair();
        let createProviderMsaOp = ExtrinsicHelper.createMsa(providerKeys);
        await createProviderMsaOp.fundAndSend();
        let createProviderOp = ExtrinsicHelper.createProvider(providerKeys, "PrivateProvider");
        let [providerEvent] = await createProviderOp.fundAndSend();
        assert.notEqual(providerEvent, undefined, "setup should return a ProviderCreated event");
        if (providerEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(providerEvent)) {
            providerId = providerEvent.data.providerId;
        }
        assert.notEqual(providerId, undefined, "setup should populate providerId");

        // Create a schema for Paginated PayloadLocation
        const createSchema = ExtrinsicHelper.createSchema(providerKeys, AVRO_CHAT_MESSAGE, "AvroBinary", "Paginated");
        const [event] = await createSchema.fundAndSend();
        if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
            [, schemaId] = event.data;
        }
        assert.notEqual(schemaId, undefined, "setup should populate schemaId");
        // Create non supported schema
        const createSchema2 = ExtrinsicHelper.createSchema(providerKeys, AVRO_CHAT_MESSAGE, "AvroBinary", "OnChain");
        const [event2] = await createSchema2.fundAndSend();
        assert.notEqual(event2, undefined, "setup should return a SchemaCreated event");
        if (event2 && createSchema2.api.events.schemas.SchemaCreated.is(event2)) {
            const [, schemaId_unsupported] = event2.data;
            assert.notEqual(schemaId_unsupported, undefined, "setup should populate schemaId_unsupported");
        }
        
        let keys = await createAndFundKeypair();

        // Create a  delegator msa
        const createMsa = ExtrinsicHelper.createMsa(keys);
        await createMsa.fundOperation();
        const [msaCreatedEvent, chainEvents] = await createMsa.signAndSend();
        assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
        assert.notEqual(msaCreatedEvent, undefined, "should have returned  an MsaCreated event");
        assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
        if (msaCreatedEvent && ExtrinsicHelper.api.events.msa.MsaCreated.is(msaCreatedEvent)) {
            msa_id = msaCreatedEvent.data.msaId;
        }
        assert.notEqual(msa_id, undefined, "setup should populate msa_id");
        
        // Grant delegation to the provider
        const payload = await generateDelegationPayload({
            authorizedMsaId: providerId,
            schemaIds: [schemaId],
        });
        const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

        const grantDelegationOp = ExtrinsicHelper.grantDelegation(keys, providerKeys, signPayloadSr25519(keys, addProviderData), payload);
        await grantDelegationOp.fundOperation();
        const [grantDelegationEvent] = await grantDelegationOp.signAndSend();
        assert.notEqual(grantDelegationEvent, undefined, "should have returned DelegationGranted event");
        if (grantDelegationEvent && grantDelegationOp.api.events.msa.DelegationGranted.is(grantDelegationEvent)) {
            assert.deepEqual(grantDelegationEvent.data.providerId, providerId, 'provider IDs should match');
            assert.deepEqual(grantDelegationEvent.data.delegatorId, msa_id, 'delegator IDs should match');
        }
    });

    describe("Paginated Storage Upsert Tests 😊/😥", () => {

        it("✅ should be able to call upsert page and add a page and remove a page via id", async function () {
            
            // Add and update actions
            const payload_1 = {
                "message": "Hello World",
            }

            const payload_2 =  {
                "message": "Hello World Again",
            }

            let paginated_add_result_1 = ExtrinsicHelper.upsertPage(providerKeys, schemaId, msa_id, 0, payload_1);
            await paginated_add_result_1.fundOperation();
            const [pageUpdateEvent1, chainEvents ] = await paginated_add_result_1.signAndSend();
            assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
            assert.notEqual(pageUpdateEvent1, undefined, "should have returned a PalletStatefulStoragepaginatedActionApplied event");
            
            // Add another page 
            let paginated_add_result_2 = ExtrinsicHelper.upsertPage(providerKeys, schemaId, msa_id, 1, payload_2);
            await paginated_add_result_2.fundOperation();
            const [pageUpdateEvent2, chainEvents2 ] = await paginated_add_result_2.signAndSend();
            assert.notEqual(chainEvents2["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(chainEvents2["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
            assert.notEqual(pageUpdateEvent2, undefined, "should have returned a PalletStatefulStoragepaginatedActionApplied event");

            // Remove the second page
            let paginated_remove_result_1 = ExtrinsicHelper.removePage(providerKeys, schemaId, msa_id, 1);
            await paginated_remove_result_1.fundOperation();
            const [pageRemove, chainEvents3] = await paginated_remove_result_1.signAndSend();
            assert.notEqual(chainEvents3["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(chainEvents3["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
            assert.notEqual(pageRemove, undefined, "should have returned a event");
        }).timeout(10000);

        it("🛑 should fail to call upsert page with invalid schemaId", async function () {
            const payload_1 = {
                "message": "Hello World",
            }
            let fake_schema_id = new u16(ExtrinsicHelper.api.registry, 999);      
            let paginated_add_result_1 = ExtrinsicHelper.upsertPage(providerKeys, fake_schema_id, msa_id, 0, payload_1);
            await paginated_add_result_1.fundOperation();
            await assert.rejects(async () => {
                await paginated_add_result_1.signAndSend();
            }
            , (err) => {
                assert.notEqual(err, undefined, "should have returned an error");
                return true;
            });
        }).timeout(10000);
        
        it("🛑 should fail to call upsert page with invalid schema location", async function () {
            const payload_1 = {
                "message": "Hello World",
            }
            let paginated_add_result_1 = ExtrinsicHelper.upsertPage(providerKeys, schemaId_unsupported, msa_id, 0, payload_1);
            await paginated_add_result_1.fundOperation();
            await assert.rejects(async () => {
                await paginated_add_result_1.signAndSend();
            }
            , (err) => {
                assert.notEqual(err, undefined, "should have returned an error");
                return true;
            });
        }).timeout(10000);

        it("🛑 should fail to call upsert page with for un-delegated attempts", async function () {
            const payload_1 = {
                "message": "Hello World",
            }
            let bad_msa_id =  new u64(ExtrinsicHelper.api.registry, 999)

            let paginated_add_result_1 = ExtrinsicHelper.upsertPage(providerKeys, schemaId, bad_msa_id, 0, payload_1);
            await paginated_add_result_1.fundOperation();
            await assert.rejects(async () => {
                await paginated_add_result_1.signAndSend();
            }
            , (err) => {
                assert.notEqual(err, undefined, "should have returned an error");
                return true;
            });
        }).timeout(10000);

        it("🛑 should fail to call remove page with invalid page index", async function () {
            let bad_page_index =  new u32(ExtrinsicHelper.api.registry, 999)

            let paginated_add_result_1 = ExtrinsicHelper.removePage(providerKeys, schemaId, msa_id, bad_page_index);
            await paginated_add_result_1.fundOperation();
            await assert.rejects(async () => {
                await paginated_add_result_1.signAndSend();
            }
            , (err) => {
                assert.notEqual(err, undefined, "should have returned an error");
                return true;
            });
        }).timeout(10000);

        it("🛑 should fail to call remove page with invalid schemaId", async function () {
            let fake_schema_id = new u16(ExtrinsicHelper.api.registry, 999);      
            let paginated_add_result_1 = ExtrinsicHelper.removePage(providerKeys, fake_schema_id, msa_id, 0);
            await paginated_add_result_1.fundOperation();
            await assert.rejects(async () => {
                await paginated_add_result_1.signAndSend();
            }
            , (err) => {
                assert.notEqual(err, undefined, "should have returned an error");
                return true;
            });
        }).timeout(10000);

        it("🛑 should fail to call remove page with invalid schema location", async function () {
            let paginated_add_result_1 = ExtrinsicHelper.removePage(providerKeys, schemaId_unsupported, msa_id, 0);
            await paginated_add_result_1.fundOperation();
            await assert.rejects(async () => {
                await paginated_add_result_1.signAndSend();
            }
            , (err) => {
                assert.notEqual(err, undefined, "should have returned an error");
                return true;
            });
        }).timeout(10000);

        it("🛑 should fail to call remove page with for un-delegated attempts", async function () {
            let bad_msa_id =  new u64(ExtrinsicHelper.api.registry, 999)

            let paginated_add_result_1 = ExtrinsicHelper.removePage(providerKeys, schemaId, bad_msa_id, 0);
            await paginated_add_result_1.fundOperation();
            await assert.rejects(async () => {
                await paginated_add_result_1.signAndSend();
            }
            , (err) => {
                assert.notEqual(err, undefined, "should have returned an error");
                return true;
            });
        }).timeout(10000);
    });
});