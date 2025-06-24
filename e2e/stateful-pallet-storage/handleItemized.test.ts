// E2E tests for pallets/stateful-pallet-storage/handleItemized.ts
import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  DOLLARS,
  createDelegatorAndDelegation,
  createMsa,
  createProviderKeysAndId,
  getCurrentItemizedHash,
  getOrCreateAvroChatMessageItemizedSchema,
  assertExtrinsicSucceededAndFeesPaid,
  createAndFundKeypair,
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { AVRO_CHAT_MESSAGE } from '../stateful-pallet-storage/fixtures/itemizedSchemaType';
import { MessageSourceId, SchemaId } from '@frequency-chain/api-augment/interfaces';
import { Bytes, u16, u64 } from '@polkadot/types';
import { getFundingSource } from '../scaffolding/funding';

let fundingSource: KeyringPair;

describe('📗 Stateful Pallet Storage Itemized', function () {
  let schemaId_deletable: SchemaId;
  let schemaId_unsupported: SchemaId;
  let delegatorKeys: KeyringPair;
  let msa_id: MessageSourceId;
  let providerId: MessageSourceId;
  let providerKeys: KeyringPair;
  let badMsaId: u64;

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);
    [
      // Create a provider for the MSA, the provider will be used to grant delegation
      [providerKeys, providerId],
      // Delegator Keys
      delegatorKeys,
      schemaId_deletable,
      schemaId_unsupported,
    ] = await Promise.all([
      createProviderKeysAndId(fundingSource, 2n * DOLLARS),
      createAndFundKeypair(fundingSource, 2n * DOLLARS),
      getOrCreateAvroChatMessageItemizedSchema(fundingSource),
      ExtrinsicHelper.getOrCreateSchemaV3(
        fundingSource,
        AVRO_CHAT_MESSAGE,
        'AvroBinary',
        'OnChain',
        [],
        'test.handleItemizedUnsupported'
      ),
    ]);
    assert.notEqual(providerId, undefined, 'setup should populate providerId');
    assert.notEqual(providerKeys, undefined, 'setup should populate providerKeys');

    [
      // Create a MSA for the delegator and delegate to the provider
      [delegatorKeys, msa_id],
      // Create an MSA that is not a provider to be used for testing failure cases
      [badMsaId],
    ] = await Promise.all([
      createDelegatorAndDelegation(
        fundingSource,
        schemaId_deletable,
        providerId,
        providerKeys,
        'sr25519',
        delegatorKeys
      ),
      createMsa(fundingSource),
    ]);
    assert.notEqual(msa_id, undefined, 'setup should populate msa_id');
    assert.notEqual(badMsaId, undefined, 'setup should populate badMsaId');
  });

  describe('Itemized Storage Tests 😊/😥', function () {
    it('✅ should be able to call applyItemizedAction and apply actions', async function () {
      // Add and update actions
      const payload_1 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency');

      const add_action = {
        Add: payload_1,
      };

      const payload_2 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World Again From Frequency');

      const update_action = {
        Add: payload_2,
      };

      const target_hash = await getCurrentItemizedHash(msa_id, schemaId_deletable);

      const add_actions = [add_action, update_action];
      const itemized_add_result_1 = ExtrinsicHelper.applyItemActions(
        providerKeys,
        schemaId_deletable,
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

    it('🛑 should fail call to applyItemizedAction with invalid schemaId', async function () {
      const payload_1 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency');
      const add_action = {
        Add: payload_1,
      };
      const add_actions = [add_action];
      const fake_schema_id = new u16(ExtrinsicHelper.api.registry, 65_534);
      const itemized_add_result_1 = ExtrinsicHelper.applyItemActions(
        delegatorKeys,
        fake_schema_id,
        msa_id,
        add_actions,
        0
      );
      await assert.rejects(itemized_add_result_1.fundAndSend(fundingSource), {
        name: 'InvalidSchemaId',
        section: 'statefulStorage',
      });
    });

    it('🛑 should fail call to applyItemizedAction with invalid schema location', async function () {
      const payload_1 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency');
      const add_action = {
        Add: payload_1,
      };
      const add_actions = [add_action];
      const itemized_add_result_1 = ExtrinsicHelper.applyItemActions(
        delegatorKeys,
        schemaId_unsupported,
        msa_id,
        add_actions,
        0
      );
      await assert.rejects(itemized_add_result_1.fundAndSend(fundingSource), {
        name: 'SchemaPayloadLocationMismatch',
        section: 'statefulStorage',
      });
    });

    it('🛑 should fail call to applyItemizedAction with for un-delegated attempts', async function () {
      const payload_1 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency');
      const add_action = {
        Add: payload_1,
      };
      const add_actions = [add_action];

      const itemized_add_result_1 = ExtrinsicHelper.applyItemActions(
        providerKeys,
        schemaId_deletable,
        badMsaId,
        add_actions,
        0
      );
      await assert.rejects(itemized_add_result_1.fundAndSend(fundingSource), {
        name: 'UnauthorizedDelegate',
        section: 'statefulStorage',
      });
    });

    it('🛑 should fail call to applyItemizedAction for target hash mismatch', async function () {
      // Add and update actions
      const payload_1 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency');

      const add_action = {
        Add: payload_1,
      };

      const payload_2 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World Again From Frequency');

      const update_action = {
        Add: payload_2,
      };

      const add_actions = [add_action, update_action];
      const itemized_add_result_1 = ExtrinsicHelper.applyItemActions(
        providerKeys,
        schemaId_deletable,
        msa_id,
        add_actions,
        0
      );
      await assert.rejects(itemized_add_result_1.fundAndSend(fundingSource), { name: 'StalePageState' });
    });
  });

  describe('Itemized Storage Remove Action Tests', function () {
    it('✅ should be able to call applyItemizedAction and apply remove actions', async function () {
      let target_hash = await getCurrentItemizedHash(msa_id, schemaId_deletable);

      // Delete action
      const idx_1: u16 = new u16(ExtrinsicHelper.api.registry, 1);
      const remove_action_1 = {
        Delete: idx_1,
      };

      target_hash = await getCurrentItemizedHash(msa_id, schemaId_deletable);

      const remove_actions = [remove_action_1];
      const itemized_remove_result_1 = ExtrinsicHelper.applyItemActions(
        providerKeys,
        schemaId_deletable,
        msa_id,
        remove_actions,
        target_hash
      );
      const { target: pageUpdateEvent2, eventMap: chainEvents2 } =
        await itemized_remove_result_1.fundAndSend(fundingSource);
      assert.notEqual(
        chainEvents2['system.ExtrinsicSuccess'],
        undefined,
        'should have returned an ExtrinsicSuccess event'
      );
      assert.notEqual(chainEvents2['balances.Withdraw'], undefined, 'should have returned a balances.Withdraw event');
      assert.notEqual(pageUpdateEvent2, undefined, 'should have returned a event');
    });

    it('🛑 should fail call to remove action with invalid schemaId', async function () {
      const idx_1: u16 = new u16(ExtrinsicHelper.api.registry, 1);
      const remove_action_1 = {
        Delete: idx_1,
      };
      const remove_actions = [remove_action_1];
      const fake_schema_id = new u16(ExtrinsicHelper.api.registry, 65_534);
      const itemized_remove_result_1 = ExtrinsicHelper.applyItemActions(
        delegatorKeys,
        fake_schema_id,
        msa_id,
        remove_actions,
        0
      );
      await assert.rejects(itemized_remove_result_1.fundAndSend(fundingSource), {
        name: 'InvalidSchemaId',
        section: 'statefulStorage',
      });
    });

    it('🛑 should fail call to remove action with invalid schema location', async function () {
      const idx_1: u16 = new u16(ExtrinsicHelper.api.registry, 1);
      const remove_action_1 = {
        Delete: idx_1,
      };
      const remove_actions = [remove_action_1];
      const itemized_remove_result_1 = ExtrinsicHelper.applyItemActions(
        delegatorKeys,
        schemaId_unsupported,
        msa_id,
        remove_actions,
        0
      );
      await assert.rejects(itemized_remove_result_1.fundAndSend(fundingSource), {
        name: 'SchemaPayloadLocationMismatch',
        section: 'statefulStorage',
      });
    });

    it('🛑 should fail call to remove action with invalid msa_id', async function () {
      const idx_1: u16 = new u16(ExtrinsicHelper.api.registry, 1);
      const remove_action_1 = {
        Delete: idx_1,
      };
      const remove_actions = [remove_action_1];

      const itemized_remove_result_1 = ExtrinsicHelper.applyItemActions(
        providerKeys,
        schemaId_deletable,
        badMsaId,
        remove_actions,
        0
      );
      await assert.rejects(itemized_remove_result_1.fundAndSend(fundingSource), {
        name: 'UnauthorizedDelegate',
        section: 'statefulStorage',
      });
    });

    it('🛑 should fail call to remove action with stale state hash', async function () {
      const idx_1: u16 = new u16(ExtrinsicHelper.api.registry, 1);
      const remove_action = {
        Delete: idx_1,
      };
      const remove_actions = [remove_action];
      const op = ExtrinsicHelper.applyItemActions(providerKeys, schemaId_deletable, msa_id, remove_actions, 0);
      await assert.rejects(op.fundAndSend(fundingSource), { name: 'StalePageState' });
    });
  });

  describe('Itemized Storage RPC Tests', function () {
    it('✅ should be able to call getItemizedStorage and get data for itemized schema', async function () {
      const result = await ExtrinsicHelper.getItemizedStorage(msa_id, schemaId_deletable);
      assert.notEqual(result.hash, undefined, 'should have returned a hash');
      assert.notEqual(result.size, undefined, 'should have returned a itemized responses');
    });
  });
});
