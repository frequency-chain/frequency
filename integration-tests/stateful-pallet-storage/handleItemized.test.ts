// Integration tests for pallets/stateful-pallet-storage/handleItemized.ts
import "@frequency-chain/api-augment";
import assert from "assert";
import { createDelegatorAndDelegation, createProviderKeysAndId } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { AVRO_CHAT_MESSAGE } from "../stateful-pallet-storage/fixtures/itemizedSchemaType";
import { MessageSourceId, SchemaId } from "@frequency-chain/api-augment/interfaces";
import { u16, u64 } from "@polkadot/types";

describe("📗 Stateful Pallet Storage", () => {
    let schemaId: SchemaId;
    let schemaId_unsupported: SchemaId;
    let msa_id: MessageSourceId;
    let providerId: MessageSourceId;
    let providerKeys: KeyringPair;

    before(async function () {
        
        // Create a provider for the MSA, the provider will be used to grant delegation
        [providerKeys, providerId] = await createProviderKeysAndId();
        assert.notEqual(providerId, undefined, "setup should populate providerId");
        assert.notEqual(providerKeys, undefined, "setup should populate providerKeys");

        // Create a schema for Itemized PayloadLocation
        const createSchema = ExtrinsicHelper.createSchema(providerKeys, AVRO_CHAT_MESSAGE, "AvroBinary", "Itemized");
        const [event] = await createSchema.fundAndSend();
        if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
            schemaId = event.data.schemaId;
        }
        assert.notEqual(schemaId, undefined, "setup should populate schemaId");
        // Create non supported schema
        const createSchema2 = ExtrinsicHelper.createSchema(providerKeys, AVRO_CHAT_MESSAGE, "AvroBinary", "OnChain");
        const [event2] = await createSchema2.fundAndSend();
        assert.notEqual(event2, undefined, "setup should return a SchemaCreated event");
        if (event2 && createSchema2.api.events.schemas.SchemaCreated.is(event2)) {
            schemaId_unsupported = event2.data.schemaId;
            assert.notEqual(schemaId_unsupported, undefined, "setup should populate schemaId_unsupported");
        }

        // Create a MSA for the delegator and delegate to the provider
        [ , msa_id] = await createDelegatorAndDelegation(schemaId, providerId, providerKeys);
        assert.notEqual(msa_id, undefined, "setup should populate msa_id");
    });

    describe("Itemized Storage Tests 😊/😥", () => {

        it("✅ should be able to call applyItemizedAction and apply actions", async function () {
            
            // Add and update actions
            const payload_1 = {
                "message": "Hello World",
            }
            const add_action = {
                "Add" : payload_1
            }

            const payload_2 =  {
                "message": "Hello World Again",
            }

            const update_action = {
                "Add" : payload_2
            }

            let add_actions = [add_action, update_action];
            let itemized_add_result_1 = ExtrinsicHelper.applyItemActions(providerKeys, schemaId, msa_id, add_actions);
            await itemized_add_result_1.fundOperation();
            const [pageUpdateEvent1, chainEvents ] = await itemized_add_result_1.signAndSend();
            assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
            assert.notEqual(pageUpdateEvent1, undefined, "should have returned a PalletStatefulStorageItemizedActionApplied event");
        }).timeout(10000);

        it("🛑 should fail to call applyItemizedAction with invalid schemaId", async function () {
            const payload_1 = {
                "message": "Hello World",
            }
            const add_action = {
                "Add" : payload_1
            }
            let add_actions = [add_action];
            let fake_schema_id = new u16(ExtrinsicHelper.api.registry, 999);      
            let itemized_add_result_1 = ExtrinsicHelper.applyItemActions(providerKeys, fake_schema_id, msa_id, add_actions);
            await itemized_add_result_1.fundOperation();
            await assert.rejects(async () => {
                await itemized_add_result_1.signAndSend();
            },{
                name: 'InvalidSchemaId',
                section: 'statefulStorage',
            });
        }).timeout(10000);
        
        it("🛑 should fail to call applyItemizedAction with invalid schema location", async function () {
            const payload_1 = {
                "message": "Hello World",
            }
            const add_action = {
                "Add" : payload_1
            }
            let add_actions = [add_action];    
            let itemized_add_result_1 = ExtrinsicHelper.applyItemActions(providerKeys, schemaId_unsupported, msa_id, add_actions);
            await itemized_add_result_1.fundOperation();
            await assert.rejects(async () => {
                await itemized_add_result_1.signAndSend();
            },{
                name: 'SchemaPayloadLocationMismatch',
                section: 'statefulStorage',
            });
        }).timeout(10000);

        it("🛑 should fail to call applyItemizedAction with for un-delegated attempts", async function () {
            const payload_1 = {
                "message": "Hello World",
            }
            const add_action = {
                "Add" : payload_1
            }
            let add_actions = [add_action];
            let bad_msa_id =  new u64(ExtrinsicHelper.api.registry, 999)

            let itemized_add_result_1 = ExtrinsicHelper.applyItemActions(providerKeys, schemaId, bad_msa_id, add_actions);
            await itemized_add_result_1.fundOperation();
            await assert.rejects(async () => {
                await itemized_add_result_1.signAndSend();
            },{
                name: 'UnAuthorizedDelegate',
                section: 'statefulStorage',
            });
        }).timeout(10000);
    });

    describe("Itemized Storage Remove Action Tests", () => {

        it("✅ should be able to call applyItemizedAction and apply remove actions", async function () {
            // Add and update actions
            const payload_1 = {
                "message": "Hello World",
            }
            const add_action = {
                "Add" : payload_1
            }

            const payload_2 =  {
                "message": "Hello World Again",
            }

            const update_action = {
                "Add" : payload_2
            }

            let add_actions = [add_action, update_action];
            let itemized_add_result_1 = ExtrinsicHelper.applyItemActions(providerKeys, schemaId, msa_id, add_actions);
            await itemized_add_result_1.fundOperation();
            const [pageUpdateEvent1, chainEvents ] = await itemized_add_result_1.signAndSend();
            assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
            assert.notEqual(pageUpdateEvent1, undefined, "should have returned a PalletStatefulStorageItemizedActionApplied event");
            // Delete action
            const idx_1: u16 = new u16(ExtrinsicHelper.api.registry, 1)
            const remove_action_1 ={
                "Delete": idx_1,
            }
            let remove_actions = [remove_action_1];
            let itemized_remove_result_1 = ExtrinsicHelper.applyItemActions(providerKeys, schemaId, msa_id, remove_actions);
            await itemized_remove_result_1.fundOperation();
            const [pageUpdateEvent2, chainEvents2 ] = await itemized_remove_result_1.signAndSend();
            assert.notEqual(chainEvents2["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(chainEvents2["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
            assert.notEqual(pageUpdateEvent2, undefined, "should have returned a event");
        }).timeout(10000);

        it("🛑 should fail to call remove action with invalid schemaId", async function () {
            const idx_1: u16 = new u16(ExtrinsicHelper.api.registry, 1)
            const remove_action_1 ={
                "Delete": idx_1,
            }
            let remove_actions = [remove_action_1];
            let fake_schema_id = new u16(ExtrinsicHelper.api.registry, 999);      
            let itemized_remove_result_1 = ExtrinsicHelper.applyItemActions(providerKeys, fake_schema_id, msa_id, remove_actions);
            await itemized_remove_result_1.fundOperation();
            await assert.rejects(async () => {
                await itemized_remove_result_1.signAndSend();
            }, {
                name: 'InvalidSchemaId',
                section: 'statefulStorage',
            });
        }).timeout(10000);

        it("🛑 should fail to call remove action with invalid schema location", async function () {
            const idx_1: u16 = new u16(ExtrinsicHelper.api.registry, 1)
            const remove_action_1 ={
                "Delete": idx_1,
            }
            let remove_actions = [remove_action_1];
            let itemized_remove_result_1 = ExtrinsicHelper.applyItemActions(providerKeys, schemaId_unsupported, msa_id, remove_actions);
            await itemized_remove_result_1.fundOperation();
            await assert.rejects(async () => {
                await itemized_remove_result_1.signAndSend();
            }, {
                name: 'SchemaPayloadLocationMismatch',
                section: 'statefulStorage',
            });
        }).timeout(10000);

        it("🛑 should fail to call remove action with invalid msa_id", async function () {
            const idx_1: u16 = new u16(ExtrinsicHelper.api.registry, 1)
            const remove_action_1 ={
                "Delete": idx_1,
            }
            let remove_actions = [remove_action_1];
            let bad_msa_id =  new u64(ExtrinsicHelper.api.registry, 999)
            let itemized_remove_result_1 = ExtrinsicHelper.applyItemActions(providerKeys, schemaId, bad_msa_id, remove_actions);
            await itemized_remove_result_1.fundOperation();
            await assert.rejects(async () => {
                await itemized_remove_result_1.signAndSend();
            }, {
                name: 'UnAuthorizedDelegate',
                section: 'statefulStorage',
            });
        }).timeout(10000);
    });

    describe("Itemized Storage RPC Tests", () => {
        it("✅ should be able to call getItemizedStorages and get data for itemized schema", async function () {
            const result = await ExtrinsicHelper.getItemizedStorages(msa_id, schemaId);
            assert.notEqual(result.hash, undefined, "should have returned a hash");
            assert.notEqual(result.size, undefined, "should have returned a itemized responses");
        }).timeout(10000);
    });
});