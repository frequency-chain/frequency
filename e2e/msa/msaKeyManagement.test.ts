import "@frequency-chain/api-augment";
import assert from "assert";
import { createKeys, createAndFundKeypair, signPayloadSr25519, Sr25519Signature, generateAddKeyPayload, createProviderKeysAndId, CENTS } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { AddKeyData, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { u64 } from "@polkadot/types";
import { Codec } from "@polkadot/types/types";
import { getFundingSource } from "../scaffolding/funding";

const maxU64 = 18_446_744_073_709_551_615n;

describe("MSA Key management", function () {
    const fundingSource = getFundingSource("msa-key-management");

    describe("addPublicKeyToMsa", function () {
        let keys: KeyringPair;
        let msaId: u64;
        let secondaryKey: KeyringPair;
        let defaultPayload: AddKeyData = {};
        let payload: AddKeyData;
        let ownerSig: Sr25519Signature;
        let newSig: Sr25519Signature;
        let badSig: Sr25519Signature;
        let addKeyData: Codec;

        before(async function () {
            // Setup an MSA with one key and a secondary funded key
            keys = await createAndFundKeypair(fundingSource, 5n * CENTS);
            const { target } = await ExtrinsicHelper.createMsa(keys).signAndSend();
            assert.notEqual(target?.data.msaId, undefined, "MSA Id not in expected event");
            msaId = target!.data.msaId;

            secondaryKey = await createAndFundKeypair(fundingSource, 5n * CENTS);

            // Default payload making it easier to test `addPublicKeyToMsa`
            defaultPayload.msaId = msaId;
            defaultPayload.newPublicKey = secondaryKey.publicKey;
        });

        beforeEach(async function () {
            payload = await generateAddKeyPayload(defaultPayload);
        });

        it("should fail to add public key if origin is not one of the signers of the payload (MsaOwnershipInvalidSignature)", async function () {
            const badKeys: KeyringPair = createKeys();
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);
            newSig = signPayloadSr25519(secondaryKey, addKeyData);
            badSig = signPayloadSr25519(badKeys, addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(keys, badSig, newSig, payload);
            await assert.rejects(op.fundAndSend(fundingSource), {
                name: 'MsaOwnershipInvalidSignature',
            });
        })

        it("should fail to add public key if new keypair is not one of the signers of the payload (NewKeyOwnershipInvalidSignature)", async function () {
            const badKeys: KeyringPair = createKeys()
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);
            ownerSig = signPayloadSr25519(keys, addKeyData);
            badSig = signPayloadSr25519(badKeys, addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, badSig, payload);
            await assert.rejects(op.fundAndSend(fundingSource), {
                name: 'NewKeyOwnershipInvalidSignature',
            });
        });

        it("should fail to add public key if origin does not have an MSA (NoKeyExists)", async function () {
            const newOriginKeys = await createAndFundKeypair(fundingSource, 50_000_000n);
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);
            ownerSig = signPayloadSr25519(newOriginKeys, addKeyData);
            newSig = signPayloadSr25519(secondaryKey, addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(newOriginKeys, ownerSig, newSig, payload);
            await assert.rejects(op.fundAndSend(fundingSource), {
                name: 'NoKeyExists',
            });
        })

        it("should fail to add public key if origin does not own MSA (NotMsaOwner)", async function () {
            const newPayload = await generateAddKeyPayload({
                ...defaultPayload,
                msaId: new u64(ExtrinsicHelper.api.registry, maxU64),
            });
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", newPayload);
            ownerSig = signPayloadSr25519(keys, addKeyData);
            newSig = signPayloadSr25519(secondaryKey, addKeyData);
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
            newSig = signPayloadSr25519(secondaryKey, addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, newPayload);
            await assert.rejects(op.fundAndSend(fundingSource), {
                name: 'ProofHasExpired',
            });
        })

        it("should fail if expiration is not yet valid (ProofNotYetValid)", async function () {
            const maxMortality = ExtrinsicHelper.api.consts.msa.mortalityWindowSize.toNumber();
            const newPayload = await generateAddKeyPayload({
                ...defaultPayload,
                expiration: (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber() + maxMortality + 999,
            });
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", newPayload);
            ownerSig = signPayloadSr25519(keys, addKeyData);
            newSig = signPayloadSr25519(secondaryKey, addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, newPayload);
            await assert.rejects(op.fundAndSend(fundingSource), {
                name: 'ProofNotYetValid',
            });
        })

        it("should successfully add a new public key to an existing MSA & disallow duplicate signed payload submission (SignatureAlreadySubmitted)", async function () {
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);

            ownerSig = signPayloadSr25519(keys, addKeyData);
            newSig = signPayloadSr25519(secondaryKey, addKeyData);
            const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);

            const { target: publicKeyEvents } = await addPublicKeyOp.fundAndSend(fundingSource);

            assert.notEqual(publicKeyEvents, undefined, 'should have added public key');

            await assert.rejects(addPublicKeyOp.fundAndSend(fundingSource), "should reject sending the same signed payload twice");
        });

        it("should fail if attempting to add the same key more than once (KeyAlreadyRegistered)", async function () {
            const addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);

            const ownerSig = signPayloadSr25519(keys, addKeyData);
            const newSig = signPayloadSr25519(secondaryKey, addKeyData);
            const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);

            await assert.rejects(addPublicKeyOp.fundAndSend(fundingSource), {
                name: 'KeyAlreadyRegistered',
            })
        });

        it("should allow new keypair to act for/on MSA", async function () {
            const thirdKey = createKeys();
            const newPayload = await generateAddKeyPayload({
                ...defaultPayload,
                newPublicKey: thirdKey.publicKey,
            });
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", newPayload);
            ownerSig = signPayloadSr25519(secondaryKey, addKeyData);
            newSig = signPayloadSr25519(thirdKey, addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(secondaryKey, ownerSig, newSig, newPayload);
            const { target: event } = await op.fundAndSend(fundingSource);
            assert.notEqual(event, undefined, 'should have added public key');

            // Cleanup
            await assert.doesNotReject(ExtrinsicHelper.deletePublicKey(keys, thirdKey.publicKey).signAndSend('current'));
        });
    });

    describe("provider msa", function () {

        it("should disallow retiring MSA belonging to a provider", async function () {
            const [providerKeys] = await createProviderKeysAndId(fundingSource);
            const retireOp = ExtrinsicHelper.retireMsa(providerKeys);
            await assert.rejects(retireOp.signAndSend('current'), {
                name: 'RpcError',
                message: /Custom error: 2/,
            });
        });
    });

    describe("delete keys and retire", function () {
        let keys: KeyringPair;
        let secondaryKey: KeyringPair;
        let msaId: u64;

        before(async function () {
            // Generates a msa with two control keys
            keys = await createAndFundKeypair(fundingSource, 50_000_000n);
            secondaryKey = await createAndFundKeypair(fundingSource, 50_000_000n);

            const { target } = await ExtrinsicHelper.createMsa(keys).signAndSend();
            assert.notEqual(target?.data.msaId, undefined, "MSA Id not in expected event");
            msaId = target!.data.msaId;

            const payload = await generateAddKeyPayload({
                msaId,
                newPublicKey: secondaryKey.publicKey,
            });
            const payloadData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);
            const ownerSig = signPayloadSr25519(keys, payloadData);
            const newSig = signPayloadSr25519(secondaryKey, payloadData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);
            const { target: event } = await op.signAndSend();
            assert.notEqual(event, undefined, 'should have added public key');
        });

        it("should disallow retiring an MSA with more than one key authorized", async function () {
            const retireOp = ExtrinsicHelper.retireMsa(keys);
            await assert.rejects(retireOp.signAndSend('current'), {
                name: 'RpcError',
                message: /Custom error: 3/,
            });
        });

        it("should fail to delete public key for self", async function () {
            const op = ExtrinsicHelper.deletePublicKey(keys, keys.publicKey);
            await assert.rejects(op.signAndSend('current'), {
                name: 'RpcError',
                message: /Custom error: 4/,
            });
        });

        it("should fail to delete key if not authorized for key's MSA", async function () {
            const [providerKeys] = await createProviderKeysAndId(fundingSource);

            const op = ExtrinsicHelper.deletePublicKey(providerKeys, keys.publicKey);
            await assert.rejects(op.signAndSend('current'), {
                name: 'RpcError',
                message: /Custom error: 5/,
            });
        });

        it("should test for 'NoKeyExists' error", async function () {
            const key = createKeys("nothing key");
            const op = ExtrinsicHelper.deletePublicKey(keys, key.publicKey);
            await assert.rejects(op.signAndSend('current'), {
                name: 'RpcError',
                message: /Custom error: 1/,
            });
        });

        it("should delete secondary key", async function () {
            const op = ExtrinsicHelper.deletePublicKey(keys, secondaryKey.publicKey);
            const { target: event } = await op.signAndSend('current');
            assert.notEqual(event, undefined, "should have returned PublicKeyDeleted event");
        });

        it("should allow retiring MSA after additional keys have been deleted", async function () {
            const retireMsaOp = ExtrinsicHelper.retireMsa(keys);
            const { target: event, eventMap } = await retireMsaOp.signAndSend('current');

            assert.notEqual(eventMap["msa.PublicKeyDeleted"], undefined, 'should have deleted public key (retired)');
            assert.notEqual(event, undefined, 'should have retired msa');
        });

    });
})
