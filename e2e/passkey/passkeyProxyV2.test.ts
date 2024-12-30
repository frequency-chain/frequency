import '@frequency-chain/api-augment';
import assert from 'assert';
import { createAndFundKeypair, getBlockNumber, getNonce, Sr25519Signature } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import { u8aToHex, u8aWrapBytes } from '@polkadot/util';
import { createPassKeyAndSignAccount, createPassKeyCallV2, createPasskeyPayloadV2 } from '../scaffolding/P256';
import { getUnifiedAddress, getUnifiedPublicKey } from '../scaffolding/ethereum';
const fundingSource = getFundingSource(import.meta.url);

describe('Passkey Pallet Proxy V2 Tests', function () {
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
      const { passKeyPrivateKey, passKeyPublicKey } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeys.sign(u8aWrapBytes(passKeyPublicKey));
      const multiSignature: Sr25519Signature = { Sr25519: u8aToHex(accountSignature) };
      const passkeyCall = await createPassKeyCallV2(accountPKey, nonce, remarksCalls);
      const passkeyPayload = await createPasskeyPayloadV2(
        multiSignature,
        passKeyPrivateKey,
        passKeyPublicKey,
        passkeyCall,
        false
      );

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxyV2(fundedKeys, passkeyPayload);
      await assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });

    it('should fail to transfer balance due to bad account ownership proof', async function () {
      const accountPKey = getUnifiedPublicKey(fundedKeys);
      const nonce = await getNonce(fundedKeys);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedPublicKey(receiverKeys), 0n);
      const { passKeyPrivateKey, passKeyPublicKey } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeys.sign('badPasskeyPublicKey');
      const multiSignature: Sr25519Signature = { Sr25519: u8aToHex(accountSignature) };
      const passkeyCall = await createPassKeyCallV2(accountPKey, nonce, transferCalls);
      const passkeyPayload = await createPasskeyPayloadV2(
        multiSignature,
        passKeyPrivateKey,
        passKeyPublicKey,
        passkeyCall,
        false
      );

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxyV2(fundedKeys, passkeyPayload);
      await assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });

    it('should fail to transfer balance due to bad passkey signature', async function () {
      const accountPKey = getUnifiedPublicKey(fundedKeys);
      const nonce = await getNonce(fundedKeys);
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedPublicKey(receiverKeys), 0n);
      const { passKeyPrivateKey, passKeyPublicKey } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeys.sign(u8aWrapBytes(passKeyPublicKey));
      const multiSignature: Sr25519Signature = { Sr25519: u8aToHex(accountSignature) };
      const passkeyCall = await createPassKeyCallV2(accountPKey, nonce, transferCalls);
      const passkeyPayload = await createPasskeyPayloadV2(
        multiSignature,
        passKeyPrivateKey,
        passKeyPublicKey,
        passkeyCall,
        true
      );

      const passkeyProxy = ExtrinsicHelper.executePassKeyProxyV2(fundedKeys, passkeyPayload);
      await assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource));
    });

    it('should transfer small balance from fundedKeys to receiverKeys', async function () {
      const startingBalance = (await ExtrinsicHelper.getAccountInfo(receiverKeys)).data.free.toBigInt();
      const accountPKey = getUnifiedPublicKey(fundedKeys);
      const nonce = await getNonce(fundedKeys);
      const transferAmount = 100_000_000n;
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(
        getUnifiedPublicKey(receiverKeys),
        transferAmount
      );
      const { passKeyPrivateKey, passKeyPublicKey } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedKeys.sign(u8aWrapBytes(passKeyPublicKey));
      const multiSignature: Sr25519Signature = { Sr25519: u8aToHex(accountSignature) };
      const passkeyCall = await createPassKeyCallV2(accountPKey, nonce, transferCalls);
      const passkeyPayload = await createPasskeyPayloadV2(
        multiSignature,
        passKeyPrivateKey,
        passKeyPublicKey,
        passkeyCall,
        false
      );
      const passkeyProxy = ExtrinsicHelper.executePassKeyProxyV2(fundedKeys, passkeyPayload);
      try {
        const {
          target,
          eventMap: { 'balances.Transfer': transferEvent },
        } = await passkeyProxy.fundAndSendUnsigned(fundingSource);
        assert.notEqual(target, undefined, 'Target event should not be undefined');
        assert.equal(
          ExtrinsicHelper.api.events.balances.Transfer.is(transferEvent),
          true,
          'Transfer event should be of correct type'
        );
        if (transferEvent && ExtrinsicHelper.api.events.balances.Transfer.is(transferEvent)) {
          const { from, to, amount } = transferEvent.data;
          assert.equal(from.toString(), getUnifiedAddress(fundedKeys), 'From address should be the funded key');
          assert.equal(to.toString(), getUnifiedAddress(receiverKeys), 'To address should be the receiver key');
          assert.equal(amount.toBigInt(), transferAmount, `Transfer amount should be ${transferAmount}`);
        }
      } catch (e: any) {
        assert.fail(e);
      }

      /*
       * Normally these checks would be unnecessary, but we are testing the passkey pallet
       * which has additional logic surrounding mapping account keys, so we want to make sure
       * that the nonce and balance are updated correctly.
       */
      const nonceAfter = (await ExtrinsicHelper.getAccountInfo(fundedKeys)).nonce.toNumber();
      assert.equal(nonce + 1, nonceAfter, 'Nonce should be incremented by 1');

      const balanceAfter = (await ExtrinsicHelper.getAccountInfo(receiverKeys)).data.free.toBigInt();
      assert.equal(
        balanceAfter,
        startingBalance + transferAmount,
        'Receiver balance should be incremented by transfer amount'
      );
    });
  });
});
