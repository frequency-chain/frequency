import '@frequency-chain/api-augment';
import assert from 'assert';
import { DOLLARS, createAndFundKeypair, createKeys } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import { getUnifiedAddress } from '../scaffolding/ethereum';

const fundingSource: KeyringPair = getFundingSource(import.meta.url);

describe('Balance transfer ethereum', function () {
  describe('setup', function () {
    let senderSr25519Keys: KeyringPair;
    let senderEthereumKeys: KeyringPair;
    let ethereumKeys: KeyringPair;
    let ethereumKeys2: KeyringPair;
    let sr25519Keys: KeyringPair;

    before(async function () {
      senderSr25519Keys = await createAndFundKeypair(fundingSource, 30n * DOLLARS);
      senderEthereumKeys = await createAndFundKeypair(fundingSource, 30n * DOLLARS, undefined, undefined, 'ethereum');
      ethereumKeys = createKeys('balance-key-1', 'ethereum');
      ethereumKeys2 = createKeys('balance-key-2', 'ethereum');
      sr25519Keys = createKeys('another-sr25519', 'sr25519');
    });

    it('should transfer from sr25519 to ethereum style key', async function () {
      const transferAmount = 10n * DOLLARS;
      const extrinsic = new Extrinsic(
        () => ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedAddress(ethereumKeys), transferAmount),
        senderSr25519Keys,
        ExtrinsicHelper.api.events.balances.Transfer
      );
      const { target } = await extrinsic.signAndSend();
      assert.notEqual(target, undefined, 'should have returned Transfer event');
      const accountInfo = await ExtrinsicHelper.getAccountInfo(ethereumKeys);
      assert(accountInfo.data.free.toBigInt() >= transferAmount);
    });

    it('should transfer from sr25519 to ethereum 20 byte address', async function () {
      const transferAmount = 10n * DOLLARS;
      const extrinsic = new Extrinsic(
        // this is using MultiAddress::Address20 type in Rust since addressRaw is 20 bytes ethereum address
        () => ExtrinsicHelper.api.tx.balances.transferKeepAlive(ethereumKeys2.addressRaw, transferAmount),
        senderSr25519Keys,
        ExtrinsicHelper.api.events.balances.Transfer
      );
      const { target } = await extrinsic.signAndSend();
      assert.notEqual(target, undefined, 'should have returned Transfer event');
      const accountInfo = await ExtrinsicHelper.getAccountInfo(ethereumKeys2);
      assert(accountInfo.data.free.toBigInt() >= transferAmount);
    });

    it('should transfer from an ethereum key to sr25519 key', async function () {
      const transferAmount = 10n * DOLLARS;
      const extrinsic = new Extrinsic(
        () => ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedAddress(sr25519Keys), transferAmount),
        senderEthereumKeys,
        ExtrinsicHelper.api.events.balances.Transfer
      );
      const { target } = await extrinsic.signAndSend();
      assert.notEqual(target, undefined, 'should have returned Transfer event');
      const accountInfo = await ExtrinsicHelper.getAccountInfo(sr25519Keys);
      assert(accountInfo.data.free.toBigInt() >= transferAmount);
    });
  });
});
