import '@frequency-chain/api-augment';
import assert from 'assert';
import { AuthorizedKeyData, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
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
  generateAuthorizedKeyPayload,
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
    let msaKeys: KeyringPair;
    let msaId: u64;
    let msaAddress: H160;
    let otherMsaKeys: KeyringPair;
    let secondaryKey: KeyringPair;
    let defaultPayload: AuthorizedKeyData;
    let payload: AuthorizedKeyData;
    let ownerSig: Sr25519Signature;
    let badSig: Sr25519Signature;
    let authorizedKeyData: Codec;

    before(async function () {
      // Setup an MSA with tokens
      msaKeys = await createAndFundKeypair(fundingSource, 5n * CENTS);
      let { target } = await ExtrinsicHelper.createMsa(msaKeys).signAndSend();
      assert.notEqual(target?.data.msaId, undefined, 'MSA Id not in expected event');
      msaId = target!.data.msaId;

      // Setup another MSA control key
      otherMsaKeys = await createAndFundKeypair(fundingSource, 5n * CENTS);
      ({ target } = await ExtrinsicHelper.createMsa(otherMsaKeys).signAndSend());
      assert.notEqual(target?.data.msaId, undefined, 'MSA Id not in expected event');

      const { accountId } = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getEthereumAddressForMsaId(msaId);
      msaAddress = accountId;

      // Create unfunded keys; this extrinsic should be free
      secondaryKey = createKeys();

      // Default payload making it easier to test `withdrawTokens`
      defaultPayload = {
        msaId,
        authorizedPublicKey: getUnifiedPublicKey(secondaryKey),
      };
    });

    beforeEach(async function () {
      payload = await generateAuthorizedKeyPayload(defaultPayload);
    });

    it('should fail if origin is not address contained in the payload (NotKeyOwner)', async function () {
      const badPayload = { ...payload, authorizedPublicKey: getUnifiedAddress(createKeys()) }; // Invalid MSA ID
      authorizedKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAuthorizedKeyData', badPayload);
      ownerSig = signPayloadSr25519(msaKeys, authorizedKeyData);
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, msaKeys, ownerSig, badPayload);
      await assert.rejects(op.signAndSend(), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 5', // NotKeyOwner,
      });
    });

    it('should fail if MSA owner signature is invalid (MsaOwnershipInvalidSignature)', async function () {
      authorizedKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAuthorizedKeyData', payload);
      badSig = signPayloadSr25519(createKeys(), authorizedKeyData); // Invalid signature
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, msaKeys, badSig, payload);
      await assert.rejects(op.signAndSend(), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 8', // MsaOwnershipInvalidSignature
      });
    });

    it('should fail if expiration has passed (ProofHasExpired)', async function () {
      const newPayload = await generateAuthorizedKeyPayload({
        ...defaultPayload,
        expiration: (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber(),
      });
      authorizedKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAuthorizedKeyData', newPayload);
      ownerSig = signPayloadSr25519(msaKeys, authorizedKeyData);
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, msaKeys, ownerSig, newPayload);
      await assert.rejects(op.signAndSend(), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 14', // ProofHasExpired,
      });
    });

    it('should fail if expiration is not yet valid (ProofNotYetValid)', async function () {
      const maxMortality = ExtrinsicHelper.api.consts.msa.mortalityWindowSize.toNumber();
      const newPayload = await generateAuthorizedKeyPayload({
        ...defaultPayload,
        expiration: (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber() + maxMortality + 999,
      });
      authorizedKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAuthorizedKeyData', newPayload);
      ownerSig = signPayloadSr25519(msaKeys, authorizedKeyData);
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, msaKeys, ownerSig, newPayload);
      await assert.rejects(op.signAndSend(), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 13', // ProofNotYetValid,
      });
    });

    it('should fail if origin is an MSA control key (IneligibleOrigin)', async function () {
      authorizedKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAuthorizedKeyData', payload);
      const newPayload = await generateAuthorizedKeyPayload({
        ...defaultPayload,
        authorizedPublicKey: getUnifiedPublicKey(otherMsaKeys),
      });
      authorizedKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAuthorizedKeyData', newPayload);
      ownerSig = signPayloadSr25519(msaKeys, authorizedKeyData);
      const op = ExtrinsicHelper.withdrawTokens(otherMsaKeys, msaKeys, ownerSig, newPayload);
      await assert.rejects(op.signAndSend(), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 12', // IneligibleOrigin,
      });
    });

    it('should fail if payload signer does not control the MSA in the signed payload (NotMsaOwner)', async function () {
      const newPayload = await generateAuthorizedKeyPayload({
        ...defaultPayload,
        msaId: new u64(ExtrinsicHelper.api.registry, 9999),
      });
      authorizedKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAuthorizedKeyData', newPayload);
      ownerSig = signPayloadSr25519(msaKeys, authorizedKeyData);
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, msaKeys, ownerSig, newPayload);
      await assert.rejects(op.signAndSend(), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 17', // NotMsaOwner,
      });
    });

    it('should fail if payload signer is not an MSA control key (NoKeyExists)', async function () {
      const badSigner = createKeys();
      const newPayload = await generateAuthorizedKeyPayload({
        ...defaultPayload,
        msaId: new u64(ExtrinsicHelper.api.registry, 9999),
      });
      authorizedKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAuthorizedKeyData', newPayload);
      ownerSig = signPayloadSr25519(badSigner, authorizedKeyData);
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, badSigner, ownerSig, newPayload);
      await assert.rejects(op.signAndSend(), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 16', // NoKeyExists,
      });
    });

    it('should fail if MSA does not have a balance', async function () {
      authorizedKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAuthorizedKeyData', payload);
      ownerSig = signPayloadSr25519(msaKeys, authorizedKeyData);
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, msaKeys, ownerSig, payload);
      await assert.rejects(op.signAndSend(), {
        name: 'RpcError',
        code: 1010,
        data: 'Custom error: 9', // InsufficientBalanceToWithdraw,
      });
    });

    it('should succeed', async function () {
      const transferAmount = 1n * DOLLARS;
      const {
        data: { free: startingBalance },
      } = await ExtrinsicHelper.getAccountInfo(secondaryKey);

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

      authorizedKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAuthorizedKeyData', payload);
      ownerSig = signPayloadSr25519(msaKeys, authorizedKeyData);
      const op = ExtrinsicHelper.withdrawTokens(secondaryKey, msaKeys, ownerSig, payload);
      await assert.doesNotReject(op.signAndSend(), 'token transfer transaction should have succeeded');

      // Destination account should have had balance increased
      const {
        data: { free: endingBalance },
      } = await ExtrinsicHelper.getAccountInfo(secondaryKey);

      assert(
        startingBalance.toBigInt() + transferAmount === endingBalance.toBigInt(),
        'balance of recieve should have increased by the transfer amount minus fee'
      );
    });
  });
});
