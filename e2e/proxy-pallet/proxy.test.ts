import '@frequency-chain/api-augment';
import assert from 'assert';
import { createAndFundKeypair } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import { getUnifiedAddress } from '@frequency-chain/ethereum-utils';

const DOLLARS = 100000000n; // 100_000_000

const fundingSource: KeyringPair = getFundingSource(import.meta.url);

describe('Proxy', function () {
  describe('Basic Any Proxy Successes', function () {
    let stashKeys: KeyringPair;
    let proxyKeys: KeyringPair;

    before(async function () {
      stashKeys = await createAndFundKeypair(fundingSource, 100n * DOLLARS);
      proxyKeys = await createAndFundKeypair(fundingSource, 1n * DOLLARS);
    });

    it('Creates a Proxy', async function () {
      const extrinsic = new Extrinsic(
        () => ExtrinsicHelper.api.tx.proxy.addProxy(getUnifiedAddress(proxyKeys), 'Any', 0),
        stashKeys,
        ExtrinsicHelper.api.events.proxy.ProxyAdded
      );

      const { target } = await extrinsic.signAndSend();
      assert.notEqual(target, undefined, 'should have returned ProxyAdded event');
    });

    it('Sends a transfer', async function () {
      const extrinsic = new Extrinsic(
        () =>
          ExtrinsicHelper.api.tx.proxy.proxy(
            getUnifiedAddress(stashKeys),
            'Any',
            ExtrinsicHelper.api.tx.balances.transferAllowDeath(getUnifiedAddress(proxyKeys), 1n * DOLLARS)
          ),
        proxyKeys,
        ExtrinsicHelper.api.events.balances.Transfer
      );

      const { target } = await extrinsic.signAndSend();
      assert.notEqual(target, undefined, 'should have returned Transfer event');
    });

    it('Can remove the proxy', async function () {
      const extrinsic = new Extrinsic(
        () => ExtrinsicHelper.api.tx.proxy.removeProxy(getUnifiedAddress(proxyKeys), 'Any', 0),
        stashKeys,
        ExtrinsicHelper.api.events.proxy.ProxyRemoved
      );

      const { target } = await extrinsic.signAndSend();
      assert.notEqual(target, undefined, 'should have returned ProxyRemoved event');
    });
  });

  describe('Basic NonTransfer Proxy', function () {
    let stashKeys: KeyringPair;
    let proxyKeys: KeyringPair;

    before(async function () {
      stashKeys = await createAndFundKeypair(fundingSource, 100n * DOLLARS);
      proxyKeys = await createAndFundKeypair(fundingSource, 1n * DOLLARS);
    });

    it('Creates a Proxy', async function () {
      const extrinsic = new Extrinsic(
        () => ExtrinsicHelper.api.tx.proxy.addProxy(getUnifiedAddress(proxyKeys), 'NonTransfer', 0),
        stashKeys,
        ExtrinsicHelper.api.events.proxy.ProxyAdded
      );

      const { target } = await extrinsic.signAndSend();
      assert.notEqual(target, undefined, 'should have returned ProxyAdded event');
    });

    it('CANNOT send a transfer', async function () {
      const extrinsic = new Extrinsic(
        () =>
          ExtrinsicHelper.api.tx.proxy.proxy(
            getUnifiedAddress(stashKeys),
            'Any',
            ExtrinsicHelper.api.tx.balances.transferAllowDeath(getUnifiedAddress(proxyKeys), 1n * DOLLARS)
          ),
        proxyKeys,
        ExtrinsicHelper.api.events.system.ExtrinsicFailed
      );

      // Filtered Out ExtrinsicFailed
      await assert.rejects(extrinsic.signAndSend(), {
        name: 'NotProxy',
      });
    });

    it('CANNOT send a transfer via utility batch', async function () {
      const extrinsic = new Extrinsic(
        () =>
          ExtrinsicHelper.api.tx.proxy.proxy(
            getUnifiedAddress(stashKeys),
            'Any',
            ExtrinsicHelper.api.tx.utility.batch([
              ExtrinsicHelper.api.tx.balances.transferAllowDeath(getUnifiedAddress(proxyKeys), 1n * DOLLARS),
            ])
          ),
        proxyKeys,
        ExtrinsicHelper.api.events.system.ExtrinsicFailed
      );

      // Filtered Out ExtrinsicFailed
      await assert.rejects(extrinsic.signAndSend(), {
        name: 'NotProxy',
      });
    });

    it('Can remove the proxy', async function () {
      const extrinsic = new Extrinsic(
        () => ExtrinsicHelper.api.tx.proxy.removeProxy(getUnifiedAddress(proxyKeys), 'NonTransfer', 0),
        stashKeys,
        ExtrinsicHelper.api.events.proxy.ProxyRemoved
      );

      const { target } = await extrinsic.signAndSend();
      assert.notEqual(target, undefined, 'should have returned ProxyRemoved event');
    });
  });
});
