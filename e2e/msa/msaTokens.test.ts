/* eslint-disable mocha/no-skipped-tests */
import '@frequency-chain/api-augment';
import assert from 'assert';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { ethereumAddressToKeyringPair } from '../scaffolding/ethereum';
import { getFundingSource } from '../scaffolding/funding';
import { H160 } from '@polkadot/types/interfaces';
import { bnToU8a, hexToU8a, stringToU8a } from '@polkadot/util';
import { KeyringPair } from '@polkadot/keyring/types';
import { keccak256AsU8a } from '@polkadot/util-crypto';
import { getExistentialDeposit } from '../scaffolding/helpers';

const fundingSource = getFundingSource(import.meta.url);

/**
 *
 * @param msaId
 * @returns Ethereum address generated from the MSA ID
 *
 * This function generates an Ethereum address based on the provided MSA ID,
 * using a specific hashing algorithm and a salt value, as follows:
 *
 * Domain prefix: 0xD9
 * MSA ID: Big-endian bytes representation of the 64-bit MSA ID
 * Salt: Keccak256 hash of the string "MSA Generated"
 *
 * Hash = keccak256(0xD9 || MSA ID bytes || Salt)
 *
 * Address = Hash[-20:]
 */
function generateMsaAddress(msaId: string | number | bigint): H160 {
  const msa64 = ExtrinsicHelper.api.registry.createType('u64', msaId);
  const msaBytes = bnToU8a(msa64.toBn(), { isLe: false, bitLength: 64 });
  const salt = keccak256AsU8a(stringToU8a('MSA Generated'));
  const combined = new Uint8Array([0xd9, ...msaBytes, ...salt]);
  const hash = keccak256AsU8a(combined);

  return ExtrinsicHelper.api.registry.createType('H160', hash.slice(-20));
}

describe('MSAs Holding Tokens', function () {
  const MSA_ID_1234 = 1234; // MSA ID for testing
  const CHECKSUMMED_ETH_ADDR_1234 = '0x65928b9a88Db189Eea76F72d86128Af834d64c32'; // Checksummed Ethereum address for MSA ID 1234
  let ethKeys: KeyringPair;
  let ethAddress20: H160;

  before(async function () {
    ethAddress20 = ExtrinsicHelper.apiPromise.createType('H160', hexToU8a(CHECKSUMMED_ETH_ADDR_1234));
    ethKeys = ethereumAddressToKeyringPair(ethAddress20);
  });

  describe('getEthereumAddressForMsaId', function () {
    it('should return the correct address for a given MSA ID', async function () {
      const expectedAddress = CHECKSUMMED_ETH_ADDR_1234.toLowerCase();
      const { accountId, accountIdChecksummed } =
        await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getEthereumAddressForMsaId(MSA_ID_1234);
      assert.equal(accountId.toHex(), expectedAddress, `Expected address ${expectedAddress}, but got ${accountId}`);
      assert.equal(
        accountIdChecksummed.toString(),
        CHECKSUMMED_ETH_ADDR_1234,
        `Expected checksummed address ${CHECKSUMMED_ETH_ADDR_1234}, but got ${accountIdChecksummed.toString()}`
      );
    });

    it('should validate the Ethereum address for an MSA ID', async function () {
      const isValid = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.validateEthAddressForMsa(
        generateMsaAddress(MSA_ID_1234),
        MSA_ID_1234
      );
      assert.equal(isValid, true, 'Expected the Ethereum address to be valid for the given MSA ID');
    });

    it('should fail to validate the Ethereum address for an incorrect MSA ID', async function () {
      const isValid = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.validateEthAddressForMsa(
        CHECKSUMMED_ETH_ADDR_1234,
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
});
