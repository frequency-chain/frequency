/* eslint-disable mocha/no-skipped-tests */
import '@frequency-chain/api-augment';
import assert from 'assert';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';

const msaId = 1234; // Example MSA ID for testing

describe('MSAs Holding Tokens', function () {
  describe('getEthereumAddressForMsaId', function () {
    it('should return the correct address for a given MSA ID', async function () {
      const expectedChecksum = '0x315A79Bd2D2f70b56EAEbf3abfc1c50C5c73E02C';
      const expectedAddress = expectedChecksum.toLowerCase();
      const { accountId, accountIdChecksummed } = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getEthereumAddressForMsaId(msaId);
      assert.equal(accountId.toHex(), expectedAddress, `Expected address ${expectedAddress}, but got ${accountId}`);
      assert.equal(accountIdChecksummed.toString(), expectedChecksum, `Expected checksummed address ${expectedChecksum}, but got ${accountIdChecksummed.toString()}`);
    });
  });

  describe('Send tokens to MSA', function () {
    // eslint-disable-next-line @typescript-eslint/no-empty-function
    it.skip('should send tokens to the MSA', async function () {});
  });

  describe('Withdraw tokens from MSA', function () {
    // eslint-disable-next-line @typescript-eslint/no-empty-function
    it.skip('should be able to withraw all tokens from the MSA', function () {});
    // eslint-disable-next-line @typescript-eslint/no-empty-function
    it.skip('withdrawing tokens from an MSA to another MSA should fail', function () {});
  });
});
