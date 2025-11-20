import '@frequency-chain/api-augment';

import assert from 'assert';

import { AVRO_GRAPH_CHANGE, PARQUET_BROADCAST } from './fixtures';
import type { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  createKeys,
  createAndFundKeypair,
  assertExtrinsicSuccess,
  generateSchemaPartialName,
} from '../scaffolding/helpers';
import { getFundingSource } from '../scaffolding/funding';
import { Type } from 'cborg';
import undefined = Type.undefined;

let fundingSource: KeyringPair;

describe('#schemas pallet tests', function () {
  let keys: KeyringPair;
  let accountWithNoFunds: KeyringPair;
  const createdIntents: { intentId: number; intentName: string; schemaIds: number[] }[] = [];
  const createdIntentGroups: { intentGroupId: number; intentIds: number[]; intentGroupName: string }[] = [];

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);
    keys = await createAndFundKeypair(fundingSource, 50_000_000n);
    accountWithNoFunds = createKeys();

    const intentName = 'e-e.' + generateSchemaPartialName(20);
    const f = ExtrinsicHelper.createIntent(keys, 'OnChain', [], intentName);
    const { target: createIntentEvent } = await f.fundAndSend(fundingSource);
    if (createIntentEvent && ExtrinsicHelper.apiPromise.events.schemas.IntentCreated.is(createIntentEvent)) {
      createdIntents.push({ intentId: createIntentEvent.data.intentId.toNumber(), intentName, schemaIds: [] });
    }

    assert.equal(createdIntents.length, 1);
  });

  describe('intents', function () {
    const cases = [
      { id: 'if no name is supplied', name: null, error: 'InvalidSchemaNameStructure' },
      { id: 'with invalid character in name', name: 'test@.invalid', error: 'InvalidSchemaNameCharacters' },
      { id: 'with invalid name structure', name: 'test', error: 'InvalidSchemaNameStructure' },
      { id: 'with invalid name encoding', name: 'ñòò.ò', error: 'InvalidSchemaNameEncoding' },
      { id: 'with invalid namespace length', name: 'a.b', error: 'InvalidSchemaNamespaceLength' },
    ];

    it('create intent should fail if account does not have enough tokens', async function () {
      await assert.rejects(
        ExtrinsicHelper.createIntent(accountWithNoFunds, 'OnChain', [], 'unk.random_name').signAndSend('current')
      );
    });

    cases.forEach(({ id, name, error }) => {

      it(`create intent should fail ${id}`, async function () {
        // @ts-expect-error allow null string
        const f = ExtrinsicHelper.createIntent(keys, 'OnChain', [], name);
        await assert.rejects(f.fundAndSend(fundingSource), { name: error });
      });
    });

    it('should successfully create an intent with a name', async function () {
      const intentName = 'e-e.' + generateSchemaPartialName(20);
      const f = ExtrinsicHelper.createIntent(keys, 'OnChain', [], intentName);
      const { target: createIntentEvent, eventMap } = await f.fundAndSend(fundingSource);
      if (createIntentEvent && ExtrinsicHelper.apiPromise.events.schemas.IntentCreated.is(createIntentEvent)) {
        createdIntents.push({
          intentId: createIntentEvent.data.intentId.toNumber(),
          intentName: createIntentEvent.data.intentName.toString(),
          schemaIds: [],
        });
      }

      assertExtrinsicSuccess(eventMap);
      assert.notEqual(createIntentEvent, undefined);
    });

    it('create intent should fail if name is a duplicate', async function () {
      const intentName = 'e-e.' + generateSchemaPartialName(20);
      const f = ExtrinsicHelper.createIntent(keys, 'OnChain', [], intentName);
      const { target: createIntentEvent, eventMap } = await f.fundAndSend(fundingSource);
      if (createIntentEvent && ExtrinsicHelper.apiPromise.events.schemas.IntentCreated.is(createIntentEvent)) {
        createdIntents.push({
          intentId: createIntentEvent.data.intentId.toNumber(),
          intentName: createIntentEvent.data.intentName.toString(),
          schemaIds: [],
        });
      }

      assertExtrinsicSuccess(eventMap);
      assert.notEqual(createIntentEvent, undefined);

      await assert.rejects(f.fundAndSend(fundingSource), {
        name: 'NameAlreadyExists',
      });
    });
  });

  describe('schemas', function () {
    const hugeSchema = {
      type: 'record',
      fields: [],
    };

    const cases: {
      id: string;
      intentId: () => number;
      model: unknown;
      modelType: 'AvroBinary' | 'Parquet';
      error: string;
    }[] = [
      {
        id: 'with a non-existent intent id',
        intentId: () => 2 ** 16 - 1,
        model: AVRO_GRAPH_CHANGE,
        modelType: 'AvroBinary',
        error: 'InvalidIntentId',
      },
      {
        id: 'if Avro schema is invalid',
        intentId: () => createdIntents[0].intentId,
        model: [1000, 3],
        modelType: 'AvroBinary',
        error: 'InvalidSchema',
      },
      {
        id: 'if Parquet schema is invalid',
        intentId: () => createdIntents[0].intentId,
        model: PARQUET_BROADCAST.map((field) => ({ ...field, unknown_field: true })),
        modelType: 'Parquet',
        error: 'InvalidSchema',
      },
      {
        id: 'if schema is less than minimum size',
        intentId: () => createdIntents[0].intentId,
        model: {},
        modelType: 'AvroBinary',
        error: 'LessThanMinSchemaModelBytes',
      },
      {
        id: 'if schema is greater than maximum size',
        intentId: () => createdIntents[0].intentId,
        model: hugeSchema,
        modelType: 'AvroBinary',
        error: 'ExceedsMaxSchemaModelBytes',
      },
    ];

    before(async function () {
      const maxBytes = (await ExtrinsicHelper.getSchemaMaxBytes()).toNumber();

      // Create a schema whose JSON representation is exactly 1 byte larger than the max allowed
      const hugeSize = JSON.stringify(hugeSchema).length;
      const sizeToFill = maxBytes - hugeSize - ',"name":""'.length + 1;
      hugeSchema['name'] = Array.from(Array(sizeToFill).keys())
        .map(() => 'a')
        .join('');
    });

    it('should fail if account does not have enough tokens v4', async function () {
      await assert.rejects(
        ExtrinsicHelper.createSchemaV4(accountWithNoFunds, 1, AVRO_GRAPH_CHANGE, 'AvroBinary').signAndSend('current'),
        {
          name: 'RpcError',
          message: /Inability to pay some fees/,
        }
      );
    });

    cases.forEach(({ id, intentId, model, modelType, error }) => {

      it(`create schema v4 should fail ${id}`, async function () {
        const f = ExtrinsicHelper.createSchemaV4(keys, intentId(), model, modelType);
        await assert.rejects(f.fundAndSend(fundingSource), { name: error });
      });
    });

    it('should successfully create a schema v4 and add it to an existing intent', async function () {
      const f = ExtrinsicHelper.createSchemaV4(keys, createdIntents[0].intentId, AVRO_GRAPH_CHANGE, 'AvroBinary');
      const { target: schemaCreatedEvent, eventMap } = await f.fundAndSend(fundingSource);

      assertExtrinsicSuccess(eventMap);
      if (schemaCreatedEvent && ExtrinsicHelper.apiPromise.events.schemas.SchemaCreated.is(schemaCreatedEvent)) {
        createdIntents[0].schemaIds = [schemaCreatedEvent.data.schemaId.toNumber()];
      }

      const response = await ExtrinsicHelper.apiPromise.call.schemasRuntimeApi.getIntentById(
        createdIntents[0].intentId,
        true
      );
      assert(response.isSome);
      assert(response.unwrap().schemaIds.isSome);
      assert(
        response
          .unwrap()
          .schemaIds.unwrap()
          .some((s) => s.toNumber() === createdIntents[0].schemaIds[0])
      );
    });
  });

  describe('context groups', function () {
    it('should fail to create an intent group with an invalid intent id', async function () {
      const intentGroupName = 'e-e.' + generateSchemaPartialName(20);
      const f = ExtrinsicHelper.createIntentGroup(keys, [createdIntents[0].intentId, 2 ** 16 - 1], intentGroupName);

      await assert.rejects(f.fundAndSend(fundingSource), {
        name: 'InvalidIntentId',
      });
    });

    it('should successfully create an intent group with a name', async function () {
      const intentGroupName = 'e-e.' + generateSchemaPartialName(20);
      const f = ExtrinsicHelper.createIntentGroup(keys, [createdIntents[0].intentId], intentGroupName);

      const { target: createIntentGroupEvent, eventMap } = await f.fundAndSend(fundingSource);
      if (
        createIntentGroupEvent &&
        ExtrinsicHelper.apiPromise.events.schemas.IntentGroupCreated.is(createIntentGroupEvent)
      ) {
        createdIntentGroups.push({
          intentGroupId: createIntentGroupEvent.data.intentGroupId.toNumber(),
          intentGroupName,
          intentIds: [createdIntents[0].intentId],
        });
      }

      assertExtrinsicSuccess(eventMap);
      assert.notEqual(createIntentGroupEvent, undefined);
    });

    it('should fail to update a non-existent intent group', async function () {
      const f = ExtrinsicHelper.updateIntentGroup(keys, 2 ** 16 - 1, [createdIntents[1].intentId]);

      await assert.rejects(f.fundAndSend(fundingSource), {
        name: 'InvalidIntentGroupId',
      });
    });

    it('should fail to update an intent group with an invalid intent id', async function () {
      const f = ExtrinsicHelper.updateIntentGroup(keys, createdIntentGroups[0].intentGroupId, [
        createdIntents[0].intentId,
        2 ** 16 - 1,
      ]);

      await assert.rejects(f.fundAndSend(fundingSource), {
        name: 'InvalidIntentId',
      });
    });

    it('should successfully update an existing intent group', async function () {
      const intentIds = [...createdIntents.map((intent) => intent.intentId)];
      const f = ExtrinsicHelper.updateIntentGroup(keys, createdIntentGroups[0].intentGroupId, intentIds);
      const { target: updateIntentGroupEvent, eventMap } = await f.fundAndSend(fundingSource);

      assertExtrinsicSuccess(eventMap);
      assert.notEqual(updateIntentGroupEvent, undefined);

      const intentGroup = await ExtrinsicHelper.apiPromise.call.schemasRuntimeApi.getIntentGroupById(
        createdIntentGroups[0].intentGroupId
      );
      assert(intentGroup.isSome);
      intentGroup.unwrap().intentIds.forEach((intentId) => assert(intentIds.includes(intentId.toNumber())));
      createdIntentGroups[0].intentIds = intentIds;
    });
  });

  describe('runtime apis', function () {
    it('get registered entities runtime call should return all registered items using the same protocol', async function () {
      const protocol = 'e-e';

      const versions = await ExtrinsicHelper.apiPromise.call.schemasRuntimeApi.getRegisteredEntitiesByName(protocol);
      assert(versions.isSome);
      const versions_response_value = versions.unwrap();
      assert(versions_response_value.length > 3);
      const intents = versions_response_value
        .filter((v) => v.entityId.isIntent)
        .map((v) => v.entityId.asIntent.toNumber());
      const intentGroups = versions_response_value
        .filter((v) => v.entityId.isIntentGroup)
        .map((v) => v.entityId.asIntentGroup.toNumber());
      createdIntents.forEach((intent) => {
        assert(intents.some((i) => i === intent.intentId));
      });
      createdIntentGroups.forEach((intentGroup) => {
        assert(intentGroups.some((i) => i === intentGroup.intentGroupId));
      });
    });

    it('get intent by id should return the correct intent', async function () {
      const response = await ExtrinsicHelper.apiPromise.call.schemasRuntimeApi.getIntentById(
        createdIntents[0].intentId,
        true
      );
      assert(response.isSome);
      assert(response.unwrap().intentId.toNumber() === createdIntents[0].intentId);
    });

    it('get intent group by id should return the correct intent group', async function () {
      const response = await ExtrinsicHelper.apiPromise.call.schemasRuntimeApi.getIntentGroupById(
        createdIntentGroups[0].intentGroupId
      );
      assert(response.isSome);
      assert(response.unwrap().intentGroupId.toNumber() === createdIntentGroups[0].intentGroupId);
      assert.deepEqual(
        new Set(response.unwrap().intentIds.map((id) => id.toNumber())),
        new Set(createdIntentGroups[0].intentIds)
      );
    });

    it('get schema by id should return the correct schema', async function () {
      const response = await ExtrinsicHelper.apiPromise.call.schemasRuntimeApi.getSchemaById(
        createdIntents[0].schemaIds[0]
      );
      assert(response.isSome);
      assert(response.unwrap().schemaId.toNumber() === createdIntents[0].schemaIds[0]);
      assert(response.unwrap().intentId.toNumber() === createdIntents[0].intentId);
    });

    // Deprecated runtime API--should get removed in a future version
    it('get schema versions by name should return the correct schema versions', async function () {
      const response = await ExtrinsicHelper.apiPromise.call.schemasRuntimeApi.getSchemaVersionsByName(
        createdIntents[0].intentName
      );
      assert(response.isSome);
      const schemaVersion = response
        .unwrap()
        .toArray()
        .filter((v) => v.schemaId.toNumber() === createdIntents[0].schemaIds[0]);
      assert(schemaVersion.length === 1);
      assert(schemaVersion[0].schemaId.toNumber() === createdIntents[0].schemaIds[0]);
      assert(schemaVersion[0].schemaVersion.toNumber() === 1);
    });
  });
});
