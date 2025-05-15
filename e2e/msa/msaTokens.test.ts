import '@frequency-chain/api-augment';
import assert from 'assert';
import { AddKeyData, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { ethereumAddressToKeyringPair, getUnifiedAddress, getUnifiedPublicKey } from '../scaffolding/ethereum';
import { getFundingSource } from '../scaffolding/funding';
import { H160 } from '@polkadot/types/interfaces';
import { bnToU8a, hexToU8a, stringToU8a } from '@polkadot/util';
import { KeyringPair } from '@polkadot/keyring/types';
import { keccak256AsU8a } from '@polkadot/util-crypto';
import {
  CENTS,
  createAndFundKeypair,
  createKeys,
  DOLLARS,
  generateAddKeyPayload,
  getExistentialDeposit,
  signPayloadSr25519,
  Sr25519Signature,
} from '../scaffolding/helpers';
import { u64 } from '@polkadot/types';
import { Codec } from '@polkadot/types/types';

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
      const ed = getExistentialDeposit();
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

  describe('withdrawTokens', function () {
    let keys: KeyringPair;
    let msaId: u64;
    let msaAddress: H160;
    let secondaryKey: KeyringPair;
    const defaultPayload: AddKeyData = {};
    let payload: AddKeyData;
    let ownerSig: Sr25519Signature;
    let badSig: Sr25519Signature;
    let addKeyData: Codec;

    before(async function () {
      // Setup an MSA with tokens
      keys = await createAndFundKeypair(fundingSource, 5n * CENTS);
      const { target } = await ExtrinsicHelper.createMsa(keys).signAndSend();
      assert.notEqual(target?.data.msaId, undefined, 'MSA Id not in expected event');
      msaId = target!.data.msaId;

      const { accountId } = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getEthereumAddressForMsaId(msaId);
      msaAddress = accountId;

      secondaryKey = await createAndFundKeypair(fundingSource, 5n * CENTS);

      // Default payload making it easier to test `withdrawTokens`
      defaultPayload.msaId = msaId;
      defaultPayload.newPublicKey = getUnifiedPublicKey(secondaryKey);
    });

    beforeEach(async function () {
      payload = await generateAddKeyPayload(defaultPayload);
    });

    it('should fail if origin is not address contained in the payload (NotKeyOwner)', async function () {
      const badPayload = { ...payload, newPublicKey: getUnifiedAddress(createKeys()) }; // Invalid MSA ID
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', badPayload);
      ownerSig = signPayloadSr25519(keys, addKeyData);
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, keys, ownerSig, badPayload);
      await assert.rejects(op.fundAndSend(fundingSource), {
        name: 'NotKeyOwner',
      });
    });

    it('should fail if MSA owner signature is invalid (MsaOwnershipInvalidSignature)', async function () {
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', payload);
      badSig = signPayloadSr25519(createKeys(), addKeyData); // Invalid signature
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, keys, badSig, payload);
      await assert.rejects(op.fundAndSend(fundingSource), {
        name: 'MsaOwnershipInvalidSignature',
      });
    });

    it('should fail if expiration has passed (ProofHasExpired)', async function () {
      const newPayload = await generateAddKeyPayload({
        ...defaultPayload,
        expiration: (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber(),
      });
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', newPayload);
      ownerSig = signPayloadSr25519(keys, addKeyData);
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, keys, ownerSig, newPayload);
      await assert.rejects(op.fundAndSend(fundingSource), {
        name: 'ProofHasExpired',
      });
    });

    it('should fail if expiration is not yet valid (ProofNotYetValid)', async function () {
      const maxMortality = ExtrinsicHelper.api.consts.msa.mortalityWindowSize.toNumber();
      const newPayload = await generateAddKeyPayload({
        ...defaultPayload,
        expiration: (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber() + maxMortality + 999,
      });
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', newPayload);
      ownerSig = signPayloadSr25519(keys, addKeyData);
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, keys, ownerSig, newPayload);
      await assert.rejects(op.fundAndSend(fundingSource), {
        name: 'ProofNotYetValid',
      });
    });

    it('should fail if payload signer does not control the MSA in the signed payload (NotMsaOwner)', async function () {
      const newPayload = await generateAddKeyPayload({
        ...defaultPayload,
        msaId: new u64(ExtrinsicHelper.api.registry, 9999),
      });
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', newPayload);
      ownerSig = signPayloadSr25519(keys, addKeyData);
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, keys, ownerSig, newPayload);
      await assert.rejects(op.fundAndSend(fundingSource), {
        name: 'NotMsaOwner',
      });
    });

    it('should fail if payload signer is not an MSA control key (NoKeyExists)', async function () {
      const badSigner = createKeys();
      const newPayload = await generateAddKeyPayload({
        ...defaultPayload,
        msaId: new u64(ExtrinsicHelper.api.registry, 9999),
      });
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', newPayload);
      ownerSig = signPayloadSr25519(badSigner, addKeyData);
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, badSigner, ownerSig, newPayload);
      await assert.rejects(op.fundAndSend(fundingSource), {
        name: 'NoKeyExists',
      });
    });

    it('should fail if MSA does not have a balance', async function () {
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', payload);
      ownerSig = signPayloadSr25519(keys, addKeyData);
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, keys, ownerSig, payload);
      await assert.rejects(op.fundAndSend(fundingSource), {
        name: 'InsufficientBalanceToWithdraw',
      });
    });

    it('should succeed', async function () {
      // Fund receiver with known amount to pay for transaction
      const startingAmount = 1n * DOLLARS;
      const transferAmount = 1n * DOLLARS;
      const tertiaryKeys = await createAndFundKeypair(fundingSource, startingAmount);
      const {
        data: { free: startingBalance },
      } = await ExtrinsicHelper.getAccountInfo(tertiaryKeys);

      // Send tokens to MSA
      try {
        const { target: transferEvent } = await ExtrinsicHelper.transferFunds(
          fundingSource,
          ethereumAddressToKeyringPair(msaAddress),
          transferAmount
        ).signAndSend();
        assert.notEqual(transferEvent, undefined, 'should have transferred tokens to MSA');
      } catch (err: any) {
        console.error('Error sending tokens to MSA', err.message);
      }

      const newPayload = await generateAddKeyPayload({ ...payload, newPublicKey: getUnifiedPublicKey(tertiaryKeys) });
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', newPayload);
      ownerSig = signPayloadSr25519(keys, addKeyData);
      const op = ExtrinsicHelper.withdrawTokens(tertiaryKeys, keys, ownerSig, newPayload);
      const { eventMap } = await op.fundAndSend(fundingSource);
      const feeAmount = (eventMap['transactionPayment.TransactionFeePaid'].data as unknown as any).actualFee;

      // Destination account should have had balance increased
      const {
        data: { free: endingBalance },
      } = await ExtrinsicHelper.getAccountInfo(tertiaryKeys);

      assert(
        startingBalance.toBigInt() + transferAmount - feeAmount.toBigInt() === endingBalance.toBigInt(),
        'balance of recieve should have increased by the transfer amount minus fee'
      );
    });
  });
});
