/* eslint-disable mocha/no-skipped-tests */
import '@frequency-chain/api-augment';
import assert from 'assert';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { HexString } from '@polkadot/util/types';
import { isDev, isTestnet } from '../scaffolding/env';

const msaId = 1234; // Example MSA ID for testing
let checksummedEthAddress: HexString;

if (isTestnet()) {
  checksummedEthAddress = '0x05500A07f5fD359e9E785c74E21d5b180e63B63b'; // Example checksummed Ethereum address for MSA ID 1234 on Paseo testnet
} else {
  checksummedEthAddress = '0xF5b82ff246a2F4226749bd78B1bDaE28Cfffb9f7'; // Example checksummed Ethereum address for MSA ID 1234 on a development chain
}

describe('MSAs Holding Tokens', function () {
  describe('getEthereumAddressForMsaId', function () {
    it('should return the correct address for a given MSA ID', async function () {
      const expectedAddress = checksummedEthAddress.toLowerCase();
      const { accountId, accountIdChecksummed } =
        await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getEthereumAddressForMsaId(msaId);
      assert.equal(accountId.toHex(), expectedAddress, `Expected address ${expectedAddress}, but got ${accountId}`);
      assert.equal(
        accountIdChecksummed.toString(),
        checksummedEthAddress,
        `Expected checksummed address ${checksummedEthAddress}, but got ${accountIdChecksummed.toString()}`
      );
    });

    it('should validate the Ethereum address for an MSA ID', async function () {
      const isValid = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.validateEthAddressForMsa(
        checksummedEthAddress,
        msaId
      );
      assert.equal(isValid, true, 'Expected the Ethereum address to be valid for the given MSA ID');
    });

    it('should fail to validate the Ethereum address for an incorrect MSA ID', async function () {
      const isValid = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.validateEthAddressForMsa(
        checksummedEthAddress,
        4321
      );
      assert.equal(isValid, false, 'Expected the Ethereum address to be invalid for a different MSA ID');
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
