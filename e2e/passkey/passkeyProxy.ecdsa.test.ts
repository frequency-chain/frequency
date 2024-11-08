import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  createAndFundKeypair, fundKeypair,
  getBlockNumber,
  getNonce
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import {Extrinsic, ExtrinsicHelper} from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import {hexToU8a, u8aToHex, u8aWrapBytes} from '@polkadot/util';
import { createPassKeyAndSignAccount, createPassKeyCall, createPasskeyPayload } from '../scaffolding/P256';
import {
  getConvertedEthereumPublicKey,
  getEthereumStyleSigner, getEthereumStyleSignerTest,
  getKeyringPairFromSecp256k1PrivateKey,
  getUnifiedAddress
} from "../scaffolding/ethereum";
import {Keyring} from "@polkadot/api";
const fundingSource = getFundingSource('passkey-proxy');

describe('Passkey Pallet Tests for ECDSA keys', function () {
  describe('proxy basic tests for ECDSA keys', function () {
    let fundedKeysSr25519: KeyringPair;
    let fundedEthereumKeys: KeyringPair;
    let ethereumKeysFromPrivateKey: KeyringPair;
    let receiverEthereumKeys: KeyringPair;

    before(async function () {
      // fundedKeysSr25519 = await createAndFundKeypair(fundingSource, 300_000_000n);
      const keyring = new Keyring({ type: 'sr25519'});
      fundedKeysSr25519 = keyring.addFromUri('//Eve');

      // fundedEthereumKeys = await createAndFundKeypair(fundingSource, 300_000_000n, undefined, undefined, 'ethereum');
      ethereumKeysFromPrivateKey = getKeyringPairFromSecp256k1PrivateKey(hexToU8a('0x4fa1fa06b8ad980d739473280ab1c362c425fa5883dd661a5d90e57e6d2969ce'));
      // receiverEthereumKeys = await createAndFundKeypair(fundingSource, undefined, undefined, undefined, 'ethereum');
      // console.log(`fundedEthereumKeys ${JSON.stringify(fundedEthereumKeys.toJson())} ${u8aToHex(fundedEthereumKeys.publicKey)} ${u8aToHex(fundedEthereumKeys.addressRaw)}   ${getUnifiedAddress(fundedEthereumKeys)}`);
      // console.log(`receiverEthereumKeys ${JSON.stringify(receiverEthereumKeys.toJson())} ${u8aToHex(receiverEthereumKeys.publicKey)} ${u8aToHex(receiverEthereumKeys.addressRaw)}   ${getUnifiedAddress(receiverEthereumKeys)}`);
    });

    // it ('should transfer from sr25519 to ethereum style key', async function () {
    //   const extrinsic = new Extrinsic(
    //     () => ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedAddress(receiverEthereumKeys), 33_000_000n),
    //     fundedKeysSr25519,
    //     ExtrinsicHelper.api.events.balances.Transfer
    //   );
    //   const {target} = await extrinsic.signAndSend();
    //   assert.notEqual(target, undefined, 'should have returned Transfer event');
    // })

    it ('should transfer from metamask injected signature to sr25519', async function () {
      const unifiedAddress = getUnifiedAddress(ethereumKeysFromPrivateKey);
      const extrinsic = ExtrinsicHelper.apiPromise.tx.balances.transferKeepAlive(getUnifiedAddress(fundedKeysSr25519), 33_000_000n);
      await extrinsic.signAndSend(unifiedAddress, { signer: getEthereumStyleSignerTest(
          "0x0a0300e659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e0229de0715000000008300000001000000c7b499ff539de473280e53ab077e100f7152a76f4d24025e9b56bebf74024ffd7cc514216c7605c9965a8740d74c4e8fd70066603ba54edc03a55a1e546e3bde00",
          "0x0c8dcadc5671485638b43a0f953ecd0e0eb6fa16213a6b605a7290e1df8b49db365c3f7c82eb9d0b857e97ad4307b0f423cae19cfd9a4b602ba8f06335cad3cc1c"
        ) }, (status) => {
        console.log(status.toHuman());
      });
    })

    // it ('should transfer from an ethereum key created from private key to sr25519', async function () {
    //   const unifiedAddress = getUnifiedAddress(ethereumKeysFromPrivateKey);
    //   await fundKeypair(fundedKeysSr25519, ethereumKeysFromPrivateKey, 100_000_000n);
    //   const extrinsic = ExtrinsicHelper.apiPromise.tx.balances.transferKeepAlive(getUnifiedAddress(fundedKeysSr25519), 44_000_000n);
    //   await extrinsic.signAndSend(unifiedAddress, { signer: getEthereumStyleSigner(ethereumKeysFromPrivateKey) }, (status) => {
    //     console.log(status.toHuman());
    //   });
    // })

    // it('should transfer via passkeys with root sr25519 key into an ethereum style account', async function () {
    //   const initialReceiverBalance = await ExtrinsicHelper.getAccountInfo(getUnifiedAddress(receiverEthereumKeys));
    //   const accountPKey = fundedKeysSr25519.publicKey;
    //   const nonce = await getNonce(fundedKeysSr25519);
    //   const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedAddress(receiverEthereumKeys), 55_000_000n);
    //   const { passKeyPrivateKey, passKeyPublicKey } = createPassKeyAndSignAccount(accountPKey);
    //   const accountSignature = fundedKeysSr25519.sign(u8aWrapBytes(passKeyPublicKey));
    //   const passkeyCall = await createPassKeyCall(accountPKey, nonce, { Sr25519: accountSignature} as MultiSignature, transferCalls);
    //   const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, false);
    //   const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeysSr25519, passkeyPayload);
    //   assert.doesNotReject(passkeyProxy.fundAndSendUnsigned(fundingSource));
    //   await ExtrinsicHelper.waitForFinalization((await getBlockNumber()) + 2);
    //   const receiverBalance = await ExtrinsicHelper.getAccountInfo(getUnifiedAddress(receiverEthereumKeys));
    //   // adding some delay before fetching the nonce to ensure it is updated
    //   await new Promise((resolve) => setTimeout(resolve, 1000));
    //   const nonceAfter = (await ExtrinsicHelper.getAccountInfo(getUnifiedAddress(fundedKeysSr25519))).nonce.toNumber();
    //   assert.equal(nonce + 1, nonceAfter);
    //   assert(receiverBalance.data.free.toBigInt() - initialReceiverBalance.data.free.toBigInt() > 0n);
    // });
    //
    // it ('should transfer via passkeys with root ethereum style key into another one', async function () {
    //   const initialReceiverBalance = await ExtrinsicHelper.getAccountInfo(getUnifiedAddress(receiverEthereumKeys));
    //   const accountPKey = getConvertedEthereumPublicKey(fundedEthereumKeys);
    //   console.log(`accountPKey ${u8aToHex(accountPKey)}`);
    //   const nonce = await getNonce(fundedEthereumKeys);
    //   const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedAddress(receiverEthereumKeys), 66_000_000n);
    //   const { passKeyPrivateKey, passKeyPublicKey } = createPassKeyAndSignAccount(accountPKey);
    //   const accountSignature = fundedEthereumKeys.sign(u8aWrapBytes(passKeyPublicKey));
    //   console.log(`accountSignature ${u8aToHex(accountSignature)}`);
    //   const passkeyCall = await createPassKeyCall(accountPKey, nonce, { Ecdsa: accountSignature} as MultiSignature, transferCalls);
    //   const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, false);
    //   const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundingSource, passkeyPayload);
    //   assert.doesNotReject(passkeyProxy.sendUnsigned());
    //   await ExtrinsicHelper.waitForFinalization((await getBlockNumber()) + 2);
    //   const receiverBalance = await ExtrinsicHelper.getAccountInfo(getUnifiedAddress(receiverEthereumKeys));
    //   // adding some delay before fetching the nonce to ensure it is updated
    //   await new Promise((resolve) => setTimeout(resolve, 1000));
    //   const nonceAfter = (await ExtrinsicHelper.getAccountInfo(getUnifiedAddress(fundedEthereumKeys))).nonce.toNumber();
    //   assert.equal(nonce + 1, nonceAfter);
    //   assert(receiverBalance.data.free.toBigInt() - initialReceiverBalance.data.free.toBigInt() > 0n);
    // })
  });
});

