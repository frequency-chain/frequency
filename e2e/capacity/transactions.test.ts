import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { Bytes, u64, u16 } from "@polkadot/types";
import { Codec } from "@polkadot/types/types";
import { u8aToHex } from "@polkadot/util/u8a/toHex";
import { u8aWrapBytes } from "@polkadot/util";
import assert from "assert";
import { AddKeyData, AddProviderPayload, EventMap, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { base64 } from 'multiformats/bases/base64';
import { SchemaId, MessageResponse } from "@frequency-chain/api-augment/interfaces";
import {
  createKeys,
  createAndFundKeypair,
  createMsaAndProvider,
  generateDelegationPayload,
  getBlockNumber,
  signPayloadSr25519,
  stakeToProvider,
  fundKeypair,
  generateAddKeyPayload,
  CENTS,
  DOLLARS,
  getOrCreateGraphChangeSchema,
  TokenPerCapacity,
  Sr25519Signature,
  assertEvent,
  getCurrentItemizedHash,
  getCurrentPaginatedHash,
  generateItemizedSignaturePayload,
  createDelegator,
  generatePaginatedUpsertSignaturePayload,
  generatePaginatedDeleteSignaturePayload,
  getOrCreateDummySchema,
  getOrCreateAvroChatMessageItemizedSchema,
  getOrCreateParquetBroadcastSchema,
  getOrCreateAvroChatMessagePaginatedSchema,
  generateItemizedSignaturePayloadV2,
  generatePaginatedUpsertSignaturePayloadV2,
  generatePaginatedDeleteSignaturePayloadV2,
  getCapacity,
  getTestHandle
} from "../scaffolding/helpers";
import { FeeDetails } from "@polkadot/types/interfaces";
import { ipfsCid } from "../messages/ipfs";
import { getFundingSource } from "../scaffolding/funding";

describe("Capacity Transactions", function () {
  const fundingSource = getFundingSource("capacity-transactions");

  const FUNDS_AMOUNT: bigint = 50n * DOLLARS;

  describe("pay_with_capacity", function () {
    describe("when caller has a Capacity account", async function () {
      let schemaId: u16;
      const amountStaked = 2n * DOLLARS;

      before(async function () {
        // Create schemas for testing with Grant Delegation to test pay_with_capacity
        schemaId = await getOrCreateGraphChangeSchema(fundingSource);
        assert.notEqual(schemaId, undefined, "setup should populate schemaId");
      });

      async function assertAddNewKey(capacityKeys: KeyringPair, addKeyPayload: AddKeyData, newControlKeypair: KeyringPair) {
        const addKeyPayloadCodec: Codec = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", addKeyPayload);
        const ownerSig: Sr25519Signature = signPayloadSr25519(capacityKeys, addKeyPayloadCodec);
        const newSig: Sr25519Signature = signPayloadSr25519(newControlKeypair, addKeyPayloadCodec);
        const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(capacityKeys, ownerSig, newSig, addKeyPayload);
        const { eventMap } = await addPublicKeyOp.signAndSend();
        assertEvent(eventMap, "system.ExtrinsicSuccess");
        assertEvent(eventMap, "msa.PublicKeyAdded");
      }

      function getCapacityFee(chainEvents: EventMap): bigint {
        if (chainEvents["capacity.CapacityWithdrawn"] &&
          ExtrinsicHelper.api.events.capacity.CapacityWithdrawn.is(chainEvents["capacity.CapacityWithdrawn"])) {
          return chainEvents["capacity.CapacityWithdrawn"].data.amount.toBigInt();
        }
        return 0n;
      }

      it("fails when a provider makes an eligible extrinsic call using non-funded control key", async function () {
        const capacityKeys = createKeys("CapacityKeysNSF");
        const capacityProvider = await createMsaAndProvider(fundingSource, capacityKeys, "CapProvNSF", FUNDS_AMOUNT);
        // this will first fund 'capacityKeys' before staking
        await assert.doesNotReject(stakeToProvider(fundingSource, capacityKeys, capacityProvider, amountStaked));

        // As current owner, add a new set of control keys that do not have a balance.
        let newControlKeypair = createKeys("NewKeyNoBalance");
        const newPublicKey = newControlKeypair.publicKey;
        const addKeyPayload: AddKeyData = await generateAddKeyPayload(
          { msaId: capacityProvider, newPublicKey: newPublicKey }
        );
        await assertAddNewKey(capacityKeys, addKeyPayload, newControlKeypair);

        // attempt a capacity transaction using the new unfunded key: claimHandle
        const handle = getTestHandle();
        const expiration = (await getBlockNumber()) + 10;
        const handle_vec = new Bytes(ExtrinsicHelper.api.registry, handle);
        const handlePayload = {
          baseHandle: handle_vec,
          expiration: expiration,
        };
        const claimHandlePayload = ExtrinsicHelper.api.registry.createType("CommonPrimitivesHandlesClaimHandlePayload", handlePayload);
        const claimHandle = ExtrinsicHelper.claimHandle(newControlKeypair, claimHandlePayload);
        await assert.rejects(claimHandle.payWithCapacity('current'), {
          name: "RpcError", message:
            "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low"
        });
      });

      describe("when capacity eligible transaction is from the msa pallet", async function () {
        let capacityKeys: KeyringPair;
        let capacityProvider: u64;
        let delegatorKeys: KeyringPair;
        let payload: any = {};
        const stakedForMsa = 15n * DOLLARS

        before(async function () {
          capacityKeys = createKeys("CapacityKeys");
          capacityProvider = await createMsaAndProvider(fundingSource, capacityKeys, "CapacityProvider", FUNDS_AMOUNT);
          await assert.doesNotReject(stakeToProvider(fundingSource, capacityKeys, capacityProvider, stakedForMsa));
        })

        beforeEach(async function () {
          delegatorKeys = createKeys("delegatorKeys")
          payload = await generateDelegationPayload({
            authorizedMsaId: capacityProvider,
            schemaIds: [schemaId],
          });
        })

        it("successfully pays with Capacity for eligible transaction - addPublicKeytoMSA", async function () {
          let authorizedKeys: KeyringPair[] = [];
          let defaultPayload: AddKeyData = {};
          let payload: AddKeyData;
          let ownerSig: Sr25519Signature;
          let newSig: Sr25519Signature;
          let addKeyData: Codec;

          authorizedKeys.push(await createAndFundKeypair(fundingSource, 50_000_000n));
          defaultPayload.msaId = capacityProvider
          defaultPayload.newPublicKey = authorizedKeys[0].publicKey;

          payload = await generateAddKeyPayload(defaultPayload);
          addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);
          ownerSig = signPayloadSr25519(capacityKeys, addKeyData);
          newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
          const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(capacityKeys, ownerSig, newSig, payload);

          const { eventMap } = await addPublicKeyOp.payWithCapacity();
          assertEvent(eventMap, "system.ExtrinsicSuccess");
          assertEvent(eventMap, "capacity.CapacityWithdrawn");
          assertEvent(eventMap, "msa.PublicKeyAdded");
        });

        it("successfully pays with Capacity for eligible transaction - createSponsoredAccountWithDelegation", async function () {
          const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
          const call = ExtrinsicHelper.createSponsoredAccountWithDelegation(
            delegatorKeys, capacityKeys,
            signPayloadSr25519(delegatorKeys, addProviderData),
            payload
          );
          const { eventMap } = await call.payWithCapacity();
          assertEvent(eventMap, "system.ExtrinsicSuccess");
          assertEvent(eventMap, "capacity.CapacityWithdrawn");
          assertEvent(eventMap, "msa.DelegationGranted");
        });

        it("successfully pays with Capacity for eligible transaction - grantDelegation", async function () {

          await fundKeypair(fundingSource, delegatorKeys, 10n * CENTS);

          let { eventMap } = await ExtrinsicHelper.createMsa(delegatorKeys).signAndSend();
          assertEvent(eventMap, "msa.MsaCreated");

          const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
          const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, capacityKeys,
            signPayloadSr25519(delegatorKeys, addProviderData), payload);

          ({ eventMap } = await grantDelegationOp.payWithCapacity());

          // Check for remaining capacity to be reduced
          const capacityStaked = await getCapacity(capacityProvider);

          assertEvent(eventMap, "system.ExtrinsicSuccess");
          assertEvent(eventMap, "capacity.CapacityWithdrawn");
          assertEvent(eventMap, "msa.DelegationGranted");

          let fee = getCapacityFee(eventMap);
          // assuming no other txns charged against capacity (b/c of async tests), this should be the maximum amount left.
          const maximumExpectedRemaining = stakedForMsa / TokenPerCapacity - fee

          let remaining = capacityStaked.remainingCapacity.toBigInt();
          assert(remaining <= maximumExpectedRemaining, `expected ${remaining} to be <= ${maximumExpectedRemaining}`);
          assert.equal(capacityStaked.totalTokensStaked.toBigInt(), stakedForMsa);
          assert.equal(capacityStaked.totalCapacityIssued.toBigInt(), stakedForMsa / TokenPerCapacity);
        });
      });

      describe("when capacity eligible transaction is from the messages pallet", async function () {
        let starting_block: number;
        let capacityKeys: KeyringPair;
        let capacityProvider: u64;

        before(async function () {
          capacityKeys = createKeys("CapacityKeys");
          capacityProvider = await createMsaAndProvider(fundingSource, capacityKeys, "CapacityProvider", FUNDS_AMOUNT);
        })

        beforeEach(async function () {
          starting_block = (await ExtrinsicHelper.apiPromise.rpc.chain.getHeader()).number.toNumber();
          await assert.doesNotReject(stakeToProvider(fundingSource, capacityKeys, capacityProvider, amountStaked));
        });

        it("successfully pays with Capacity for eligible transaction - addIPFSMessage", async function () {
          let schemaId = await getOrCreateParquetBroadcastSchema(fundingSource);
          const ipfs_payload_data = "This is a test of Frequency.";
          const ipfs_payload_len = ipfs_payload_data.length + 1;
          const ipfs_cid_64 = (await ipfsCid(ipfs_payload_data, './e2e_test.txt')).toString(base64);
          const call = ExtrinsicHelper.addIPFSMessage(capacityKeys, schemaId, ipfs_cid_64, ipfs_payload_len);

          const { eventMap } = await call.payWithCapacity();
          assertEvent(eventMap, "capacity.CapacityWithdrawn");
          assertEvent(eventMap, "messages.MessagesInBlock");
        });

        it("successfully pays with Capacity for eligible transaction - addOnchainMessage", async function () {
          // Create a dummy on-chain schema
          let dummySchemaId: u16 = await getOrCreateDummySchema(fundingSource);
          const call = ExtrinsicHelper.addOnChainMessage(capacityKeys, dummySchemaId, "0xdeadbeef");
          const { eventMap } = await call.payWithCapacity();
          assertEvent(eventMap, "capacity.CapacityWithdrawn");
          assertEvent(eventMap, "messages.MessagesInBlock");
          const get = await ExtrinsicHelper.apiPromise.rpc.messages.getBySchemaId(
            dummySchemaId,
            {
              from_block: starting_block,
              from_index: 0,
              to_block: starting_block + 999,
              page_size: 999
            }
          );
          const response: MessageResponse = get.content[get.content.length - 1];
          assert.equal(response.payload, "0xdeadbeef", "payload should be 0xdeadbeef");
        });
      });

      describe("when capacity eligible transaction is from the StatefulStorage pallet", async function () {
        let delegatorKeys: KeyringPair;
        let delegatorProviderId: u64;
        let capacityKeys: KeyringPair;
        let capacityProvider: u64;

        before(async function () {
          capacityKeys = createKeys("CapacityKeys");
          capacityProvider = await createMsaAndProvider(fundingSource, capacityKeys, "CapacityProvider", FUNDS_AMOUNT);
        })
        beforeEach(async function () {
          await assert.doesNotReject(stakeToProvider(fundingSource, capacityKeys, capacityProvider, amountStaked));
        });

        it("successfully pays with Capacity for eligible transaction - applyItemActions", async function () {
          // Create a schema to allow delete actions
          let schemaId_deletable: SchemaId = await getOrCreateAvroChatMessageItemizedSchema(fundingSource)

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
          const { eventMap } = await call.payWithCapacity();
          assertEvent(eventMap, "system.ExtrinsicSuccess");
          assertEvent(eventMap, "capacity.CapacityWithdrawn");
          assertEvent(eventMap, "statefulStorage.ItemizedPageUpdated");
        });

        it("successfully pays with Capacity for eligible transaction - upsertPage; deletePage", async function () {
          // Get a schema for Paginated PayloadLocation
          schemaId = await getOrCreateAvroChatMessagePaginatedSchema(fundingSource);
          let page_id = 0;
          let target_hash = await getCurrentPaginatedHash(capacityProvider, schemaId, page_id)

          // Add and update actions
          const payload_1 = new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency");
          const call = ExtrinsicHelper.upsertPage(capacityKeys, schemaId, capacityProvider, page_id, payload_1, target_hash);
          const { eventMap } = await call.payWithCapacity();
          assertEvent(eventMap, "system.ExtrinsicSuccess");
          assertEvent(eventMap, "capacity.CapacityWithdrawn");
          assertEvent(eventMap, "statefulStorage.PaginatedPageUpdated")

          // Remove the page
          target_hash = await getCurrentPaginatedHash(capacityProvider, schemaId, page_id)
          const call2 = ExtrinsicHelper.removePage(capacityKeys, schemaId, capacityProvider, page_id, target_hash);
          const { eventMap: eventMap2 } = await call2.payWithCapacity();
          assertEvent(eventMap2, "system.ExtrinsicSuccess");
          assertEvent(eventMap2, "capacity.CapacityWithdrawn");
          assertEvent(eventMap2, "statefulStorage.PaginatedPageDeleted");
        });

        it("successfully pays with Capacity for eligible transaction - applyItemActionsWithSignature", async function () {
          // Create a schema for Itemized PayloadLocation
          let itemizedSchemaId: SchemaId = await getOrCreateAvroChatMessageItemizedSchema(fundingSource);

          // Create a MSA for the delegator
          [delegatorKeys, delegatorProviderId] = await createDelegator(fundingSource);
          assert.notEqual(delegatorKeys, undefined, "setup should populate delegator_key");
          assert.notEqual(delegatorProviderId, undefined, "setup should populate msa_id");

          // Add and update actions
          let payload_1 = new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency");
          const add_action = {
            "Add": payload_1
          }

          let payload_2 = new Bytes(ExtrinsicHelper.api.registry, "Hello World Again From Frequency");
          const update_action = {
            "Add": payload_2
          }

          const target_hash = await getCurrentItemizedHash(delegatorProviderId, itemizedSchemaId);

          let add_actions = [add_action, update_action];
          const payload = await generateItemizedSignaturePayload({
            msaId: delegatorProviderId,
            targetHash: target_hash,
            schemaId: itemizedSchemaId,
            actions: add_actions,
          });
          const itemizedPayloadData = ExtrinsicHelper.api.registry.createType("PalletStatefulStorageItemizedSignaturePayload", payload);
          let itemized_add_result_1 = ExtrinsicHelper.applyItemActionsWithSignature(delegatorKeys, capacityKeys, signPayloadSr25519(delegatorKeys, itemizedPayloadData), payload);
          const { target: pageUpdateEvent1, eventMap } = await itemized_add_result_1.payWithCapacity();
          assertEvent(eventMap, "system.ExtrinsicSuccess");
          assertEvent(eventMap, "capacity.CapacityWithdrawn");
          assert.notEqual(pageUpdateEvent1, undefined, "should have returned a PalletStatefulStorageItemizedActionApplied event");
        });

        it("successfully pays with Capacity for eligible transaction - applyItemActionsWithSignatureV2", async function () {
          // Create a schema for Itemized PayloadLocation
          let itemizedSchemaId: SchemaId = await getOrCreateAvroChatMessageItemizedSchema(fundingSource);

          // Create a MSA for the delegator
          [delegatorKeys, delegatorProviderId] = await createDelegator(fundingSource);
          assert.notEqual(delegatorKeys, undefined, "setup should populate delegator_key");
          assert.notEqual(delegatorProviderId, undefined, "setup should populate msa_id");

          // Add and update actions
          let payload_1 = new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency");
          const add_action = {
            "Add": payload_1
          }

          let payload_2 = new Bytes(ExtrinsicHelper.api.registry, "Hello World Again From Frequency");
          const update_action = {
            "Add": payload_2
          }

          const target_hash = await getCurrentItemizedHash(delegatorProviderId, itemizedSchemaId);

          let add_actions = [add_action, update_action];
          const payload = await generateItemizedSignaturePayloadV2({
            targetHash: target_hash,
            schemaId: itemizedSchemaId,
            actions: add_actions,
          });
          const itemizedPayloadData = ExtrinsicHelper.api.registry.createType("PalletStatefulStorageItemizedSignaturePayloadV2", payload);
          let itemized_add_result_1 = ExtrinsicHelper.applyItemActionsWithSignatureV2(delegatorKeys, capacityKeys, signPayloadSr25519(delegatorKeys, itemizedPayloadData), payload);
          const { target: pageUpdateEvent1, eventMap } = await itemized_add_result_1.payWithCapacity();
          assertEvent(eventMap, "system.ExtrinsicSuccess");
          assertEvent(eventMap, "capacity.CapacityWithdrawn");
          assert.notEqual(pageUpdateEvent1, undefined, "should have returned a PalletStatefulStorageItemizedActionApplied event");
        });

        it("successfully pays with Capacity for eligible transaction - upsertPageWithSignature; deletePageWithSignature", async function () {
          let paginatedSchemaId: SchemaId = await getOrCreateAvroChatMessagePaginatedSchema(fundingSource);

          // Create a MSA for the delegator
          [delegatorKeys, delegatorProviderId] = await createDelegator(fundingSource);
          assert.notEqual(delegatorKeys, undefined, "setup should populate delegator_key");
          assert.notEqual(delegatorProviderId, undefined, "setup should populate msa_id");

          let page_id = new u16(ExtrinsicHelper.api.registry, 1);

          // Add and update actions
          let target_hash = await getCurrentPaginatedHash(delegatorProviderId, paginatedSchemaId, page_id.toNumber());
          const upsertPayload = await generatePaginatedUpsertSignaturePayload({
            msaId: delegatorProviderId,
            targetHash: target_hash,
            schemaId: paginatedSchemaId,
            pageId: page_id,
            payload: new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency"),
          });
          const upsertPayloadData = ExtrinsicHelper.api.registry.createType("PalletStatefulStoragePaginatedUpsertSignaturePayload", upsertPayload);
          let upsert_result = ExtrinsicHelper.upsertPageWithSignature(delegatorKeys, capacityKeys, signPayloadSr25519(delegatorKeys, upsertPayloadData), upsertPayload);
          const { target: pageUpdateEvent, eventMap: eventMap1 } = await upsert_result.payWithCapacity();
          assertEvent(eventMap1, "system.ExtrinsicSuccess");
          assertEvent(eventMap1, "capacity.CapacityWithdrawn");
          assert.notEqual(pageUpdateEvent, undefined, "should have returned a PalletStatefulStoragePaginatedPageUpdate event");

          // Remove the page
          target_hash = await getCurrentPaginatedHash(delegatorProviderId, paginatedSchemaId, page_id.toNumber());
          const deletePayload = await generatePaginatedDeleteSignaturePayload({
            msaId: delegatorProviderId,
            targetHash: target_hash,
            schemaId: paginatedSchemaId,
            pageId: page_id,
          });
          const deletePayloadData = ExtrinsicHelper.api.registry.createType("PalletStatefulStoragePaginatedDeleteSignaturePayload", deletePayload);
          let remove_result = ExtrinsicHelper.deletePageWithSignature(delegatorKeys, capacityKeys, signPayloadSr25519(delegatorKeys, deletePayloadData), deletePayload);
          const { target: pageRemove, eventMap: eventMap2 } = await remove_result.payWithCapacity();
          assertEvent(eventMap2, "system.ExtrinsicSuccess");
          assertEvent(eventMap2, "capacity.CapacityWithdrawn");
          assert.notEqual(pageRemove, undefined, "should have returned a event");

          // no pages should exist
          const result = await ExtrinsicHelper.getPaginatedStorage(delegatorProviderId, paginatedSchemaId);
          assert.notEqual(result, undefined, "should have returned a valid response");
          assert.equal(result.length, 0, "should returned no paginated pages");
        });

        it("successfully pays with Capacity for eligible transaction - upsertPageWithSignatureV2; deletePageWithSignatureV2", async function () {
          let paginatedSchemaId: SchemaId = await getOrCreateAvroChatMessagePaginatedSchema(fundingSource);

          // Create a MSA for the delegator
          [delegatorKeys, delegatorProviderId] = await createDelegator(fundingSource);
          assert.notEqual(delegatorKeys, undefined, "setup should populate delegator_key");
          assert.notEqual(delegatorProviderId, undefined, "setup should populate msa_id");

          let page_id = new u16(ExtrinsicHelper.api.registry, 1);

          // Add and update actions
          let target_hash = await getCurrentPaginatedHash(delegatorProviderId, paginatedSchemaId, page_id.toNumber());
          const upsertPayload = await generatePaginatedUpsertSignaturePayloadV2({
            targetHash: target_hash,
            schemaId: paginatedSchemaId,
            pageId: page_id,
            payload: new Bytes(ExtrinsicHelper.api.registry, "Hello World From Frequency"),
          });
          const upsertPayloadData = ExtrinsicHelper.api.registry.createType("PalletStatefulStoragePaginatedUpsertSignaturePayloadV2", upsertPayload);
          let upsert_result = ExtrinsicHelper.upsertPageWithSignatureV2(delegatorKeys, capacityKeys, signPayloadSr25519(delegatorKeys, upsertPayloadData), upsertPayload);
          const { target: pageUpdateEvent, eventMap: eventMap1 } = await upsert_result.payWithCapacity();
          assertEvent(eventMap1, "system.ExtrinsicSuccess");
          assertEvent(eventMap1, "capacity.CapacityWithdrawn");
          assert.notEqual(pageUpdateEvent, undefined, "should have returned a PalletStatefulStoragePaginatedPageUpdate event");

          // Remove the page
          target_hash = await getCurrentPaginatedHash(delegatorProviderId, paginatedSchemaId, page_id.toNumber());
          const deletePayload = await generatePaginatedDeleteSignaturePayloadV2({
            targetHash: target_hash,
            schemaId: paginatedSchemaId,
            pageId: page_id,
          });
          const deletePayloadData = ExtrinsicHelper.api.registry.createType("PalletStatefulStoragePaginatedDeleteSignaturePayloadV2", deletePayload);
          let remove_result = ExtrinsicHelper.deletePageWithSignatureV2(delegatorKeys, capacityKeys, signPayloadSr25519(delegatorKeys, deletePayloadData), deletePayload);
          const { target: pageRemove, eventMap: eventMap2 } = await remove_result.payWithCapacity();
          assertEvent(eventMap2, "system.ExtrinsicSuccess");
          assertEvent(eventMap2, "capacity.CapacityWithdrawn");
          assert.notEqual(pageRemove, undefined, "should have returned a event");

          // no pages should exist
          const result = await ExtrinsicHelper.getPaginatedStorage(delegatorProviderId, paginatedSchemaId);
          assert.notEqual(result, undefined, "should have returned a valid response");
          assert.equal(result.length, 0, "should returned no paginated pages");
        });
      });

      describe("when capacity eligible transaction is from the handles pallet", async function () {
        let capacityKeys: KeyringPair;
        let capacityProvider: u64;

        before(async function () {
          capacityKeys = createKeys("CapacityKeys");
          capacityProvider = await createMsaAndProvider(fundingSource, capacityKeys, "CapacityProvider", FUNDS_AMOUNT);
        })
        it("successfully pays with Capacity for eligible transaction - claimHandle", async function () {
          await assert.doesNotReject(stakeToProvider(fundingSource, capacityKeys, capacityProvider, amountStaked));

          const handle = getTestHandle();
          const expiration = (await getBlockNumber()) + 10;
          const handle_vec = new Bytes(ExtrinsicHelper.api.registry, handle);
          const handlePayload = {
            baseHandle: handle_vec,
            expiration: expiration,
          };
          const claimHandlePayload = ExtrinsicHelper.api.registry.createType("CommonPrimitivesHandlesClaimHandlePayload", handlePayload);
          const claimHandle = ExtrinsicHelper.claimHandle(capacityKeys, claimHandlePayload);
          const { eventMap } = await claimHandle.payWithCapacity();
          assertEvent(eventMap, "system.ExtrinsicSuccess");
          assertEvent(eventMap, "capacity.CapacityWithdrawn");
          assertEvent(eventMap, "handles.HandleClaimed");
        });
      });

      // When a user attempts to pay for a non-capacity transaction with Capacity,
      // it should error and drop the transaction from the transaction-pool.
      it("fails to pay with Capacity for a non-capacity transaction", async function () {
        const capacityKeys = createKeys("CapacityKeys");
        const capacityProvider = await createMsaAndProvider(fundingSource, capacityKeys, "CapacityProvider", FUNDS_AMOUNT);
        const nonCapacityTxn = ExtrinsicHelper.stake(capacityKeys, capacityProvider, 1n * CENTS);
        await assert.rejects(nonCapacityTxn.payWithCapacity('current'), {
          name: "RpcError", message:
            "1010: Invalid Transaction: Custom error: 0"
        });
      });

      // When a user does not have enough capacity to pay for the transaction fee
      // and is NOT eligible to replenish Capacity, it should error and be dropped
      // from the transaction pool.
      it("fails to pay for a transaction with empty capacity", async function () {
        const capacityKeys = createKeys("CapKeysEmpty");
        const capacityProvider = await createMsaAndProvider(fundingSource, capacityKeys, "CapProvEmpty", FUNDS_AMOUNT);
        let noCapacityKeys = createKeys("noCapacityKeys");
        let _providerId = await createMsaAndProvider(fundingSource, noCapacityKeys, "NoCapProvider");

        let delegatorKeys = createKeys("delegatorKeys");

        const payload = await generateDelegationPayload({
          authorizedMsaId: capacityProvider,
          schemaIds: [schemaId],
        });
        const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
        const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, noCapacityKeys,
          signPayloadSr25519(noCapacityKeys, addProviderData), payload);

        await assert.rejects(grantDelegationOp.payWithCapacity('current'), {
          name: "RpcError", message:
            "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low"
        });
      });

      // *All keys should have at least an EXISTENTIAL_DEPOSIT = 1M.
      it("fails to pay for transaction when key does has not met the min deposit", async function () {
        const capacityKeys = createKeys("CapKeysUnder");
        const capacityProvider = await createMsaAndProvider(fundingSource, capacityKeys, "CapProvUnder", FUNDS_AMOUNT);
        const noTokensKeys = createKeys("noTokensKeys");
        const delegatorKeys = await createAndFundKeypair(fundingSource, 2n * DOLLARS, "delegatorKeys");

        await assert.doesNotReject(stakeToProvider(fundingSource, capacityKeys, capacityProvider, 1n * DOLLARS));

        // Add new key
        let newKeyPayload: AddKeyData = await generateAddKeyPayload({
          msaId: new u64(ExtrinsicHelper.api.registry, capacityProvider), newPublicKey: noTokensKeys.publicKey
        });
        let addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", newKeyPayload);

        let ownerSig = signPayloadSr25519(capacityKeys, addKeyData);
        let newSig = signPayloadSr25519(noTokensKeys, addKeyData);
        const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(capacityKeys, ownerSig, newSig, newKeyPayload);

        const { target: publicKeyEvent } = await addPublicKeyOp.fundAndSend(fundingSource);
        assert.notEqual(publicKeyEvent, undefined, 'should have added public key');

        const createMsaOp = ExtrinsicHelper.createMsa(delegatorKeys);
        const { target: MsaCreatedEvent } = await createMsaOp.fundAndSend(fundingSource);
        assert.notEqual(MsaCreatedEvent, undefined, "should have returned MsaCreated event");

        const payload = await generateDelegationPayload({
          authorizedMsaId: capacityProvider,
          schemaIds: [schemaId],
        });
        const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
        const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, noTokensKeys,
          signPayloadSr25519(delegatorKeys, addProviderData), payload);

        await assert.rejects(grantDelegationOp.payWithCapacity('current'), {
          name: 'RpcError',
          message: "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low",
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
        delegatorProviderId = await createMsaAndProvider(fundingSource, delegatorKeys, "Delegator", FUNDS_AMOUNT);
        schemaId = new u16(ExtrinsicHelper.api.registry, 0);
      });

      describe("but has an MSA account that has not been registered as a Provider", async function () {
        it("fails to pay for a transaction", async function () {
          // Create a keypair with msaId, but no provider
          const noProviderKeys = await createAndFundKeypair(fundingSource, FUNDS_AMOUNT, "NoProviderKeys");

          const createMsaOp = ExtrinsicHelper.createMsa(noProviderKeys);

          const { target: msaCreatedEvent } = await createMsaOp.fundAndSend(fundingSource);
          assert.notEqual(msaCreatedEvent, undefined, "should have returned MsaCreated event");

          // When a user is not a registered provider and attempts to pay with Capacity,
          // it should error with InvalidTransaction::Payment, which is a 1010 error, Inability to pay some fees.
          const payload = await generateDelegationPayload({
            authorizedMsaId: delegatorProviderId,
            schemaIds: [schemaId],
          });
          const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
          const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, noProviderKeys,
            signPayloadSr25519(delegatorKeys, addProviderData), payload);

          await assert.rejects(grantDelegationOp.payWithCapacity('current'), {
            name: "RpcError", message:
              "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low"
          });
        });
      });

      describe("and does not have an MSA account associated to signing keys", async function () {
        it("fails to pay for a transaction", async function () {
          let emptyKeys = await createAndFundKeypair(fundingSource, 50_000_000n);

          const payload = await generateDelegationPayload({
            authorizedMsaId: delegatorProviderId,
            schemaIds: [schemaId],
          });
          const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
          const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, emptyKeys,
            signPayloadSr25519(delegatorKeys, addProviderData), payload);

          await assert.rejects(grantDelegationOp.payWithCapacity('current'), {
            name: "RpcError", message:
              "1010: Invalid Transaction: Custom error: 1"
          });
        });
      });
    });
  });

  describe("pay_with_capacity_batch_all", function () {
    let capacityProviderKeys: KeyringPair;
    let capacityProvider: u64;
    let schemaId: u16;
    let defaultPayload: AddProviderPayload;
    const amountStaked = 9n * DOLLARS;

    beforeEach(async function () {
      capacityProviderKeys = createKeys("CapacityProviderKeys");
      capacityProvider = await createMsaAndProvider(fundingSource, capacityProviderKeys, "CapacityProvider", FUNDS_AMOUNT);
      defaultPayload = {
        authorizedMsaId: capacityProvider,
        schemaIds: [schemaId],
      }
    });

    it("successfully pays with Capacity for a batch of eligible transactions - [createSponsoredAccountWithDelegation, claimHandle]", async function () {
      await assert.doesNotReject(stakeToProvider(fundingSource, capacityProviderKeys, capacityProvider, amountStaked));

      const addProviderPayload = await generateDelegationPayload({ ...defaultPayload });
      const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", addProviderPayload);
      let delegatorKeys = createKeys("delegatorKeys");
      const createSponsoredAccountWithDelegation = ExtrinsicHelper.api.tx.msa.createSponsoredAccountWithDelegation(
        delegatorKeys.publicKey,
        signPayloadSr25519(delegatorKeys, addProviderData),
        addProviderPayload
      );

      const handle = getTestHandle();
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

      const { target: batchCompletedEvent, eventMap } = await payWithCapacityBatchAllOp.signAndSend();

      if (batchCompletedEvent &&
        !(ExtrinsicHelper.api.events.utility.BatchCompleted.is(batchCompletedEvent))) {
        assert.fail("should return a BatchCompletedEvent");
      }

      assert.notEqual(eventMap["msa.DelegationGranted"], undefined, 'should have returned DelegationGranted event');
      assert.notEqual(eventMap["handles.HandleClaimed"], undefined, 'should have returned HandleClaimed event');
    });

    it("batch fails if one transaction fails - [createSponsoredAccountWithDelegation, claimHandle]", async function () {
      await assert.doesNotReject(stakeToProvider(fundingSource, capacityProviderKeys, capacityProvider, amountStaked));

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

  describe("Capacity RPC", function () {
    let capacityProviderKeys: KeyringPair;
    let capacityProvider: u64;
    let schemaId: u16;
    let defaultPayload: AddProviderPayload;
    const amountStaked = 9n * DOLLARS;

    beforeEach(async function () {
      capacityProviderKeys = createKeys("CapacityProviderKeys");
      capacityProvider = await createMsaAndProvider(fundingSource, capacityProviderKeys, "CapacityProvider", FUNDS_AMOUNT);
      defaultPayload = {
        authorizedMsaId: capacityProvider,
        schemaIds: [schemaId],
      }
    });

    it("Returns `FeeDetails` when requesting capacity cost of a transaction", async function () {
      const addProviderPayload = await generateDelegationPayload({ ...defaultPayload });
      const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", addProviderPayload);
      let delegatorKeys = createKeys("delegatorKeys");
      const call = ExtrinsicHelper.api.tx.msa.createSponsoredAccountWithDelegation(
        delegatorKeys.publicKey,
        signPayloadSr25519(delegatorKeys, addProviderData),
        addProviderPayload
      );

      // Actual weights and fee
      const { weight: { refTime, proofSize }, } = await ExtrinsicHelper.apiPromise.call.transactionPaymentApi.queryInfo(call.toHex(), 0);
      const weightFee = await ExtrinsicHelper.apiPromise.call.transactionPaymentApi.queryWeightToFee({refTime, proofSize});

      const feeDetails: FeeDetails = await ExtrinsicHelper.apiPromise.rpc.frequencyTxPayment.computeCapacityFeeDetails(call.toHex(), null);
      assert.notEqual(feeDetails, undefined, "should have returned a feeDetails");
      assert.notEqual(feeDetails.inclusionFee, undefined, "should have returned a partialFee");
      assert(feeDetails.inclusionFee.isSome, "should have returned a partialFee");
      const { baseFee, lenFee, adjustedWeightFee } = feeDetails.inclusionFee.toJSON() as any;
      assert(Math.abs(baseFee - 106382) < 10_000, "The base fee appears to be wrong or have changed more than expected");
      assert(Math.abs(lenFee - 1170000) < 100, "The len fee appears to be wrong or have changed more than expected");
      // This is comparing stable weight, which has no impact from targeted_fee_adjustment, with actual weights.
      assert(Math.abs(adjustedWeightFee - weightFee.toNumber()) < 10_000, "The adjusted weight fee appears to be wrong or have changed more than expected");
    });

    it("Returns `FeeDetails` when requesting capacity cost of a transaction when wrapped in payWithCapacity", async function () {
      const addProviderPayload = await generateDelegationPayload({ ...defaultPayload });
      const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", addProviderPayload);
      let delegatorKeys = createKeys("delegatorKeys");
      const insideTx = ExtrinsicHelper.api.tx.msa.createSponsoredAccountWithDelegation(
        delegatorKeys.publicKey,
        signPayloadSr25519(delegatorKeys, addProviderData),
        addProviderPayload
      );
      const tx = ExtrinsicHelper.api.tx.frequencyTxPayment.payWithCapacity(insideTx);
      const feeDetails: FeeDetails = await ExtrinsicHelper.apiPromise.rpc.frequencyTxPayment.computeCapacityFeeDetails(tx.toHex(), null);
      assert.notEqual(feeDetails, undefined, "should have returned a feeDetails");
      assert.notEqual(feeDetails.inclusionFee, undefined, "should have returned a partialFee");
      assert(feeDetails.inclusionFee.isSome, "should have returned a partialFee");
      const { baseFee, lenFee, adjustedWeightFee } = feeDetails.inclusionFee.toJSON() as any;

      // payWithCapacity costs will fluctuate, so just checking that they are valid positive numbers
      assert(baseFee > 1, "The base fee appears to be wrong.");
      assert(lenFee > 1, "The len fee appears to be wrong.");
      assert(adjustedWeightFee > 1, "The adjusted weight fee appears to be wrong");
    });

    it("Returns nothing when requesting capacity cost of a non-capacity transaction", async function () {
      const tx = ExtrinsicHelper.api.tx.msa.retireMsa();
      const feeDetails: FeeDetails = await ExtrinsicHelper.apiPromise.rpc.frequencyTxPayment.computeCapacityFeeDetails(tx.toHex(), null);
      assert.notEqual(feeDetails, undefined, "should have returned a feeDetails");
      assert(feeDetails.inclusionFee.isNone, "should have returned something for the inclusionFee");
    });

    it("Returns nothing when requesting pay with capacity call with a non-capacity transaction", async function () {
      const insideTx = ExtrinsicHelper.api.tx.msa.retireMsa();
      const tx = ExtrinsicHelper.api.tx.frequencyTxPayment.payWithCapacity(insideTx);
      const feeDetails: FeeDetails = await ExtrinsicHelper.apiPromise.rpc.frequencyTxPayment.computeCapacityFeeDetails(tx.toHex(), null);
      assert.notEqual(feeDetails, undefined, "should have returned a feeDetails");
      assert(feeDetails.inclusionFee.isNone, "should have returned something for the inclusionFee");
    });
  });
});
