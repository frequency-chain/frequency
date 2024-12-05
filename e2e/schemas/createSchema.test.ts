import '@frequency-chain/api-augment';

import assert from 'assert';

import { AVRO_GRAPH_CHANGE } from './fixtures/avroGraphChangeSchemaType';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  createKeys,
  createAndFundKeypair,
  assertExtrinsicSuccess,
  generateSchemaPartialName,
} from '../scaffolding/helpers';
import { getFundingSource } from '../scaffolding/funding';

const fundingSource = getFundingSource(import.meta.url);

describe('#createSchema', function () {
  let keys: KeyringPair;
  let accountWithNoFunds: KeyringPair;

  before(async function () {
    keys = await createAndFundKeypair(fundingSource, 50_000_000n);
    accountWithNoFunds = createKeys();
  });

  it('should fail if account does not have enough tokens', async function () {
    await assert.rejects(
      ExtrinsicHelper.createSchema(accountWithNoFunds, AVRO_GRAPH_CHANGE, 'AvroBinary', 'OnChain').signAndSend(
        'current'
      ),
      {
        name: 'RpcError',
        message: /Inability to pay some fees/,
      }
    );
  });

  it('should fail if account does not have enough tokens v2', async function () {
    await assert.rejects(
      ExtrinsicHelper.createSchemaV2(accountWithNoFunds, AVRO_GRAPH_CHANGE, 'AvroBinary', 'OnChain', []).signAndSend(
        'current'
      ),
      {
        name: 'RpcError',
        message: /Inability to pay some fees/,
      }
    );
  });

  it('should fail to create invalid schema', async function () {
    const f = ExtrinsicHelper.createSchema(keys, [1000, 3], 'AvroBinary', 'OnChain');

    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'InvalidSchema',
    });
  });

  it('should fail to create invalid schema v2', async function () {
    const f = ExtrinsicHelper.createSchemaV2(keys, [1000, 3], 'AvroBinary', 'OnChain', []);

    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'InvalidSchema',
    });
  });

  it('should fail to create schema less than minimum size', async function () {
    const f = ExtrinsicHelper.createSchema(keys, {}, 'AvroBinary', 'OnChain');
    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'LessThanMinSchemaModelBytes',
    });
  });

  it('should fail to create schema less than minimum size v2', async function () {
    const f = ExtrinsicHelper.createSchemaV2(keys, {}, 'AvroBinary', 'OnChain', []);
    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'LessThanMinSchemaModelBytes',
    });
  });

  it('should fail to create schema greater than maximum size', async function () {
    const maxBytes = (await ExtrinsicHelper.getSchemaMaxBytes()).toNumber();

    // Create a schema whose JSON representation is exactly 1 byte larger than the max allowed
    const hugeSchema = {
      type: 'record',
      fields: [],
    };
    const hugeSize = JSON.stringify(hugeSchema).length;
    const sizeToFill = maxBytes - hugeSize - ',"name":""'.length + 1;
    hugeSchema['name'] = Array.from(Array(sizeToFill).keys())
      .map(() => 'a')
      .join('');

    const f = ExtrinsicHelper.createSchema(keys, hugeSchema, 'AvroBinary', 'OnChain');
    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'ExceedsMaxSchemaModelBytes',
    });
  });

  it('should fail to create schema greater than maximum size v2', async function () {
    const maxBytes = (await ExtrinsicHelper.getSchemaMaxBytes()).toNumber();

    // Create a schema whose JSON representation is exactly 1 byte larger than the max allowed
    const hugeSchema = {
      type: 'record',
      fields: [],
    };
    const hugeSize = JSON.stringify(hugeSchema).length;
    const sizeToFill = maxBytes - hugeSize - ',"name":""'.length + 1;
    hugeSchema['name'] = Array.from(Array(sizeToFill).keys())
      .map(() => 'a')
      .join('');

    const f = ExtrinsicHelper.createSchemaV2(keys, hugeSchema, 'AvroBinary', 'OnChain', []);
    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'ExceedsMaxSchemaModelBytes',
    });
  });

  it('should successfully create an Avro GraphChange schema', async function () {
    const f = ExtrinsicHelper.createSchema(keys, AVRO_GRAPH_CHANGE, 'AvroBinary', 'OnChain');
    const { target: createSchemaEvent, eventMap } = await f.fundAndSend(fundingSource);

    assertExtrinsicSuccess(eventMap);
    assert.notEqual(createSchemaEvent, undefined);
  });

  it('should successfully create an Avro GraphChange schema v2', async function () {
    const f = ExtrinsicHelper.createSchemaV2(keys, AVRO_GRAPH_CHANGE, 'AvroBinary', 'OnChain', []);
    const { target: createSchemaEvent, eventMap } = await f.fundAndSend(fundingSource);

    assertExtrinsicSuccess(eventMap);
    assert.notEqual(createSchemaEvent, undefined);
  });

  it('should successfully create a schema v3 with name', async function () {
    const schemaName = 'e-e.' + generateSchemaPartialName(20);
    const f = ExtrinsicHelper.createSchemaV3(keys, AVRO_GRAPH_CHANGE, 'AvroBinary', 'OnChain', [], schemaName);
    const { target: createSchemaEvent, eventMap } = await f.fundAndSend(fundingSource);

    assertExtrinsicSuccess(eventMap);
    assert.notEqual(createSchemaEvent, undefined);
    assert.notEqual(eventMap['schemas.SchemaNameCreated'], undefined);
  });

  it('should successfully create a schema v3 without a name', async function () {
    const f = ExtrinsicHelper.createSchemaV3(keys, AVRO_GRAPH_CHANGE, 'AvroBinary', 'OnChain', [], null);
    const { target: createSchemaEvent, eventMap } = await f.fundAndSend(fundingSource);

    assertExtrinsicSuccess(eventMap);
    assert.notEqual(createSchemaEvent, undefined);
    assert.equal(eventMap['schemas.SchemaNameCreated'], undefined);
  });

  it('should fail to create schema with invalid character in name v3', async function () {
    const f = ExtrinsicHelper.createSchemaV3(keys, AVRO_GRAPH_CHANGE, 'AvroBinary', 'OnChain', [], 'test2.invalid');
    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'InvalidSchemaNameCharacters',
    });
  });

  it('should fail to create schema with invalid name structure v3', async function () {
    const f = ExtrinsicHelper.createSchemaV3(keys, AVRO_GRAPH_CHANGE, 'AvroBinary', 'OnChain', [], 'test');
    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'InvalidSchemaNameStructure',
    });
  });

  it('should fail to create schema with invalid name encoding v3', async function () {
    const f = ExtrinsicHelper.createSchemaV3(keys, AVRO_GRAPH_CHANGE, 'AvroBinary', 'OnChain', [], 'ñòò.ò');
    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'InvalidSchemaNameEncoding',
    });
  });

  it('should fail to create schema with invalid namespace length v3', async function () {
    const f = ExtrinsicHelper.createSchemaV3(keys, AVRO_GRAPH_CHANGE, 'AvroBinary', 'OnChain', [], 'a.b');
    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'InvalidSchemaNamespaceLength',
    });
  });

  it('get version rpc should return all schemas using the same name', async function () {
    const namespace = generateSchemaPartialName(20);
    const aliceSchemaName = namespace + '.alice';
    const bobSchemaName = namespace + '.bob';
    const f = ExtrinsicHelper.createSchemaV3(keys, AVRO_GRAPH_CHANGE, 'AvroBinary', 'OnChain', [], aliceSchemaName);
    const { target: createSchemaEvent, eventMap } = await f.fundAndSend(fundingSource);

    assertExtrinsicSuccess(eventMap);
    assert.notEqual(createSchemaEvent, undefined);
    assert.notEqual(eventMap['schemas.SchemaNameCreated'], undefined);

    const f2 = ExtrinsicHelper.createSchemaV3(keys, AVRO_GRAPH_CHANGE, 'AvroBinary', 'OnChain', [], bobSchemaName);
    const { target: createSchemaEvent2, eventMap: eventMap2 } = await f2.fundAndSend(fundingSource);

    assertExtrinsicSuccess(eventMap2);
    assert.notEqual(createSchemaEvent2, undefined);
    assert.notEqual(eventMap2['schemas.SchemaNameCreated'], undefined);

    const versions = await ExtrinsicHelper.apiPromise.rpc.schemas.getVersions(namespace);
    assert(versions.isSome);
    const versions_response_value = versions.unwrap();
    assert.equal(versions_response_value.length, 2);
    assert(versions_response_value.toArray().some((v) => v.schema_name == aliceSchemaName));
    assert(versions_response_value.toArray().some((v) => v.schema_name == bobSchemaName));
  });
});
