import '@frequency-chain/api-augment';
import assert from 'assert';
import { createAndFundKeypair, EcdsaSignature, getNonce, log, Sr25519Signature } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import { getUnifiedPublicKey, getUnifiedAddress } from '@frequency-chain/ethereum-utils/address';
import { createPassKeyAndSignAccount, createPassKeyCallV2, createPasskeyPayloadV2 } from '../scaffolding/P256';
import { u8aToHex, u8aWrapBytes } from '@polkadot/util';
import { AccountId32 } from '@polkadot/types/interfaces';
import { ISubmittableResult } from '@polkadot/types/types';
const fundingSource = getFundingSource(import.meta.url);

describe('Passkey Pallet Proxy V2 Ethereum Tests', function () {
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
      const startingBalance = (await ExtrinsicHelper.getAccountInfo(receiverKeys)).data.free.toBigInt();
      const accountPKey = getUnifiedPublicKey(fundedSr25519Keys);
      const nonce = await getNonce(fundedSr25519Keys);
      const transferAmount = 55_000_000n;
      const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(
        getUnifiedAddress(receiverKeys),
        transferAmount
      );
      const { passKeyPrivateKey, passKeyPublicKey } = createPassKeyAndSignAccount(accountPKey);
      const accountSignature = fundedSr25519Keys.sign(u8aWrapBytes(passKeyPublicKey));
      const multiSignature: Sr25519Signature = { Sr25519: u8aToHex(accountSignature) };
      const passkeyCall = await createPassKeyCallV2(accountPKey, nonce, transferCalls);
      const passkeyPayload = await createPasskeyPayloadV2(
        multiSignature,
        passKeyPrivateKey,
        passKeyPublicKey,
        passkeyCall,
        false
      );
      const passkeyProxy = ExtrinsicHelper.executePassKeyProxyV2(fundedSr25519Keys, passkeyPayload);
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
          assert.equal(
            from.toString(),
            getUnifiedAddress(fundedSr25519Keys),
            'From address should be the funded sr25519 key'
          );
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
      const nonceAfter = (await ExtrinsicHelper.getAccountInfo(fundedSr25519Keys)).nonce.toNumber();
      assert.equal(nonce + 1, nonceAfter, 'Nonce should be incremented by 1');

      const balanceAfter = (await ExtrinsicHelper.getAccountInfo(receiverKeys)).data.free.toBigInt();
      assert.equal(
        balanceAfter,
        startingBalance + transferAmount,
        'Receiver balance should be incremented by transfer amount'
      );
    });

    describe('successful transfer via passkeys with root ethereum-style key into a polkadot-style key', function () {
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

      const transferAmount = 66_000_000n;

      before(async function () {
        startingReceiverBalance = (await ExtrinsicHelper.getAccountInfo(receiverKeys)).data.free.toBigInt();
        startingSenderBalance = (await ExtrinsicHelper.getAccountInfo(fundedEthereumKeys)).data.free.toBigInt();
        accountPKey = getUnifiedPublicKey(fundedEthereumKeys);
        log(`accountPKey ${u8aToHex(accountPKey)}`);
        nonce = await getNonce(fundedEthereumKeys);
        const transferCalls = ExtrinsicHelper.api.tx.balances.transferKeepAlive(
          getUnifiedAddress(receiverKeys),
          transferAmount
        );
        const { passKeyPrivateKey, passKeyPublicKey } = createPassKeyAndSignAccount(accountPKey);
        // ethereum keys should not have wrapping
        const accountSignature = fundedEthereumKeys.sign(passKeyPublicKey);
        log(`accountSignature ${u8aToHex(accountSignature)}`);
        const multiSignature: EcdsaSignature = { Ecdsa: u8aToHex(accountSignature) };
        const passkeyCall = await createPassKeyCallV2(accountPKey, nonce, transferCalls);
        const passkeyPayload = await createPasskeyPayloadV2(
          multiSignature,
          passKeyPrivateKey,
          passKeyPublicKey,
          passkeyCall,
          false
        );
        passkeyProxy = ExtrinsicHelper.executePassKeyProxyV2(fundingSource, passkeyPayload);
      });

      it('should successfully execute transfer extrinsic', async function () {
        await assert.doesNotReject(async () => {
          ({
            target,
            eventMap: { 'balances.Transfer': transferEvent, 'balances.Withdraw': feeEvent },
          } = await passkeyProxy.sendUnsigned());
        });
      });

      it('should have received a transaction execution success event', async function () {
        assert.notEqual(target, undefined, 'Target event should not be undefined');
        assert.equal(
          ExtrinsicHelper.api.events.passkey.TransactionExecutionSuccess.is(target),
          true,
          'Target event should be of correct type'
        );
      });

      it('should have debited & credited correct accounts for the correct amount', async function () {
        if (transferEvent && ExtrinsicHelper.api.events.balances.Transfer.is(transferEvent)) {
          const { from, to, amount } = transferEvent.data;
          assert.equal(from.toString(), getUnifiedAddress(fundedEthereumKeys), 'From address should be the funded key');
          assert.equal(to.toString(), getUnifiedAddress(receiverKeys), 'To address should be the receiver key');
          assert.equal(amount.toBigInt(), transferAmount, `Transfer amount should be ${transferAmount}`);
        } else {
          assert.fail('Transfer event not found');
        }
      });

      it('should have deducted fee from the sender', async function () {
        if (feeEvent && ExtrinsicHelper.api.events.balances.Withdraw.is(feeEvent)) {
          const { who, amount } = feeEvent.data;
          assert.equal(who.toString(), getUnifiedAddress(fundedEthereumKeys), 'Fee should be deducted from the sender');
          assert.equal(amount.toBigInt() > 0, true, 'Fee should be greater than 0');
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
        const nonceAfter = (await ExtrinsicHelper.getAccountInfo(fundedEthereumKeys)).nonce.toNumber();
        assert.equal(nonce + 1, nonceAfter, 'Nonce should be incremented by 1');
      });

      it('should have changed account balances correctly', async function () {
        const endingReceiverBalance = (await ExtrinsicHelper.getAccountInfo(receiverKeys)).data.free.toBigInt();
        const endingSenderBalance = (await ExtrinsicHelper.getAccountInfo(fundedEthereumKeys)).data.free.toBigInt();
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
