// E2E tests for pallets/stateful-pallet-storage/handlePaginated.ts
import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  createProviderKeysAndId,
  createDelegatorAndDelegation,
  getCurrentPaginatedHash,
  createMsa,
  DOLLARS,
  getOrCreateAvroChatMessagePaginatedSchema,
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { AVRO_CHAT_MESSAGE } from './fixtures/itemizedSchemaType';
import { MessageSourceId, SchemaId } from '@frequency-chain/api-augment/interfaces';
import { Bytes, u16, u64 } from '@polkadot/types';
import { getFundingSource } from '../scaffolding/funding';

const badSchemaId = 65_534;
const fundingSource = getFundingSource('stateful-storage-handle-paginated');

describe('📗 Stateful Pallet Storage Paginated', function () {
  let schemaId: SchemaId;
  let schemaId_unsupported: SchemaId;
  let delegatorKeys: KeyringPair;
  let msa_id: MessageSourceId;
  let providerId: MessageSourceId;
  let providerKeys: KeyringPair;
  let badMsaId: u64;

  before(async function () {
    // Create a provider for the MSA, the provider will be used to grant delegation
    [providerKeys, providerId] = await createProviderKeysAndId(fundingSource, 2n * DOLLARS);
    assert.notEqual(providerId, undefined, 'setup should populate providerId');
    assert.notEqual(providerKeys, undefined, 'setup should populate providerKeys');

    // Create a schema for Paginated PayloadLocation
    schemaId = await getOrCreateAvroChatMessagePaginatedSchema(providerKeys);

    // Create non supported schema
    schemaId_unsupported = await ExtrinsicHelper.getOrCreateSchemaV3(
      providerKeys,
      AVRO_CHAT_MESSAGE,
      'AvroBinary',
      'OnChain',
      [],
      'test.handlePaginatedUnsupported'
    );

    // Create a MSA for the delegator and delegate to the provider
    [delegatorKeys, msa_id] = await createDelegatorAndDelegation(fundingSource, schemaId, providerId, providerKeys);
    assert.notEqual(msa_id, undefined, 'setup should populate msa_id');

    // Create an MSA that is not a provider to be used for testing failure cases
    [badMsaId] = await createMsa(fundingSource);
  });

  describe('Paginated Storage Upsert/Remove Tests 😊/😥', function () {
    it('✅ should be able to call upsert page and add a page and remove a page via id', async function () {
      let page_id = 0;
      let target_hash = await getCurrentPaginatedHash(msa_id, schemaId, page_id);

      // Add and update actions
      const payload_1 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency');
      const paginated_add_result_1 = ExtrinsicHelper.upsertPage(
        providerKeys,
        schemaId,
        msa_id,
        page_id,
        payload_1,
        target_hash
      );
      const { target: pageUpdateEvent1, eventMap: chainEvents } =
        await paginated_add_result_1.fundAndSend(fundingSource);
      assert.notEqual(
        chainEvents['system.ExtrinsicSuccess'],
        undefined,
        'should have returned an ExtrinsicSuccess event'
      );
      assert.notEqual(
        chainEvents['transactionPayment.TransactionFeePaid'],
        undefined,
        'should have returned a TransactionFeePaid event'
      );
      assert.notEqual(
        pageUpdateEvent1,
        undefined,
        'should have returned a PalletStatefulStoragepaginatedActionApplied event'
      );

      // Add another page
      page_id = 1;
      target_hash = await getCurrentPaginatedHash(msa_id, schemaId, page_id);
      const paginated_add_result_2 = ExtrinsicHelper.upsertPage(
        providerKeys,
        schemaId,
        msa_id,
        page_id,
        payload_1,
        target_hash
      );
      const { target: pageUpdateEvent2, eventMap: chainEvents2 } =
        await paginated_add_result_2.fundAndSend(fundingSource);
      assert.notEqual(
        chainEvents2['system.ExtrinsicSuccess'],
        undefined,
        'should have returned an ExtrinsicSuccess event'
      );
      assert.notEqual(
        chainEvents2['transactionPayment.TransactionFeePaid'],
        undefined,
        'should have returned a TransactionFeePaid event'
      );
      assert.notEqual(
        pageUpdateEvent2,
        undefined,
        'should have returned a PalletStatefulStoragepaginatedActionApplied event'
      );

      // Remove the second page
      target_hash = await getCurrentPaginatedHash(msa_id, schemaId, page_id);
      const paginated_remove_result_1 = ExtrinsicHelper.removePage(
        providerKeys,
        schemaId,
        msa_id,
        page_id,
        target_hash
      );
      const { target: pageRemove, eventMap: chainEvents3 } = await paginated_remove_result_1.fundAndSend(fundingSource);
      assert.notEqual(
        chainEvents3['system.ExtrinsicSuccess'],
        undefined,
        'should have returned an ExtrinsicSuccess event'
      );
      assert.notEqual(
        chainEvents3['transactionPayment.TransactionFeePaid'],
        undefined,
        'should have returned a TransactionFeePaid event'
      );
      assert.notEqual(pageRemove, undefined, 'should have returned a event');
    });

    it('🛑 should fail call to upsert page with invalid schemaId', async function () {
      const page_id = 0;
      const target_hash = await getCurrentPaginatedHash(msa_id, schemaId, page_id);
      const payload_1 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency');
      const fake_schema_id = new u16(ExtrinsicHelper.api.registry, badSchemaId);
      const paginated_add_result_1 = ExtrinsicHelper.upsertPage(
        delegatorKeys,
        fake_schema_id,
        msa_id,
        page_id,
        payload_1,
        target_hash
      );
      await assert.rejects(paginated_add_result_1.fundAndSend(fundingSource), {
        name: 'InvalidSchemaId',
        section: 'statefulStorage',
      });
    });

    it('🛑 should fail call to upsert page with invalid schema location', async function () {
      const page_id = 0;
      const target_hash = await getCurrentPaginatedHash(msa_id, schemaId, page_id);
      const payload_1 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency');
      const paginated_add_result_1 = ExtrinsicHelper.upsertPage(
        delegatorKeys,
        schemaId_unsupported,
        msa_id,
        page_id,
        payload_1,
        target_hash
      );
      await assert.rejects(paginated_add_result_1.fundAndSend(fundingSource), {
        name: 'SchemaPayloadLocationMismatch',
        section: 'statefulStorage',
      });
    });

    it('🛑 should fail call to upsert page with for un-delegated attempts', async function () {
      const page_id = 0;
      const payload_1 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency');

      const target_hash = await getCurrentPaginatedHash(msa_id, schemaId, page_id);
      const paginated_add_result_1 = ExtrinsicHelper.upsertPage(
        providerKeys,
        schemaId,
        badMsaId,
        page_id,
        payload_1,
        target_hash
      );
      await assert.rejects(paginated_add_result_1.fundAndSend(fundingSource), {
        name: 'UnauthorizedDelegate',
        section: 'statefulStorage',
      });
    });

    it('🛑 should fail call to upsert page with stale target hash', async function () {
      const page_id = 0;
      const payload_1 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency');

      const paginated_add_result_1 = ExtrinsicHelper.upsertPage(providerKeys, schemaId, msa_id, page_id, payload_1, 0);
      await assert.rejects(paginated_add_result_1.signAndSend('current'), {
        name: 'StalePageState',
        section: 'statefulStorage',
      });
    });
  });

  describe('Paginated Storage Removal Negative Tests 😊/😥', function () {
    it('🛑 should fail call to remove page with invalid schemaId', async function () {
      const page_id = 0;
      const paginated_add_result_1 = ExtrinsicHelper.removePage(delegatorKeys, badSchemaId, msa_id, page_id, 0);
      await assert.rejects(paginated_add_result_1.fundAndSend(fundingSource), {
        name: 'InvalidSchemaId',
        section: 'statefulStorage',
      });
    });

    it('🛑 should fail call to remove page with invalid schema location', async function () {
      const page_id = 0;
      const paginated_add_result_1 = ExtrinsicHelper.removePage(
        delegatorKeys,
        schemaId_unsupported,
        msa_id,
        page_id,
        0
      );
      await assert.rejects(paginated_add_result_1.fundAndSend(fundingSource), {
        name: 'SchemaPayloadLocationMismatch',
        section: 'statefulStorage',
      });
    });
  });

  describe('Paginated Storage RPC Tests', function () {
    it('✅ should be able to call get_paginated_storage and get paginated data', async function () {
      const result = await ExtrinsicHelper.getPaginatedStorage(msa_id, schemaId);
      assert.notEqual(result, undefined, 'should have returned a valid response');
      assert.notEqual(result.length, 0, 'should have returned paginated responses');
      assert.notEqual(result[0].hash, undefined, 'should have returned a valid page');
    });
  });
});
