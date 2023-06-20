import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { u16, u64 } from "@polkadot/types";
import assert from "assert";
import { AddProviderPayload, Extrinsic, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { createAndFundKeypair, createKeys, generateDelegationPayload, signPayloadSr25519 } from "../scaffolding/helpers";
import { SchemaId } from "@frequency-chain/api-augment/interfaces";
import { firstValueFrom } from "rxjs";

describe("Delegation Scenario Tests", function () {
    let keys: KeyringPair;
    let otherMsaKeys: KeyringPair;
    let noMsaKeys: KeyringPair;
    let providerKeys: KeyringPair;
    let otherProviderKeys: KeyringPair;
    let schemaId: u16;
    let schemaId2: SchemaId;
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
        let [createSchemaEvent] = await createSchemaOp.fundAndSend();
        assert.notEqual(createSchemaEvent, undefined, "setup should return SchemaCreated event");
        if (createSchemaEvent && ExtrinsicHelper.api.events.schemas.SchemaCreated.is(createSchemaEvent)) {
            schemaId = createSchemaEvent.data.schemaId;
        }
        assert.notEqual(schemaId, undefined, "setup should populate schemaId");

        // Create a second schema
        [createSchemaEvent] = await createSchemaOp.fundAndSend();
        assert.notEqual(createSchemaEvent, undefined, "setup should return SchemaCreated event");
        if (createSchemaEvent && ExtrinsicHelper.api.events.schemas.SchemaCreated.is(createSchemaEvent)) {
            schemaId2 = createSchemaEvent.data.schemaId;
        }
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

        it('initial granted schemas should be correct', async function () {
            let schemaIds = await firstValueFrom(ExtrinsicHelper.api.rpc.msa.grantedSchemaIdsByMsaId(msaId, providerId));
            assert.equal(schemaIds.isSome, true);
            assert.deepEqual([schemaId], schemaIds.unwrap().toArray());
        });

        /// TODO: As of this writing there is no extrinsic available to ADD schema permissions to a grant.
        ///       The only method available is `grantDelegation` which *overwrites* any existing list of
        ///       schema permissions. If a "add_schema_permission" extrinsic becomes available, rewrite this test.
        it('should grant additional schema permissions', async function() {
            const payload = await generateDelegationPayload({
                authorizedMsaId: providerId,
                schemaIds: [schemaId2],
            });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);

            const grantDelegationOp = ExtrinsicHelper.grantDelegation(keys, providerKeys, signPayloadSr25519(keys, addProviderData), payload);
            const [grantDelegationEvent] = await grantDelegationOp.fundAndSend();
            assert.notEqual(grantDelegationEvent, undefined, "should have returned DelegationGranted event");
            if (grantDelegationEvent && grantDelegationOp.api.events.msa.DelegationGranted.is(grantDelegationEvent)) {
                assert.deepEqual(grantDelegationEvent.data.providerId, providerId, 'provider IDs should match');
                assert.deepEqual(grantDelegationEvent.data.delegatorId, msaId, 'delegator IDs should match');
            }
            let grantedSchemaIds = await firstValueFrom(ExtrinsicHelper.api.rpc.msa.grantedSchemaIdsByMsaId(msaId, providerId));
            // NOTE: As of this writing, this is a false pass. The returned array includes `schemaId`, but `schemaId` has actually been
            //       revoked as of the current block, because of the way `grantDelegation` behaves. But we only get back an array of ids,
            //       without the context of the block number at which they have been revoked (zero for not revoked)
            assert.deepEqual([schemaId, schemaId2], grantedSchemaIds.unwrap().toArray());
        })

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

        it("should successfully revoke granted schema", async function () {
            const op = ExtrinsicHelper.revokeSchemaPermissions(keys, providerId, [schemaId]);
            await assert.doesNotReject(op.fundAndSend());
            let grantedSchemaIds = await firstValueFrom(ExtrinsicHelper.api.rpc.msa.grantedSchemaIdsByMsaId(msaId, providerId));
            // NOTE: As of this writing, this test fails because the RPC only returns a list of SchemaIds present in the storage map for
            //       the delegation, which includes revoked schema permissions.
            assert.deepEqual([schemaId2], grantedSchemaIds.unwrap().toArray(), "granted schema permissions should not include revoked schema");
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
