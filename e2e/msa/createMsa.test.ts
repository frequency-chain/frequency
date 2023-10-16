import "@frequency-chain/api-augment";
import assert from "assert";
import { createKeys, createAndFundKeypair, signPayloadSr25519, Sr25519Signature, generateAddKeyPayload } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { AddKeyData, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { u64 } from "@polkadot/types";
import { Codec } from "@polkadot/types/types";
import { getFundingSource } from "../scaffolding/funding";

describe("Create Accounts", function () {
    const fundingSource = getFundingSource("msa-create-msa");

    let keys: KeyringPair;
    let msaId: u64;
    let authorizedKeys: KeyringPair[] = [];

    before(async function () {
        keys = await createAndFundKeypair(fundingSource);
    });

    describe("createMsa", function () {

        it("should successfully create an MSA account", async function () {
            const f = ExtrinsicHelper.createMsa(keys);
            const [msaCreatedEvent, chainEvents] = await f.fundAndSend(fundingSource);

            assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(msaCreatedEvent, undefined, "should have returned  an MsaCreated event");
            assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");

            if (msaCreatedEvent && ExtrinsicHelper.api.events.msa.MsaCreated.is(msaCreatedEvent)) {
                msaId = msaCreatedEvent.data.msaId;
            }
        });

        it("should fail to create an MSA for a keypair already associated with an MSA", async function () {
            const op = ExtrinsicHelper.createMsa(keys);
            await assert.rejects(op.fundAndSend(fundingSource), {
                name: 'KeyAlreadyRegistered',
            });
        });
    });

    describe("addPublicKeyToMsa", function () {
        let badKeys: KeyringPair;
        let defaultPayload: AddKeyData = {};
        let payload: AddKeyData;
        let ownerSig: Sr25519Signature;
        let newSig: Sr25519Signature;
        let badSig: Sr25519Signature;
        let addKeyData: Codec;

        before(async function () {
            authorizedKeys.push(await createAndFundKeypair(fundingSource));
            badKeys = createKeys();
            defaultPayload.msaId = msaId;
            defaultPayload.newPublicKey = authorizedKeys[0].publicKey;
        });

        beforeEach(async function () {
            payload = await generateAddKeyPayload(defaultPayload);
        });

        it("should fail to add public key if origin is not one of the signers of the payload (MsaOwnershipInvalidSignature)", async function () {
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);
            newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            badSig = signPayloadSr25519(badKeys, addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(keys, badSig, newSig, payload);
            await assert.rejects(op.fundAndSend(fundingSource), {
                name: 'MsaOwnershipInvalidSignature',
            });
        })

        it("should fail to add public key if new keypair is not one of the signers of the payload (NewKeyOwnershipInvalidSignature)", async function () {
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);
            ownerSig = signPayloadSr25519(keys, addKeyData);
            badSig = signPayloadSr25519(badKeys, addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, badSig, payload);
            await assert.rejects(op.fundAndSend(fundingSource), {
                name: 'NewKeyOwnershipInvalidSignature',
            });
        });

        it("should fail to add public key if origin does not have an MSA (NoKeyExists)", async function () {
            const newOriginKeys = await createAndFundKeypair(fundingSource);
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);
            ownerSig = signPayloadSr25519(newOriginKeys, addKeyData);
            newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(newOriginKeys, ownerSig, newSig, payload);
            await assert.rejects(op.fundAndSend(fundingSource), {
                name: 'NoKeyExists',
            });
        })

        it("should fail to add public key if origin does not own MSA (NotMsaOwner)", async function () {
            const newPayload = await generateAddKeyPayload({
                ...defaultPayload,
                msaId: new u64(ExtrinsicHelper.api.registry, 999), // If we create more than 999 MSAs in our test suites, this will fail
            });
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", newPayload);
            ownerSig = signPayloadSr25519(keys, addKeyData);
            newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, newPayload);
            await assert.rejects(op.fundAndSend(fundingSource), {
                name: 'NotMsaOwner',
            });
        });

        it("should fail if expiration has passed (ProofHasExpired)", async function () {
            const newPayload = await generateAddKeyPayload({
                ...defaultPayload,
                expiration: (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber(),
            });
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", newPayload);
            ownerSig = signPayloadSr25519(keys, addKeyData);
            newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, newPayload);
            await assert.rejects(op.fundAndSend(fundingSource), {
                name: 'ProofHasExpired',
            });
        })

        it("should fail if expiration is not yet valid (ProofNotYetValid)", async function () {
            const maxMortality = ExtrinsicHelper.api.consts.msa.mortalityWindowSize.toNumber();
            const newPayload = await generateAddKeyPayload({
                ...defaultPayload,
                expiration: (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber() + maxMortality + 5,
            });
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", newPayload);
            ownerSig = signPayloadSr25519(keys, addKeyData);
            newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, newPayload);
            await assert.rejects(op.fundAndSend(fundingSource), {
                name: 'ProofNotYetValid',
            });
        })

        it("should successfully add a new public key to an existing MSA & disallow duplicate signed payload submission (SignatureAlreadySubmitted)", async function () {
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);

            ownerSig = signPayloadSr25519(keys, addKeyData);
            newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);

            const [publicKeyEvents] = await addPublicKeyOp.fundAndSend(fundingSource);

            assert.notEqual(publicKeyEvents, undefined, 'should have added public key');

            await assert.rejects(addPublicKeyOp.fundAndSend(fundingSource), "should reject sending the same signed payload twice");
        });

        it("should fail if attempting to add the same key more than once (KeyAlreadyRegistered)", async function () {
            const addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);

            const ownerSig = signPayloadSr25519(keys, addKeyData);
            const newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);

            await assert.rejects(addPublicKeyOp.fundAndSend(fundingSource), {
                name: 'KeyAlreadyRegistered',
            })
        });

        it("should allow new keypair to act for/on MSA", async function () {
            const additionalKeys = createKeys();
            const newPayload = await generateAddKeyPayload({
                ...defaultPayload,
                newPublicKey: additionalKeys.publicKey,
            });
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", newPayload);
            ownerSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            newSig = signPayloadSr25519(additionalKeys, addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(authorizedKeys[0], ownerSig, newSig, newPayload);
            const [event] = await op.fundAndSend(fundingSource);
            assert.notEqual(event, undefined, 'should have added public key');
            authorizedKeys.push(additionalKeys);
        });
    });

    describe("retire/delete keys", function () {
        let providerKeys: KeyringPair;

        before(async function () {
            providerKeys = await createAndFundKeypair(fundingSource);
            const createMsaOp = ExtrinsicHelper.createMsa(providerKeys);
            await createMsaOp.fundAndSend(fundingSource);

            const createProviderOp = ExtrinsicHelper.createProvider(providerKeys, 'Test Provider');
            await createProviderOp.fundAndSend(fundingSource);
        });

        it("should disallow retiring MSA belonging to a provider", async function () {
            const retireOp = ExtrinsicHelper.retireMsa(providerKeys);
            await assert.rejects(retireOp.fundAndSend(fundingSource), {
                name: 'RpcError',
                message: /Custom error: 2/,
            });
        });

        it("should disallow retiring an MSA with more than one key authorized", async function () {
            const retireOp = ExtrinsicHelper.retireMsa(keys);
            await assert.rejects(retireOp.fundAndSend(fundingSource), {
                name: 'RpcError',
                message: /Custom error: 3/,
            });
        });

        it("should fail to delete public key for self", async function () {
            const op = ExtrinsicHelper.deletePublicKey(keys, keys.publicKey);
            await assert.rejects(op.fundAndSend(fundingSource), {
                name: 'RpcError',
                message: /Custom error: 4/,
            });
        });

        it("should fail to delete key if not authorized for key's MSA", async function () {
            const op = ExtrinsicHelper.deletePublicKey(providerKeys, keys.publicKey);
            await assert.rejects(op.fundAndSend(fundingSource), {
                name: 'RpcError',
                message: /Custom error: 5/,
            });
        });

        it("should test for 'NoKeyExists' error");

        it("should delete all other authorized keys", async function () {
            for (const key of authorizedKeys) {
                const op = ExtrinsicHelper.deletePublicKey(keys, key.publicKey);
                const [event] = await op.fundAndSend(fundingSource);
                assert.notEqual(event, undefined, "should have returned PublicKeyDeleted event");
            };
            authorizedKeys = [];
        });

        it("should allow retiring MSA after additional keys have been deleted", async function () {
            const retireMsaOp = ExtrinsicHelper.retireMsa(keys);
            const [event, eventMap] = await retireMsaOp.fundAndSend(fundingSource);

            assert.notEqual(eventMap["msa.PublicKeyDeleted"], undefined, 'should have deleted public key (retired)');
            assert.notEqual(event, undefined, 'should have retired msa');
        });

    });
})
