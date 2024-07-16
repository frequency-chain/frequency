import '@frequency-chain/api-augment';
import assert from 'assert';
import { createAndFundKeypair, getNonce } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import { u8aWrapBytes } from '@polkadot/util';
import { createPassKeyAndSignAccount, createPassKeyCall, createPasskeyPayload } from '../scaffolding/P256';
const fundingSource = getFundingSource('passkey-proxy');

describe('Passkey Pallet Tests', function () {
  describe('proxy basic tests', function () {
    let fundedKeys: KeyringPair;
    let receiverKeys: KeyringPair;

    beforeEach(async function () {
      fundedKeys = await createAndFundKeypair(fundingSource, 100_000_000n);
      receiverKeys = await createAndFundKeypair(fundingSource);
    });

    it('should fail due to unsupported call', async function () {
      const accountPKey = fundedKeys.publicKey;
      const nonce = await getNonce(fundedKeys);

      const remarksCalls = ExtrinsicHelper.api.tx.system.remark('passkey-test');
      const { passKeyPrivateKey, passKeyPublicKey, passkeySignature } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeys.sign(u8aWrapBytes(passKeyPublicKey));
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, remarksCalls);
      const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, false);

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });

    it('should fail to transfer balance due to bad account ownership proof', async function () {
      const accountPKey = fundedKeys.publicKey;
      const nonce = await getNonce(fundedKeys);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(receiverKeys.publicKey, 0n);
      const { passKeyPrivateKey, passKeyPublicKey, passkeySignature } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeys.sign('badPasskeyPublicKey');
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, false);

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });

    it('should fail to transfer balance due to bad passkey signature', async function () {
      const accountPKey = fundedKeys.publicKey;
      const nonce = await getNonce(fundedKeys);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(receiverKeys.publicKey, 0n);
      const { passKeyPrivateKey, passKeyPublicKey, passkeySignature } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeys.sign(u8aWrapBytes(passKeyPublicKey));
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, true);

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });

    it('should transfer small balance from fundedKeys to receiverKeys', async function () {
      const accountPKey = fundedKeys.publicKey;
      const nonce = await getNonce(fundedKeys);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(receiverKeys.publicKey, 100n);
      const { passKeyPrivateKey, passKeyPublicKey, passkeySignature } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeys.sign(u8aWrapBytes(passKeyPublicKey));
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, false);
      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      assert.doesNotReject(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });
  });
});
