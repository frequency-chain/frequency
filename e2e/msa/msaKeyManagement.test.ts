import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  createKeys,
  createAndFundKeypair,
  createAndFundKeypairs,
  signPayloadSr25519,
  Sr25519Signature,
  generateAddKeyPayload,
  createProviderKeysAndId,
  DOLLARS,
  assertExtrinsicSucceededAndFeesPaid,
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { AddKeyData, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { u64 } from '@polkadot/types';
import { Codec } from '@polkadot/types/types';
import { getFundingSource } from '../scaffolding/funding';
import { getUnifiedPublicKey } from '@frequency-chain/ethereum-utils';
import { getFundingSource, getRootFundingSource, getSudo } from '../scaffolding/funding';
import { getUnifiedAddress, getUnifiedPublicKey } from '../scaffolding/ethereum';
import { BigInt } from '@polkadot/x-bigint';

const maxU64 = 18_446_744_073_709_551_615n;
const fundingSource = getFundingSource(import.meta.url);

describe('MSA Key management', function () {
  describe('addPublicKeyToMsa', function () {
    let keys: KeyringPair;
    let msaId: u64;
    let secondaryKey: KeyringPair;
    const defaultPayload: AddKeyData = {};
    let payload: AddKeyData;
    let ownerSig: Sr25519Signature;
    let newSig: Sr25519Signature;
    let badSig: Sr25519Signature;
    let addKeyData: Codec;

    before(async function () {
      // Setup an MSA with one key and a secondary funded key
      [keys, secondaryKey] = await createAndFundKeypairs(fundingSource, ['keys', 'secondaryKey'], 2n * DOLLARS);
      const { target } = await ExtrinsicHelper.createMsa(keys).signAndSend();
      assert.notEqual(target?.data.msaId, undefined, 'MSA Id not in expected event');
      msaId = target!.data.msaId;

      // Default payload making it easier to test `addPublicKeyToMsa`
      defaultPayload.msaId = msaId;
      defaultPayload.newPublicKey = getUnifiedPublicKey(secondaryKey);
    });

    beforeEach(async function () {
      payload = await generateAddKeyPayload(defaultPayload);
    });

    it('should fail to add public key if origin is not one of the signers of the payload (MsaOwnershipInvalidSignature)', async function () {
      const badKeys: KeyringPair = createKeys();
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', payload);
      newSig = signPayloadSr25519(secondaryKey, addKeyData);
      badSig = signPayloadSr25519(badKeys, addKeyData);
      const op = ExtrinsicHelper.addPublicKeyToMsa(keys, badSig, newSig, payload);
      await assert.rejects(op.fundAndSend(fundingSource), {
        name: 'MsaOwnershipInvalidSignature',
      });
    });

    it('should fail to add public key if new keypair is not one of the signers of the payload (NewKeyOwnershipInvalidSignature)', async function () {
      const badKeys: KeyringPair = createKeys();
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', payload);
      ownerSig = signPayloadSr25519(keys, addKeyData);
      badSig = signPayloadSr25519(badKeys, addKeyData);
      const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, badSig, payload);
      await assert.rejects(op.fundAndSend(fundingSource), {
        name: 'NewKeyOwnershipInvalidSignature',
      });
    });

    it('should fail to add public key if origin does not have an MSA (NoKeyExists)', async function () {
      const newOriginKeys = await createAndFundKeypair(fundingSource, 50_000_000n);
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', payload);
      ownerSig = signPayloadSr25519(newOriginKeys, addKeyData);
      newSig = signPayloadSr25519(secondaryKey, addKeyData);
      const op = ExtrinsicHelper.addPublicKeyToMsa(newOriginKeys, ownerSig, newSig, payload);
      await assert.rejects(op.fundAndSend(fundingSource), {
        name: 'NoKeyExists',
      });
    });

    it('should fail to add public key if origin does not own MSA (NotMsaOwner)', async function () {
      const newPayload = await generateAddKeyPayload({
        ...defaultPayload,
        msaId: new u64(ExtrinsicHelper.api.registry, maxU64),
      });
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', newPayload);
      ownerSig = signPayloadSr25519(keys, addKeyData);
      newSig = signPayloadSr25519(secondaryKey, addKeyData);
      const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, newPayload);
      await assert.rejects(op.fundAndSend(fundingSource), {
        name: 'NotMsaOwner',
      });
    });

    it('should fail if expiration has passed (ProofHasExpired)', async function () {
      const newPayload = await generateAddKeyPayload({
        ...defaultPayload,
        expiration: (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber(),
      });
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', newPayload);
      ownerSig = signPayloadSr25519(keys, addKeyData);
      newSig = signPayloadSr25519(secondaryKey, addKeyData);
      const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, newPayload);
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
      newSig = signPayloadSr25519(secondaryKey, addKeyData);
      const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, newPayload);
      await assert.rejects(op.fundAndSend(fundingSource), {
        name: 'ProofNotYetValid',
      });
    });

    it('should successfully add a new public key to an existing MSA & disallow duplicate signed payload submission (SignatureAlreadySubmitted)', async function () {
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', payload);

      ownerSig = signPayloadSr25519(keys, addKeyData);
      newSig = signPayloadSr25519(secondaryKey, addKeyData);
      const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);

      const { target: publicKeyEvents, eventMap: chainEvents } = await addPublicKeyOp.fundAndSend(fundingSource);

      assert.notEqual(publicKeyEvents, undefined, 'should have added public key');
      assertExtrinsicSucceededAndFeesPaid(chainEvents);
      assert.notEqual(chainEvents['transactionPayment.TransactionFeePaid'], undefined);

      await assert.rejects(
        addPublicKeyOp.fundAndSend(fundingSource),
        'should reject sending the same signed payload twice'
      );
    });

    it('should fail if attempting to add the same key more than once (KeyAlreadyRegistered)', async function () {
      const addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', payload);

      const ownerSig = signPayloadSr25519(keys, addKeyData);
      const newSig = signPayloadSr25519(secondaryKey, addKeyData);
      const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);

      await assert.rejects(addPublicKeyOp.fundAndSend(fundingSource), {
        name: 'KeyAlreadyRegistered',
      });
    });

    it('should allow new keypair to act for/on MSA', async function () {
      const thirdKey = createKeys();
      const newPayload = await generateAddKeyPayload({
        ...defaultPayload,
        newPublicKey: getUnifiedPublicKey(thirdKey),
      });
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', newPayload);
      ownerSig = signPayloadSr25519(secondaryKey, addKeyData);
      newSig = signPayloadSr25519(thirdKey, addKeyData);
      const op = ExtrinsicHelper.addPublicKeyToMsa(secondaryKey, ownerSig, newSig, newPayload);
      // Need to wait for finalization to use the key
      const { target: event } = await op.fundAndSend(fundingSource, false);
      assert.notEqual(event, undefined, 'should have added public key');

      // Cleanup
      await assert.doesNotReject(ExtrinsicHelper.deletePublicKey(keys, getUnifiedPublicKey(thirdKey)).signAndSend());
    });
  });

  describe('provider msa', function () {
    it('should disallow retiring MSA belonging to a provider', async function () {
      const [providerKeys] = await createProviderKeysAndId(fundingSource);
      // Make sure we are finalized before trying to retire
      await ExtrinsicHelper.waitForFinalization();
      const retireOp = ExtrinsicHelper.retireMsa(providerKeys);
      await assert.rejects(retireOp.signAndSend('current'), {
        name: 'RpcError',
        message: /Custom error: 2/,
      });
    });
  });
});
