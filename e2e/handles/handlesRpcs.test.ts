//  Handles RPC test suite
import '@frequency-chain/api-augment';
import assert from 'assert';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';

describe('Handles RPCs', function () {
  describe('Suffixes Integrity Check', function () {
    it('should return same suffixes for `abcdefg` from chain as hardcoded', async function () {
      const suffixes = await ExtrinsicHelper.getNextSuffixesForHandle('abcdefg', 10);
      const suffixes_expected = [23, 65, 16, 53, 25, 75, 29, 26, 10, 87];
      const resp_suffixes_number = suffixes.suffixes.map((x) => x.toNumber());
      assert.deepEqual(resp_suffixes_number, suffixes_expected, 'suffixes should be equal to suffixes_expected');
    });
  });

  describe('validateHandle basic test', function () {
    it('returns true for good handle, and false for bad handle', async function () {
      let res = await ExtrinsicHelper.validateHandle('Robert`DROP TABLE STUDENTS;--');
      assert.equal(res.toHuman(), false);
      res = await ExtrinsicHelper.validateHandle('Little Bobby Tables');
      assert.equal(res.toHuman(), true);
      res = await ExtrinsicHelper.validateHandle('Bobbay😀😀');
      assert.equal(res.toHuman(), false);
    });
  });

  describe('checkHandle basic test', function () {
    it('expected outcome for a good handle', async function () {
      const res = await ExtrinsicHelper.apiPromise.call.handlesRuntimeApi.checkHandle('Little Bobby Tables');
      assert(!res.isEmpty, 'Expected a response');
      assert.deepEqual(res.toHuman(), {
        baseHandle: 'Little Bobby Tables',
        canonicalBase: 'l1tt1eb0bbytab1es',
        suffixIndex: '0',
        suffixesAvailable: true,
        valid: true,
      });
    });

    it('expected outcome for a bad handle', async function () {
      const res = await ExtrinsicHelper.apiPromise.call.handlesRuntimeApi.checkHandle('Robert`DROP TABLE STUDENTS;--');
      assert(!res.isEmpty, 'Expected a response');
      assert.deepEqual(res.toHuman(), {
        baseHandle: 'Robert`DROP TABLE STUDENTS;--',
        canonicalBase: '',
        suffixIndex: '0',
        suffixesAvailable: false,
        valid: false,
      });
    });

    it('expected outcome for a good handle with complex whitespace', async function () {
      const res = await ExtrinsicHelper.apiPromise.call.handlesRuntimeApi.checkHandle('नी हुन्‍न् ।');
      assert(!res.isEmpty, 'Expected a response');
      assert.deepEqual(res.toHuman(), {
        baseHandle: '0xe0a4a8e0a58020e0a4b9e0a581e0a4a8e0a58de2808de0a4a8e0a58d20e0a5a4',
        canonicalBase: '0xe0a4a8e0a4b9e0a4a8e0a4a8e0a5a4',
        suffixIndex: '0',
        suffixesAvailable: true,
        valid: true,
      });
    });
  });
});
