import '@frequency-chain/api-augment';
import assert from 'assert';
import { DOLLARS, createAndFundKeypair } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import { getUnifiedAddress } from '../scaffolding/ethereum';

const fundingSource: KeyringPair = getFundingSource('frequency-balance-ethereum');

describe('Balance transfer ethereum', function () {
  describe('setup', function () {
    let sr25519Keys: KeyringPair;
    let ethereumKeys: KeyringPair;

    before(async function () {
      sr25519Keys = await createAndFundKeypair(fundingSource, 100n * DOLLARS);
      ethereumKeys = await createAndFundKeypair(fundingSource, 100n * DOLLARS, undefined, undefined, 'ethereum');
    });

    it('should transfer from sr25519 to ethereum style key', async function () {
      const extrinsic = new Extrinsic(
        () => ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedAddress(ethereumKeys), 10n * DOLLARS),
        sr25519Keys,
        ExtrinsicHelper.api.events.balances.Transfer
      );
      const { target } = await extrinsic.signAndSend();
      assert.notEqual(target, undefined, 'should have returned Transfer event');
    });

    it('should transfer from sr25519 to ethereum 20 byte address', async function () {
      const extrinsic = new Extrinsic(
        // this is using MultiAddress::Address20 type in Rust since addressRaw is 20 bytes ethereum address
        () => ExtrinsicHelper.api.tx.balances.transferKeepAlive(ethereumKeys.addressRaw, 10n * DOLLARS),
        sr25519Keys,
        ExtrinsicHelper.api.events.balances.Transfer
      );
      const { target } = await extrinsic.signAndSend();
      assert.notEqual(target, undefined, 'should have returned Transfer event');
    });

    it('should transfer from an ethereum key to sr25519 key', async function () {
      const extrinsic = new Extrinsic(
        () => ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedAddress(sr25519Keys), 30n * DOLLARS),
        ethereumKeys,
        ExtrinsicHelper.api.events.balances.Transfer
      );
      const { target } = await extrinsic.signAndSend();
      assert.notEqual(target, undefined, 'should have returned Transfer event');
    });
  });
});
