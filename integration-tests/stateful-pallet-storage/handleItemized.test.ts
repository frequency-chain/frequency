// Integration tests for pallets/stateful-pallet-storage/handleItemized.ts
import "@frequency-chain/api-augment";
import assert from "assert";
import { createAndFundKeypair, generateDelegationPayload, signPayloadSr25519 } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { AVRO_CHAT_MESSAGE } from "../stateful-pallet-storage/fixtures/itemizedSchemaType";
import { MessageSourceId, SchemaId } from "@frequency-chain/api-augment/interfaces";
import { PalletStatefulStorageItemAction } from "@polkadot/types/lookup";

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
        const [grantDelegationEvent] = await grantDelegationOp.fundAndSend();
        assert.notEqual(grantDelegationEvent, undefined, "should have returned DelegationGranted event");
        if (grantDelegationEvent && grantDelegationOp.api.events.msa.DelegationGranted.is(grantDelegationEvent)) {
            assert.deepEqual(grantDelegationEvent.data.providerId, providerId, 'provider IDs should match');
            assert.deepEqual(grantDelegationEvent.data.delegatorId, msa_id, 'delegator IDs should match');
        }
    });

    describe("Itemized", () => {
        it("should be able to call applyItemizedAction and apply actions", async function () {
            
            // Add and update actions
            const add_action =({
                Add: "0x0000000000000000000000000000000000000000000000000000000000000001"
            });
            const update_action =({
                Add: "0x0000000000000000000000000000000000000000000000000000000000000002"
            });
            let actions = [add_action, update_action];
            let itemized_add_result_1 = ExtrinsicHelper.applyItemActions(providerKeys, schemaId, msa_id, actions);
            await itemized_add_result_1.fundAndSend();
            const [ ,chainEvents ] = await itemized_add_result_1.fundAndSend();
            assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");

            // Delete action
            const remove_action_1 =({
                Remove: 0,
            });
            const remove_action_2 =({
                Remove: 1,
            });
            
            let remove_actions = [remove_action_1, remove_action_2];
            let itemized_remove_result_1 = ExtrinsicHelper.applyItemActions(providerKeys, schemaId, msa_id, remove_actions);
            await itemized_remove_result_1.fundAndSend();
            const [ , chainEvents2 ] = await itemized_remove_result_1.fundAndSend();
            assert.notEqual(chainEvents2["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(chainEvents2["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
        }).timeout(10000);
    });
});