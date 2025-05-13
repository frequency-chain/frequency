import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  createAndFundKeypair,
  getBlockNumber,
  getNextEpochBlock,
  getNonce,
  Sr25519Signature,
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import { u8aToHex, u8aWrapBytes } from '@polkadot/util';
import { createPassKeyAndSignAccount, createPassKeyCall, createPasskeyPayload } from '../scaffolding/P256';
import { getUnifiedPublicKey } from '@frequency-chain/ethereum-utils/address';
const fundingSource = getFundingSource(import.meta.url);

describe('Passkey Pallet Tests', function () {
  describe('proxy basic tests', function () {
    let fundedKeys: KeyringPair;
    let receiverKeys: KeyringPair;

    before(async function () {
      fundedKeys = await createAndFundKeypair(fundingSource, 300_000_000n);
      receiverKeys = await createAndFundKeypair(fundingSource);
    });

    it('should fail due to unsupported call', async function () {
      const accountPKey = getUnifiedPublicKey(fundedKeys);
      const nonce = await getNonce(fundedKeys);

      const remarksCalls = ExtrinsicHelper.api.tx.system.remark('passkey-test');
      const { passKeyPrivateKey, passKeyPublicKey, passkeySignature } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeys.sign(u8aWrapBytes(passKeyPublicKey));
      const multiSignature: Sr25519Signature = { Sr25519: u8aToHex(accountSignature) };
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, multiSignature, remarksCalls);
      const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, false);

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      await assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });

    it('should fail to transfer balance due to bad account ownership proof', async function () {
      const accountPKey = getUnifiedPublicKey(fundedKeys);
      const nonce = await getNonce(fundedKeys);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedPublicKey(receiverKeys), 0n);
      const { passKeyPrivateKey, passKeyPublicKey, passkeySignature } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeys.sign('badPasskeyPublicKey');
      const multiSignature: Sr25519Signature = { Sr25519: u8aToHex(accountSignature) };
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, multiSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, false);

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      await assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });

    it('should fail to transfer balance due to bad passkey signature', async function () {
      const accountPKey = getUnifiedPublicKey(fundedKeys);
      const nonce = await getNonce(fundedKeys);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedPublicKey(receiverKeys), 0n);
      const { passKeyPrivateKey, passKeyPublicKey, passkeySignature } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeys.sign(u8aWrapBytes(passKeyPublicKey));
      const multiSignature: Sr25519Signature = { Sr25519: u8aToHex(accountSignature) };
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, multiSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, true);

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      await assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });

    it('should transfer small balance from fundedKeys to receiverKeys', async function () {
      const accountPKey = getUnifiedPublicKey(fundedKeys);
      const nonce = await getNonce(fundedKeys);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(
        getUnifiedPublicKey(receiverKeys),
        100_000_000n
      );
      const { passKeyPrivateKey, passKeyPublicKey } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeys.sign(u8aWrapBytes(passKeyPublicKey));
      const multiSignature: Sr25519Signature = { Sr25519: u8aToHex(accountSignature) };
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, multiSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, false);
      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
      await assert.doesNotReject(passkeyProxy.fundAndSendUnsigned(fundingSource));
      await ExtrinsicHelper.waitForFinalization((await getBlockNumber()) + 2);
      const receiverBalance = await ExtrinsicHelper.getAccountInfo(receiverKeys);
      // adding some delay before fetching the nonce to ensure it is updated
      await new Promise((resolve) => setTimeout(resolve, 2000));
      const nonceAfter = (await ExtrinsicHelper.getAccountInfo(fundedKeys)).nonce.toNumber();
      assert.equal(nonce + 1, nonceAfter);
      assert(receiverBalance.data.free.toBigInt() > 0n);
    });
  });
});
