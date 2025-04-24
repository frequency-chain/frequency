/* eslint-disable mocha/no-skipped-tests */
import '@frequency-chain/api-augment';
import assert from 'assert';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { u8aToString } from '@polkadot/util';

describe('MSAs Holding Tokens', function () {
  describe('getEthereumAddressForMsaId', function () {
    it('should return the correct address for a given MSA ID', async function () {
      const msaId = 1234;
      const expectedAddress = '0x00000000000004d247656E657261746564000000';
      const raw = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getEthereumAddressForMsaId(msaId);
      const address = u8aToString(raw);
      assert.equal(address, expectedAddress, `Expected address ${expectedAddress}, but got ${address}`);
    });
  });

  describe('Send tokens to MSA', function () {
    // eslint-disable-next-line @typescript-eslint/no-empty-function
    it.skip('should send tokens to the MSA', function () {});
  });

  describe('Withdraw tokens from MSA', function () {
    // eslint-disable-next-line @typescript-eslint/no-empty-function
    it.skip('should be able to withraw all tokens from the MSA', function () {});
    // eslint-disable-next-line @typescript-eslint/no-empty-function
    it.skip('withdrawing tokens from an MSA to another MSA should fail', function () {});
  });
});
