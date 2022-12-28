import "@frequency-chain/api-augment";
import assert from "assert";
import { createKeys, createAndFundKeypair, signPayloadSr25519, Sr25519Signature } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { AddKeyData, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { u64 } from "@polkadot/types";
import { Codec } from "@polkadot/types/types";

describe("Create Accounts", function () {
    let keys: KeyringPair;
    let msaId: u64;
    let authorizedKeys: KeyringPair[] = [];

    before(async function () {
        keys = await createAndFundKeypair();
    });

    describe("createMsa", function () {

        it("should successfully create an MSA account", async function () {
            const f = ExtrinsicHelper.createMsa(keys);
            const chainEvents = await f.fundAndSend();
            const msaCreatedEvent = chainEvents["msa.MsaCreated"];

            assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined, "should have returned an ExtrinsicSuccess event");
            assert.notEqual(msaCreatedEvent, undefined, "should have returned  an MsaCreated event");
            assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined, "should have returned a TransactionFeePaid event");

            if (ExtrinsicHelper.api.events.msa.MsaCreated.is(msaCreatedEvent)) {
                msaId = msaCreatedEvent.data.msaId;
            }
        });

        it("should fail to create an MSA for a keypair already associated with an MSA", async function () {
            const op = ExtrinsicHelper.createMsa(keys);
            await assert.rejects(op.fundAndSend(), {
                name: 'KeyAlreadyRegistered',
            });
        });
    });

    describe("addPublicKeyToMsa", function () {
        let badKeys: KeyringPair;
        let payload: AddKeyData = {};
        let ownerSig: Sr25519Signature;
        let newSig: Sr25519Signature;
        let badSig: Sr25519Signature;
        let addKeyData: Codec;

        before(async function () {
            authorizedKeys.push(await createAndFundKeypair());
            badKeys = createKeys();
            payload.msaId = msaId;
            payload.newPublicKey = authorizedKeys[0].publicKey;
            payload.expiration = (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber() + 30;
        });

        it("should fail to add public key if origin is not one of the signers of the payload", async function () {
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);
            newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            badSig = signPayloadSr25519(badKeys, addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(keys, badSig, newSig, payload);
            await assert.rejects(op.fundAndSend(), {
                name: 'MsaOwnershipInvalidSignature',
            });
        })

        it("should fail to add public key if new keypair is not one of the signers of the payload", async function () {
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);
            ownerSig = signPayloadSr25519(keys, addKeyData);
            badSig = signPayloadSr25519(badKeys, addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, badSig, payload);
            await assert.rejects(op.fundAndSend(), {
                name: 'NewKeyOwnershipInvalidSignature',
            });
        });

        it("should fail to add public key if origin does not have an MSA", async function () {
            const newOriginKeys = await createAndFundKeypair();
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);
            ownerSig = signPayloadSr25519(newOriginKeys, addKeyData);
            newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(newOriginKeys, ownerSig, newSig, payload);
            await assert.rejects(op.fundAndSend(), {
                name: 'NoKeyExists',
            });
        })

        it("should fail to add public key if origin does not own MSA", async function () {
            const newPayload = {
                ...payload,
                msaId: 999, // If we create more than 999 MSAs in our test suites, this will fail
            }
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", newPayload);
            ownerSig = signPayloadSr25519(keys, addKeyData);
            newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, newPayload);
            await assert.rejects(op.fundAndSend(), {
                name: 'NotMsaOwner',
            });
        });

        it("should fail if expiration has passed", async function () {
            const newPayload = {
                ...payload,
                expiration: (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber(),
            }
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", newPayload);
            ownerSig = signPayloadSr25519(keys, addKeyData);
            newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, newPayload);
            await assert.rejects(op.fundAndSend(), {
                name: 'ProofHasExpired',
            });
        })

        it("should fail if expiration is not yet valid", async function () {
            const maxMortality = ExtrinsicHelper.api.consts.msa.mortalityWindowSize.toNumber();
            const newPayload = {
                ...payload,
                expiration: (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber() + maxMortality + 5,
            }
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", newPayload);
            ownerSig = signPayloadSr25519(keys, addKeyData);
            newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, newPayload);
            await assert.rejects(op.fundAndSend(), {
                name: 'ProofNotYetValid',
            });
        })

        it("should successfully add a new public key to an existing MSA", async function () {
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);

            ownerSig = signPayloadSr25519(keys, addKeyData);
            newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);

            const publicKeyEvents = await addPublicKeyOp.fundAndSend();

            assert.notEqual(publicKeyEvents["msa.PublicKeyAdded"], undefined, 'should have added public key');
        });

        // NOTE: This test depends on signature & payload variables remaining set from the immediately preceding test
        it("should fail if submitting the same signature more than once", async function () {
            const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);
            await assert.rejects(addPublicKeyOp.fundAndSend(), {
                name: 'SignatureAlreadySubmitted',
            });
        })

        it("should fail if attempting to add the same key more than once", async function () {
            const addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);

            const ownerSig = signPayloadSr25519(keys, addKeyData);
            const newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);

            await assert.rejects(addPublicKeyOp.fundAndSend(), {
                name: 'KeyAlreadyRegistered',
            })
        });

        it("should allow new keypair to act for/on MSA", async function () {
            const additionalKeys = createKeys();
            const newPayload = {
                ...payload,
                newPublicKey: additionalKeys.publicKey,
            }
            addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", newPayload);
            ownerSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
            newSig = signPayloadSr25519(additionalKeys, addKeyData);
            const op = ExtrinsicHelper.addPublicKeyToMsa(authorizedKeys[0], ownerSig, newSig, newPayload);
            const events = await op.fundAndSend();
            assert.notEqual(events["msa.PublicKeyAdded"], undefined, 'should have added public key');
            authorizedKeys.push(additionalKeys);
        });
    });

    describe("retire/delete keys", function () {
        let providerKeys: KeyringPair;

        before(async function () {
            providerKeys = await createAndFundKeypair();
            const createMsaOp = ExtrinsicHelper.createMsa(providerKeys);
            await createMsaOp.fundAndSend();

            const createProviderOp = ExtrinsicHelper.createProvider(providerKeys, 'Test Provider');
            await createProviderOp.fundAndSend();
        });

        it("should disallow retiring MSA belonging to a provider", async function () {
            const retireOp = ExtrinsicHelper.retireMsa(providerKeys);
            await assert.rejects(retireOp.fundAndSend(), {
                name: 'RpcError',
                message: /Custom error: 2/,
            });
        });

        it("should disallow retiring an MSA with more than one key authorized", async function () {
            const retireOp = ExtrinsicHelper.retireMsa(keys);
            await assert.rejects(retireOp.fundAndSend(), {
                name: 'RpcError',
                message: /Custom error: 3/,
            });
        });

        it("should fail to delete public key for self", async function () {
            const op = ExtrinsicHelper.deletePublicKey(keys, keys.publicKey);
            await assert.rejects(op.fundAndSend(), {
                name: 'RpcError',
                message: /Custom error: 4/,
            });
        });

        it("should fail to delete key if not authorized for key's MSA", async function () {
            const op = ExtrinsicHelper.deletePublicKey(providerKeys, keys.publicKey);
            await assert.rejects(op.fundAndSend(), {
                name: 'RpcError',
                message: /Custom error: 5/,
            });
        });

        it("should test for 'NoKeyExists' error");

        it("should delete all other authorized keys", async function () {
            for (const key of authorizedKeys) {
                const op = ExtrinsicHelper.deletePublicKey(keys, key.publicKey);
                const event = (await op.fundAndSend())?.["msa.PublicKeyDeleted"];
                assert.notEqual(event, undefined, "should have returned PublicKeyDeleted event");
            };
            authorizedKeys = [];
        });

        it("should allow retiring MSA after additional keys have been deleted", async function () {
            const retireMsaOp = ExtrinsicHelper.retireMsa(keys);
            const retireMsaEvents = await retireMsaOp.fundAndSend();

            assert.notEqual(retireMsaEvents["msa.PublicKeyDeleted"], undefined, 'should have deleted public key (retired)');
            assert.notEqual(retireMsaEvents["msa.MsaRetired"], undefined, 'should have retired msa');
        });

    });
})
