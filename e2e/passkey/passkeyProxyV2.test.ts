import '@frequency-chain/api-augment';
import assert from 'assert';
import { createAndFundKeypair, getBlockNumber, getNonce, Sr25519Signature } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import { u8aToHex, u8aWrapBytes } from '@polkadot/util';
import { createPassKeyAndSignAccount, createPassKeyCallV2, createPasskeyPayloadV2 } from '../scaffolding/P256';
import { getUnifiedAddress, getUnifiedPublicKey } from '@frequency-chain/ethereum-utils';
import { AccountId32 } from '@polkadot/types/interfaces';
import { ISubmittableResult } from '@polkadot/types/types';
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
      await assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource), /Transaction call is not expected/);
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
      await assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource), /Invalid signing address/);
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
      await assert.rejects(passkeyProxy.fundAndSendUnsigned(fundingSource), /Custom error: 4/);
    });

    describe('successful transfer small balance from fundedKeys to receiverKeys', function () {
      let startingReceiverBalance: bigint;
      let startingSenderBalance: bigint;
      let accountPKey: Uint8Array<ArrayBufferLike>;
      let nonce: number;
      let target: any;
      let transferEvent: any;
      let feeEvent: any;
      let passkeyProxy: Extrinsic<
        {
          accountId: AccountId32;
        },
        ISubmittableResult,
        [accountId: AccountId32]
      >;

      const transferAmount = 100_000_000n;

      before(async function () {
        startingReceiverBalance = (await ExtrinsicHelper.getAccountInfo(receiverKeys)).data.free.toBigInt();
        startingSenderBalance = (await ExtrinsicHelper.getAccountInfo(fundedKeys)).data.free.toBigInt();
        accountPKey = getUnifiedPublicKey(fundedKeys);
        nonce = await getNonce(fundedKeys);
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
        passkeyProxy = ExtrinsicHelper.executePassKeyProxyV2(fundedKeys, passkeyPayload);
      });

      it('should successfully execute transfer extrinsic', async function () {
        await assert.doesNotReject(async () => {
          ({
            target,
            eventMap: { 'balances.Transfer': transferEvent, 'balances.Withdraw': feeEvent },
          } = await passkeyProxy.fundAndSendUnsigned(fundingSource));
        });
      });

      it('should have received a transaction execution success event', async function () {
        assert.notEqual(target, undefined, 'Target event should not be undefined');
        assert.equal(
          ExtrinsicHelper.api.events.balances.Transfer.is(transferEvent),
          true,
          'Transfer event should be of correct type'
        );
      });

      it('should have debited and credited correct accounts for the correct amount', async function () {
        if (transferEvent && ExtrinsicHelper.api.events.balances.Transfer.is(transferEvent)) {
          const { from, to, amount } = transferEvent.data;
          assert.equal(from.toString(), getUnifiedAddress(fundedKeys), 'From address should be the funded key');
          assert.equal(to.toString(), getUnifiedAddress(receiverKeys), 'To address should be the receiver key');
          assert.equal(amount.toBigInt(), transferAmount, `Transfer amount should be ${transferAmount}`);
        } else {
          assert.fail('Transfer event not found');
        }
      });

      it('should have deducted the correct fee from the sender', async function () {
        if (feeEvent && ExtrinsicHelper.api.events.balances.Withdraw.is(feeEvent)) {
          const { who, amount } = feeEvent.data;
          assert.equal(who.toString(), getUnifiedAddress(fundedKeys), 'Fee should be deducted from the sender');
          assert.equal(amount.toBigInt() > 0n, true, 'Fee should be greater than 0');
        } else {
          assert.fail('Fee event not found');
        }
      });

      /*
       * Normally these checks would be unnecessary, but we are testing the passkey pallet
       * which has additional logic surrounding mapping account keys, so we want to make sure
       * that the nonce and balance are updated correctly.
       */
      it('should have incremented nonce by 1 for sender', async function () {
        const nonceAfter = (await ExtrinsicHelper.getAccountInfo(fundedKeys)).nonce.toNumber();
        assert.equal(nonce + 1, nonceAfter, 'Nonce should be incremented by 1');
      });

      it('should have changed account balances correctly', async function () {
        const endingReceiverBalance = (await ExtrinsicHelper.getAccountInfo(receiverKeys)).data.free.toBigInt();
        const endingSenderBalance = (await ExtrinsicHelper.getAccountInfo(fundedKeys)).data.free.toBigInt();
        assert.equal(
          endingReceiverBalance,
          startingReceiverBalance + transferAmount,
          'Receiver balance should be incremented by transfer amount'
        );
        assert.equal(
          startingSenderBalance - transferAmount >= endingSenderBalance,
          true,
          'Sender balance should be decremented by at least transfer amount'
        );
      });
    });
  });
});
