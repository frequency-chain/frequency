import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { u16, u64 } from "@polkadot/types";
import assert from "assert";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { createAndFundKeypair, signPayloadSr25519 } from "../scaffolding/helpers";

describe("Delegation Scenario Tests", function () {
    let keys: KeyringPair;
    let providerKeys: KeyringPair;
    let schemaId: u16;
    let providerId: u64;

    before(async function () {
        keys = await createAndFundKeypair();
        const createMsaOp = ExtrinsicHelper.createMsa(keys);
        await createMsaOp.fundAndSend();

        providerKeys = await createAndFundKeypair();
        const createProviderMsaOp = ExtrinsicHelper.createMsa(providerKeys);
        await createProviderMsaOp.fundAndSend();
        const createProviderOp = ExtrinsicHelper.createProvider(providerKeys, "MyPoster");
        const providerEvent = (await createProviderOp.fundAndSend())?.["msa.ProviderCreated"];
        assert.notEqual(providerEvent, undefined, "setup should return a ProviderCreated event");
        if (ExtrinsicHelper.api.events.msa.ProviderCreated.is(providerEvent)) {
            providerId = providerEvent.data.providerId;
        }
        assert.notEqual(providerId, undefined, "setup should populate providerId");

        const createSchemaOp = ExtrinsicHelper.createSchema(keys, {
            type: "record",
            name: "Post",
            fields: [
                {
                    name: "title",
                    type: {
                        name: "Title",
                        type: "string"
                    }
                },
                {
                    name: "content",
                    type: {
                        name: "Content",
                        type: "string"
                    }
                },
                {
                    name: "fromId",
                    type: {
                        name: "DSNPId",
                        type: "fixed",
                        size: 8,
                    },
                },
                {
                    name: "objectId",
                    type: "DSNPId",
                },
            ]
        }, "AvroBinary", "OnChain");
        const createSchemaEvent = (await createSchemaOp.fundAndSend())["schemas.SchemaCreated"];
        assert.notEqual(createSchemaEvent, undefined, "setup should return SchemaCreated event");
        if (ExtrinsicHelper.api.events.schemas.SchemaCreated.is(createSchemaEvent)) {
            schemaId = createSchemaEvent.data[1];
        }
        assert.notEqual(schemaId, undefined, "setup should populate schemaId");
    });

    it("should grant a delegation to a provider", async function () {
        const payload = {
            authorizedMsaId: providerId,
            schemaIds: [schemaId],
            expiration: (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber() + 5,
        }
        const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

        const grantDelegationOp = ExtrinsicHelper.grantDelegation(keys, providerKeys, signPayloadSr25519(keys, addProviderData), payload);
        const grantDelegationEvents = await grantDelegationOp.fundAndSend();
        assert.notEqual(grantDelegationEvents["msa.DelegationGranted"], undefined, "should have returned DelegationGranted event");
    });

    it("should grant permissions to a provider for a specific set of schemas", async function () {
        const grantSchemaPermissionsOp = ExtrinsicHelper.grantSchemaPermissions(keys, providerId, [schemaId]);
        const grantSchemaPermissionsEvents = await grantSchemaPermissionsOp.fundAndSend();
        assert.notEqual(grantSchemaPermissionsEvents["msa.DelegationUpdated"], undefined, "should have returned DelegationUpdated event");
    });

    it("should revoke a delegation by delegator", async function () {
        const revokeDelegationOp = ExtrinsicHelper.revokeDelegationByDelegator(keys, providerId);
        const revokeDelegationEvents = await revokeDelegationOp.fundAndSend();
        assert.notEqual(revokeDelegationEvents["msa.DelegationRevoked"], undefined, "should have returned DelegationRevoked event");
    });
})
