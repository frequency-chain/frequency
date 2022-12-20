import "@frequency-chain/api-augment";
import { ApiRx } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { Codec } from "@polkadot/types/types";
import assert from "assert";
import { connect } from "../scaffolding/apiConnection";
import { createMsa, createProvider, createSchema, grantDelegation, grantSchemaPermissions, revokeDelegationByDelegator } from "../scaffolding/extrinsicHelpers";
import { createAndFundAccount, DevAccounts, INITIAL_FUNDING, showTotalCost, signPayloadSr25519 } from "../scaffolding/helpers";

describe("Delegation Scenario Tests", function () {
    this.timeout(15000);

    const context = this.title;
    const source = DevAccounts.Alice;
    const amount = INITIAL_FUNDING;

    let api: ApiRx;
    let keys: KeyringPair;
    let schemaId: Codec;

    before(async function () {
        let connectApi = await connect(process.env.WS_PROVIDER_URL);
        api = connectApi
        const accountKeys = createAndFundAccount({ api, amount, source, context });
        keys = (await accountKeys).newAccount;
        await createMsa(api, keys);

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

        schemaId = createSchemaEvents["schemas.SchemaCreated"][1];
    })

    after(async function () {
        await showTotalCost(api, context);
        await api.disconnect()
    })

    it("should grant a delegation to a provider", async function () {
        const { newAccount: providerKeys } = await createAndFundAccount({ api, amount, source, context });
        await createMsa(api, providerKeys);


        let createProviderEvents = await createProvider(api, providerKeys, "MyPoster");
        const providerId = createProviderEvents["msa.ProviderCreated"][0];
        assert.notEqual(providerId, undefined);


        const payload = {
            authorizedMsaId: providerId,
            schemaIds: [schemaId],
            // If we ever have tests that exceed Block 24, this test will start failing
            expiration: 24
        }
        const addProviderData = api.registry.createType("PalletMsaAddProvider", payload);

        const grantDelegationEvents = await grantDelegation(api, keys, providerKeys, signPayloadSr25519(keys, addProviderData), payload)
        assert.notEqual(grantDelegationEvents["msa.DelegationGranted"], undefined);
    });

    it("should grant permissions to a provider for a specific set of schemas", async function () {
        const { newAccount: providerKeys } = await createAndFundAccount({ api, amount, source, context });
        await createMsa(api, providerKeys);

        let createProviderEvents = await createProvider(api, providerKeys, "MyPoster");
        const providerId = createProviderEvents["msa.ProviderCreated"][0];
        assert.notEqual(providerId, undefined);


        const payload = {
            authorizedMsaId: providerId,
            schemaIds: [schemaId],
            // If we ever have tests that exceed Block 50, this test will start failing
            expiration: 50
        }
        const addProviderData = api.registry.createType("PalletMsaAddProvider", payload);

        const grantDelegationEvents = await grantDelegation(api, keys, providerKeys, signPayloadSr25519(keys, addProviderData), payload)
        assert.notEqual(grantDelegationEvents["msa.DelegationGranted"], undefined);

        const grantSchemaPermissionsEvents = await grantSchemaPermissions(api, keys, providerId, [schemaId])
        assert.notEqual(grantSchemaPermissionsEvents["msa.DelegationUpdated"], undefined);
    });

    it("should revoke a delegation by delegator", async function () {
        const { newAccount: providerKeys } = await createAndFundAccount({ api, amount, source, context });
        await createMsa(api, providerKeys);

        let createProviderEvents = await createProvider(api, providerKeys, "MyPoster");
        const providerId = createProviderEvents["msa.ProviderCreated"][0];
        assert.notEqual(providerId, undefined);


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
    });
})
