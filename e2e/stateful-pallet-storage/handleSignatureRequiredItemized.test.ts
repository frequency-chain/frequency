// E2E tests for pallets/stateful-pallet-storage/handleItemizedWithSignatureItemized.ts
import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  DOLLARS,
  createDelegatorAndDelegation,
  createProviderKeysAndId,
  generateItemizedActions,
  generateItemizedActionsSignedPayloadV2,
  getCurrentItemizedHash,
  assertExtrinsicSucceededAndFeesPaid,
  createAndFundKeypair,
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { AVRO_CHAT_MESSAGE } from '../stateful-pallet-storage/fixtures/itemizedSchemaType';
import { MessageSourceId, SchemaId } from '@frequency-chain/api-augment/interfaces';
import { getFundingSource } from '../scaffolding/funding';

const fundingSource = getFundingSource(import.meta.url);

describe('📗 Stateful Pallet Storage Signature Required Itemized', function () {
  let itemizedSchemaId: SchemaId;
  let msa_id: MessageSourceId;
  let undelegatedProviderId: MessageSourceId;
  let undelegatedProviderKeys: KeyringPair;
  let delegatedProviderId: MessageSourceId;
  let delegatedProviderKeys: KeyringPair;
  let delegatorKeys: KeyringPair;

  let itemizedActionsSignedPayload;

  before(async function () {
    [
      // Create a provider. This provider will NOT be granted delegations;
      // methods requiring a payload signature do not require a delegation
      [undelegatedProviderKeys, undelegatedProviderId],
      // Create a provider for the MSA, the provider will be used to grant delegation
      [delegatedProviderKeys, delegatedProviderId],
      // Fund the Delegator Keys
      delegatorKeys,
      // Create a schema for Itemized PayloadLocation
      itemizedSchemaId,
    ] = await Promise.all([
      createProviderKeysAndId(fundingSource, 2n * DOLLARS),
      createProviderKeysAndId(fundingSource, 2n * DOLLARS),
      createAndFundKeypair(fundingSource, 2n * DOLLARS),
      ExtrinsicHelper.getOrCreateSchemaV3(
        fundingSource,
        AVRO_CHAT_MESSAGE,
        'AvroBinary',
        'Itemized',
        ['AppendOnly', 'SignatureRequired'],
        'test.ItemizedSignatureRequired'
      ),
    ]);
    assert.notEqual(undelegatedProviderId, undefined, 'setup should populate undelegatedProviderId');
    assert.notEqual(undelegatedProviderKeys, undefined, 'setup should populate undelegatedProviderKeys');
    assert.notEqual(delegatedProviderId, undefined, 'setup should populate delegatedProviderId');
    assert.notEqual(delegatedProviderKeys, undefined, 'setup should populate delegatedProviderKeys');

    // Create a MSA for the delegator
    [delegatorKeys, msa_id] = await createDelegatorAndDelegation(
      fundingSource,
      [itemizedSchemaId],
      delegatedProviderId,
      delegatedProviderKeys,
      'sr25519',
      delegatorKeys
    );
    assert.notEqual(delegatorKeys, undefined, 'setup should populate delegator_key');
    assert.notEqual(msa_id, undefined, 'setup should populate msa_id');

    itemizedActionsSignedPayload = await generateItemizedActionsSignedPayloadV2(
      generateItemizedActions([
        { action: 'Add', value: 'Hello, world from Frequency' },
        { action: 'Add', value: 'Hello, world again from Frequency' },
      ]),
      itemizedSchemaId,
      delegatorKeys,
      msa_id
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
    const { target: pageUpdateEvent1, eventMap: chainEvents } = await itemized_add_result_1.fundAndSend(fundingSource);
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
    const { target: pageUpdateEvent1, eventMap: chainEvents } = await itemized_add_result_1.fundAndSend(fundingSource);
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
    const { target: pageUpdateEvent1, eventMap: chainEvents } = await itemized_add_result_1.fundAndSend(fundingSource);
    assertExtrinsicSucceededAndFeesPaid(chainEvents);
    assert.notEqual(
      pageUpdateEvent1,
      undefined,
      'should have returned a PalletStatefulStorageItemizedActionApplied event'
    );
  });
});
