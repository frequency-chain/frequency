import '@frequency-chain/api-augment';
import assert from 'assert';
import { base64UrlToUint8Array, createAndFundKeypair, getNonce } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import { SubmittableExtrinsic } from '@polkadot/api/types';
import { ISubmittableResult } from '@polkadot/types/types';
import { secp256k1 } from '@noble/curves/secp256k1'; // ESM and Common.js

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
      const { passKeyPrivateKey, passKeyPublicKey, passkeySignature } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeys.sign(Buffer.from(passKeyPublicKey));
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, remarksCalls);
      const passkeyPayload = await createPasskeyPayload(
        passKeyPrivateKey,
        Buffer.from(passKeyPublicKey),
        passkeyCall,
        false
      );

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });

    it('should fail to transfer balance due to bad account ownership proof', async function () {
      const accountPKey = fundedKeys.publicKey;
      const nonce = await getNonce(fundedKeys);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferAllowDeath(receiverKeys.publicKey, 1000000);
      const { passKeyPrivateKey, passKeyPublicKey, passkeySignature } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeys.sign('badPasskeyPublicKey');
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(
        passKeyPrivateKey,
        Buffer.from(passKeyPublicKey),
        passkeyCall,
        false
      );

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });

    it('should fail to transfer balance due to bad passkey signature', async function () {
      const accountPKey = fundedKeys.publicKey;
      const nonce = await getNonce(fundedKeys);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferAllowDeath(receiverKeys.publicKey, 1000000);
      const { passKeyPrivateKey, passKeyPublicKey, passkeySignature } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeys.sign(Buffer.from(passKeyPublicKey));
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(
        passKeyPrivateKey,
        Buffer.from(passKeyPublicKey),
        passkeyCall,
        true
      );

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });
  });

  it('should transfer small balance from fundedKeys to receiverKeys', async function () {
    const accountPKey = fundedKeys.publicKey;
    const nonce = await getNonce(fundedKeys);
    const transferCalls = ExtrinsicHelper.api.tx.balances.transferAllowDeath(receiverKeys.publicKey, 1);
    const { passKeyPrivateKey, passKeyPublicKey, passkeySignature } = createPassKeyAndSignAccount(accountPKey);
    const accountSignature = fundedKeys.sign(Buffer.from(passKeyPublicKey));
    const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, transferCalls);
    const passkeyPayload = await createPasskeyPayload(
      passKeyPrivateKey,
      Buffer.from(passKeyPublicKey),
      passkeyCall,
      false
    );

    const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
    await passkeyProxy.fundAndSendUnsigned(fundingSource);
  });
});

function createPassKeyAndSignAccount(accountPKey: Uint8Array) {
  const passKeyPrivateKey = secp256k1.utils.randomPrivateKey();
  const passKeyPublicKey = secp256k1.getPublicKey(passKeyPrivateKey);
  const passkeySignature = secp256k1.sign(accountPKey, passKeyPrivateKey).toCompactRawBytes();
  console.log('passKeyPrivateKey', passKeyPrivateKey);
  console.log('passKeyPublicKey', passKeyPublicKey);
  console.log('passkeySignature', passkeySignature);
  return { passKeyPrivateKey, passKeyPublicKey, passkeySignature };
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
  passKeyPrivateKey: Uint8Array,
  passKeyPublicKey: Uint8Array,
  passkeyCallPayload: any = {},
  bad: boolean = false
) {
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

  const passKeySignature = secp256k1.sign(Buffer.from(passkeyCallType.toU8a()), passKeyPrivateKey).toCompactRawBytes();
  const passkeyPayload = {
    passkeyPublicKey: Array.from(passKeyPublicKey),
    verifiablePasskeySignature: {
      signature: Array.from(passKeySignature),
      authenticatorData: Array.from(base64UrlToUint8Array(authenticatorData)),
      clientDataJson: Array.from(base64UrlToUint8Array(clientData)),
    },
    passkeyCall: passkeyCallType,
  };

  return passkeyPayload;
}
