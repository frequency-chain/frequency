import assert from 'assert';
import { options } from '../index.js';
import { ApiPromise } from '@polkadot/api';
import { MockProvider } from '@polkadot/rpc-provider/mock';
import { TypeRegistry } from '@polkadot/types';
// @ts-expect-error engine doesn't like this for some reason
import metadataRaw from '../metadata.json' with { type: 'json' };

describe('index', function () {
  let mock: MockProvider;
  let api: ApiPromise;

  beforeEach(async function () {
    mock = new MockProvider(new TypeRegistry());

    api = await ApiPromise.create({
      ...options,
      provider: mock,
      metadata: metadataRaw as any,
    });
  });

  afterEach(async function () {
    await api.disconnect();
    await mock.disconnect();
  });

  it('should have rpc calls', async function () {
    assert.notEqual(api.rpc.frequency, undefined);
    assert.notEqual(api.rpc.frequencyTxPayment, undefined);
    assert.notEqual(api.rpc.handles, undefined);
    assert.notEqual(api.rpc.messages, undefined);
    assert.notEqual(api.rpc.msa, undefined);
    assert.notEqual(api.rpc.schemas, undefined);
    assert.notEqual(api.rpc.statefulStorage, undefined);
  });
});
