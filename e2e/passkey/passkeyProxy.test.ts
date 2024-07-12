import '@frequency-chain/api-augment';
import assert from 'assert';
import { base64UrlToUint8Array, createAndFundKeypair, getNonce } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import { SubmittableExtrinsic } from '@polkadot/api/types';
import { ISubmittableResult } from '@polkadot/types/types';
import cryptokit from "cryptokit";

const fundingSource = getFundingSource('passkey-proxy');

describe('Passkey Pallet Tests', function () {
  let fundedKeys: KeyringPair;
  let receiverKeys: KeyringPair;

  before(async function () {
    fundedKeys = await createAndFundKeypair(fundingSource, 50_000_000n);
    receiverKeys = await createAndFundKeypair(fundingSource, 0n);
  });

  describe('proxy', function () {
    it('should fail due to unsupported call', async function () {
      const accountPKey = fundedKeys.publicKey;
      const passkeyPublicKey = new Uint8Array(33);
      const nonce = await getNonce(fundedKeys);
      const accountSignature = fundedKeys.sign(passkeyPublicKey);

      // base64URL encoded strings
      const badPassKeySignatureBase64URL = Buffer.from('badPassKeySignature').toString('base64url');
      const authenticatorDataBase64URL = Buffer.from('authenticatorData').toString('base64url');
      const clientDataJsonBase64URL = Buffer.from('clientDataJson').toString('base64url');

      const remarksCalls = ExtrinsicHelper.api.tx.system.remark('passkey-test');
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, remarksCalls);
      const passkeyPayload = await createPasskeyPayload(
        passkeyPublicKey,
        badPassKeySignatureBase64URL,
        authenticatorDataBase64URL,
        clientDataJsonBase64URL,
        passkeyCall
      );

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });

    it('should fail to transfer balance due to bad account ownership proof', async function () {
      const accountPKey = fundedKeys.publicKey;
      const passkeyPublicKey = new Uint8Array(33);
      const nonce = await getNonce(fundedKeys);
      const accountSignature = fundedKeys.sign('badPasskeyPublicKey');

      // base64URL encoded strings
      const badPassKeySignatureBase64URL = Buffer.from('badPassKeySignature').toString('base64url');
      const authenticatorDataBase64URL = Buffer.from('authenticatorData').toString('base64url');
      const clientDataJsonBase64URL = Buffer.from('clientDataJson').toString('base64url');

      const transferCalls = ExtrinsicHelper.api.tx.balances.transferAllowDeath(receiverKeys.publicKey, 1000000);
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(
        passkeyPublicKey,
        badPassKeySignatureBase64URL,
        authenticatorDataBase64URL,
        clientDataJsonBase64URL,
        passkeyCall
      );

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });

    it('should fail to transfer balance due to bad passkey signature', async function () {
      const accountPKey = fundedKeys.publicKey;
      const passkeyPublicKey = new Uint8Array(33);
      const nonce = await getNonce(fundedKeys);
      const accountSignature = fundedKeys.sign(passkeyPublicKey);

      // base64URL encoded strings
      const badPassKeySignatureBase64URL = Buffer.from('badPassKeySignature').toString('base64url');
      const authenticatorDataBase64URL = Buffer.from('authenticatorData').toString('base64url');
      const clientDataJsonBase64URL = Buffer.from('clientDataJson').toString('base64url');

      const transferCalls = ExtrinsicHelper.api.tx.balances.transferAllowDeath(receiverKeys.publicKey, 1000000);
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(
        passkeyPublicKey,
        badPassKeySignatureBase64URL,
        authenticatorDataBase64URL,
        clientDataJsonBase64URL,
        passkeyCall
      );

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });
  });
});

async function createAndSignPasskeyPayload(payloadToSign: Buffer) {
  const P256Keys = await cryptokit.P256.generateKeys();
  const P256PublicKey = await cryptokit.P256.loadPublicKey(P256Keys.publicKey);
  const P256PrivateKey = await cryptokit.P256.loadPrivateKey(P256Keys.privateKey);
  const passkeyPublicKeyString = Buffer.from(await cryptokit.P256.formatPublicKeyToRaw(P256PublicKey)).toString('base64url');
  const signature = await cryptokit.P256.sign(payloadToSign, P256PrivateKey);
  return { passkeyPublicKeyString, signature };
}

async function createPassKeyCall(
  accountPKey: Uint8Array,
  nonce: number,
  accountSignature: Uint8Array,
  call: SubmittableExtrinsic<'rxjs', ISubmittableResult>
) {
  const passkeyCall = {
    accountId: accountPKey,
    accountNonce: nonce,
    accountOwnershipProof: {
      Sr25519: accountSignature,
    },
    call: call,
  };

  return passkeyCall;
}

async function createPasskeyPayload(
  passkeyPublicKey: Uint8Array,
  passkeySignature: string,
  authenticatorData: string,
  clientDataJson: string,
  passkeyCallPayload: { 
    call: SubmittableExtrinsic<'rxjs', ISubmittableResult>,
    accountId: Uint8Array,
    accountNonce: number,
    accountOwnershipProof: { Sr25519: Uint8Array }
  }
) {

  const passkeyCallType = ExtrinsicHelper.api.createType('PalletPasskeyPasskeyCall', passkeyCallPayload);
  const passkeyPayload = {
    passkeyPublicKey: Array.from(passkeyPublicKey),
    verifiablePasskeySignature: {
      signature: Array.from(base64UrlToUint8Array(passkeySignature)),
      authenticatorData: Array.from(base64UrlToUint8Array(authenticatorData)),
      clientDataJson: Array.from(base64UrlToUint8Array(clientDataJson)),
    },
    passkeyCall: passkeyCallType,
  };

  return passkeyPayload;
}
