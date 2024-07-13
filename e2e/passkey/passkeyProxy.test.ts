import '@frequency-chain/api-augment';
import assert from 'assert';
import { base64UrlToUint8Array, createAndFundKeypair, getNonce } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import { SubmittableExtrinsic } from '@polkadot/api/types';
import { ISubmittableResult } from '@polkadot/types/types';
import cryptokit from 'cryptokit';

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
      const nonce = await getNonce(fundedKeys);

      const remarksCalls = ExtrinsicHelper.api.tx.system.remark('passkey-test');
      const { P256Keys, passKeyPublicKey, passkeySignature } = await createPassKeyAndSignAccount(
        Buffer.from(accountPKey)
      );
      const accountSignature = fundedKeys.sign(Buffer.from(passKeyPublicKey));
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, remarksCalls);
      const passkeyPayload = await createPasskeyPayload(P256Keys, Buffer.from(passKeyPublicKey), passkeyCall, false);

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });

    it('should fail to transfer balance due to bad account ownership proof', async function () {
      const accountPKey = fundedKeys.publicKey;
      const nonce = await getNonce(fundedKeys);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferAllowDeath(receiverKeys.publicKey, 1000000);
      const { P256Keys, passKeyPublicKey, passkeySignature } = await createPassKeyAndSignAccount(
        Buffer.from(accountPKey)
      );
      const accountSignature = fundedKeys.sign('badPasskeyPublicKey');
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(P256Keys, Buffer.from(passKeyPublicKey), passkeyCall, false);

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });

    it('should fail to transfer balance due to bad passkey signature', async function () {
      const accountPKey = fundedKeys.publicKey;
      const nonce = await getNonce(fundedKeys);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferAllowDeath(receiverKeys.publicKey, 1000000);
      const { P256Keys, passKeyPublicKey, passkeySignature } = await createPassKeyAndSignAccount(
        Buffer.from(accountPKey)
      );
      const accountSignature = fundedKeys.sign(Buffer.from(passKeyPublicKey));
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(P256Keys, Buffer.from(passKeyPublicKey), passkeyCall, true);

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });
  });
});

async function createPassKeyAndSignAccount(accountPKey: Buffer) {
  const password = 'frequency passkey pallet tests';
  const P256Keys = await cryptokit.P256.generateKeys(password);
  const P256PublicKey = await cryptokit.P256.loadPublicKey(P256Keys.publicKey);
  const P256PrivateKey = await cryptokit.P256.loadPrivateKey(P256Keys.privateKey, password);
  const passKeyPublicKey = await cryptokit.P256.formatPublicKeyToRaw(P256PublicKey);
  const passkeySignature = await cryptokit.P256.sign(accountPKey, P256PrivateKey);
  return { P256Keys, passKeyPublicKey, passkeySignature };
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
  P256Keys: { publicKey: string; privateKey: string },
  passKeyPublicKey: Buffer,
  passkeyCallPayload: any = {},
  bad: boolean = false
) {
  const password = 'frequency passkey pallet tests';
  const signerP256PrivateKey = await cryptokit.P256.loadPrivateKey(P256Keys.privateKey, password);
  const authenticatorDataRaw = 'WJ8JTNbivTWn-433ubs148A7EgWowi4SAcYBjLWfo1EdAAAAAA';
  const replacedClientDataRaw =
    'eyJ0eXBlIjoid2ViYXV0aG4uZ2V0IiwiY2hhbGxlbmdlIjoiI3JwbGMjIiwib3JpZ2luIjoiaHR0cHM6Ly9wYXNza2V5LmFtcGxpY2EuaW86ODA4MCIsImNyb3NzT3JpZ2luIjpmYWxzZSwiYWxnIjoiSFMyNTYifQ';
  let clientData = Buffer.from(replacedClientDataRaw).toString('base64url');
  let authenticatorData = Buffer.from(authenticatorDataRaw).toString('base64url');

  if (bad) {
    authenticatorData = Buffer.from('badAuthenticatorData').toString('base64url');
    clientData = Buffer.from('badClientData').toString('base64url');
  }
  const passkeyCallType = ExtrinsicHelper.api.createType('PalletPasskeyPasskeyCall', passkeyCallPayload);

  const passKeySignature = await cryptokit.P256.sign(Buffer.from(passkeyCallType.toU8a()), signerP256PrivateKey);
  const passkeyPayload = {
    passkeyPublicKey: Array.from(passKeyPublicKey),
    verifiablePasskeySignature: {
      signature: Array.from(base64UrlToUint8Array(passKeySignature)),
      authenticatorData: Array.from(base64UrlToUint8Array(authenticatorData)),
      clientDataJson: Array.from(base64UrlToUint8Array(clientData)),
    },
    passkeyCall: passkeyCallType,
  };

  return passkeyPayload;
}
