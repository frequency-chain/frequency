import '@frequency-chain/api-augment';
import '@frequency-chain/recovery-sdk';
import assert from 'assert';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper, RecoveryCommitmentData } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import {
  createAndFundKeypairs,
  signPayloadSr25519,
  Sr25519Signature,
  DOLLARS,
  generateRecoveryCommitmentPayload,
  getBlockNumber,
  assertEvent,
} from '../scaffolding/helpers';
import { u64 } from '@polkadot/types';
import { ContactType, generateRecoverySecret, getRecoveryCommitment } from '@frequency-chain/recovery-sdk';
import { getUnifiedPublicKey } from '@frequency-chain/ethereum-utils';
import { Codec } from '@polkadot/types/types';

let fundingSource: KeyringPair;

describe('Recovery Commitment Testing', function () {
  let keys: KeyringPair;
  let msaId: u64;
  let secondaryKey: KeyringPair;
  let payload: RecoveryCommitmentData;
  let ownerSig: Sr25519Signature;
  let badSig: Sr25519Signature;
  const recoverySecret = generateRecoverySecret();
  const testEmail = 'test@example.com';
  const recoveryCommitmentHex = getRecoveryCommitment(recoverySecret, ContactType.EMAIL, testEmail);
  const recoveryCommitment = new Uint8Array(Buffer.from(recoveryCommitmentHex.slice(2), 'hex'));
  // const expiration = (await getBlockNumber()) + 10;
  const recoveryCommitmentData: RecoveryCommitmentData = {
    recoveryCommitment,
  };
  let recoveryCommitmentPayload: Codec;

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);

    // Setup an MSA with one key and a secondary funded key
    [keys, secondaryKey] = await createAndFundKeypairs(fundingSource, ['keys', 'secondaryKey'], 2n * DOLLARS);
    const { target } = await ExtrinsicHelper.createMsa(keys).signAndSend();
    assert.notEqual(target?.data.msaId, undefined, 'MSA Id not in expected event');
    msaId = target!.data.msaId;
  });

  describe('addRecoveryCommitment', function () {
    it('should successfully add an Sr25519 signed recovery commitment', async function () {
      // Pass the explicit expiration to the payload generator to ensure consistency
      const payload = await generateRecoveryCommitmentPayload({ ...recoveryCommitmentData });
      recoveryCommitmentPayload = ExtrinsicHelper.api.registry.createType(
        'PalletMsaRecoveryCommitmentPayload',
        payload
      );
      ownerSig = signPayloadSr25519(keys, recoveryCommitmentPayload);

      const addRecoveryCommitmentOp = ExtrinsicHelper.addRecoveryCommitment(keys, ownerSig, payload);

      const { eventMap } = await addRecoveryCommitmentOp.fundAndSend(fundingSource, false);
      assertEvent(eventMap, 'system.ExtrinsicSuccess');
      assertEvent(eventMap, 'msa.RecoveryCommitmentAdded');
    });

    it('should fail to add a recovery commitment when signature is re-used', async function () {
      // Pass the explicit expiration to the payload generator to ensure consistency
      const payload = await generateRecoveryCommitmentPayload({ ...recoveryCommitmentData });

      const addRecoveryCommitmentOp = ExtrinsicHelper.addRecoveryCommitment(keys, ownerSig, payload);
      await assert.rejects(addRecoveryCommitmentOp.fundAndSend(fundingSource, false), { name: 'InvalidSignature' });
    });

    it('should fail to add a recovery commitment when signature is not owner', async function () {
      // Pass the explicit expiration to the payload generator to ensure consistency
      const payload = await generateRecoveryCommitmentPayload({ ...recoveryCommitmentData });
      badSig = signPayloadSr25519(secondaryKey, recoveryCommitmentPayload);

      const addRecoveryCommitmentOp = ExtrinsicHelper.addRecoveryCommitment(keys, badSig, payload);
      await assert.rejects(addRecoveryCommitmentOp.fundAndSend(fundingSource, false), { name: 'InvalidSignature' });
    });
  });
});
