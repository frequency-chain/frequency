// E2E tests for pallets/stateful-pallet-storage/handleItemizedWithSignaturePaginated.ts
import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  DOLLARS,
  createDelegatorAndDelegation,
  createProviderKeysAndId,
  generatePaginatedDeleteSignaturePayloadV2,
  generatePaginatedUpsertSignaturePayloadV2,
  getCurrentPaginatedHash,
  signPayloadSr25519,
  assertExtrinsicSucceededAndFeesPaid,
  createMsa,
  createAndFundKeypair,
  getOrCreateIntentAndSchema,
  assertExtrinsicSuccess,
} from '../scaffolding/helpers';
import type { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { AVRO_CHAT_MESSAGE } from '../stateful-pallet-storage/fixtures/itemizedSchemaType';
import { IntentId, MessageSourceId, SchemaId } from '@frequency-chain/api-augment/interfaces';
import { Bytes, u16 } from '@polkadot/types';
import { getFundingSource } from '../scaffolding/funding';

let fundingSource: KeyringPair;

describe('📗 Stateful Pallet Storage Signature Required Paginated', function () {
  let paginatedIntentId: IntentId;
  let paginatedSchemaId: SchemaId;
  let msa_id: MessageSourceId;
  let undelegatedProviderId: MessageSourceId;
  let undelegatedProviderKeys: KeyringPair;
  let delegatedProviderId: MessageSourceId;
  let delegatedProviderKeys: KeyringPair;
  let delegatorKeys: KeyringPair;

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);
    [
      // Create a provider. This provider will NOT be granted delegations;
      // methods requiring a payload signature do not require a delegation
      [undelegatedProviderKeys, undelegatedProviderId],
      // Create a provider for the MSA, the provider will be used to grant delegation
      [delegatedProviderKeys, delegatedProviderId],
      // Fund the Delegator Keys
      delegatorKeys,
      // Create a schema for Paginated PayloadLocation
      { intentId: paginatedIntentId, schemaId: paginatedSchemaId },
    ] = await Promise.all([
      createProviderKeysAndId(fundingSource, 2n * DOLLARS),
      createProviderKeysAndId(fundingSource, 2n * DOLLARS),
      createAndFundKeypair(fundingSource, 2n * DOLLARS),
      getOrCreateIntentAndSchema(
        fundingSource,
        'test.PaginatedSignatureRequired',
        { payloadLocation: 'Paginated', settings: ['SignatureRequired'] },
        { model: AVRO_CHAT_MESSAGE, modelType: 'AvroBinary' }
      ),
    ]);
    assert.notEqual(undelegatedProviderId, undefined, 'setup should populate undelegatedProviderId');
    assert.notEqual(undelegatedProviderKeys, undefined, 'setup should populate undelegatedProviderKeys');
    assert.notEqual(delegatedProviderId, undefined, 'setup should populate delegatedProviderId');
    assert.notEqual(delegatedProviderKeys, undefined, 'setup should populate delegatedProviderKeys');

    // Create a MSA for the delegator
    [delegatorKeys, msa_id] = await createDelegatorAndDelegation(
      fundingSource,
      [paginatedIntentId],
      delegatedProviderId,
      delegatedProviderKeys,
      'sr25519',
      delegatorKeys
    );
    // ExtrinsicHelper.transferFunds(fundingSource, delegatorKeys, 2n * DOLLARS);
    assert.notEqual(delegatorKeys, undefined, 'setup should populate delegator_key');
    assert.notEqual(msa_id, undefined, 'setup should populate msa_id');
  });

  it('provider should be able to call upsertPageWithSignatureV2 a page and deletePageWithSignatureV2 it successfully', async function () {
    const page_id = new u16(ExtrinsicHelper.api.registry, 1);

    // Add and update actions
    let target_hash = await getCurrentPaginatedHash(msa_id, paginatedIntentId, page_id.toNumber());
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
    await assertExtrinsicSucceededAndFeesPaid(chainEvents1);

    assert.notEqual(
      pageUpdateEvent,
      undefined,
      'should have returned a PalletStatefulStoragePaginatedPageUpdate event'
    );

    // Remove the page
    target_hash = await getCurrentPaginatedHash(msa_id, paginatedIntentId, page_id.toNumber());
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
    await assertExtrinsicSucceededAndFeesPaid(chainEvents2);
    // no pages should exist
    const result = await ExtrinsicHelper.getPaginatedStorage(msa_id, paginatedIntentId);
    assert.notEqual(result, undefined, 'should have returned a valid response');
    const thePage = result.toArray().find((page) => page.pageId === page_id);
    assert.equal(thePage, undefined, 'inserted page should not exist');
  });

  it('delegator (owner) can upsertPageWithSignatureV2 a page and deletePageWithSignatureV2', async function () {
    const page_id = new u16(ExtrinsicHelper.api.registry, 1);

    // Add and update actions
    let target_hash = await getCurrentPaginatedHash(msa_id, paginatedIntentId, page_id.toNumber());
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
    await assertExtrinsicSucceededAndFeesPaid(chainEvents1);

    // Remove the page
    target_hash = await getCurrentPaginatedHash(msa_id, paginatedIntentId, page_id.toNumber());
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
    await assertExtrinsicSucceededAndFeesPaid(chainEvents2);
    assert.notEqual(pageRemove, undefined, 'should have returned a event');

    // no pages should exist
    const result = await ExtrinsicHelper.getPaginatedStorage(msa_id, paginatedIntentId);
    assert.notEqual(result, undefined, 'should have returned a valid response');
    const thePage = result.toArray().find((page) => page.pageId.eq(page_id));
    assert.equal(thePage, undefined, 'inserted page should not exist');
  });

  it('provider cannot upsertPage', async function () {
    const page_id = new u16(ExtrinsicHelper.api.registry, 1);

    const target_hash = await getCurrentPaginatedHash(msa_id, paginatedIntentId, page_id.toNumber());

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

    const target_hash = await getCurrentPaginatedHash(msa_id, paginatedIntentId, page_id.toNumber());

    const upsert_result = ExtrinsicHelper.upsertPage(
      delegatorKeys,
      paginatedSchemaId,
      msa_id,
      page_id,
      new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency'),
      target_hash
    );
    const { target: pageUpdateEvent, eventMap: chainEvents1 } = await upsert_result.fundAndSend(fundingSource);
    await assertExtrinsicSucceededAndFeesPaid(chainEvents1);
    assert.notEqual(
      pageUpdateEvent,
      undefined,
      'should have returned a PalletStatefulStoragePaginatedPageUpdate event'
    );
  });

  it('Provider cannot deletePage directly', async function () {
    const page_id = new u16(ExtrinsicHelper.api.registry, 1);

    const target_hash = await getCurrentPaginatedHash(msa_id, paginatedIntentId, page_id.toNumber());

    const remove_op = ExtrinsicHelper.removePage(
      undelegatedProviderKeys,
      paginatedIntentId,
      msa_id,
      page_id,
      target_hash
    );
    await assert.rejects(remove_op.fundAndSend(fundingSource), { name: 'UnauthorizedDelegate' });

    const remove_op_2 = ExtrinsicHelper.removePage(
      delegatedProviderKeys,
      paginatedIntentId,
      msa_id,
      page_id,
      target_hash
    );
    await assert.rejects(remove_op_2.fundAndSend(fundingSource), { name: 'UnsupportedOperationForIntent' });
  });

  // Fails to emit event but appears to pass
  it('delegator (owner) can deletePage', async function () {
    const page_id = new u16(ExtrinsicHelper.api.registry, 1);

    const target_hash = await getCurrentPaginatedHash(msa_id, paginatedIntentId, page_id.toNumber());
    const remove_result = ExtrinsicHelper.removePage(delegatorKeys, paginatedIntentId, msa_id, page_id, target_hash);
    const { target: pageUpdateEvent, eventMap: chainEvents1 } = await remove_result.fundAndSend(fundingSource);
    await assertExtrinsicSucceededAndFeesPaid(chainEvents1);
    assert.notEqual(pageUpdateEvent, undefined, 'should have returned a PaginatedPageDeleted event');
  });
});
