import '@frequency-chain/api-augment';
import '@frequency-chain/recovery-sdk';
import { createRecoveryCommitmentPayload, getUnifiedAddress, getUnifiedPublicKey, sign } from '@frequency-chain/ethereum-utils';
import { ContactType, generateRecoverySecret, getRecoveryCommitment } from '@frequency-chain/recovery-sdk';
import assert from 'assert';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper, RecoveryCommitmentPayload } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import {
  createAndFundKeypairs,
  signPayloadSr25519,
  Sr25519Signature,
  DOLLARS,
  generateRecoveryCommitmentPayload,
  getBlockNumber,
  assertEvent,
  getEthereumKeyPairFromUnifiedAddress,
  createKeys,
} from '../scaffolding/helpers';
import { u64 } from '@polkadot/types';
import { Codec } from '@polkadot/types/types';
import { u8aToHex } from '@polkadot/util';

let fundingSource: KeyringPair;

describe('Recovery Commitment Testing', function () {
  let keys: KeyringPair;
  let msaId: u64;
  let secondaryKey: KeyringPair;
  let payload: RecoveryCommitmentPayload;
  let ownerSig: Sr25519Signature;
  let badSig: Sr25519Signature;
  const recoverySecret = generateRecoverySecret();
  const testEmail = 'test@example.com';
  // REMOVE: Use the secret from ethereum-utils for consistency in tests
  const recoveryCommitmentHex = getRecoveryCommitment(recoverySecret, ContactType.EMAIL, testEmail);
  // const recoveryCommitmentHex = "0x5c06ce60a2a1245fabdd1c11bfbf55246836d2c6fefac2c634837e3359d0dbb3";
  // const recoveryCommitment = new Uint8Array(Buffer.from(recoveryCommitmentHex.slice(2), 'hex'));
  console.log('recoveryCommitment:', recoveryCommitmentHex);
  let recoveryCommitmentData: RecoveryCommitmentPayload;
  let recoveryCommitmentPayload: Codec;

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);

    // Setup an MSA with one key and a secondary funded key
    [keys, secondaryKey] = await createAndFundKeypairs(fundingSource, ['keys', 'secondaryKey'], 2n * DOLLARS);
    const { target } = await ExtrinsicHelper.createMsa(keys).signAndSend();
    assert.notEqual(target?.data.msaId, undefined, 'MSA Id not in expected event');
    msaId = target!.data.msaId;
    // Pass the explicit expiration to the payload generator to ensure consistency
    recoveryCommitmentData = {
      discriminant: 'RecoveryCommitmentPayload',
      recoveryCommitment: recoveryCommitmentHex,
    };
    payload = await generateRecoveryCommitmentPayload({ ...recoveryCommitmentData });
    recoveryCommitmentPayload = ExtrinsicHelper.api.registry.createType(
      'PalletMsaRecoveryCommitmentPayload',
      payload
    );
  });

  describe('addRecoveryCommitment', function () {
    it('should successfully add an Sr25519 signed recovery commitment', async function () {
      ownerSig = signPayloadSr25519(keys, recoveryCommitmentPayload);

      const addRecoveryCommitmentOp = ExtrinsicHelper.addRecoveryCommitment(keys, ownerSig, payload);

      const { eventMap } = await addRecoveryCommitmentOp.fundAndSend(fundingSource, false);
      assertEvent(eventMap, 'system.ExtrinsicSuccess');
      assertEvent(eventMap, 'msa.RecoveryCommitmentAdded');
    });

    it('should successfully add an Ethereum signed recovery commitment', async function () {
      // SET UP
      const ethereumKeyringPair = await createKeys('Ethereum', 'ethereum');
      const unifiedAddress = getUnifiedAddress(ethereumKeyringPair);
      console.log('unifiedAddress:', unifiedAddress);
      const ethereumKeyPair = getEthereumKeyPairFromUnifiedAddress(unifiedAddress);
      const ethereumSecretKey = u8aToHex( getEthereumKeyPairFromUnifiedAddress(unifiedAddress).secretKey);
      console.log('ethereumKeyPair:', ethereumKeyPair);
      console.log('ethereumSecretKey:', ethereumSecretKey);

      if (!ethereumKeyPair || !ethereumKeyPair.secretKey) {
        throw new Error('Ethereum keypair or secret key is not defined');
      }

      // Fund the Ethereum key and create an MSA for it
      await ExtrinsicHelper.transferFunds(fundingSource, ethereumKeyringPair, 2n * DOLLARS).signAndSend();
      const { target: msaCreationTarget } = await ExtrinsicHelper.createMsa(ethereumKeyringPair).signAndSend();
      assert.notEqual(msaCreationTarget?.data.msaId, undefined, 'MSA Id not in expected event');
      console.log('MSA Id:', msaCreationTarget!.data.msaId);


      const eip712RecoveryCommitmentPayload = createRecoveryCommitmentPayload(
        recoveryCommitmentHex,
        payload.expiration
      );
      console.log('recoveryCommitmentPayload:', eip712RecoveryCommitmentPayload);
      const ecdsaSignature = await sign(ethereumSecretKey, eip712RecoveryCommitmentPayload, 'Dev');
      console.log('ecdsaSignature:', ecdsaSignature);

      const addRecoveryCommitmentOp = ExtrinsicHelper.addRecoveryCommitment(ethereumKeyringPair, ecdsaSignature, payload);

      // ACT
      const { eventMap } = await addRecoveryCommitmentOp.fundAndSend(fundingSource, false);

      // ASSERT
      assertEvent(eventMap, 'system.ExtrinsicSuccess');
      assertEvent(eventMap, 'msa.RecoveryCommitmentAdded');
    });

    it('should fail to add a recovery commitment when signature is re-used', async function () {
      const addRecoveryCommitmentOp = ExtrinsicHelper.addRecoveryCommitment(keys, ownerSig, payload);
      await assert.rejects(addRecoveryCommitmentOp.fundAndSend(fundingSource, false), { name: 'SignatureAlreadySubmitted' });
    });

    it('should fail to add a recovery commitment when signature is not owner', async function () {
      badSig = signPayloadSr25519(secondaryKey, recoveryCommitmentPayload);

      const addRecoveryCommitmentOp = ExtrinsicHelper.addRecoveryCommitment(keys, badSig, payload);
      await assert.rejects(addRecoveryCommitmentOp.fundAndSend(fundingSource, false), { name: 'InvalidSignature' });
    });

  });
});
