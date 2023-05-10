import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { Bytes, u32, u64, u16, u128, Vec } from "@polkadot/types";
import { Codec } from "@polkadot/types/types";
import { u8aToHex } from "@polkadot/util/u8a/toHex";
import { u8aWrapBytes } from "@polkadot/util";
import assert from "assert";
import { AddKeyData, AddProviderPayload, EventMap, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import {
  devAccounts, createKeys, createAndFundKeypair, createMsaAndProvider,
  generateDelegationPayload, getBlockNumber, signPayloadSr25519, stakeToProvider, fundKeypair,
  TEST_EPOCH_LENGTH,
  setEpochLength,
  generateAddKeyPayload,
  CENTS,
  DOLLARS,
  createGraphChangeSchema,
  TokenPerCapacity,
  Sr25519Signature,
  assertEvent,
  getCurrentItemizedHash,
  getCurrentPaginatedHash,
  getRemainingCapacity,
} from "../scaffolding/helpers";
import { loadIpfs, getBases } from "../messages/loadIPFS";
import { PARQUET_BROADCAST } from "../schemas/fixtures/parquetBroadcastSchemaType";
import { async, firstValueFrom } from "rxjs";
import { SchemaId } from "@frequency-chain/api-augment/interfaces";
import { AVRO_CHAT_MESSAGE } from "../stateful-pallet-storage/fixtures/itemizedSchemaType";

describe("Capacity Transactions", function () {
    const FUNDS_AMOUNT: bigint = 200n * DOLLARS;

    before(async function () {
        await setEpochLength(devAccounts[0].keys, TEST_EPOCH_LENGTH);
    });

    describe("pay_with_capacity", function () {
        describe("when caller has a Capacity account", async function () {
            let capacityKeys: KeyringPair;
            let capacityProvider: u64;
            let schemaId: u16;
            const amountStaked = 25n * DOLLARS;

            beforeEach(async function () {
                capacityKeys = createKeys("CapacityKeys");
                capacityProvider = await createMsaAndProvider(capacityKeys, "CapacityProvider", FUNDS_AMOUNT);

                // Create schemas for testing with Grant Delegation to test pay_with_capacity
                schemaId = await createGraphChangeSchema();
                assert.notEqual(schemaId, undefined, "setup should populate schemaId");

            });

            it("successfully pays with Capacity for eligible transaction - addPublicKeytoMSA", async function () {
                let authorizedKeys: KeyringPair[] = [];
                let defaultPayload: AddKeyData = {};
                let payload: AddKeyData;
                let ownerSig: Sr25519Signature;
                let newSig: Sr25519Signature;
                let addKeyData: Codec;

                await assert.doesNotReject(stakeToProvider(capacityKeys, capacityProvider, amountStaked));

                authorizedKeys.push(await createAndFundKeypair());
                defaultPayload.msaId = capacityProvider
                defaultPayload.newPublicKey = authorizedKeys[0].publicKey;

                payload = await generateAddKeyPayload(defaultPayload);
                addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);
                ownerSig = signPayloadSr25519(capacityKeys, addKeyData);
                newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
                const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(capacityKeys, ownerSig, newSig, payload);

                const [_, chainEvents] = await addPublicKeyOp.payWithCapacity();

                assertEvent(chainEvents, "msa.PublicKeyAdded");
                assertEvent(chainEvents, "capacity.CapacityWithdrawn");
            });

            // REVIEW: No existing e2e test for createSponsoredAccountWithDelegation
            it("should test pays with Capacity for eligible transaction - createSponsoredAccountWithDelegation");

            it("successfully pays with Capacity for eligible transaction - grantDelegation", async function () {
                await assert.doesNotReject(stakeToProvider(capacityKeys, capacityProvider, amountStaked));

                let delegatorKeys = createKeys("delegatorKeys");
                await fundKeypair(devAccounts[0].keys, delegatorKeys, 100n * DOLLARS);

                let [_, MsaCreatedEvent] = await ExtrinsicHelper.createMsa(delegatorKeys).signAndSend();
                assertEvent(MsaCreatedEvent, "msa.MsaCreated");
                // assert.notEqual(MsaCreatedEvent, undefined, "should have returned MsaCreated event");

                const payload = await generateDelegationPayload({
                    authorizedMsaId: capacityProvider,
                    schemaIds: [schemaId],
                });
                const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
                const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, capacityKeys,
                    signPayloadSr25519(delegatorKeys, addProviderData), payload);

                const [grantDelegationEvent, chainEvents] = await grantDelegationOp.payWithCapacity();
                if (grantDelegationEvent &&
                    !(ExtrinsicHelper.api.events.msa.DelegationGranted.is(grantDelegationEvent))) {
                    assert.fail("should return a DelegationGranted event");
                }

                let fee: u128;
                if (chainEvents["capacity.CapacityWithdrawn"] &&
                    ExtrinsicHelper.api.events.capacity.CapacityWithdrawn.is(chainEvents["capacity.CapacityWithdrawn"]))
                {
                    fee = chainEvents["capacity.CapacityWithdrawn"].data.amount;
                }
                else {
                    assert.fail("should return a CapacityWithdrawn event");
                }
                const expectedRemainingCapacity = amountStaked/TokenPerCapacity - fee.toBigInt();

                // Check for remaining capacity to be reduced by correct amount
                const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(capacityProvider))).unwrap();

                let remaining = capacityStaked.remainingCapacity.toBigInt();
                assert.equal(remaining, expectedRemainingCapacity);
                assert.equal(capacityStaked.totalTokensStaked, amountStaked);
                assert.equal(capacityStaked.totalCapacityIssued, amountStaked/TokenPerCapacity);
            });

            it("successfully pays with Capacity for eligible transaction - addIPFSMessage", async function () {
                // REVIEW: need more staking?
                await assert.doesNotReject(stakeToProvider(capacityKeys, capacityProvider, amountStaked));
                // Create a schema for IPFS
                const createSchema = ExtrinsicHelper.createSchema(capacityKeys, PARQUET_BROADCAST, "Parquet", "IPFS");
                let [event] = await createSchema.fundAndSend();
                if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
                    [, schemaId] = event.data;
                }
                const ipfs_payload_data = "This is a test of Frequency.";
                const ipfs_payload_len = ipfs_payload_data.length + 1;
                let ipfs_node: any;
                let ipfs_cid_64: string;
                ipfs_node = await loadIpfs();
                const { base64, base32 } = await getBases();
                const file = await ipfs_node.add({ path: 'integration_test.txt', content: ipfs_payload_data }, { cidVersion: 1, onlyHash: true });
                ipfs_cid_64 = file.cid.toString(base64);
                const call = ExtrinsicHelper.addIPFSMessage(capacityKeys, schemaId, ipfs_cid_64, ipfs_payload_len);

                const [_, chainEvents] = await call.payWithCapacity();
                assertEvent(chainEvents, "capacity.CapacityWithdrawn");
                assertEvent(chainEvents, "messages.MessagesStored");
            });

            it("successfully pays with Capacity for eligible transaction - addOnchainMessage", async function () {
                // REVIEW: need more staking?
                await assert.doesNotReject(stakeToProvider(capacityKeys, capacityProvider, amountStaked));
                // Create a dummy on-chain schema
                let dummySchemaId: u16;
                const createDummySchema = ExtrinsicHelper.createSchema(
                    capacityKeys, 
                    { type: "record", name: "Dummy on-chain schema", fields: [] }, 
                    "AvroBinary", 
                    "OnChain"
                );
                const [dummySchemaEvent] = await createDummySchema.fundAndSend();
                if (dummySchemaEvent && createDummySchema.api.events.schemas.SchemaCreated.is(dummySchemaEvent)) {
                    [, dummySchemaId] = dummySchemaEvent.data;
                }
                else {
                    assert.fail("should have returned a SchemaCreated event");
                }
                const call = ExtrinsicHelper.addOnChainMessage(capacityKeys, dummySchemaId, "0xdeadbeef");
                const [_, chainEvents] = await call.payWithCapacity();
                assertEvent(chainEvents, "capacity.CapacityWithdrawn");
                assertEvent(chainEvents, "messages.MessagesStored");
            });

            it("successfully pays with Capacity for eligible transaction - applyItemActions", async function () {
                // REVIEW: need more staking?
                await assert.doesNotReject(stakeToProvider(capacityKeys, capacityProvider, amountStaked));
                // Create a schema to allow delete actions
                let schemaId_deletable: SchemaId;
                const createSchemaDeletable = ExtrinsicHelper.createSchema(capacityKeys, AVRO_CHAT_MESSAGE, "AvroBinary", "Itemized");
                const [eventDeletable] = await createSchemaDeletable.fundAndSend();
                if (eventDeletable && createSchemaDeletable.api.events.schemas.SchemaCreated.is(eventDeletable)) {
                    schemaId_deletable = eventDeletable.data.schemaId;
                }
                else {
                    assert.fail("setup should populate schemaId");
                }

                // Add and update actions
                let payload_1 = new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency");

                const add_action = {
                    "Add": payload_1
                }

                let payload_2 = new Bytes(ExtrinsicHelper.api.registry, "Hello World Again From Frequency");

                const update_action = {
                    "Add": payload_2
                }

                const target_hash = await getCurrentItemizedHash(capacityProvider, schemaId_deletable);

                let add_actions = [add_action, update_action];
                const call = ExtrinsicHelper.applyItemActions(capacityKeys, schemaId_deletable, capacityProvider, add_actions, target_hash);
                const [pageUpdateEvent, chainEvents] = await call.payWithCapacity();
                assertEvent(chainEvents, "system.ExtrinsicSuccess");
                assertEvent(chainEvents, "capacity.CapacityWithdrawn");
                // REVIEW: Is this event correct?
                assert.notEqual(pageUpdateEvent, undefined, "should have returned a PalletStatefulStorageItemizedActionApplied event");
 
            });

            // REVIEW: No existing e2e test for createSponsoredAccountWithDelegation
            it("successfully pays with Capacity for eligible transaction - upsertPage; deletePage", async function () {
                // REVIEW: need more staking?
                await assert.doesNotReject(stakeToProvider(capacityKeys, capacityProvider, amountStaked));

                // Create a schema for Paginated PayloadLocation
                const createSchema = ExtrinsicHelper.createSchema(capacityKeys, AVRO_CHAT_MESSAGE, "AvroBinary", "Paginated");
                const [event] = await createSchema.fundAndSend();
                if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
                    schemaId = event.data.schemaId;
                }
                assert.notEqual(schemaId, undefined, "setup should populate schemaId");

                let page_id = 0;
                let target_hash = await getCurrentPaginatedHash(capacityProvider, schemaId, page_id)

                // Add and update actions
                const payload_1 = new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency");
                let call = ExtrinsicHelper.upsertPage(capacityKeys, schemaId, capacityProvider, page_id, payload_1, target_hash);
                let [pageUpdateEvent1, chainEvents] = await call.payWithCapacity();
                assertEvent(chainEvents, "system.ExtrinsicSuccess");
                assertEvent(chainEvents, "capacity.CapacityWithdrawn");
                assert.notEqual(pageUpdateEvent1, undefined, "should have returned a PalletStatefulStoragepaginatedActionApplied event");

                // Remove the page
                target_hash = await getCurrentPaginatedHash(capacityProvider, schemaId, page_id)
                call = ExtrinsicHelper.removePage(capacityKeys, schemaId, capacityProvider, page_id, target_hash);
                let [pageRemove, chainEvents2] = await call.payWithCapacity();
                assertEvent(chainEvents2, "system.ExtrinsicSuccess");
                assertEvent(chainEvents2, "capacity.CapacityWithdrawn");
                // assertEvent(chainEvents2, "PaginatedPageDeleted");
                assert.notEqual(pageRemove, undefined, "should have returned an event");
            });


            it("successfully pays with Capacity for eligible transaction - deletePage", async function () {
                // REVIEW: need more staking?
                // await assert.doesNotReject(stakeToProvider(capacityKeys, capacityProvider, amountStaked));

                // // Create a schema for Paginated PayloadLocation
                // const createSchema = ExtrinsicHelper.createSchema(capacityKeys, AVRO_CHAT_MESSAGE, "AvroBinary", "Paginated");
                // const [event] = await createSchema.fundAndSend();
                // if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
                //     schemaId = event.data.schemaId;
                // }
                // assert.notEqual(schemaId, undefined, "setup should populate schemaId");

                // // Remove the page
                // let page_id = 0;
                // let target_hash = await getCurrentPaginatedHash(capacityProvider, schemaId, page_id)
                // let call = ExtrinsicHelper.removePage(capacityKeys, schemaId, capacityProvider, page_id, target_hash);
                // const [pageRemove, chainEvents] = await call.payWithCapacity();
                // assertEvent(chainEvents, "system.ExtrinsicSuccess");
                // assertEvent(chainEvents, "capacity.CapacityWithdrawn");
                // assertEvent(chainEvents, "PaginatedPageDeleted");
                // assert.notEqual(pageRemove, undefined, "should have returned an event");
            });

            it("successfully pays with Capacity for eligible transaction - applyItemActionsWithSignature");

            it("successfully pays with Capacity for eligible transaction - upsertPageWithSignature");

            it("successfully pays with Capacity for eligible transaction - deletePageWithSignature");

            it("successfully pays with Capacity for eligible transaction - claimHandle");

            // When a user attempts to pay for a non-capacity transaction with Capacity,
            // it should error and drop the transaction from the transaction-pool.
            it("fails to pay with Capacity for a non-capacity transaction", async function () {
                const nonCapacityTxn = ExtrinsicHelper.stake(capacityKeys, capacityProvider, 1n * CENTS);
                await assert.rejects(nonCapacityTxn.payWithCapacity(), {
                    name: "RpcError", message:
                        "1010: Invalid Transaction: Custom error: 0"
                });
            });

            // When a user does not have enough capacity to pay for the transaction fee
            // and is NOT eligible to replenish Capacity, it should error and be dropped
            // from the transaction pool.
            it("fails to pay for a transaction with empty capacity", async function () {
                let noCapacityKeys = createKeys("noCapacityKeys");
                let _providerId = await createMsaAndProvider(noCapacityKeys, "NoCapProvider");

                let delegatorKeys = createKeys("delegatorKeys");

                const payload = await generateDelegationPayload({
                    authorizedMsaId: capacityProvider,
                    schemaIds: [schemaId],
                });
                const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
                const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, noCapacityKeys,
                    signPayloadSr25519(noCapacityKeys, addProviderData), payload);

                await assert.rejects(grantDelegationOp.payWithCapacity(), {
                    name: "RpcError", message:
                        "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low"
                });
            });

            // *All keys should have at least an EXISTENTIAL_DEPOSIT = 1M.
            it("fails to pay for transaction when key does has not met the min deposit", async function () {
                let noTokensKeys = createKeys("noTokensKeys");
                let delegatorKeys = createKeys("delegatorKeys");

                await assert.doesNotReject(stakeToProvider(capacityKeys, capacityProvider, 1n * DOLLARS));

                // Add new key
                let newKeyPayload: AddKeyData = await generateAddKeyPayload({
                     msaId: new u64(ExtrinsicHelper.api.registry, capacityProvider), newPublicKey: noTokensKeys.publicKey
                });
                let addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", newKeyPayload);

                let ownerSig = signPayloadSr25519(capacityKeys, addKeyData);
                let newSig = signPayloadSr25519(noTokensKeys, addKeyData);
                const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(capacityKeys, ownerSig, newSig, newKeyPayload);

                const [publicKeyEvents] = await addPublicKeyOp.fundAndSend();
                assert.notEqual(publicKeyEvents, undefined, 'should have added public key');

                await fundKeypair(devAccounts[0].keys, delegatorKeys, 100n * DOLLARS);
                const createMsaOp = ExtrinsicHelper.createMsa(delegatorKeys);
                const [MsaCreatedEvent] = await createMsaOp.fundAndSend();
                assert.notEqual(MsaCreatedEvent, undefined, "should have returned MsaCreated event");

                const payload = await generateDelegationPayload({
                    authorizedMsaId: capacityProvider,
                    schemaIds: [schemaId],
                });
                const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
                const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, noTokensKeys,
                    signPayloadSr25519(delegatorKeys, addProviderData), payload);

                await assert.rejects(grantDelegationOp.payWithCapacity(), {
                    name: 'RpcError',
                    message: /Custom error: 3/,
                })
            });
        });

        describe("when caller does not have a Capacity account", async function () {
            let delegatorKeys: KeyringPair;
            let delegatorProviderId: u64;
            let schemaId: u16;

            beforeEach(async function () {
                // Create and fund a keypair with EXISTENTIAL_DEPOSIT
                // Use this keypair for delegator operations
                delegatorKeys = createKeys("OtherProviderKeys");
                delegatorProviderId = await createMsaAndProvider(delegatorKeys, "Delegator", FUNDS_AMOUNT);
                schemaId = new u16(ExtrinsicHelper.api.registry, 0);
            });

            describe("but has an MSA account that has not been registered as a Provider", async function () {
                it("fails to pay for a transaction", async function () {
                    // Create a keypair with msaId, but no provider
                    let noProviderKeys = createKeys("NoProviderKeys");
                    await fundKeypair(devAccounts[0].keys, noProviderKeys, FUNDS_AMOUNT);
                    const createMsaOp = ExtrinsicHelper.createMsa(noProviderKeys);

                    const [MsaCreatedEvent] = await createMsaOp.fundAndSend();
                    assert.notEqual(MsaCreatedEvent, undefined, "should have returned MsaCreated event");

                    // When a user is not a registered provider and attempts to pay with Capacity,
                    // it should error with InvalidTransaction::Payment, which is a 1010 error, Inability to pay some fees.
                    const payload = await generateDelegationPayload({
                        authorizedMsaId: delegatorProviderId,
                        schemaIds: [schemaId],
                    });
                    const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
                    const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, noProviderKeys,
                        signPayloadSr25519(delegatorKeys, addProviderData), payload);

                    await assert.rejects(grantDelegationOp.payWithCapacity(), {
                        name: "RpcError", message:
                            "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low"
                    });
                });
            });

            describe("and does not have an MSA account associated to signing keys", async function () {
                it("fails to pay for a transaction", async function () {
                    let emptyKeys = await createAndFundKeypair();

                    const payload = await generateDelegationPayload({
                        authorizedMsaId: delegatorProviderId,
                        schemaIds: [schemaId],
                    });
                    const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
                    const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, emptyKeys,
                        signPayloadSr25519(delegatorKeys, addProviderData), payload);

                    await assert.rejects(grantDelegationOp.payWithCapacity(), {
                        name: "RpcError", message:
                            "1010: Invalid Transaction: Custom error: 1"
                    });
                });
            });
        });
    });
    describe("pay_with_capacity_batch_all", function () {
        describe("when caller has a Capacity account", async function () {
            let capacityProviderKeys: KeyringPair;
            let capacityProvider: u64;
            let schemaId: u16;
            let defaultPayload: AddProviderPayload;
            const amountStaked = 9n * DOLLARS;

            beforeEach(async function () {
                capacityProviderKeys = createKeys("CapacityProviderKeys");
                capacityProvider = await createMsaAndProvider(capacityProviderKeys, "CapacityProvider", FUNDS_AMOUNT);
                defaultPayload = {
                    authorizedMsaId: capacityProvider,
                    schemaIds: [schemaId],
                }
            });

            it("successfully pays with Capacity for a batch of eligible transactions - [createSponsoredAccountWithDelegation, claimHandle]", async function () {
                await assert.doesNotReject(stakeToProvider(capacityProviderKeys, capacityProvider, amountStaked));

                const addProviderPayload = await generateDelegationPayload({ ...defaultPayload });
                const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", addProviderPayload);
                let delegatorKeys = createKeys("delegatorKeys");
                const createSponsoredAccountWithDelegation = ExtrinsicHelper.api.tx.msa.createSponsoredAccountWithDelegation(
                    delegatorKeys.publicKey,
                    signPayloadSr25519(delegatorKeys, addProviderData),
                    addProviderPayload
                );

                const handle = "test_handle";
                const handle_vec = new Bytes(ExtrinsicHelper.api.registry, handle);
                const expiration = (await getBlockNumber()) + 5;
                const handlePayload = {
                    baseHandle: handle_vec,
                    expiration: expiration,
                };
                const claimHandlePayload: any = ExtrinsicHelper.api.registry.createType("CommonPrimitivesHandlesClaimHandlePayload", handlePayload);
                const claimHandleProof = {
                    Sr25519: u8aToHex(delegatorKeys.sign(u8aWrapBytes(claimHandlePayload.toU8a()))),
                };

                const claimHandle = ExtrinsicHelper.api.tx.handles.claimHandle(
                    delegatorKeys.publicKey,
                    claimHandleProof,
                    claimHandlePayload
                );
                const calls = [
                    createSponsoredAccountWithDelegation,
                    claimHandle,
                ];

                let payWithCapacityBatchAllOp = ExtrinsicHelper.payWithCapacityBatchAll(capacityProviderKeys, calls);

                const [batchCompletedEvent, eventMap] = await payWithCapacityBatchAllOp.signAndSend();

                if (batchCompletedEvent &&
                    !(ExtrinsicHelper.api.events.utility.BatchCompleted.is(batchCompletedEvent))) {
                        assert.fail("should return a BatchCompletedEvent");
                    }

                assert.notEqual(eventMap["msa.DelegationGranted"], undefined, 'should have returned DelegationGranted event');
                assert.notEqual(eventMap["handles.HandleClaimed"], undefined, 'should have returned HandleClaimed event');
            });

            it("batch fails if one transaction fails - [createSponsoredAccountWithDelegation, claimHandle]", async function () {
                await assert.doesNotReject(stakeToProvider(capacityProviderKeys, capacityProvider, amountStaked));

                const addProviderPayload = await generateDelegationPayload({ ...defaultPayload });
                const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", addProviderPayload);
                let delegatorKeys = createKeys("delegatorKeys");
                const createSponsoredAccountWithDelegation = ExtrinsicHelper.api.tx.msa.createSponsoredAccountWithDelegation(
                    delegatorKeys.publicKey,
                    signPayloadSr25519(delegatorKeys, addProviderData),
                    addProviderPayload
                );

                const handle = "test_handle_that_exceeds_the_byte_limit";
                const handle_vec = new Bytes(ExtrinsicHelper.api.registry, handle);
                const expiration = (await getBlockNumber()) + 5;
                const handlePayload = {
                    baseHandle: handle_vec,
                    expiration: expiration,
                };
                const claimHandlePayload: any = ExtrinsicHelper.api.registry.createType("CommonPrimitivesHandlesClaimHandlePayload", handlePayload);
                const calimHandleProof = {
                    Sr25519: u8aToHex(delegatorKeys.sign(u8aWrapBytes(claimHandlePayload.toU8a()))),
                };

                const claimHandle = ExtrinsicHelper.api.tx.handles.claimHandle(
                    delegatorKeys.publicKey,
                    calimHandleProof,
                    claimHandlePayload
                );
                const calls = [
                    createSponsoredAccountWithDelegation,
                    claimHandle,
                ];

                let payWithCapacityBatchAllOp = ExtrinsicHelper.payWithCapacityBatchAll(capacityProviderKeys, calls);

                await assert.rejects(payWithCapacityBatchAllOp.signAndSend(), {
                    name: "InvalidHandleByteLength"
                });
            });
        });
    });
});
