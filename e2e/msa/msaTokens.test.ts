/* eslint-disable mocha/no-skipped-tests */
import '@frequency-chain/api-augment';
import assert from 'assert';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { HexString } from '@polkadot/util/types';
import { isTestnet } from '../scaffolding/env';
import { ethereumAddressToKeyringPair } from '../scaffolding/ethereum';
import { getFundingSource } from '../scaffolding/funding';
import { H160 } from '@polkadot/types/interfaces';
import { hexToU8a } from '@polkadot/util';
import { KeyringPair } from '@polkadot/keyring/types';
import { getExistentialDeposit } from '../scaffolding/helpers';

const fundingSource = getFundingSource(import.meta.url);
const msaId = 1234; // Example MSA ID for testing
let checksummedEthAddress: HexString;

if (isTestnet()) {
  checksummedEthAddress = '0x05500A07f5fD359e9E785c74E21d5b180e63B63b'; // Example checksummed Ethereum address for MSA ID 1234 on Paseo testnet
} else {
  checksummedEthAddress = '0xCa08F40AE1F1E311bC8516C9a43771828f0F14c2'; // Example checksummed Ethereum address for MSA ID 1234 on a development chain
}

describe('MSAs Holding Tokens', function () {
  let ethKeys: KeyringPair;
  let ethAddress20: H160;

  before(async function () {
    ethAddress20 = ExtrinsicHelper.apiPromise.createType('H160', hexToU8a(checksummedEthAddress));
    ethKeys = ethereumAddressToKeyringPair(ethAddress20);
  });

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
    it('should send tokens to the MSA', async function () {
      const ed = await getExistentialDeposit();
      const transferAmount = 1n + ed;
      let accountData = await ExtrinsicHelper.getAccountInfo(ethKeys);
      const initialBalance = accountData.data.free.toBigInt();
      const op = ExtrinsicHelper.transferFunds(
        fundingSource,
        ethereumAddressToKeyringPair(ethAddress20),
        transferAmount
      );

      const { target: transferEvent } = await op.fundAndSend(fundingSource);
      assert.notEqual(transferEvent, undefined, 'should have transferred tokens');

      accountData = await ExtrinsicHelper.getAccountInfo(ethKeys);
      const finalBalance = accountData.data.free.toBigInt();
      assert.equal(
        finalBalance,
        initialBalance + transferAmount,
        'Final balance should be increased by transfer amount'
      );
    });
  });

  describe('Withdraw tokens from MSA', function () {
    // eslint-disable-next-line @typescript-eslint/no-empty-function
    it.skip('should be able to withraw all tokens from the MSA', function () {});
    // eslint-disable-next-line @typescript-eslint/no-empty-function
    it.skip('withdrawing tokens from an MSA to another MSA should fail', function () {});
  });
});
