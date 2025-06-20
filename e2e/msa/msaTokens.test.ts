import '@frequency-chain/api-augment';
import assert from 'assert';
import { AuthorizedKeyData, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  EcdsaSignature,
  createAuthorizedKeyData,
  ethereumAddressToKeyringPair,
  getUnifiedAddress,
  getUnifiedPublicKey,
  signEip712,
} from '@frequency-chain/ethereum-utils';
import { getFundingSource } from '../scaffolding/funding';
import { H160 } from '@polkadot/types/interfaces';
import { bnToU8a, hexToU8a, stringToU8a, u8aToHex } from '@polkadot/util';
import { KeyringPair } from '@polkadot/keyring/types';
import { keccak256AsU8a } from '@polkadot/util-crypto';
import {
  CENTS,
  createAndFundKeypair,
  createKeys,
  DOLLARS,
  generateAddKeyPayload,
  generateAuthorizedKeyPayload,
  getEthereumKeyPairFromUnifiedAddress,
  signPayload,
} from '../scaffolding/helpers';
import { u64 } from '@polkadot/types';

const fundingSource = getFundingSource(import.meta.url);
const TRANSFER_AMOUNT = 1n * DOLLARS;

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

async function generateSignedAuthorizedKeyPayload(keys: KeyringPair, payload: AuthorizedKeyData) {
  const signingPayload = createAuthorizedKeyData(
    payload.msaId!.toString(),
    u8aToHex(payload.authorizedPublicKey),
    payload.expiration
  );
  const ownerSig = await signEip712(
    u8aToHex(getEthereumKeyPairFromUnifiedAddress(getUnifiedAddress(keys)).secretKey),
    signingPayload
  );

  return {
    signingPayload,
    ownerSig,
  };
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
      const result: any = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getEthereumAddressForMsaId(MSA_ID_1234);
      const accountId = result?.accountId;
      const accountIdChecksummed = result?.accountIdChecksummed;

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
      let accountData = await ExtrinsicHelper.getAccountInfo(ethKeys);
      const initialBalance = accountData.data.free.toBigInt();
      const op = ExtrinsicHelper.transferFunds(
        fundingSource,
        ethereumAddressToKeyringPair(ethAddress20),
        TRANSFER_AMOUNT
      );

      const { target: transferEvent } = await op.fundAndSend(fundingSource);
      assert.notEqual(transferEvent, undefined, 'should have transferred tokens');

      accountData = await ExtrinsicHelper.getAccountInfo(ethKeys);
      const finalBalance = accountData.data.free.toBigInt();
      assert.equal(
        finalBalance,
        initialBalance + TRANSFER_AMOUNT,
        'Final balance should be increased by transfer amount'
      );
    });
  });

  describe('withdrawTokens', function () {
    let msaKeys: KeyringPair;
    let msaId: u64;
    let msaAddress: H160;
    let otherMsaKeys: KeyringPair;
    let secondaryKey: KeyringPair;
    let defaultPayload: AuthorizedKeyData;
    let payload: AuthorizedKeyData;
    let ownerSig: EcdsaSignature;
    let badSig: EcdsaSignature;

    before(async function () {
      // Setup an MSA with tokens
      msaKeys = await createAndFundKeypair(fundingSource, 5n * CENTS, undefined, undefined, 'ethereum');
      let { target } = await ExtrinsicHelper.createMsa(msaKeys).signAndSend();
      assert.notEqual(target?.data.msaId, undefined, 'MSA Id not in expected event');
      msaId = target!.data.msaId;

      // Setup another MSA control key
      otherMsaKeys = await createAndFundKeypair(fundingSource, 5n * CENTS, undefined, undefined, 'ethereum');
      ({ target } = await ExtrinsicHelper.createMsa(otherMsaKeys).signAndSend());
      assert.notEqual(target?.data.msaId, undefined, 'MSA Id not in expected event');

      const { accountId } = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getEthereumAddressForMsaId(msaId);
      msaAddress = accountId;

      // Create unfunded keys; this extrinsic should be free
      secondaryKey = createKeys(undefined, 'ethereum');

      // Default payload making it easier to test `withdrawTokens`
      defaultPayload = {
        discriminant: 'AuthorizedKeyData',
        msaId,
        authorizedPublicKey: getUnifiedPublicKey(secondaryKey),
      };
    });

    beforeEach(async function () {
      payload = await generateAuthorizedKeyPayload(defaultPayload);
    });

    it('should fail if origin is not address contained in the payload (NotKeyOwner)', async function () {
      const badPayload = { ...payload, authorizedPublicKey: getUnifiedPublicKey(createKeys()) }; // Invalid MSA ID
      ({ ownerSig } = await generateSignedAuthorizedKeyPayload(msaKeys, payload));
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, msaKeys, ownerSig, badPayload);
      await assert.rejects(op.signAndSend('current'), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 5', // NotKeyOwner,
      });
    });

    it('should fail if signed payload is not actually an AuthorizedKeyData (MsaOwnershipInvalidSignature)', async function () {
      const { discriminant, ...badPayload } = defaultPayload;
      // Generate AddKeyData instead of AuthorizedKeyData (missing discriminator)
      const payload = await generateAddKeyPayload(badPayload);
      const signingPayload = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', payload);
      const ownerSig = signPayload(msaKeys, signingPayload);
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, msaKeys, ownerSig, payload as AuthorizedKeyData);
      await assert.rejects(op.signAndSend('current'), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 8', // MsaOwnershipInvalidSignature
      });
    });

    it('should fail if MSA owner signature is invalid (MsaOwnershipInvalidSignature)', async function () {
      ({ ownerSig: badSig } = await generateSignedAuthorizedKeyPayload(createKeys('badKeys', 'ethereum'), payload));
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, msaKeys, badSig, payload);
      await assert.rejects(op.signAndSend('current'), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 8', // MsaOwnershipInvalidSignature
      });
    });

    it('should fail if expiration has passed (MsaOwnershipInvalidSignature)', async function () {
      const newPayload = await generateAuthorizedKeyPayload({
        ...defaultPayload,
        expiration: (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber(),
      });
      ({ ownerSig } = await generateSignedAuthorizedKeyPayload(msaKeys, newPayload));
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, msaKeys, ownerSig, newPayload);
      await assert.rejects(op.signAndSend('current'), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 8', // MsaOwnershipInvalidSignature,
      });
    });

    it('should fail if expiration is not yet valid (MsaOwnershipInvalidSignature)', async function () {
      const maxMortality = ExtrinsicHelper.api.consts.msa.mortalityWindowSize.toNumber();
      const newPayload = await generateAuthorizedKeyPayload({
        ...defaultPayload,
        expiration: (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber() + maxMortality + 999,
      });
      ({ ownerSig } = await generateSignedAuthorizedKeyPayload(msaKeys, newPayload));
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, msaKeys, ownerSig, newPayload);
      await assert.rejects(op.signAndSend('current'), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 8', // MsaOwnershipInvalidSignature,
      });
    });

    it('should fail if origin is an MSA control key (IneligibleOrigin)', async function () {
      const newPayload = await generateAuthorizedKeyPayload({
        ...defaultPayload,
        authorizedPublicKey: getUnifiedPublicKey(otherMsaKeys),
      });
      ({ ownerSig } = await generateSignedAuthorizedKeyPayload(msaKeys, newPayload));
      const op = ExtrinsicHelper.withdrawTokens(otherMsaKeys, msaKeys, ownerSig, newPayload);
      await assert.rejects(op.signAndSend('current'), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 10', // IneligibleOrigin,
      });
    });

    it('should fail if payload signer does not control the MSA in the signed payload (InvalidMsaKey)', async function () {
      const newPayload = await generateAuthorizedKeyPayload({
        ...defaultPayload,
        msaId: new u64(ExtrinsicHelper.api.registry, 9999),
      });
      ({ ownerSig } = await generateSignedAuthorizedKeyPayload(msaKeys, newPayload));
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, msaKeys, ownerSig, newPayload);
      await assert.rejects(op.signAndSend('current'), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 1', // InvalidMsaKey,
      });
    });

    it('should fail if payload signer is not an MSA control key (InvalidMsaKey)', async function () {
      const badSigner = createKeys(undefined, 'ethereum');
      const newPayload = await generateAuthorizedKeyPayload({
        ...defaultPayload,
        msaId: new u64(ExtrinsicHelper.api.registry, 9999),
      });
      ({ ownerSig } = await generateSignedAuthorizedKeyPayload(badSigner, newPayload));
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, badSigner, ownerSig, newPayload);
      await assert.rejects(op.signAndSend('current'), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 1', // InvalidMsaKey,
      });
    });

    it('should fail if MSA does not have a balance', async function () {
      ({ ownerSig } = await generateSignedAuthorizedKeyPayload(msaKeys, payload));
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, msaKeys, ownerSig, payload);
      await assert.rejects(op.signAndSend('current'), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 9', // InsufficientBalanceToWithdraw,
      });
    });

    it('should succeed', async function () {
      const {
        data: { free: startingBalance },
      } = await ExtrinsicHelper.getAccountInfo(secondaryKey);
      // Send tokens to MSA
      const op1 = ExtrinsicHelper.transferFunds(
        fundingSource,
        ethereumAddressToKeyringPair(msaAddress),
        TRANSFER_AMOUNT
      );
      await assert.doesNotReject(op1.signAndSend(), 'MSA funding failed');
      ({ ownerSig } = await generateSignedAuthorizedKeyPayload(msaKeys, payload));
      const op2 = ExtrinsicHelper.withdrawTokens(secondaryKey, msaKeys, ownerSig, payload);
      await assert.doesNotReject(op2.signAndSend('current'), 'token transfer transaction should have succeeded');
      // Destination account should have had balance increased
      const {
        data: { free: endingBalance },
      } = await ExtrinsicHelper.getAccountInfo(secondaryKey);

      assert(
        startingBalance.toBigInt() + TRANSFER_AMOUNT === endingBalance.toBigInt(),
        'balance of recieve should have increased by the transfer amount minus fee'
      );
    });

    it('should fail for duplicate signature submission (MsaOwnershipInvalidSignature)', async function () {
      // In order to test this, we need to create a new keypair and fund it, because otherwise the nonce will
      // be the same for both transactions (and, because we're using Edcs signatures, the signature will be the same).
      // Not sure exactly what happens in this case, but it seems to be that the second transaction is siliently dropped
      // by the node, but the status call back in polkadot.js still resolves (ie, gets 'isInBlock' or 'isFinalized')
      const keys = await createAndFundKeypair(fundingSource, 5n * CENTS, undefined, undefined, 'ethereum');
      payload.authorizedPublicKey = getUnifiedPublicKey(keys);

      const op1 = ExtrinsicHelper.transferFunds(
        fundingSource,
        ethereumAddressToKeyringPair(msaAddress),
        TRANSFER_AMOUNT
      );
      await assert.doesNotReject(op1.signAndSend(), 'MSA funding failed');

      ({ ownerSig } = await generateSignedAuthorizedKeyPayload(msaKeys, payload));
      let op2 = ExtrinsicHelper.withdrawTokens(keys, msaKeys, ownerSig, payload);
      await assert.doesNotReject(op2.signAndSend('current'), 'token withdrawal should have succeeded');

      // Re-fund MSA so we don't fail for that
      await assert.doesNotReject(op1.signAndSend(), 'MSA re-funding failed');
      op2 = ExtrinsicHelper.withdrawTokens(keys, msaKeys, ownerSig, payload);
      await assert.rejects(op2.signAndSend('current'), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 8', // MsaOwnershipInvalidSignature,
      });
    });
  });
});
