// E2E tests for pallets/stateful-pallet-storage/handleItemizedWithSignature.ts
import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  DOLLARS,
  createDelegatorAndDelegation,
  createProviderKeysAndId,
  generateItemizedActions,
  generateItemizedActionsSignedPayloadV2,
  generatePaginatedDeleteSignaturePayloadV2,
  generatePaginatedUpsertSignaturePayloadV2,
  getCurrentPaginatedHash,
  signPayload,
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { AVRO_CHAT_MESSAGE } from '../stateful-pallet-storage/fixtures/itemizedSchemaType';
import { MessageSourceId, SchemaId } from '@frequency-chain/api-augment/interfaces';
import { Bytes, u16 } from '@polkadot/types';
import { getFundingSource } from '../scaffolding/funding';

const fundingSource = getFundingSource(import.meta.url);

describe('📗 Stateful Pallet Storage Ethereum', function () {
  let itemizedSchemaId: SchemaId;
  let paginatedSchemaId: SchemaId;
  let msa_id: MessageSourceId;
  let undelegatedProviderId: MessageSourceId;
  let undelegatedProviderKeys: KeyringPair;
  let delegatedProviderId: MessageSourceId;
  let delegatedProviderKeys: KeyringPair;
  let ethereumDelegatorKeys: KeyringPair;

  before(async function () {
    // Create a provider. This provider will NOT be granted delegations;
    // methods requiring a payload signature do not require a delegation
    [undelegatedProviderKeys, undelegatedProviderId] = await createProviderKeysAndId(fundingSource, 2n * DOLLARS);
    assert.notEqual(undelegatedProviderId, undefined, 'setup should populate undelegatedProviderId');
    assert.notEqual(undelegatedProviderKeys, undefined, 'setup should populate undelegatedProviderKeys');

    // Create a provider for the MSA, the provider will be used to grant delegation
    [delegatedProviderKeys, delegatedProviderId] = await createProviderKeysAndId(fundingSource, 2n * DOLLARS);
    assert.notEqual(delegatedProviderId, undefined, 'setup should populate delegatedProviderId');
    assert.notEqual(delegatedProviderKeys, undefined, 'setup should populate delegatedProviderKeys');

    // Create a schema for Itemized PayloadLocation
    itemizedSchemaId = await ExtrinsicHelper.getOrCreateSchemaV3(
      undelegatedProviderKeys,
      AVRO_CHAT_MESSAGE,
      'AvroBinary',
      'Itemized',
      ['AppendOnly', 'SignatureRequired'],
      'test.ItemizedSignatureRequired'
    );

    // Create a schema for Paginated PayloadLocation
    paginatedSchemaId = await ExtrinsicHelper.getOrCreateSchemaV3(
      undelegatedProviderKeys,
      AVRO_CHAT_MESSAGE,
      'AvroBinary',
      'Paginated',
      ['SignatureRequired'],
      'test.PaginatedSignatureRequired'
    );

    // Create a MSA for the delegator
    [ethereumDelegatorKeys, msa_id] = await createDelegatorAndDelegation(
      fundingSource,
      [itemizedSchemaId, paginatedSchemaId],
      delegatedProviderId,
      delegatedProviderKeys,
      'ethereum'
    );
    console.log('after createDelegatorAndDelegation');
    assert.notEqual(ethereumDelegatorKeys, undefined, 'setup should populate delegator_key');
    assert.notEqual(msa_id, undefined, 'setup should populate msa_id');
  });

  describe('Itemized With Signature Storage Tests', function () {
    it('provider should be able to call applyItemizedActionWithSignatureV2 and apply actions with Ethereum keys', async function () {
      const { payload, signature } = await generateItemizedActionsSignedPayloadV2(
        generateItemizedActions([
          { action: 'Add', value: 'Hello, world from Frequency' },
          { action: 'Add', value: 'Hello, world again from Frequency' },
        ]),
        itemizedSchemaId,
        ethereumDelegatorKeys,
        msa_id
      );

      const itemized_add_result_1 = ExtrinsicHelper.applyItemActionsWithSignatureV2(
        ethereumDelegatorKeys,
        undelegatedProviderKeys,
        signature,
        payload
      );
      const { target: pageUpdateEvent1, eventMap: chainEvents } =
        await itemized_add_result_1.fundAndSend(fundingSource);
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
        'should have returned a PalletStatefulStorageItemizedActionApplied event'
      );
    });
  });

  describe('Paginated With Signature Storage Tests with Ethereum keys', function () {
    it('provider should be able to call upsertPageWithSignatureV2 a page and deletePageWithSignatureV2 it successfully with Ethereum keys', async function () {
      const page_id = new u16(ExtrinsicHelper.api.registry, 1);

      // Add and update actions
      let target_hash = await getCurrentPaginatedHash(msa_id, paginatedSchemaId, page_id.toNumber());
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
        ethereumDelegatorKeys,
        undelegatedProviderKeys,
        signPayload(ethereumDelegatorKeys, upsertPayloadData),
        upsertPayload
      );
      const { target: pageUpdateEvent, eventMap: chainEvents1 } = await upsert_result.fundAndSend(fundingSource);
      assert.notEqual(
        chainEvents1['system.ExtrinsicSuccess'],
        undefined,
        'should have returned an ExtrinsicSuccess event'
      );
      assert.notEqual(
        chainEvents1['transactionPayment.TransactionFeePaid'],
        undefined,
        'should have returned a TransactionFeePaid event'
      );
      assert.notEqual(
        pageUpdateEvent,
        undefined,
        'should have returned a PalletStatefulStoragePaginatedPageUpdate event'
      );

      // Remove the page
      target_hash = await getCurrentPaginatedHash(msa_id, paginatedSchemaId, page_id.toNumber());
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
        ethereumDelegatorKeys,
        undelegatedProviderKeys,
        signPayload(ethereumDelegatorKeys, deletePayloadData),
        deletePayload
      );
      const { target: pageRemove, eventMap: chainEvents2 } = await remove_result.fundAndSend(fundingSource);
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
      assert.notEqual(pageRemove, undefined, 'should have returned a event');

      // no pages should exist
      const result = await ExtrinsicHelper.getPaginatedStorage(msa_id, paginatedSchemaId);
      assert.notEqual(result, undefined, 'should have returned a valid response');
      const thePage = result.toArray().find((page) => page.page_id === page_id);
      assert.equal(thePage, undefined, 'inserted page should not exist');
    });
  });
});
