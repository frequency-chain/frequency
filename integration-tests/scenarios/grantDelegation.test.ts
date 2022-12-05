import "@frequency-chain/api-augment";
import { ApiRx } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import assert from "assert";
import { connect, createKeys } from "../scaffolding/apiConnection";
import { createMsa, createProvider, createSchema, grantDelegation, grantSchemaPermissions, revokeDelegationByDelegator } from "../scaffolding/extrinsicHelpers";
import { signPayloadSr25519 } from "../scaffolding/helpers";

describe("Delegation Scenario Tests", () => {
    let api: ApiRx;
    let keys: KeyringPair;

    before(async () => {
        let connectApi = await connect(process.env.WS_PROVIDER_URL);
        keys = createKeys("//Alice")
        api = connectApi
    })

    after(() => {
        api.disconnect()
    })

    it("should grant a delegation to a provider", async () => {
        const providerKeys = createKeys("//Charlie");
        await createMsa(api, keys);
        await createMsa(api, providerKeys);

        let createSchemaEvents = await createSchema(api, keys, {
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

        assert.notEqual(createSchemaEvents["schemas.SchemaCreated"], undefined);

        let createProviderEvents = await createProvider(api, providerKeys, "MyPoster");
        assert.notEqual(createProviderEvents["msa.ProviderCreated"], undefined);

        const schemaId = createSchemaEvents["schemas.SchemaCreated"][1];
        const providerId = createProviderEvents["msa.ProviderCreated"][0];

        const payload = {
            authorizedMsaId: providerId,
            schemaIds: [schemaId],
            // If we ever have tests that exceed Block 24, this test will start failing
            expiration: 24
        }
        const addProviderData = api.registry.createType("PalletMsaAddProvider", payload); 

        const grantDelegationEvents = await grantDelegation(api, keys, providerKeys, signPayloadSr25519(keys, addProviderData), payload)
        assert.notEqual(grantDelegationEvents["msa.DelegationGranted"], undefined);
    }).timeout(15000);

    it("should grant permissions to a provider for a specific set of schemas", async () => {
        const providerKeys = createKeys("//Dave");
        await createMsa(api, keys);
        await createMsa(api, providerKeys);

        let createSchemaEvents = await createSchema(api, keys, {
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

        assert.notEqual(createSchemaEvents["schemas.SchemaCreated"], undefined);

        let createProviderEvents = await createProvider(api, providerKeys, "MyPoster");
        assert.notEqual(createProviderEvents["msa.ProviderCreated"], undefined);

        const schemaId = createSchemaEvents["schemas.SchemaCreated"][1];
        const providerId = createProviderEvents["msa.ProviderCreated"][0];

        const payload = {
            authorizedMsaId: providerId,
            schemaIds: [schemaId],
            // If we ever have tests that exceed Block 24, this test will start failing
            expiration: 24
        }
        const addProviderData = api.registry.createType("PalletMsaAddProvider", payload); 

        const grantDelegationEvents = await grantDelegation(api, keys, providerKeys, signPayloadSr25519(keys, addProviderData), payload)
        assert.notEqual(grantDelegationEvents["msa.DelegationGranted"], undefined);

        const grantSchemaPermissionsEvents = await grantSchemaPermissions(api, keys, providerId, [schemaId])
        assert.notEqual(grantSchemaPermissionsEvents["msa.DelegationUpdated"], undefined);
    }).timeout(15000);

    it("should revoke a delegation by delegator", async () => {
        const providerKeys = createKeys("//Bob");
        await createMsa(api, keys);
        await createMsa(api, providerKeys);

        let createSchemaEvents = await createSchema(api, keys, {
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

        assert.notEqual(createSchemaEvents["schemas.SchemaCreated"], undefined);

        let createProviderEvents = await createProvider(api, providerKeys, "MyPoster");
        assert.notEqual(createProviderEvents["msa.ProviderCreated"], undefined);

        const schemaId = createSchemaEvents["schemas.SchemaCreated"][1];
        const providerId = createProviderEvents["msa.ProviderCreated"][0];

        const payload = {
            authorizedMsaId: providerId,
            schemaIds: [schemaId],
            expiration: 36
        }
        const addProviderData = api.registry.createType("PalletMsaAddProvider", payload); 

        const grantDelegationEvents = await grantDelegation(api, keys, providerKeys, signPayloadSr25519(keys, addProviderData), payload)
        assert.notEqual(grantDelegationEvents["msa.DelegationGranted"], undefined);

        const revokeDelegationEvents = await revokeDelegationByDelegator(api, keys, providerId)
        assert.notEqual(revokeDelegationEvents["msa.DelegationRevoked"], undefined);
    }).timeout(15000);
})