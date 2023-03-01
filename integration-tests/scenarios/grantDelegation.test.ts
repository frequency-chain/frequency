import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { u16, u64 } from "@polkadot/types";
import assert from "assert";
import { AddProviderPayload, Extrinsic, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { createAndFundKeypair, createKeys, generateDelegationPayload, signPayloadSr25519 } from "../scaffolding/helpers";

describe("Delegation Scenario Tests", function () {
    let keys: KeyringPair;
    let otherMsaKeys: KeyringPair;
    let noMsaKeys: KeyringPair;
    let providerKeys: KeyringPair;
    let otherProviderKeys: KeyringPair;
    let schemaId: u16;
    let providerId: u64;
    let otherProviderId: u64;
    let msaId: u64;
    let otherMsaId: u64;

    before(async function () {
        noMsaKeys = await createAndFundKeypair();

        keys = await createAndFundKeypair();
        const createMsaOp = ExtrinsicHelper.createMsa(keys);
        let [msaCreatedEvent] = await createMsaOp.fundAndSend();
        if (msaCreatedEvent && createMsaOp.api.events.msa.MsaCreated.is(msaCreatedEvent)) {
            msaId = msaCreatedEvent.data.msaId;
        }
        assert.notEqual(msaId, undefined, 'setup should populate msaId');

        otherMsaKeys = await createAndFundKeypair();
        [msaCreatedEvent] = await ExtrinsicHelper.createMsa(otherMsaKeys).fundAndSend();
        if (msaCreatedEvent && createMsaOp.api.events.msa.MsaCreated.is(msaCreatedEvent)) {
            otherMsaId = msaCreatedEvent.data.msaId;
        }
        assert.notEqual(otherMsaId, undefined, 'setup should populate otherMsaId');

        providerKeys = await createAndFundKeypair();
        let createProviderMsaOp = ExtrinsicHelper.createMsa(providerKeys);
        await createProviderMsaOp.fundAndSend();
        let createProviderOp = ExtrinsicHelper.createProvider(providerKeys, "MyPoster");
        let [providerEvent] = await createProviderOp.fundAndSend();
        assert.notEqual(providerEvent, undefined, "setup should return a ProviderCreated event");
        if (providerEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(providerEvent)) {
            providerId = providerEvent.data.providerId;
        }
        assert.notEqual(providerId, undefined, "setup should populate providerId");

        otherProviderKeys = await createAndFundKeypair();
        createProviderMsaOp = ExtrinsicHelper.createMsa(otherProviderKeys);
        await createProviderMsaOp.fundAndSend();
        createProviderOp = ExtrinsicHelper.createProvider(otherProviderKeys, "MyPoster");
        [providerEvent] = await createProviderOp.fundAndSend();
        assert.notEqual(providerEvent, undefined, "setup should return a ProviderCreated event");
        if (providerEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(providerEvent)) {
            otherProviderId = providerEvent.data.providerId;
        }
        assert.notEqual(otherProviderId, undefined, "setup should populate providerId");

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
        const [createSchemaEvent] = await createSchemaOp.fundAndSend();
        assert.notEqual(createSchemaEvent, undefined, "setup should return SchemaCreated event");
        if (createSchemaEvent && ExtrinsicHelper.api.events.schemas.SchemaCreated.is(createSchemaEvent)) {
            schemaId = createSchemaEvent.data[1];
        }
        assert.notEqual(schemaId, undefined, "setup should populate schemaId");
    });

    describe("delegation grants", function () {

        it("should fail to grant delegation if payload not signed by delegator (AddProviderSignatureVerificationFailed)", async function () {
            const payload = await generateDelegationPayload({
                authorizedMsaId: providerId,
                schemaIds: [schemaId],
            });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

            const grantDelegationOp = ExtrinsicHelper.grantDelegation(keys, providerKeys, signPayloadSr25519(providerKeys, addProviderData), payload);
            await assert.rejects(grantDelegationOp.fundAndSend(), { name: 'AddProviderSignatureVerificationFailed' });
        });

        it("should fail to delegate to self (InvalidSelfProvider)", async function () {
            const payload = await generateDelegationPayload({
                authorizedMsaId: providerId,
                schemaIds: [schemaId],
            });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

            const grantDelegationOp = ExtrinsicHelper.grantDelegation(providerKeys, providerKeys, signPayloadSr25519(providerKeys, addProviderData), payload);
            await assert.rejects(grantDelegationOp.fundAndSend(), { name: 'InvalidSelfProvider' });
        });

        it("should fail to grant delegation to an MSA that is not a registered provider (ProviderNotRegistered)", async function () {
            const payload = await generateDelegationPayload({
                authorizedMsaId: otherMsaId,
                schemaIds: [schemaId],
            });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

            const grantDelegationOp = ExtrinsicHelper.grantDelegation(keys, otherMsaKeys, signPayloadSr25519(keys, addProviderData), payload);
            await assert.rejects(grantDelegationOp.fundAndSend(), { name: 'ProviderNotRegistered' });
        });

        it("should fail to grant delegation if ID in payload does not match origin (UnauthorizedDelegator)", async function () {
            const payload = await generateDelegationPayload({
                authorizedMsaId: otherMsaId,
                schemaIds: [schemaId],
            });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

            const grantDelegationOp = ExtrinsicHelper.grantDelegation(keys, providerKeys, signPayloadSr25519(keys, addProviderData), payload);
            await assert.rejects(grantDelegationOp.fundAndSend(), { name: 'UnauthorizedDelegator' });
        });

        it("should grant a delegation to a provider", async function () {
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
                assert.deepEqual(grantDelegationEvent.data.delegatorId, msaId, 'delegator IDs should match');
            }
        });

    });

    describe("schema permission grants", function () {
        it("should fail to grant schema permissions to non-MSA (NoKeyExists)", async function () {
            const nonMsaKeys = await createAndFundKeypair();
            const op = ExtrinsicHelper.grantSchemaPermissions(nonMsaKeys, providerId, [schemaId]);
            await assert.rejects(op.fundAndSend(), { name: 'NoKeyExists' });
        });

        it("should fail to grant schema permissions to a provider for which no delegation exists (DelegationNotFound)", async function () {
            const op = ExtrinsicHelper.grantSchemaPermissions(keys, otherProviderId, [schemaId]);
            await assert.rejects(op.fundAndSend(), { name: 'DelegationNotFound' });
        })

        it("should grant permissions to a provider for a specific set of schemas", async function () {
            const grantSchemaPermissionsOp = ExtrinsicHelper.grantSchemaPermissions(keys, providerId, [schemaId]);
            const [grantSchemaPermissionsEvent] = await grantSchemaPermissionsOp.fundAndSend();
            assert.notEqual(grantSchemaPermissionsEvent, undefined, "should have returned DelegationUpdated event");
            if (grantSchemaPermissionsEvent && grantSchemaPermissionsOp.api.events.msa.DelegationUpdated.is(grantSchemaPermissionsEvent)) {
                assert.deepEqual(grantSchemaPermissionsEvent.data.providerId, providerId, 'provider ids should be equal');
                assert.deepEqual(grantSchemaPermissionsEvent.data.delegatorId, msaId, 'delegator ids should be equal');
            }
        });

        it("should fail to grant more then maxSchemaGrantsPerDelegation (ExceedsMaxSchemaGrantsPerDelegation)", async function () {
            const max_schema_grants = ExtrinsicHelper.api.consts.msa.maxSchemaGrantsPerDelegation.toNumber();
            let schemaIds: u16[] = [];

            const schema = {
                name: "dummySchema",
                type: "record",
                fields: [],
            }

            for (const [index, _] of new Array<number>(max_schema_grants).entries()) {
                const f = ExtrinsicHelper.createSchema(keys, { ...schema, name: `${schema.name}${index}` }, "AvroBinary", "OnChain");
                const [event] = await f.fundAndSend();
                if (event && f.api.events.schemas.SchemaCreated.is(event)) {
                    schemaIds.push(event.data[1]);
                }
            }

            const op = ExtrinsicHelper.grantSchemaPermissions(keys, providerId, schemaIds);
            await assert.rejects(op.fundAndSend(), { name: 'ExceedsMaxSchemaGrantsPerDelegation' });
        }).timeout(10000);
    });

    describe("revoke schema permissions", function () {
        it("should fail to revoke schema permissions from non-MSA (NoKeyExists)", async function () {
            const nonMsaKeys = await createAndFundKeypair();
            const op = ExtrinsicHelper.revokeSchemaPermissions(nonMsaKeys, providerId, [schemaId]);
            await assert.rejects(op.fundAndSend(), { name: 'NoKeyExists' });
        });

        it("should fail to revoke schema permissions from a provider for which no delegation exists (DelegationNotFound)", async function () {
            const op = ExtrinsicHelper.revokeSchemaPermissions(keys, otherProviderId, [schemaId]);
            await assert.rejects(op.fundAndSend(), { name: 'DelegationNotFound' });
        })

        it("should fail to revoke schema permission which was never granted (SchemaNotGranted)", async function () {
            const schema = {
                name: "nonGrantedSchema",
                type: "record",
                fields: [],
            }

            const sop = ExtrinsicHelper.createSchema(keys, schema, "AvroBinary", "OnChain");
            const [sevent] = await sop.fundAndSend();
            let sid: u16 | undefined;
            if (!!sevent && sop.api.events.schemas.SchemaCreated.is(sevent)) {
                sid = sevent.data[1];
            }
            assert.notEqual(sid, undefined, "should have created schema");

            const op = ExtrinsicHelper.revokeSchemaPermissions(keys, providerId, [sid]);
            await assert.rejects(op.fundAndSend(), { name: 'SchemaNotGranted' });
        });
    });

    describe("revoke delegations", function () {
        it("should fail to revoke a delegation if no MSA exists (InvalidMsaKey)", async function () {
            const nonMsaKeys = await createAndFundKeypair();
            const op = ExtrinsicHelper.revokeDelegationByDelegator(nonMsaKeys, providerId);
            await assert.rejects(op.fundAndSend(), { name: 'RpcError', message: /Custom error: 1$/ });
        });

        it("should revoke a delegation by delegator", async function () {
            const revokeDelegationOp = ExtrinsicHelper.revokeDelegationByDelegator(keys, providerId);
            const [revokeDelegationEvent] = await revokeDelegationOp.fundAndSend();
            assert.notEqual(revokeDelegationEvent, undefined, "should have returned DelegationRevoked event");
            if (!!revokeDelegationEvent && revokeDelegationOp.api.events.msa.DelegationRevoked.is(revokeDelegationEvent)) {
                assert.deepEqual(revokeDelegationEvent.data.providerId, providerId, 'provider ids should be equal');
                assert.deepEqual(revokeDelegationEvent.data.delegatorId, msaId, 'delegator ids should be equal');
            }
        });

        it("should fail to revoke a delegation that has already been revoked (InvalidDelegation)", async function () {
            const op = ExtrinsicHelper.revokeDelegationByDelegator(keys, providerId);
            await assert.rejects(op.fundAndSend(), { name: 'RpcError', message: /Custom error: 0$/ });
        });

        it("should fail to revoke delegation where no delegation exists (DelegationNotFound)", async function () {
            const op = ExtrinsicHelper.revokeDelegationByDelegator(keys, otherProviderId);
            await assert.rejects(op.fundAndSend(), { name: 'RpcError', message: /Custom error: 0$/ });
        });

        it("should revoke a delegation by provider", async function () {
            const newKeys = createKeys();
            const payload = await generateDelegationPayload({ authorizedMsaId: providerId, schemaIds: [schemaId] });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
            let op = ExtrinsicHelper.createSponsoredAccountWithDelegation(newKeys, providerKeys, signPayloadSr25519(newKeys, addProviderData), payload);
            const [msaEvent] = await op.fundAndSend();
            const msaId = (msaEvent && op.api.events.msa.MsaCreated.is(msaEvent) ? msaEvent.data.msaId : undefined);
            assert.notEqual(msaId, undefined, 'should have returned an MSA');
            op = ExtrinsicHelper.revokeDelegationByProvider(msaId as u64, providerKeys);
            const [revokeEvent] = await op.fundAndSend();
            assert.notEqual(revokeEvent, undefined, "should have returned a DelegationRevoked event");
            if (revokeEvent && op.api.events.msa.DelegationRevoked.is(revokeEvent)) {
                assert.deepEqual(revokeEvent.data.delegatorId, msaId, 'delegator ids should match');
                assert.deepEqual(revokeEvent.data.providerId, providerId, 'provider ids should match');
            }
        });
    });

    describe("createSponsoredAccountWithDelegation", function () {
        let sponsorKeys: KeyringPair;
        let op: Extrinsic;
        let defaultPayload: AddProviderPayload;

        before(async function () {
            sponsorKeys = await createAndFundKeypair();
            defaultPayload = {
                authorizedMsaId: providerId,
                schemaIds: [schemaId],
            }
        });

        it("should fail to create delegated account if provider ids don't match (UnauthorizedProvider)", async function () {
            const payload = await generateDelegationPayload({ ...defaultPayload, authorizedMsaId: otherProviderId });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

            op = ExtrinsicHelper.createSponsoredAccountWithDelegation(sponsorKeys, providerKeys, signPayloadSr25519(sponsorKeys, addProviderData), payload);
            await assert.rejects(op.fundAndSend(), { name: 'UnauthorizedProvider' });
        });

        it("should fail to create delegated account if payload signature cannot be verified (InvalidSignature)", async function () {
            const payload = await generateDelegationPayload({ ...defaultPayload, schemaIds: [] });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

            op = ExtrinsicHelper.createSponsoredAccountWithDelegation(sponsorKeys, providerKeys, signPayloadSr25519(sponsorKeys, addProviderData), { ...payload, ...defaultPayload });
            await assert.rejects(op.fundAndSend(), { name: 'InvalidSignature' });
        });

        it("should fail to create delegated account if no MSA exists for origin (NoKeyExists)", async function () {
            const payload = await generateDelegationPayload(defaultPayload);
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

            op = ExtrinsicHelper.createSponsoredAccountWithDelegation(sponsorKeys, noMsaKeys, signPayloadSr25519(sponsorKeys, addProviderData), payload);
            await assert.rejects(op.fundAndSend(), { name: 'NoKeyExists' });
        });

        it("should fail to create delegated account if MSA already exists for delegator (KeyAlreadyRegistered)", async function () {
            const payload = await generateDelegationPayload(defaultPayload);
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

            op = ExtrinsicHelper.createSponsoredAccountWithDelegation(keys, providerKeys, signPayloadSr25519(keys, addProviderData), payload);
            await assert.rejects(op.fundAndSend(), { name: 'KeyAlreadyRegistered' });
        })

        it("should fail to create delegated account if provider is not registered (ProviderNotRegistered)", async function () {
            const payload = await generateDelegationPayload({ ...defaultPayload, authorizedMsaId: otherMsaId });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

            op = ExtrinsicHelper.createSponsoredAccountWithDelegation(keys, otherMsaKeys, signPayloadSr25519(keys, addProviderData), payload);
            await assert.rejects(op.fundAndSend(), { name: 'ProviderNotRegistered' });
        })

        it("should fail to create delegated account if provider if payload proof is too far in the future (ProofNotYetValid)", async function () {
            const payload = await generateDelegationPayload(defaultPayload, 999);
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

            op = ExtrinsicHelper.createSponsoredAccountWithDelegation(sponsorKeys, providerKeys, signPayloadSr25519(sponsorKeys, addProviderData), payload);
            await assert.rejects(op.fundAndSend(), { name: 'ProofNotYetValid' });
        })

        it("should fail to create delegated account if provider if payload proof has expired (ProofHasExpired))", async function () {
            const payload = await generateDelegationPayload(defaultPayload, -1);
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

            op = ExtrinsicHelper.createSponsoredAccountWithDelegation(sponsorKeys, providerKeys, signPayloadSr25519(sponsorKeys, addProviderData), payload);
            await assert.rejects(op.fundAndSend(), { name: 'ProofHasExpired' });
        })

        it("should successfully create a delegated account", async function () {
            const payload = await generateDelegationPayload(defaultPayload);
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

            op = ExtrinsicHelper.createSponsoredAccountWithDelegation(sponsorKeys, providerKeys, signPayloadSr25519(sponsorKeys, addProviderData), payload);
            const [event, eventMap] = await op.fundAndSend();
            assert.notEqual(event, undefined, 'should have returned MsaCreated event');
            assert.notEqual(eventMap["msa.DelegationGranted"], undefined, 'should have returned DelegationGranted event');
            await assert.rejects(op.fundAndSend(), { name: 'SignatureAlreadySubmitted' }, "should reject double submission");
        })
    })
})
