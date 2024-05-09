import assert from 'assert';
import { TypeRegistry } from '@polkadot/types';
import { types } from '../index.js';

describe('types', function () {
  it('should be able to successfully register', function () {
    const registry = new TypeRegistry();
    assert.doesNotThrow(() => {
      registry.register(types);
    });
  });
});
