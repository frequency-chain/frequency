import '@frequency-chain/api-augment';
import assert from 'assert';
import { DOLLARS, createAndFundKeypair, getBlockNumber, getNonce } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import { u8, Option } from '@polkadot/types';
import { u8aToHex } from '@polkadot/util/u8a/toHex';
import { getUnifiedAddress, getUnifiedPublicKey } from '@frequency-chain/ethereum-utils';

const fundingSource: KeyringPair = getFundingSource(import.meta.url);

describe('Frequency', function () {
  describe('setup', function () {
    let keypairA: KeyringPair;
    let keypairB: KeyringPair;

    before(async function () {
      keypairA = await createAndFundKeypair(fundingSource, 100n * DOLLARS);
      keypairB = await createAndFundKeypair(fundingSource, 100n * DOLLARS);
    });

    it('Get events successfully', async function () {
      const balance_pallet = new u8(ExtrinsicHelper.api.registry, 10);
      const transfer_event = new u8(ExtrinsicHelper.api.registry, 2);
      const dest_account = u8aToHex(getUnifiedPublicKey(keypairB)).slice(2);
      const beforeBlockNumber = await getBlockNumber();

      const extrinsic = new Extrinsic(
        () => ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedAddress(keypairB), 1n * DOLLARS),
        keypairA,
        ExtrinsicHelper.api.events.balances.Transfer
      );
      const { target } = await extrinsic.signAndSend();
      assert.notEqual(target, undefined, 'should have returned Transfer event');

      const afterBlockNumber = await getBlockNumber();
      let found = false;

      for (let i = beforeBlockNumber + 1; i <= afterBlockNumber; i++) {
        const block = await ExtrinsicHelper.apiPromise.rpc.chain.getBlockHash(i);
        const events = await ExtrinsicHelper.getFrequencyEvents(block);
        if (
          events.find(
            (e) => e.pallet.eq(balance_pallet) && e.event.eq(transfer_event) && e.data.toHex().includes(dest_account)
          )
        ) {
          found = true;
          break;
        }
      }

      assert(found, 'Could not find the desired event');
    });

    it('Get missing nonce successfully', async function () {
      const nonce = await getNonce(keypairB);
      for (let i = 0; i < 10; i += 2) {
        const extrinsic = new Extrinsic(
          () => ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedAddress(keypairA), 1n * DOLLARS),
          keypairB,
          ExtrinsicHelper.api.events.balances.Transfer
        );
        // intentionally we don't want an await here
        extrinsic.signAndSend(nonce + i);
      }
      // wait a little for all of the above transactions to get queued
      await new Promise((resolve) => setTimeout(resolve, 1000));
      const missingNonce = await ExtrinsicHelper.getMissingNonceValues(getUnifiedPublicKey(keypairB));
      assert.equal(missingNonce.length, 4, 'Could not get missing nonce values');

      // applying the missing nonce values to next transactions to unblock the stuck ones
      for (const missing of missingNonce) {
        const extrinsic = new Extrinsic(
          () => ExtrinsicHelper.api.tx.balances.transferKeepAlive(getUnifiedAddress(keypairA), 1n * DOLLARS),
          keypairB,
          ExtrinsicHelper.api.events.balances.Transfer
        );
        const { target } = await extrinsic.signAndSend(missing.toNumber());
        assert.notEqual(target, undefined, 'should have returned Transfer event');
      }
    });
  });
});
