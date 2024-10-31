import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  createAndFundKeypair,
  DOLLARS,
  getBlockNumber,
  getConvertedEthereumAccount,
  getNextEpochBlock,
  getNonce
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import {Extrinsic, ExtrinsicHelper} from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import {hexToU8a, u8aToHex, u8aWrapBytes} from '@polkadot/util';
import { createPassKeyAndSignAccount, createPassKeyCall, createPasskeyPayload } from '../scaffolding/P256';
import {encodeAddress, ethereumEncode} from "@polkadot/util-crypto";
import {secp256k1Expand} from "@polkadot/util-crypto/secp256k1/expand";
import {HexString} from "@polkadot/util/types";
import {SignerResult} from "@polkadot/types/types";
const fundingSource = getFundingSource('passkey-proxy');

describe('Passkey Pallet Tests for ECDSA keys', function () {
  describe('proxy basic tests for ECDSA keys', function () {
    let fundedKeys: KeyringPair;
    let receiverKeys: KeyringPair;
    let fundedKeysSr25519: KeyringPair;
    let fundedKeysEth: KeyringPair;

    before(async function () {
      fundedKeys = await createAndFundKeypair(fundingSource, 300_000_000n, "test_fund", undefined, 'ecdsa');
      receiverKeys = await createAndFundKeypair(fundingSource, undefined, "test_receive", undefined, 'ecdsa');
      fundedKeysSr25519 = await createAndFundKeypair(fundingSource, 300_000_000n);
      // fundedKeysEth = await createAndFundKeypair(fundingSource, 300_000_000n, "test_eth", undefined, 'ethereum');
      console.log(`fundedKeys ${JSON.stringify(fundedKeys.toJson())}  ${encodeAddress(fundedKeys.address)}`);
      console.log(`receiverKeys ${JSON.stringify(receiverKeys.toJson())}   ${encodeAddress(receiverKeys.address)} ${u8aToHex(receiverKeys.addressRaw)}`);
      // console.log(`fundedKeysEth ${JSON.stringify(fundedKeysEth.toJson())}`);
    });

    // it('should fail due to unsupported call', async function () {
    //   const accountPKey = fundedKeys.publicKey;
    //   const nonce = await getNonce(fundedKeys);
    //
    //   const remarksCalls = ExtrinsicHelper.api.tx.system.remark('passkey-test');
    //   const { passKeyPrivateKey, passKeyPublicKey, passkeySignature } = createPassKeyAndSignAccount(accountPKey);
    //   const accountSignature = fundedKeys.sign(u8aWrapBytes(passKeyPublicKey));
    //   const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, remarksCalls);
    //   const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, false);
    //
    //   const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
    //   assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    // });
    //
    // it('should fail to transfer balance due to bad account ownership proof', async function () {
    //   const accountPKey = fundedKeys.publicKey;
    //   const nonce = await getNonce(fundedKeys);
    //   const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(receiverKeys.publicKey, 0n);
    //   const { passKeyPrivateKey, passKeyPublicKey, passkeySignature } = createPassKeyAndSignAccount(accountPKey);
    //   const accountSignature = fundedKeys.sign('badPasskeyPublicKey');
    //   const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, transferCalls);
    //   const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, false);
    //
    //   const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
    //   assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    // });
    //
    // it('should fail to transfer balance due to bad passkey signature', async function () {
    //   const accountPKey = fundedKeys.publicKey;
    //   const nonce = await getNonce(fundedKeys);
    //   const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(receiverKeys.publicKey, 0n);
    //   const { passKeyPrivateKey, passKeyPublicKey, passkeySignature } = createPassKeyAndSignAccount(accountPKey);
    //   const accountSignature = fundedKeys.sign(u8aWrapBytes(passKeyPublicKey));
    //   const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, transferCalls);
    //   const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, true);
    //
    //   const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
    //   assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    // });

    // it('should transfer via passkeys', async function () {
    //   const receiverBalance1 = await ExtrinsicHelper.getAccountInfo(receiverKeys.address);
    //   const balanceBefore = receiverBalance1.data.free.toBigInt();
    //   console.log(`before ${balanceBefore}`);
    //
    //   const accountPKey = fundedKeys.addressRaw;
    //   console.log(`public key size ${accountPKey.length}`)
    //   const nonce = await getNonce(fundedKeys);
    //   const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive({
    //     Id: receiverKeys.address
    //   }, 100_000_000n);
    //   const { passKeyPrivateKey, passKeyPublicKey } = createPassKeyAndSignAccount(accountPKey);
    //   const accountSignature = fundedKeys.sign(u8aWrapBytes(passKeyPublicKey));
    //   const passkeyCall = await createPassKeyCall(accountPKey, nonce, accountSignature, transferCalls);
    //   const passkeyPayload = await createPasskeyPayload(passKeyPrivateKey, passKeyPublicKey, passkeyCall, false);
    //   const passkeyProxy = ExtrinsicHelper.executePassKeyProxy(fundedKeys, passkeyPayload);
    //   assert.doesNotReject(passkeyProxy.fundAndSendUnsigned(fundingSource));
    //   await ExtrinsicHelper.waitForFinalization((await getBlockNumber()) + 2);
    //   const receiverBalance = await ExtrinsicHelper.getAccountInfo(receiverKeys.address);
    //   const nonceAfter = (await ExtrinsicHelper.getAccountInfo(fundedKeys.address)).nonce.toNumber();
    //   assert.equal(nonce + 1, nonceAfter);
    //   console.log(`after ${receiverBalance}`);
    //   assert(receiverBalance.data.free.toBigInt() - balanceBefore > 0n);
    // });

    // it ('it should send from secp256k1 to secp256k1', async function () {
    //   const receiverBalance1 = await ExtrinsicHelper.getAccountInfo(receiverKeys.address);
    //   const balanceBefore = receiverBalance1.data.free.toBigInt();
    //   console.log(`before ${balanceBefore}`);
    //
    //   const extrinsic = new Extrinsic(
    //     () => ExtrinsicHelper.api.tx.balances.transferKeepAlive(receiverKeys.address, 100_000_000n),
    //     fundedKeys,
    //     ExtrinsicHelper.api.events.balances.Transfer
    //   );
    //   const {target} = await extrinsic.signAndSend();
    //   assert.notEqual(target, undefined, 'should have returned Transfer event');
    //
    //   const receiverBalance2 = await ExtrinsicHelper.getAccountInfo(receiverKeys.address);
    //   const balanceBefore2 = receiverBalance2.data.free.toBigInt();
    //   console.log(`after ${balanceBefore2}`);
    // })

    it ('it should send from sr25519 accountId32 to secp256k1 accountId20', async function () {
      const etheAddress = ethereumEncode(receiverKeys.publicKey);
      console.log(`eth receiver address ${etheAddress}`);
      let ethAddress20 = Array.from(hexToU8a(etheAddress));
      console.log(`ss ${ethAddress20}`);

      const extrinsic = new Extrinsic(
        () => ExtrinsicHelper.api.tx.balances.transferKeepAlive({
          Address20: ethAddress20
        }, 50_000_000n),
        fundedKeysSr25519,
        ExtrinsicHelper.api.events.balances.Transfer
      );
      const {target} = await extrinsic.signAndSend();
      assert.notEqual(target, undefined, 'should have returned Transfer event');

      // transfer back
      const extrinsic2 = ExtrinsicHelper.apiPromise.tx.balances.transferKeepAlive(fundedKeysSr25519.address, 1_000_000n);
      let convertedAddress = getConvertedEthereumAccount(etheAddress);
      const rawPayload = await ExtrinsicHelper.getRawPayloadForSigning(extrinsic2, convertedAddress);
      console.log(`signer payload => ${JSON.stringify(rawPayload)}`);

      await extrinsic2.signAndSend(convertedAddress, {signer: {
        signRaw: async (payload): Promise<SignerResult> => {
          console.log(`inside payload => ${JSON.stringify(payload)}`);
          const sig = receiverKeys.sign(payload.data);
          const prefixedSignature = new Uint8Array(sig.length + 1);
          prefixedSignature[0]=2;
          prefixedSignature.set(sig, 1);
          const hex = u8aToHex(prefixedSignature);
          console.log(`signature => ${hex}`);
          return {
              signature: hex,
          } as SignerResult;
        },
      }}, (status) => {
        console.log(status.toHuman());
      });



      // const extrinsic2 = new Extrinsic(
      //   () => ExtrinsicHelper.api.tx.balances.transferKeepAlive(fundedKeysSr25519.address, 500_000n),
      //   receiverKeys,
      //   ExtrinsicHelper.api.events.balances.Transfer
      // );
      // const {target2} = await extrinsic2.signAndSend();
      // assert.notEqual(target2, undefined, 'should have returned Transfer event');

    })


    // it ('it should send from secp256k1 accountId20 to sr25519', async function () {
    //   const etheAddress = ethereumEncode(fundedKeysEth.publicKey);
    //   console.log(`eth sender address ${etheAddress}`);
    //   let convertedAddress = getConvertedEthereumAccount(etheAddress);
    //   console.log(`convertedAddress ${convertedAddress}`);
    //   // const receiverBalance1 = await ExtrinsicHelper.getAccountInfo(receiverKeys.address);
    //   // const balanceBefore = receiverBalance1.data.free.toBigInt();
    //   // console.log(`before ${balanceBefore}`);
    //
    //   const extrinsic = ExtrinsicHelper.api.tx.balances.transferKeepAlive(fundedKeysSr25519.address, 1_000_000n);
    //   const rawPayload = await ExtrinsicHelper.getRawPayloadForSigning(extrinsic, convertedAddress);
    //   console.log(`signer payload => ${JSON.stringify(rawPayload)}`);
    //
    //   // TODO check whatever
    //   // const {target} = await extrinsic.signAndSend();
    //   // assert.notEqual(target, undefined, 'should have returned Transfer event');
    //
    //   // const receiverBalance2 = await ExtrinsicHelper.getAccountInfo(receiverKeys.address);
    //   // const balanceBefore2 = receiverBalance2.data.free.toBigInt();
    //   // console.log(`after ${balanceBefore2}`);
    // })
  });
});

