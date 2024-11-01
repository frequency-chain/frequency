import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  createAndFundKeypair,
  getBlockNumber,
  getNonce
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import {Extrinsic, ExtrinsicHelper} from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import { u8aToHex, u8aWrapBytes} from '@polkadot/util';
import { createPassKeyAndSignAccount, createPassKeyCall, createPasskeyPayload } from '../scaffolding/P256';
import {getConvertedEthereumPublicKey, getEthereumStyleSigner, getUnifiedAddress} from "../scaffolding/ethereum";
import {MultiSignature} from "@polkadot/types/interfaces";
const fundingSource = getFundingSource('passkey-proxy');

describe('Passkey Pallet Tests for ECDSA keys', function () {
  describe('proxy basic tests for ECDSA keys', function () {
    let fundedKeysSr25519: KeyringPair;
    let fundedEthereumKeys: KeyringPair;
    let receiverEthereumKeys: KeyringPair;

    before(async function () {
      fundedKeysSr25519 = await createAndFundKeypair(fundingSource, 300_000_000n);
      fundedEthereumKeys = await createAndFundKeypair(fundingSource, 300_000_000n, undefined, undefined, 'ethereum');
      receiverEthereumKeys = await createAndFundKeypair(fundingSource, undefined, undefined, undefined, 'ethereum');
      console.log(`ethereumKeys ${JSON.stringify(fundedEthereumKeys.toJson())} ${u8aToHex(fundedEthereumKeys.publicKey)} ${u8aToHex(fundedEthereumKeys.addressRaw)}   ${getUnifiedAddress(fundedEthereumKeys)}`);
      console.log(`receiverEthereumKeys ${JSON.stringify(receiverEthereumKeys.toJson())} ${u8aToHex(receiverEthereumKeys.publicKey)} ${u8aToHex(receiverEthereumKeys.addressRaw)}   ${getUnifiedAddress(receiverEthereumKeys)}`);
    });

    it ('should transfer from sr25519 to ethereum style key', async function () {
      const extrinsic = new Extrinsic(
        () => ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedAddress(receiverEthereumKeys), 33_000_000n),
        fundedKeysSr25519,
        ExtrinsicHelper.api.events.balances.Transfer
      );
      const {target} = await extrinsic.signAndSend();
      assert.notEqual(target, undefined, 'should have returned Transfer event');
    })

    it ('should transfer from ethereum style key to sr25519', async function () {
      const unifiedAddress = getUnifiedAddress(fundedEthereumKeys);
      const extrinsic = ExtrinsicHelper.apiPromise.tx.balances.transferKeepAlive(getUnifiedAddress(fundedKeysSr25519), 44_000_000n);
      await extrinsic.signAndSend(unifiedAddress, { signer: getEthereumStyleSigner(fundedEthereumKeys) }, (status) => {
        console.log(status.toHuman());
      });
    })

    it('should transfer via passkeys with root sr25519 key into an ethereum style account', async function () {
      const initialReceiverBalance = await ExtrinsicHelper.getAccountInfo(getUnifiedAddress(receiverEthereumKeys));
      const accountPKey = fundedKeysSr25519.publicKey;
      const nonce = await getNonce(fundedKeysSr25519);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedAddress(receiverEthereumKeys), 55_000_000n);
      const { passKeyPrivateKey, passKeyPublicKey } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeysSr25519.sign(u8aWrapBytes(passKeyPublicKey));
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, { Sr25519: accountSignature} as MultiSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, false);
      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeysSr25519, passkeyPayload);
      assert.doesNotReject(passkeyProxy.fundAndSendUnsigned(fundingSource));
      await ExtrinsicHelper.waitForFinalization((await getBlockNumber()) + 2);
      const receiverBalance = await ExtrinsicHelper.getAccountInfo(getUnifiedAddress(receiverEthereumKeys));
      // adding some delay before fetching the nonce to ensure it is updated
      await new Promise((resolve) => setTimeout(resolve, 1000));
      const nonceAfter = (await ExtrinsicHelper.getAccountInfo(getUnifiedAddress(fundedKeysSr25519))).nonce.toNumber();
      assert.equal(nonce + 1, nonceAfter);
      assert(receiverBalance.data.free.toBigInt() - initialReceiverBalance.data.free.toBigInt() > 0n);
    });

    it ('should transfer via passkeys with root ethereum style key into another one', async function () {
      const initialReceiverBalance = await ExtrinsicHelper.getAccountInfo(getUnifiedAddress(receiverEthereumKeys));
      const accountPKey = getConvertedEthereumPublicKey(fundedEthereumKeys);
      console.log(`accountPKey ${u8aToHex(accountPKey)}`);
      const nonce = await getNonce(fundedEthereumKeys);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedAddress(receiverEthereumKeys), 66_000_000n);
      const { passKeyPrivateKey, passKeyPublicKey } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedEthereumKeys.sign(u8aWrapBytes(passKeyPublicKey));
      console.log(`accountSignature ${u8aToHex(accountSignature)}`);
      const passkeyCall = await createPassKeyCall(accountPKey, nonce, { Ecdsa: accountSignature} as MultiSignature, transferCalls);
      const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, false);
      const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundingSource, passkeyPayload);
      assert.doesNotReject(passkeyProxy.sendUnsigned());
      await ExtrinsicHelper.waitForFinalization((await getBlockNumber()) + 2);
      const receiverBalance = await ExtrinsicHelper.getAccountInfo(getUnifiedAddress(receiverEthereumKeys));
      // adding some delay before fetching the nonce to ensure it is updated
      await new Promise((resolve) => setTimeout(resolve, 1000));
      const nonceAfter = (await ExtrinsicHelper.getAccountInfo(getUnifiedAddress(fundedEthereumKeys))).nonce.toNumber();
      assert.equal(nonce + 1, nonceAfter);
      assert(receiverBalance.data.free.toBigInt() - initialReceiverBalance.data.free.toBigInt() > 0n);
    })
  });
});

