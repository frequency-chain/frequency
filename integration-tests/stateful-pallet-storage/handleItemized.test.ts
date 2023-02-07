// Integration tests for pallets/stateful-pallet-storage/handleItemized.ts
import "@frequency-chain/api-augment";
import assert from "assert";
import { createAndFundKeypair } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { AVRO_CHAT_MESSAGE } from "../stateful-pallet-storage/fixtures/itemizedSchemaType";
import { MessageSourceId, SchemaId } from "@frequency-chain/api-augment/interfaces";

describe("Stateful Pallet Storage", () => {
    let keys: KeyringPair;
    let schemaId: SchemaId;
    let msa_id: MessageSourceId;

    before(async function () {        
        keys = await createAndFundKeypair();
        // Create a new MSA
        const createMsa = ExtrinsicHelper.createMsa(keys);
        await createMsa.fundOperation();

        // Create a schema for Itemized PayloadLocation
        const [msaCreatedEvent, chainEvents] = await createMsa.signAndSend();
        assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
        assert.notEqual(msaCreatedEvent, undefined, "should have returned  an MsaCreated event");
        assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
        if (msaCreatedEvent && ExtrinsicHelper.api.events.msa.MsaCreated.is(msaCreatedEvent)) {
            msa_id = msaCreatedEvent.data.msaId;
        }

        // Create a schema for Itemized PayloadLocation
        const createSchema = ExtrinsicHelper.createSchema(keys, AVRO_CHAT_MESSAGE, "AvroBinary", "Itemized");
        const [event] = await createSchema.fundAndSend();
        if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
            [, schemaId] = event.data;
        }
    });
    describe("Itemized", () => {
        it("should be able to call applyItemizedAction and apply actions", async function () {
            let add_action  = [{
                Add: {
                    fromId: "0x0000000000000000000000000000000000000000000000000000000000000001",
                    message: "Hello World",
                    inReplyTo: null,
                    url: null
                }
            }];
            let update_action = {
                Add: {
                    fromId: "0x0000000000000000000000000000000000000000000000000000000000000001",
                    message: "Hello World Again",
                    inReplyTo: null,
                    url: null            
                }
            };
            let remove_action_0 = {
                Remove: {
                    index: 0
                }
            };
            let remove_action_1 = {
                Remove: {
                    index: 1
                }
            };
            let actions = [add_action, update_action, remove_action_0, remove_action_1];
            let itemized_add_result = ExtrinsicHelper.applyItemizedAction(keys, schemaId, msa_id, actions);
            await itemized_add_result.fundOperation();
            const [ mutatableEvent, chainEvents ] = await itemized_add_result.signAndSend();
            assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(mutatableEvent, undefined, "should have returned  an pallet-stateful-storage event");
            assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");
        }).timeout(500);
    });
});