import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  createAndFundKeypair,
  EcdsaSignature,
  getBlockNumber,
  getNextEpochBlock,
  getNonce,
  Sr25519Signature,
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import { getConvertedEthereumPublicKey, getUnifiedAddress } from '../scaffolding/ethereum';
import { createPassKeyAndSignAccount, createPassKeyCall, createPasskeyPayload } from '../scaffolding/P256';
import { u8aToHex, u8aWrapBytes } from '@polkadot/util';
const fundingSource = getFundingSource('passkey-proxy-ethereum');

describe('Passkey Pallet Ethereum Tests', function () {
  describe('passkey ethereum tests', function () {
    let fundedSr25519Keys: KeyringPair;
    let fundedEthereumKeys: KeyringPair;
    let receiverKeys: KeyringPair;

    before(async function () {
      fundedSr25519Keys = await createAndFundKeypair(fundingSource, 300_000_000n);
      fundedEthereumKeys = await createAndFundKeypair(fundingSource, 300_000_000n, undefined, undefined, 'ethereum');
      receiverKeys = await createAndFundKeypair(fundingSource);
    });

    it('should transfer via passkeys with root sr25519 key into an ethereum style account', async function () {
      const initialReceiverBalance = await ExtrinsicHelper.getAccountInfo(receiverKeys);
      const accountPKey = fundedSr25519Keys.publicKey;
      const nonce = await getNonce(fundedSr25519Keys);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(
        getUnifiedAddress(receiverKeys),
        55_000_000n
      );
      const { passKeyPrivateKey, passKeyPublicKey } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedSr25519Keys.sign(u8aWrapBytes(passKeyPublicKey));
      const multiSignature: Sr25519Signature = { Sr25519: u8aToHex(accountSignature) };
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, multiSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, false);
      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedSr25519Keys, passkeyPayload);
      assert.doesNotReject(passkeyProxy.fundAndSendUnsigned(fundingSource));
      await ExtrinsicHelper.waitForFinalization((await getBlockNumber()) + 2);
      const receiverBalance = await ExtrinsicHelper.getAccountInfo(receiverKeys);
      // adding some delay before fetching the nonce to ensure it is updated
      await new Promise((resolve) => setTimeout(resolve, 1000));
      const nonceAfter = (await ExtrinsicHelper.getAccountInfo(fundedSr25519Keys)).nonce.toNumber();
      assert.equal(nonce + 1, nonceAfter);
      assert(receiverBalance.data.free.toBigInt() - initialReceiverBalance.data.free.toBigInt() > 0n);
    });

    it('should transfer via passkeys with root ethereum style key into another one', async function () {
      const initialReceiverBalance = await ExtrinsicHelper.getAccountInfo(receiverKeys);
      const accountPKey = getConvertedEthereumPublicKey(fundedEthereumKeys);
      console.log(`accountPKey ${u8aToHex(accountPKey)}`);
      const nonce = await getNonce(fundedEthereumKeys);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(
        getUnifiedAddress(receiverKeys),
        66_000_000n
      );
      const { passKeyPrivateKey, passKeyPublicKey } = createPassKeyAndSignAccount(accountPKey);
      // ethereum keys should not have wrapping
      const accountSignature = fundedEthereumKeys.sign(passKeyPublicKey);
      console.log(`accountSignature ${u8aToHex(accountSignature)}`);
      const multiSignature: EcdsaSignature = { Ecdsa: u8aToHex(accountSignature) };
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, multiSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, false);
      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundingSource, passkeyPayload);
      assert.doesNotReject(passkeyProxy.sendUnsigned());
      await ExtrinsicHelper.waitForFinalization((await getBlockNumber()) + 2);
      const receiverBalance = await ExtrinsicHelper.getAccountInfo(receiverKeys);
      // adding some delay before fetching the nonce to ensure it is updated
      await new Promise((resolve) => setTimeout(resolve, 1000));
      const nonceAfter = (await ExtrinsicHelper.getAccountInfo(fundedEthereumKeys)).nonce.toNumber();
      assert.equal(nonce + 1, nonceAfter);
      assert(receiverBalance.data.free.toBigInt() - initialReceiverBalance.data.free.toBigInt() > 0n);
    });
  });
});