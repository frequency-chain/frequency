import '@frequency-chain/api-augment';
import { KeyringPair } from '@polkadot/keyring/types';
import { Bytes, u64, u16 } from '@polkadot/types';
import assert from 'assert';
import { AddKeyData, RecoveryCommitmentPayload, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { base64 } from 'multiformats/bases/base64';
import { SchemaId } from '@frequency-chain/api-augment/interfaces';
import {
  generateRecoverySecret,
  getRecoveryCommitment,
  ContactType,
  getIntermediaryHashes,
} from '@frequency-chain/recovery-sdk';
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
  getTokenPerCapacity,
  assertEvent,
  getCurrentItemizedHash,
  getCurrentPaginatedHash,
  generateItemizedSignaturePayload,
  getOrCreateDummySchema,
  getOrCreateAvroChatMessageItemizedSchema,
  getOrCreateParquetBroadcastSchema,
  getOrCreateAvroChatMessagePaginatedSchema,
  generatePaginatedUpsertSignaturePayloadV2,
  generatePaginatedDeleteSignaturePayloadV2,
  getCapacity,
  getTestHandle,
  assertHasMessage,
  createMsa,
  generateRecoveryCommitmentPayload,
} from '../scaffolding/helpers';
import { ipfsCid } from '../messages/ipfs';
import { getFundingSource } from '../scaffolding/funding';
import { getUnifiedPublicKey } from '@frequency-chain/ethereum-utils';

const FUNDS_AMOUNT: bigint = 50n * DOLLARS;
let fundingSource: KeyringPair;

describe('Capacity Transactions', function () {
  describe('pay_with_capacity', function () {
    describe('when caller has a Capacity account', function () {
      let schemaId: u16;
      const amountStaked = 3n * DOLLARS;

      before(async function () {
        fundingSource = await getFundingSource(import.meta.url);
        // Create schemas for testing with Grant Delegation to test pay_with_capacity
        schemaId = await getOrCreateGraphChangeSchema(fundingSource);
        assert.notEqual(schemaId, undefined, 'setup should populate schemaId');
      });

      describe('when capacity eligible transaction is from the msa pallet', function () {
        let capacityKeys: KeyringPair;
        let capacityProvider: u64;
        let delegatorKeys: KeyringPair;
        let payload: any = {};
        const stakedForMsa = 15n * DOLLARS;

        before(async function () {
          capacityKeys = createKeys('CapacityKeys');
          capacityProvider = await createMsaAndProvider(fundingSource, capacityKeys, 'CapacityProvider', FUNDS_AMOUNT);
          // Stake enough for all transactions
          await assert.doesNotReject(stakeToProvider(fundingSource, fundingSource, capacityProvider, stakedForMsa));
        });

        beforeEach(async function () {
          delegatorKeys = createKeys('delegatorKeys');
          payload = await generateDelegationPayload({
            authorizedMsaId: capacityProvider,
            schemaIds: [schemaId],
          });
        });

        it('successfully pays with Capacity for eligible transaction - addPublicKeytoMSA', async function () {
          const authorizedKeys: KeyringPair[] = [];
          const defaultPayload: AddKeyData = {};

          authorizedKeys.push(await createAndFundKeypair(fundingSource, 50_000_000n));
          defaultPayload.msaId = capacityProvider;
          defaultPayload.newPublicKey = getUnifiedPublicKey(authorizedKeys[0]);

          const payload = await generateAddKeyPayload(defaultPayload);
          const addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', payload);
          const ownerSig = signPayloadSr25519(capacityKeys, addKeyData);
          const newSig = signPayloadSr25519(authorizedKeys[0], addKeyData);
          const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(capacityKeys, ownerSig, newSig, payload);

          const { eventMap } = await addPublicKeyOp.payWithCapacity();
          assertEvent(eventMap, 'system.ExtrinsicSuccess');
          assertEvent(eventMap, 'capacity.CapacityWithdrawn');
          assertEvent(eventMap, 'msa.PublicKeyAdded');
        });

        it('successfully pays with Capacity for eligible transaction - createSponsoredAccountWithDelegation', async function () {
          const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);
          const call = ExtrinsicHelper.createSponsoredAccountWithDelegation(
            delegatorKeys,
            capacityKeys,
            signPayloadSr25519(delegatorKeys, addProviderData),
            payload
          );
          const { eventMap } = await call.payWithCapacity();
          assertEvent(eventMap, 'system.ExtrinsicSuccess');
          assertEvent(eventMap, 'capacity.CapacityWithdrawn');
          assertEvent(eventMap, 'msa.DelegationGranted');
        });

        it('successfully pays with Capacity for eligible transaction - grantDelegation', async function () {
          await fundKeypair(fundingSource, delegatorKeys, 10n * CENTS);

          let { eventMap } = await ExtrinsicHelper.createMsa(delegatorKeys).signAndSend();
          assertEvent(eventMap, 'msa.MsaCreated');

          const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);
          const grantDelegationOp = ExtrinsicHelper.grantDelegation(
            delegatorKeys,
            capacityKeys,
            signPayloadSr25519(delegatorKeys, addProviderData),
            payload
          );

          ({ eventMap } = await grantDelegationOp.payWithCapacity());

          // Check for remaining capacity to be reduced
          const capacityStaked = await getCapacity(capacityProvider);

          assertEvent(eventMap, 'system.ExtrinsicSuccess');
          assertEvent(eventMap, 'capacity.CapacityWithdrawn');
          assertEvent(eventMap, 'msa.DelegationGranted');

          const fee = ExtrinsicHelper.getCapacityFee(eventMap);
          // assuming no other txns charged against capacity (b/c of async tests), this should be the maximum amount left.
          const maximumExpectedRemaining = stakedForMsa / getTokenPerCapacity() - fee;

          const remaining = capacityStaked.remainingCapacity.toBigInt();
          assert(remaining <= maximumExpectedRemaining, `expected ${remaining} to be <= ${maximumExpectedRemaining}`);
          assert.equal(capacityStaked.totalTokensStaked.toBigInt(), stakedForMsa);
          assert.equal(capacityStaked.totalCapacityIssued.toBigInt(), stakedForMsa / getTokenPerCapacity());
        });

        it('successfully pays with Capacity for eligible transaction - addRecoveryCommitment', async function () {
          // Generate a recovery secret using the Recovery SDK
          const recoverySecret = generateRecoverySecret();

          // Generate Recovery Commitment using the Recovery SDK with test email contact
          const testEmail = 'test@example.com';
          const recoveryCommitment = getRecoveryCommitment(recoverySecret, ContactType.EMAIL, testEmail);

          const expiration = (await getBlockNumber()) + 10;
          const recoveryCommitmentData: RecoveryCommitmentPayload = {
            discriminant: 'RecoveryCommitmentPayload',
            recoveryCommitment,
            expiration,
          };

          const payload = await generateRecoveryCommitmentPayload(recoveryCommitmentData);
          const recoveryCommitmentPayload = ExtrinsicHelper.api.registry.createType(
            'PalletMsaRecoveryCommitmentPayload',
            payload
          );
          const signature = signPayloadSr25519(capacityKeys, recoveryCommitmentPayload);
          const addRecoveryCommitmentOp = ExtrinsicHelper.addRecoveryCommitment(capacityKeys, signature, payload);

          const { eventMap } = await addRecoveryCommitmentOp.payWithCapacity();
          assertEvent(eventMap, 'system.ExtrinsicSuccess');
          assertEvent(eventMap, 'capacity.CapacityWithdrawn');
          assertEvent(eventMap, 'msa.RecoveryCommitmentAdded');
        });

        it('successfully pays with Capacity for eligible transaction - recoverAccount', async function () {
          const defaultPayload: AddKeyData = {};
          const lostKey = await createAndFundKeypair(fundingSource, 50_000_000n);
          const recoveryKey = createKeys('RecoveryKey');

          // Create an MSA to use as the lost key
          const { eventMap: setupEventMap } = await ExtrinsicHelper.createMsa(lostKey).signAndSend();
          assertEvent(setupEventMap, 'msa.MsaCreated');
          const msaCreatedEvent = setupEventMap['msa.MsaCreated'];

          // Store the msaId and recovery key that will be used in the recovery payload
          defaultPayload.msaId = msaCreatedEvent.data[0] as u64;
          defaultPayload.newPublicKey = getUnifiedPublicKey(recoveryKey);

          // Generate a recovery secret using the Recovery SDK
          const recoverySecret = generateRecoverySecret();
          const testEmail = 'test@example.com';

          // Add a recovery commitment to the lostKey MSA
          const recoveryCommitment = getRecoveryCommitment(recoverySecret, ContactType.EMAIL, testEmail);
          const expiration = (await getBlockNumber()) + 10;
          const recoveryCommitmentData: RecoveryCommitmentPayload = {
            discriminant: 'RecoveryCommitmentPayload',
            recoveryCommitment,
            expiration,
          };

          const recoveryPayload = await generateRecoveryCommitmentPayload(recoveryCommitmentData);
          const recoveryCommitmentCodec = ExtrinsicHelper.api.registry.createType(
            'PalletMsaRecoveryCommitmentPayload',
            recoveryPayload
          );
          const recoverySignature = signPayloadSr25519(lostKey, recoveryCommitmentCodec);
          const addRecoveryCommitmentOp = ExtrinsicHelper.addRecoveryCommitment(
            lostKey,
            recoverySignature,
            recoveryPayload
          );
          await addRecoveryCommitmentOp.signAndSend();

          // Create and approve a recovery provider - use the capacityKeys that already has capacity staked
          const recoveryProviderKeys = capacityKeys;

          // Approve the recovery provider
          await ExtrinsicHelper.approveRecoveryProvider(fundingSource, recoveryProviderKeys).signAndSend();

          // Generate the payload for adding a new control key, required for recovery
          const payload = await generateAddKeyPayload(defaultPayload);
          const addKeyDataCodec = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', payload);
          const newSig = signPayloadSr25519(recoveryKey, addKeyDataCodec);

          // Generate Recovery Intermediary Hashes
          const { a, b } = getIntermediaryHashes(recoverySecret, ContactType.EMAIL, testEmail);

          // Recover the account using the recovery provider
          const recoverAccountOp = ExtrinsicHelper.recoverAccount(recoveryProviderKeys, a, b, newSig, payload);

          const { eventMap } = await recoverAccountOp.payWithCapacity();
          assertEvent(eventMap, 'system.ExtrinsicSuccess');
          assertEvent(eventMap, 'capacity.CapacityWithdrawn');
          assertEvent(eventMap, 'msa.AccountRecovered');
        });
      });

      describe('when capacity eligible transaction is from the messages pallet', function () {
        let starting_block: number;
        let capacityKeys: KeyringPair;
        let capacityProvider: u64;

        before(async function () {
          capacityKeys = createKeys('CapacityKeys');
          capacityProvider = await createMsaAndProvider(fundingSource, capacityKeys, 'CapacityProvider', FUNDS_AMOUNT);
          const numberOfTests = BigInt(this.test!.parent!.tests.length);
          // Stake the amount for each test
          await assert.doesNotReject(
            stakeToProvider(fundingSource, fundingSource, capacityProvider, numberOfTests * amountStaked)
          );
        });

        beforeEach(async function () {
          starting_block = (await ExtrinsicHelper.apiPromise.rpc.chain.getHeader()).number.toNumber();
        });

        it('successfully pays with Capacity for eligible transaction - addIPFSMessage', async function () {
          const schemaId = await getOrCreateParquetBroadcastSchema(fundingSource);
          const ipfs_payload_data = 'This is a test of Frequency.';
          const ipfs_payload_len = ipfs_payload_data.length + 1;
          const ipfs_cid_64 = (await ipfsCid(ipfs_payload_data, './e2e_test.txt')).toString(base64);
          const call = ExtrinsicHelper.addIPFSMessage(capacityKeys, schemaId, ipfs_cid_64, ipfs_payload_len);

          const { eventMap } = await call.payWithCapacity();
          assertEvent(eventMap, 'capacity.CapacityWithdrawn');
          // messages.MessagesInBlock in block might not be on this transaction if there are others
          assertEvent(eventMap, 'system.ExtrinsicSuccess');
        });

        it('successfully pays with Capacity for eligible transaction - addOnchainMessage', async function () {
          // Create a dummy on-chain schema
          const dummySchemaId: u16 = await getOrCreateDummySchema(fundingSource);
          const call = ExtrinsicHelper.addOnChainMessage(capacityKeys, dummySchemaId, '0xdeadbeef');
          const { eventMap } = await call.payWithCapacity();
          assertEvent(eventMap, 'capacity.CapacityWithdrawn');
          // messages.MessagesInBlock in block might not be on this transaction if there are others
          assertEvent(eventMap, 'system.ExtrinsicSuccess');
          const get = await ExtrinsicHelper.apiPromise.rpc.messages.getBySchemaId(dummySchemaId, {
            from_block: starting_block,
            from_index: 0,
            to_block: starting_block + 999,
            page_size: 999,
          });
          assertHasMessage(get, (x) => x.payload.isSome && x.payload.toString() === '0xdeadbeef');
        });
      });

      describe('when capacity eligible transaction is from the StatefulStorage pallet', function () {
        let delegatorKeys: KeyringPair;
        let delegatorProviderId: u64;
        let capacityKeys: KeyringPair;
        let capacityProvider: u64;

        before(async function () {
          capacityKeys = createKeys('CapacityKeys');
          capacityProvider = await createMsaAndProvider(fundingSource, capacityKeys, 'CapacityProvider', FUNDS_AMOUNT);
          // Create a MSA for the delegator
          [delegatorProviderId, delegatorKeys] = await createMsa(fundingSource);
          assert.notEqual(delegatorKeys, undefined, 'setup should populate delegator_key');
          assert.notEqual(delegatorProviderId, undefined, 'setup should populate msa_id');

          // Stake the amount for each test
          const numberOfTests = BigInt(this.test!.parent!.tests.length);
          await assert.doesNotReject(
            stakeToProvider(fundingSource, fundingSource, capacityProvider, numberOfTests * amountStaked)
          );
        });

        it('successfully pays with Capacity for eligible transaction - applyItemActions', async function () {
          // Create a schema to allow delete actions
          const schemaId_deletable: SchemaId = await getOrCreateAvroChatMessageItemizedSchema(fundingSource);

          // Add and update actions
          const payload_1 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency');
          const add_action = {
            Add: payload_1,
          };

          const payload_2 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World Again From Frequency');
          const update_action = {
            Add: payload_2,
          };

          const target_hash = await getCurrentItemizedHash(capacityProvider, schemaId_deletable);

          const add_actions = [add_action, update_action];
          const call = ExtrinsicHelper.applyItemActions(
            capacityKeys,
            schemaId_deletable,
            capacityProvider,
            add_actions,
            target_hash
          );
          const { eventMap } = await call.payWithCapacity();
          assertEvent(eventMap, 'system.ExtrinsicSuccess');
          assertEvent(eventMap, 'capacity.CapacityWithdrawn');
          assertEvent(eventMap, 'statefulStorage.ItemizedPageUpdated');
        });

        it('successfully pays with Capacity for eligible transaction - upsertPage; deletePage', async function () {
          // Get a schema for Paginated PayloadLocation
          schemaId = await getOrCreateAvroChatMessagePaginatedSchema(fundingSource);
          const page_id = 0;
          let target_hash = await getCurrentPaginatedHash(capacityProvider, schemaId, page_id);

          // Add and update actions
          const payload_1 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency');
          const call = ExtrinsicHelper.upsertPage(
            capacityKeys,
            schemaId,
            capacityProvider,
            page_id,
            payload_1,
            target_hash
          );
          const { eventMap } = await call.payWithCapacity();
          assertEvent(eventMap, 'system.ExtrinsicSuccess');
          assertEvent(eventMap, 'capacity.CapacityWithdrawn');
          assertEvent(eventMap, 'statefulStorage.PaginatedPageUpdated');

          // Remove the page
          target_hash = await getCurrentPaginatedHash(capacityProvider, schemaId, page_id);
          const call2 = ExtrinsicHelper.removePage(capacityKeys, schemaId, capacityProvider, page_id, target_hash);
          const { eventMap: eventMap2 } = await call2.payWithCapacity();
          assertEvent(eventMap2, 'system.ExtrinsicSuccess');
          assertEvent(eventMap2, 'capacity.CapacityWithdrawn');
          assertEvent(eventMap2, 'statefulStorage.PaginatedPageDeleted');
        });

        it('successfully pays with Capacity for eligible transaction - applyItemActionsWithSignatureV2', async function () {
          // Create a schema for Itemized PayloadLocation
          const itemizedSchemaId: SchemaId = await getOrCreateAvroChatMessageItemizedSchema(fundingSource);

          // Add and update actions
          const payload_1 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency');
          const add_action = {
            Add: payload_1,
          };

          const payload_2 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World Again From Frequency');
          const update_action = {
            Add: payload_2,
          };

          const target_hash = await getCurrentItemizedHash(delegatorProviderId, itemizedSchemaId);

          const add_actions = [add_action, update_action];
          const payload = await generateItemizedSignaturePayload({
            targetHash: target_hash,
            schemaId: itemizedSchemaId,
            actions: add_actions,
          });
          const itemizedPayloadData = ExtrinsicHelper.api.registry.createType(
            'PalletStatefulStorageItemizedSignaturePayloadV2',
            payload
          );
          const itemized_add_result_1 = ExtrinsicHelper.applyItemActionsWithSignatureV2(
            delegatorKeys,
            capacityKeys,
            signPayloadSr25519(delegatorKeys, itemizedPayloadData),
            payload
          );
          const { target: pageUpdateEvent1, eventMap } = await itemized_add_result_1.payWithCapacity();
          assertEvent(eventMap, 'system.ExtrinsicSuccess');
          assertEvent(eventMap, 'capacity.CapacityWithdrawn');
          assert.notEqual(
            pageUpdateEvent1,
            undefined,
            'should have returned a PalletStatefulStorageItemizedActionApplied event'
          );
        });

        it('successfully pays with Capacity for eligible transaction - upsertPageWithSignatureV2; deletePageWithSignatureV2', async function () {
          const paginatedSchemaId: SchemaId = await getOrCreateAvroChatMessagePaginatedSchema(fundingSource);

          const page_id = new u16(ExtrinsicHelper.api.registry, 1);

          // Add and update actions
          let target_hash = await getCurrentPaginatedHash(delegatorProviderId, paginatedSchemaId, page_id.toNumber());
          const upsertPayload = await generatePaginatedUpsertSignaturePayloadV2({
            targetHash: target_hash,
            schemaId: paginatedSchemaId,
            pageId: page_id,
            payload: new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency'),
          });
          const upsertPayloadData = ExtrinsicHelper.api.registry.createType(
            'PalletStatefulStoragePaginatedUpsertSignaturePayloadV2',
            upsertPayload
          );
          const upsert_result = ExtrinsicHelper.upsertPageWithSignatureV2(
            delegatorKeys,
            capacityKeys,
            signPayloadSr25519(delegatorKeys, upsertPayloadData),
            upsertPayload
          );
          const { target: pageUpdateEvent, eventMap: eventMap1 } = await upsert_result.payWithCapacity();
          assertEvent(eventMap1, 'system.ExtrinsicSuccess');
          assertEvent(eventMap1, 'capacity.CapacityWithdrawn');
          assert.notEqual(
            pageUpdateEvent,
            undefined,
            'should have returned a PalletStatefulStoragePaginatedPageUpdate event'
          );

          // Remove the page
          target_hash = await getCurrentPaginatedHash(delegatorProviderId, paginatedSchemaId, page_id.toNumber());
          const deletePayload = await generatePaginatedDeleteSignaturePayloadV2({
            targetHash: target_hash,
            schemaId: paginatedSchemaId,
            pageId: page_id,
          });
          const deletePayloadData = ExtrinsicHelper.api.registry.createType(
            'PalletStatefulStoragePaginatedDeleteSignaturePayloadV2',
            deletePayload
          );
          const remove_result = ExtrinsicHelper.deletePageWithSignatureV2(
            delegatorKeys,
            capacityKeys,
            signPayloadSr25519(delegatorKeys, deletePayloadData),
            deletePayload
          );
          const { target: pageRemove, eventMap: eventMap2 } = await remove_result.payWithCapacity();
          assertEvent(eventMap2, 'system.ExtrinsicSuccess');
          assertEvent(eventMap2, 'capacity.CapacityWithdrawn');
          assert.notEqual(pageRemove, undefined, 'should have returned a event');

          // no pages should exist
          const result = await ExtrinsicHelper.getPaginatedStorage(delegatorProviderId, paginatedSchemaId);
          assert.notEqual(result, undefined, 'should have returned a valid response');
          assert.equal(result.length, 0, 'should returned no paginated pages');
        });
      });

      describe('when capacity eligible transaction is from the handles pallet', function () {
        let capacityKeys: KeyringPair;
        let capacityProvider: u64;

        before(async function () {
          capacityKeys = createKeys('CapacityKeys');
          capacityProvider = await createMsaAndProvider(fundingSource, capacityKeys, 'CapacityProvider', FUNDS_AMOUNT);
        });

        it('successfully pays with Capacity for eligible transaction - claimHandle', async function () {
          await assert.doesNotReject(stakeToProvider(fundingSource, fundingSource, capacityProvider, amountStaked));

          const handle = getTestHandle();
          const expiration = (await getBlockNumber()) + 10;
          const handle_vec = new Bytes(ExtrinsicHelper.api.registry, handle);
          const handlePayload = {
            baseHandle: handle_vec,
            expiration: expiration,
          };
          const claimHandlePayload = ExtrinsicHelper.api.registry.createType(
            'CommonPrimitivesHandlesClaimHandlePayload',
            handlePayload
          );
          const claimHandle = ExtrinsicHelper.claimHandle(capacityKeys, claimHandlePayload);
          const { eventMap } = await claimHandle.payWithCapacity();
          assertEvent(eventMap, 'system.ExtrinsicSuccess');
          assertEvent(eventMap, 'capacity.CapacityWithdrawn');
          assertEvent(eventMap, 'handles.HandleClaimed');
        });
      });

      describe('when capacity eligible transaction and balance less than ED', function () {
        let capacityKeys: KeyringPair;
        let capacityProvider: u64;

        before(async function () {
          capacityKeys = createKeys('CapacityKeys');
          capacityProvider = await createMsaAndProvider(fundingSource, capacityKeys, 'CapacityProvider', FUNDS_AMOUNT);
        });

        it('successfully pays with Capacity for eligible transaction - claimHandle [available balance < ED]', async function () {
          await assert.doesNotReject(stakeToProvider(fundingSource, capacityKeys, capacityProvider, amountStaked));
          // Empty the account to ensure the balance is less than ED
          await ExtrinsicHelper.emptyAccount(capacityKeys, fundingSource).signAndSend();
          // Confirm that the available balance is less than ED
          // The available balance is the free balance minus the frozen balance
          const capacityAcctInfo = await ExtrinsicHelper.getAccountInfo(capacityKeys);
          assert.equal(capacityAcctInfo.data.frozen.toBigInt(), amountStaked);
          assert.equal(capacityAcctInfo.data.free.toBigInt(), amountStaked);

          // Confirm that a transfer fails because the available balance is 0
          const failTransferObj = ExtrinsicHelper.transferFunds(capacityKeys, fundingSource, 1n * CENTS);
          assert.rejects(failTransferObj.signAndSend('current'), {
            name: 'RpcError',
            message: '1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low',
          });

          const handle = getTestHandle();
          const expiration = (await getBlockNumber()) + 10;
          const handle_vec = new Bytes(ExtrinsicHelper.api.registry, handle);
          const handlePayload = {
            baseHandle: handle_vec,
            expiration: expiration,
          };
          const claimHandlePayload = ExtrinsicHelper.api.registry.createType(
            'CommonPrimitivesHandlesClaimHandlePayload',
            handlePayload
          );
          const claimHandle = ExtrinsicHelper.claimHandle(capacityKeys, claimHandlePayload);
          const { eventMap } = await claimHandle.payWithCapacity();
          assertEvent(eventMap, 'system.ExtrinsicSuccess');
          assertEvent(eventMap, 'capacity.CapacityWithdrawn');
          assertEvent(eventMap, 'handles.HandleClaimed');
        });
      });
    });
  });
});
