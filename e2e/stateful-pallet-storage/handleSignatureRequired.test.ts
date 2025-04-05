// E2E tests for pallets/stateful-pallet-storage/handleItemizedWithSignature.ts
import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  DOLLARS,
  createDelegatorAndDelegation,
  createProviderKeysAndId,
  generateItemizedActions,
  generateItemizedActionsSignedPayload,
  generateItemizedActionsSignedPayloadV2,
  generatePaginatedDeleteSignaturePayload,
  generatePaginatedDeleteSignaturePayloadV2,
  generatePaginatedUpsertSignaturePayload,
  generatePaginatedUpsertSignaturePayloadV2,
  getCurrentItemizedHash,
  getCurrentPaginatedHash,
  signPayloadSr25519, assertExtrinsicSucceededAndFeesPaid,
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { AVRO_CHAT_MESSAGE } from '../stateful-pallet-storage/fixtures/itemizedSchemaType';
import { MessageSourceId, SchemaId } from '@frequency-chain/api-augment/interfaces';
import { Bytes, u16 } from '@polkadot/types';
import { getFundingSource } from '../scaffolding/funding';

const fundingSource = getFundingSource(import.meta.url);

describe('📗 Stateful Pallet Storage Signature Required', function () {
  let itemizedSchemaId: SchemaId;
  let paginatedSchemaId: SchemaId;
  let msa_id: MessageSourceId;
  let undelegatedProviderId: MessageSourceId;
  let undelegatedProviderKeys: KeyringPair;
  let delegatedProviderId: MessageSourceId;
  let delegatedProviderKeys: KeyringPair;
  let delegatorKeys: KeyringPair;

  let itemizedActionsSignedPayload;

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
    [delegatorKeys, msa_id] = await createDelegatorAndDelegation(
      fundingSource,
      [itemizedSchemaId, paginatedSchemaId],
      delegatedProviderId,
      delegatedProviderKeys
    );
    assert.notEqual(delegatorKeys, undefined, 'setup should populate delegator_key');
    assert.notEqual(msa_id, undefined, 'setup should populate msa_id');

    itemizedActionsSignedPayload = await generateItemizedActionsSignedPayload(
        generateItemizedActions([
          { action: 'Add', value: 'Hello, world from Frequency' },
          { action: 'Add', value: 'Hello, world again from Frequency' },
        ]),
        itemizedSchemaId,
        delegatorKeys,
        msa_id
    );

  });

  describe('Itemized With Signature Storage Tests', function () {
    // passes
    it('provider should be able to call applyItemizedActionWithSignature and apply actions', async function () {
      const { payload, signature } = itemizedActionsSignedPayload;

      const itemized_add_result_1 = ExtrinsicHelper.applyItemActionsWithSignature(
        delegatorKeys,
        undelegatedProviderKeys,
        signature,
        payload
      );
      const { target: pageUpdateEvent1, eventMap: chainEvents } = await itemized_add_result_1.fundAndSend(fundingSource);

      assertExtrinsicSucceededAndFeesPaid(chainEvents);

      assert.notEqual(
        pageUpdateEvent1,
        undefined,
        'should have returned a PalletStatefulStorageItemizedActionApplied event'
      );
    });

    // fails
    it('delegator (owner) should be able to call applyItemizedActionWithSignature and apply actions', async function () {
      const { payload, signature } = itemizedActionsSignedPayload;

      const itemized_add_result_1 = ExtrinsicHelper.applyItemActionsWithSignature(
        delegatorKeys,
        delegatorKeys,
        signature,
        payload
      );
      const { target: pageUpdateEvent1, eventMap: chainEvents } =
        await itemized_add_result_1.fundAndSend(fundingSource);

      assertExtrinsicSucceededAndFeesPaid(chainEvents);

      assert.notEqual(
        pageUpdateEvent1,
        undefined,
        'should have returned a PalletStatefulStorageItemizedActionApplied event'
      );
    });

    it('provider should be able to call applyItemizedActionWithSignatureV2 and apply actions', async function () {
      const { payload, signature } = await generateItemizedActionsSignedPayloadV2(
        generateItemizedActions([
          { action: 'Add', value: 'Hello, world from Frequency' },
          { action: 'Add', value: 'Hello, world again from Frequency' },
        ]),
        itemizedSchemaId,
        delegatorKeys,
        msa_id
      );

      const itemized_add_result_1 = ExtrinsicHelper.applyItemActionsWithSignatureV2(
        delegatorKeys,
        undelegatedProviderKeys,
        signature,
        payload
      );
      const { target: pageUpdateEvent1, eventMap: chainEvents } =
        await itemized_add_result_1.fundAndSend(fundingSource);
      assertExtrinsicSucceededAndFeesPaid(chainEvents);
      assert.notEqual(
        pageUpdateEvent1,
        undefined,
        'should have returned a PalletStatefulStorageItemizedActionApplied event'
      );
    });

    it('delegator (owner) should be able to call applyItemizedActionWithSignatureV2 and apply actions', async function () {
      const { payload, signature } = await generateItemizedActionsSignedPayloadV2(
        generateItemizedActions([
          { action: 'Add', value: 'Hello, world from Frequency' },
          { action: 'Add', value: 'Hello, world again from Frequency' },
        ]),
        itemizedSchemaId,
        delegatorKeys,
        msa_id
      );

      const itemized_add_result_1 = ExtrinsicHelper.applyItemActionsWithSignatureV2(
        delegatorKeys,
        delegatorKeys,
        signature,
        payload
      );
      const { target: pageUpdateEvent1, eventMap: chainEvents } =
        await itemized_add_result_1.fundAndSend(fundingSource);
      assertExtrinsicSucceededAndFeesPaid(chainEvents);
      assert.notEqual(
        pageUpdateEvent1,
        undefined,
        'should have returned a PalletStatefulStorageItemizedActionApplied event'
      );
    });

    it('provider should not be able to call applyItemizedAction', async function () {
      const add_actions = generateItemizedActions([
        { action: 'Add', value: 'Hello, world from Frequency' },
        { action: 'Add', value: 'Hello, world again from Frequency' },
      ]);

      const target_hash = await getCurrentItemizedHash(msa_id, itemizedSchemaId);

      const itemized_add_result_1 = ExtrinsicHelper.applyItemActions(
        undelegatedProviderKeys,
        itemizedSchemaId,
        msa_id,
        add_actions,
        target_hash
      );
      await assert.rejects(itemized_add_result_1.fundAndSend(fundingSource), { name: 'UnauthorizedDelegate' });

      const itemized_add_result_2 = ExtrinsicHelper.applyItemActions(
        delegatedProviderKeys,
        itemizedSchemaId,
        msa_id,
        add_actions,
        target_hash
      );
      await assert.rejects(itemized_add_result_2.fundAndSend(fundingSource), { name: 'UnsupportedOperationForSchema' });
    });

    it('owner should be able to call applyItemizedAction', async function () {
      const add_actions = generateItemizedActions([
        { action: 'Add', value: 'Hello, world from Frequency' },
        { action: 'Add', value: 'Hello, world again from Frequency' },
      ]);

      const target_hash = await getCurrentItemizedHash(msa_id, itemizedSchemaId);

      const itemized_add_result_1 = ExtrinsicHelper.applyItemActions(
        delegatorKeys,
        itemizedSchemaId,
        msa_id,
        add_actions,
        target_hash
      );
      const { target: pageUpdateEvent1, eventMap: chainEvents } =
        await itemized_add_result_1.fundAndSend(fundingSource);
      assertExtrinsicSucceededAndFeesPaid(chainEvents);
      assert.notEqual(
        pageUpdateEvent1,
        undefined,
        'should have returned a PalletStatefulStorageItemizedActionApplied event'
      );
    });
  });

  describe('Paginated With Signature Storage Tests', function () {
    // passes with PaginatedPageDeleted event
    it('provider can call upsertPageWithSignature and deletePageWithSignature', async function () {
      const page_id = new u16(ExtrinsicHelper.api.registry, 1);

      // Add and update actions
      let target_hash = await getCurrentPaginatedHash(msa_id, paginatedSchemaId, page_id.toNumber());
      const upsertPayload = await generatePaginatedUpsertSignaturePayload({
        msaId: msa_id,
        targetHash: target_hash,
        schemaId: paginatedSchemaId,
        pageId: page_id,
        payload: new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency'),
      });
      const upsertPayloadData = ExtrinsicHelper.api.registry.createType(
        'PalletStatefulStoragePaginatedUpsertSignaturePayload',
        upsertPayload
      );
      const upsert_result = ExtrinsicHelper.upsertPageWithSignature(
        delegatorKeys,
        undelegatedProviderKeys,
        signPayloadSr25519(delegatorKeys, upsertPayloadData),
        upsertPayload
      );
      const { target: pageUpdateEvent, eventMap: chainEvents1 } = await upsert_result.fundAndSend(fundingSource);
      assertExtrinsicSucceededAndFeesPaid(chainEvents1)
      assert.notEqual(
        pageUpdateEvent,
        undefined,
        'should have returned a PalletStatefulStoragePaginatedPageUpdate event'
      );

      // Remove the page
      target_hash = await getCurrentPaginatedHash(msa_id, paginatedSchemaId, page_id.toNumber());
      const deletePayload = await generatePaginatedDeleteSignaturePayload({
        msaId: msa_id,
        targetHash: target_hash,
        schemaId: paginatedSchemaId,
        pageId: page_id,
      });
      const deletePayloadData = ExtrinsicHelper.api.registry.createType(
        'PalletStatefulStoragePaginatedDeleteSignaturePayload',
        deletePayload
      );
      const remove_result = ExtrinsicHelper.deletePageWithSignature(
        delegatorKeys,
        undelegatedProviderKeys,
        signPayloadSr25519(delegatorKeys, deletePayloadData),
        deletePayload
      );
      const { target: pageRemove, eventMap: chainEvents2 } = await remove_result.fundAndSend(fundingSource);
      assertExtrinsicSucceededAndFeesPaid(chainEvents2);
      assert.notEqual(pageRemove, undefined, 'should have returned a event');

      // no pages should exist
      const result = await ExtrinsicHelper.getPaginatedStorage(msa_id, paginatedSchemaId);
      assert.notEqual(result, undefined, 'should have returned a valid response');
      const thePage = result.toArray().find((page) => page.page_id === page_id);
      assert.equal(thePage, undefined, 'inserted page should not exist');
    });

    // fails to find the PaginatedPageDeleted event
    it('delegator (owner) can upsertPageWithSignature and deletePageWithSignature', async function () {
      const page_id = new u16(ExtrinsicHelper.api.registry, 1);

      // Add and update actions
      let target_hash = await getCurrentPaginatedHash(msa_id, paginatedSchemaId, page_id.toNumber());
      const upsertPayload = await generatePaginatedUpsertSignaturePayload({
        msaId: msa_id,
        targetHash: target_hash,
        schemaId: paginatedSchemaId,
        pageId: page_id,
        payload: new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency'),
      });
      const upsertPayloadData = ExtrinsicHelper.api.registry.createType(
        'PalletStatefulStoragePaginatedUpsertSignaturePayload',
        upsertPayload
      );
      const upsert_result = ExtrinsicHelper.upsertPageWithSignature(
        delegatorKeys,
        delegatorKeys,
        signPayloadSr25519(delegatorKeys, upsertPayloadData),
        upsertPayload
      );


      const { target: pageUpdateEvent, eventMap: chainEvents1 } = await upsert_result.fundAndSend(fundingSource);
      assertExtrinsicSucceededAndFeesPaid(chainEvents1);
      assert.notEqual(
        pageUpdateEvent,
        undefined,
        'should have returned a PalletStatefulStoragePaginatedPageUpdate event'
      );

      // Remove the page
      target_hash = await getCurrentPaginatedHash(msa_id, paginatedSchemaId, page_id.toNumber());
      const deletePayload = await generatePaginatedDeleteSignaturePayload({
        msaId: msa_id,
        targetHash: target_hash,
        schemaId: paginatedSchemaId,
        pageId: page_id,
      });
      const deletePayloadData = ExtrinsicHelper.api.registry.createType(
        'PalletStatefulStoragePaginatedDeleteSignaturePayload',
        deletePayload
      );
      const remove_result = ExtrinsicHelper.deletePageWithSignature(
        delegatorKeys,
        delegatorKeys,
        signPayloadSr25519(delegatorKeys, deletePayloadData),
        deletePayload
      );
      const { target: pageRemove, eventMap: chainEvents2 } = await remove_result.fundAndSend(fundingSource);
      assertExtrinsicSucceededAndFeesPaid(chainEvents2);
      assert.notEqual(pageRemove, undefined, 'should have returned a PaginatedPageDeleted');

      // no pages should exist
      const result = await ExtrinsicHelper.getPaginatedStorage(msa_id, paginatedSchemaId);
      assert.notEqual(result, undefined, 'should have returned a valid response');
      const thePage = result.toArray().find((page) => page.page_id === page_id);
      assert.equal(thePage, undefined, 'inserted page should not exist');
    });

    it('provider can upsertPageWithSignatureV2 a page and deletePageWithSignatureV2', async function () {
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
        delegatorKeys,
        undelegatedProviderKeys,
        signPayloadSr25519(delegatorKeys, upsertPayloadData),
        upsertPayload
      );
      const { target: pageUpdateEvent, eventMap: chainEvents1 } = await upsert_result.fundAndSend(fundingSource);
      assertExtrinsicSucceededAndFeesPaid(chainEvents1);

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
        delegatorKeys,
        undelegatedProviderKeys,
        signPayloadSr25519(delegatorKeys, deletePayloadData),
        deletePayload
      );
      const { target: pageRemove, eventMap: chainEvents2 } = await remove_result.fundAndSend(fundingSource);
      assert.notEqual(pageRemove, undefined, 'should have returned a event');
      assertExtrinsicSucceededAndFeesPaid(chainEvents2);
      // no pages should exist
      const result = await ExtrinsicHelper.getPaginatedStorage(msa_id, paginatedSchemaId);
      assert.notEqual(result, undefined, 'should have returned a valid response');
      const thePage = result.toArray().find((page) => page.page_id === page_id);
      assert.equal(thePage, undefined, 'inserted page should not exist');
    });

    it('delegator (owner) can upsertPageWithSignatureV2 a page and deletePageWithSignatureV2', async function () {
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
        delegatorKeys,
        delegatorKeys,
        signPayloadSr25519(delegatorKeys, upsertPayloadData),
        upsertPayload
      );
      const { target: pageUpdateEvent, eventMap: chainEvents1 } = await upsert_result.fundAndSend(fundingSource);
      assert.notEqual(
        chainEvents1['system.ExtrinsicSuccess'],
        undefined,
        'should have returned an ExtrinsicSuccess event'
      );
      assertExtrinsicSucceededAndFeesPaid(chainEvents1)

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
        delegatorKeys,
        delegatorKeys,
        signPayloadSr25519(delegatorKeys, deletePayloadData),
        deletePayload
      );
      const { target: pageRemove, eventMap: chainEvents2 } = await remove_result.fundAndSend(fundingSource);
      assertExtrinsicSucceededAndFeesPaid(chainEvents2);
      assert.notEqual(pageRemove, undefined, 'should have returned a event');

      // no pages should exist
      const result = await ExtrinsicHelper.getPaginatedStorage(msa_id, paginatedSchemaId);
      assert.notEqual(result, undefined, 'should have returned a valid response');
      const thePage = result.toArray().find((page) => page.page_id === page_id);
      assert.equal(thePage, undefined, 'inserted page should not exist');
    });

    it('provider cannot upsertPage', async function () {
      const page_id = new u16(ExtrinsicHelper.api.registry, 1);

      const target_hash = await getCurrentPaginatedHash(msa_id, paginatedSchemaId, page_id.toNumber());

      const upsert = ExtrinsicHelper.upsertPage(
        undelegatedProviderKeys,
        paginatedSchemaId,
        msa_id,
        page_id,
        new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency'),
        target_hash
      );
      await assert.rejects(upsert.fundAndSend(fundingSource), { name: 'UnauthorizedDelegate' });

      const upsert_2 = ExtrinsicHelper.upsertPage(
        delegatedProviderKeys,
        paginatedSchemaId,
        msa_id,
        page_id,
        new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency'),
        target_hash
      );
      await assert.rejects(upsert_2.fundAndSend(fundingSource), { name: 'UnsupportedOperationForSchema' });
    });

    it('owner can upsertPage', async function () {
      const page_id = new u16(ExtrinsicHelper.api.registry, 1);

      const target_hash = await getCurrentPaginatedHash(msa_id, paginatedSchemaId, page_id.toNumber());

      const upsert_result = ExtrinsicHelper.upsertPage(
        delegatorKeys,
        paginatedSchemaId,
        msa_id,
        page_id,
        new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency'),
        target_hash
      );
      const { target: pageUpdateEvent, eventMap: chainEvents1 } = await upsert_result.fundAndSend(fundingSource);
      assertExtrinsicSucceededAndFeesPaid(chainEvents1);
      assert.notEqual(
        pageUpdateEvent,
        undefined,
        'should have returned a PalletStatefulStoragePaginatedPageUpdate event'
      );
    });

    it('Provider cannot deletePage directly', async function () {
      const page_id = new u16(ExtrinsicHelper.api.registry, 1);

      const target_hash = await getCurrentPaginatedHash(msa_id, paginatedSchemaId, page_id.toNumber());

      const remove_op = ExtrinsicHelper.removePage(
        undelegatedProviderKeys,
        paginatedSchemaId,
        msa_id,
        page_id,
        target_hash
      );
      await assert.rejects(remove_op.fundAndSend(fundingSource), { name: 'UnauthorizedDelegate' });

      const remove_op_2 = ExtrinsicHelper.removePage(
        delegatedProviderKeys,
        paginatedSchemaId,
        msa_id,
        page_id,
        target_hash
      );
      await assert.rejects(remove_op_2.fundAndSend(fundingSource), { name: 'UnsupportedOperationForSchema' });
    });

    // Fails to emit event but appears to pass
    it('delegator (owner) can deletePage', async function () {
      const page_id = new u16(ExtrinsicHelper.api.registry, 1);

      const target_hash = await getCurrentPaginatedHash(msa_id, paginatedSchemaId, page_id.toNumber());

      const remove_result = ExtrinsicHelper.removePage(
          delegatorKeys, paginatedSchemaId, msa_id, page_id, target_hash);
      const { target: pageUpdateEvent, eventMap: chainEvents1 } = await remove_result.fundAndSend(fundingSource);
      assertExtrinsicSucceededAndFeesPaid(chainEvents1);
      assert.notEqual(
        pageUpdateEvent,
        undefined,
        'should have returned a PaginatedPageDeleted event'
      );
    });
  });

});
