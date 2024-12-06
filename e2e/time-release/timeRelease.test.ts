import '@frequency-chain/api-augment';
import assert from 'assert';
import { createAndFundKeypair } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper, ReleaseSchedule } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';

const DOLLARS: number = 100000000; // 100_000_000

function getBlocksInMonthPeriod(blockTime: number, periodInMonths: number) {
  const secondsPerMonth = 2592000; // Assuming 30 days in a month

  // Calculate the number of blocks in the given period
  const blocksInPeriod = Math.floor((periodInMonths * secondsPerMonth) / blockTime);

  return blocksInPeriod;
}

function calculateReleaseSchedule(amount: number | bigint): ReleaseSchedule {
  const start = 0;
  const period = getBlocksInMonthPeriod(6, 4);
  const periodCount = 4;

  const perPeriod = BigInt(amount) / BigInt(periodCount);

  return {
    start,
    period,
    periodCount,
    perPeriod,
  };
}

const fundingSource: KeyringPair = getFundingSource(import.meta.url);

describe('TimeRelease', function () {
  let vesterKeys: KeyringPair;

  before(async function () {
    vesterKeys = await createAndFundKeypair(fundingSource, 50_000_000n);
  });

  describe('vested transfer and claim flow', function () {
    it('creates a vested transfer', async function () {
      const amount = 100000n * BigInt(DOLLARS);
      const schedule: ReleaseSchedule = calculateReleaseSchedule(amount);

      const vestedTransferTx = ExtrinsicHelper.timeReleaseTransfer(fundingSource, vesterKeys, schedule);
      const { target } = await vestedTransferTx.signAndSend();
      assert.notEqual(target, undefined, 'should have returned ReleaseScheduleAdded event');
    });
  });
});
