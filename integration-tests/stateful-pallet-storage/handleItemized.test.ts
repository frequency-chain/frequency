// Integration tests for pallets/stateful-pallet-storage/handleItemized.ts
import "@frequency-chain/api-augment";
import assert from "assert";
import { createAndFundKeypair, generateDelegationPayload, signPayloadSr25519 } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { AVRO_CHAT_MESSAGE } from "../stateful-pallet-storage/fixtures/itemizedSchemaType";
import { MessageSourceId, SchemaId } from "@frequency-chain/api-augment/interfaces";
import { u16 } from "@polkadot/types";

describe("Stateful Pallet Storage", () => {
    let schemaId: SchemaId;
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

        // Create a schema for Itemized PayloadLocation
        const createSchema = ExtrinsicHelper.createSchema(providerKeys, AVRO_CHAT_MESSAGE, "AvroBinary", "Itemized");
        const [event] = await createSchema.fundAndSend();
        if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
            [, schemaId] = event.data;
        }
        assert.notEqual(schemaId, undefined, "setup should populate schemaId");

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

    describe("Itemized", () => {
        it("should be able to call applyItemizedAction and apply actions", async function () {
            
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
            const [palletEvent1, chainEvents ] = await itemized_add_result_1.signAndSend();
            assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
            assert.notEqual(palletEvent1, undefined, "should have returned a PalletStatefulStorageItemizedActionApplied event");
            // Delete action
            const idx_1: u16 = new u16(ExtrinsicHelper.api.registry, 1)
            const remove_action_1 ={
                "Remove": idx_1,
            }
            let remove_actions = [remove_action_1];
            let itemized_remove_result_1 = ExtrinsicHelper.applyItemActions(providerKeys, schemaId, msa_id, remove_actions);
            await itemized_remove_result_1.fundOperation();
            const [palletEvent2, chainEvents2 ] = await itemized_remove_result_1.signAndSend();
            assert.notEqual(chainEvents2["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(chainEvents2["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
            assert.notEqual(palletEvent2, undefined, "should have returned a event");
        }).timeout(10000);
    });
});