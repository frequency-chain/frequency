import '@frequency-chain/api-augment';
import '@frequency-chain/recovery-sdk';
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
  assertEvent,
  createKeys,
  getEthereumKeyPairFromUnifiedAddress,
} from '../scaffolding/helpers';
import { u64 } from '@polkadot/types';
import { Codec } from '@polkadot/types/types';
import { createRecoveryCommitmentPayload, getUnifiedAddress, HexString, sign } from '@frequency-chain/ethereum-utils';
import { u8aToHex } from '@polkadot/util';

let fundingSource: KeyringPair;

describe('Recovery Commitment Testing', function () {
  // SET UP: Common variables used across tests
  let keys: KeyringPair;
  let msaId: u64;
  let secondaryKey: KeyringPair;
  let payload: RecoveryCommitmentPayload;
  let ownerSig: Sr25519Signature;
  let badSig: Sr25519Signature;
  const recoverySecret = generateRecoverySecret();
  const testEmail = 'test@example.com';
  const recoveryCommitment = getRecoveryCommitment(recoverySecret, ContactType.EMAIL, testEmail);
  let recoveryCommitmentData: RecoveryCommitmentPayload;
  let codecForPayload: Codec;

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);

    // Setup an MSA with one key and a secondary funded key
    [keys, secondaryKey] = await createAndFundKeypairs(fundingSource, ['keys', 'secondaryKey'], 2n * DOLLARS);
    const { target } = await ExtrinsicHelper.createMsa(keys).signAndSend();
    assert.notEqual(target?.data.msaId, undefined, 'MSA Id not in expected event');
    msaId = target!.data.msaId;

    recoveryCommitmentData = {
      discriminant: 'RecoveryCommitmentPayload',
      recoveryCommitment: recoveryCommitment,
    };
    payload = await generateRecoveryCommitmentPayload({ ...recoveryCommitmentData });
    codecForPayload = ExtrinsicHelper.api.registry.createType('PalletMsaRecoveryCommitmentPayload', payload);
  });

  describe('addRecoveryCommitment', function () {
    it('should successfully add an Sr25519 signed recovery commitment', async function () {
      ownerSig = signPayloadSr25519(keys, codecForPayload);

      const addRecoveryCommitmentOp = ExtrinsicHelper.addRecoveryCommitment(keys, ownerSig, payload);

      const { eventMap } = await addRecoveryCommitmentOp.fundAndSend(fundingSource, false);
      assertEvent(eventMap, 'system.ExtrinsicSuccess');
      assertEvent(eventMap, 'msa.RecoveryCommitmentAdded');
    });

    it('should fail to add a recovery commitment when signature is re-used', async function () {
      const addRecoveryCommitmentOp = ExtrinsicHelper.addRecoveryCommitment(keys, ownerSig, payload);
      await assert.rejects(addRecoveryCommitmentOp.fundAndSend(fundingSource, false), {
        name: 'SignatureAlreadySubmitted',
      });
    });

    it('should fail to add a recovery commitment when signature is not owner', async function () {
      badSig = signPayloadSr25519(secondaryKey, codecForPayload);

      const addRecoveryCommitmentOp = ExtrinsicHelper.addRecoveryCommitment(keys, badSig, payload);
      await assert.rejects(addRecoveryCommitmentOp.fundAndSend(fundingSource, false), { name: 'InvalidSignature' });
    });

    it('should successfully add an Ethereum signed recovery commitment', async function () {
      // SET UP
      const ethereumKeyringPair = createKeys('Ethereum', 'ethereum');
      const unifiedAddress = getUnifiedAddress(ethereumKeyringPair);
      const ethereumKeyPair = getEthereumKeyPairFromUnifiedAddress(unifiedAddress);
      const ethereumSecretKey = u8aToHex(getEthereumKeyPairFromUnifiedAddress(unifiedAddress).secretKey);

      if (!ethereumKeyPair || !ethereumKeyPair.secretKey) {
        throw new Error('Ethereum keypair or secret key is not defined');
      }

      // Fund the Ethereum key and create an MSA for it
      await ExtrinsicHelper.transferFunds(fundingSource, ethereumKeyringPair, 2n * DOLLARS).signAndSend();
      const { target: msaCreationTarget } = await ExtrinsicHelper.createMsa(ethereumKeyringPair).signAndSend();
      assert.notEqual(msaCreationTarget?.data.msaId, undefined, 'MSA Id not in expected event');

      const eip712RecoveryCommitmentPayload = createRecoveryCommitmentPayload(
        recoveryCommitment as HexString,
        payload.expiration
      );
      const ecdsaSignature = await sign(ethereumSecretKey, eip712RecoveryCommitmentPayload, 'Dev');
      const addRecoveryCommitmentOp = ExtrinsicHelper.addRecoveryCommitment(
        ethereumKeyringPair,
        ecdsaSignature,
        payload
      );

      // ACT
      const { eventMap } = await addRecoveryCommitmentOp.fundAndSend(fundingSource, false);

      // ASSERT
      assertEvent(eventMap, 'system.ExtrinsicSuccess');
      assertEvent(eventMap, 'msa.RecoveryCommitmentAdded');
    });
  });
});
