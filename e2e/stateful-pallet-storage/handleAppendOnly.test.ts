// E2E tests for pallets/stateful-pallet-storage/handleItemizedWithSignature.ts
import "@frequency-chain/api-augment";
import assert from "assert";
import {
  createDelegatorAndDelegation,
  createProviderKeysAndId, devAccounts,
  getCurrentItemizedHash,
} from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { AVRO_CHAT_MESSAGE } from "./fixtures/itemizedSchemaType";
import { MessageSourceId, SchemaId } from "@frequency-chain/api-augment/interfaces";
import {Bytes, u16} from "@polkadot/types";

describe("📗 Stateful Pallet Storage AppendOnly Schemas", () => {
    let itemizedSchemaId: SchemaId;
    let msa_id: MessageSourceId;
    let providerId: MessageSourceId;
    let providerKeys: KeyringPair;
    let sudoKey: KeyringPair;

    before(async function () {
        // Using Alice as sudoKey
        sudoKey = devAccounts[0].keys;

        // Create a provider for the MSA, the provider will be used to grant delegation
        [providerKeys, providerId] = await createProviderKeysAndId();
        assert.notEqual(providerId, undefined, "setup should populate providerId");
        assert.notEqual(providerKeys, undefined, "setup should populate providerKeys");

        // Create a schema for Itemized PayloadLocation
        const createSchema = ExtrinsicHelper.createSchemaWithSettingsGov(providerKeys, sudoKey, AVRO_CHAT_MESSAGE, "AvroBinary", "Itemized", "AppendOnly");
        const [event] = await createSchema.sudoSignAndSend();
        if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
            itemizedSchemaId = event.data.schemaId;
        }
        assert.notEqual(itemizedSchemaId, undefined, "setup should populate schemaId");

        // Create a MSA for the delegator and delegate to the provider for the itemized schema
        [, msa_id] = await createDelegatorAndDelegation(itemizedSchemaId, providerId, providerKeys);
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
          name: 'UnsupportedOperationForSchema',
          section: 'statefulStorage',
      });
    });
  });
});
