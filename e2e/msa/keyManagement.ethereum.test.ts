import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  createKeys,
  createAndFundKeypair,
  generateAddKeyPayload,
  CENTS,
  signPayload,
  MultiSignatureType,
  signEip712AddKeyData,
  getKeyPairFromName,
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { AddKeyData, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { u64 } from '@polkadot/types';
import { Codec } from '@polkadot/types/types';
import { getFundingSource } from '../scaffolding/funding';
import { getUnifiedPublicKey } from '../scaffolding/ethereum';

const maxU64 = 18_446_744_073_709_551_615n;
const fundingSource = getFundingSource(import.meta.url);

describe('MSA Key management Ethereum', function () {
  describe('addPublicKeyToMsa Ethereum', function () {
    let keysName: string;
    let keys: KeyringPair;
    let msaId: u64;
    let secondaryKeyName: string;
    let secondaryKey: KeyringPair;
    const defaultPayload: AddKeyData = {};
    let payload: AddKeyData;
    let ownerSig: MultiSignatureType;
    let newSig: MultiSignatureType;
    let badSig: MultiSignatureType;
    let addKeyData: Codec;

    before(async function () {
      // Setup an MSA with one key and a secondary funded key
      keysName = 'key-management-1';
      keys = await createAndFundKeypair(fundingSource, 5n * CENTS, keysName, undefined, 'ethereum');
      const { target } = await ExtrinsicHelper.createMsa(keys).signAndSend();
      assert.notEqual(target?.data.msaId, undefined, 'MSA Id not in expected event');
      msaId = target!.data.msaId;

      secondaryKeyName = 'key-management-2';
      secondaryKey = await createAndFundKeypair(fundingSource, 5n * CENTS, secondaryKeyName, undefined, 'ethereum');

      // Default payload making it easier to test `addPublicKeyToMsa`
      defaultPayload.msaId = msaId;
      defaultPayload.newPublicKey = getUnifiedPublicKey(secondaryKey);
    });

    beforeEach(async function () {
      payload = await generateAddKeyPayload(defaultPayload);
    });

    it('should fail to add public key if origin is not one of the signers of the payload (MsaOwnershipInvalidSignature) for a Ethereum key', async function () {
      const badKeys: KeyringPair = createKeys();
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', payload);
      newSig = signPayload(secondaryKey, addKeyData);
      badSig = signPayload(badKeys, addKeyData);
      const op = ExtrinsicHelper.addPublicKeyToMsa(keys, badSig, newSig, payload);
      await assert.rejects(op.fundAndSend(fundingSource), {
        name: 'MsaOwnershipInvalidSignature',
      });
    });

    it('should fail to add public key if origin does not own MSA (NotMsaOwner) for a Ethereum key', async function () {
      const newPayload = await generateAddKeyPayload({
        ...defaultPayload,
        msaId: new u64(ExtrinsicHelper.api.registry, maxU64),
      });
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', newPayload);
      ownerSig = signPayload(keys, addKeyData);
      newSig = signPayload(secondaryKey, addKeyData);
      const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, newPayload);
      await assert.rejects(op.fundAndSend(fundingSource), {
        name: 'NotMsaOwner',
      });
    });

    it('should successfully add a new public key to an existing MSA & disallow duplicate signed payload submission (SignatureAlreadySubmitted) for a Ethereum key', async function () {
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', payload);

      ownerSig = signPayload(keys, addKeyData);
      newSig = signPayload(secondaryKey, addKeyData);
      const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);

      const { target: publicKeyEvents } = await addPublicKeyOp.fundAndSend(fundingSource);

      assert.notEqual(publicKeyEvents, undefined, 'should have added public key');

      await assert.rejects(
        addPublicKeyOp.fundAndSend(fundingSource),
        'should reject sending the same signed payload twice'
      );
    });

    it('should fail if attempting to add the same key more than once (KeyAlreadyRegistered) for a Ethereum key', async function () {
      const addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', payload);

      const ownerSig = signPayload(keys, addKeyData);
      const newSig = signPayload(secondaryKey, addKeyData);
      const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);

      await assert.rejects(addPublicKeyOp.fundAndSend(fundingSource), {
        name: 'KeyAlreadyRegistered',
      });
    });

    it('should allow new keypair to act for/on MSA for a Ethereum key', async function () {
      const thirdKey = createKeys();
      const newPayload = await generateAddKeyPayload({
        ...defaultPayload,
        newPublicKey: getUnifiedPublicKey(thirdKey),
      });
      addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', newPayload);
      ownerSig = signPayload(secondaryKey, addKeyData);
      newSig = signPayload(thirdKey, addKeyData);
      const op = ExtrinsicHelper.addPublicKeyToMsa(secondaryKey, ownerSig, newSig, newPayload);
      const { target: event } = await op.fundAndSend(fundingSource);
      assert.notEqual(event, undefined, 'should have added public key');

      // Cleanup
      await assert.doesNotReject(ExtrinsicHelper.deletePublicKey(keys, getUnifiedPublicKey(thirdKey)).signAndSend());
    });

    it('should allow using eip-712 signatures to add a new key', async function () {
      const thirdKeyName = 'eth-key-3';
      const ethereumKeys3 = createKeys(thirdKeyName, 'ethereum');
      const newPayload = await generateAddKeyPayload({
        ...defaultPayload,
        newPublicKey: getUnifiedPublicKey(ethereumKeys3),
      });

      ownerSig = await signEip712AddKeyData(getKeyPairFromName(secondaryKeyName), newPayload);
      newSig = await signEip712AddKeyData(getKeyPairFromName(thirdKeyName), newPayload);
      const op = ExtrinsicHelper.addPublicKeyToMsa(secondaryKey, ownerSig, newSig, newPayload);
      const { target: event } = await op.fundAndSend(fundingSource);
      assert.notEqual(event, undefined, 'should have added public key via eip-712');

      // Cleanup
      await assert.doesNotReject(
        ExtrinsicHelper.deletePublicKey(keys, getUnifiedPublicKey(ethereumKeys3)).signAndSend()
      );
    });
  });
});
