import '@frequency-chain/api-augment';
import assert from 'assert';
import { createAndFundKeypair, getBlockNumber, calculateReleaseSchedule } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper, ReleaseSchedule } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';

const DOLLARS: number = 100000000; // 100_000_000

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

  describe('Schedule transfers', function () {
    it('create a schedule transfer and cancel', async function () {
      const amount = 100000n * BigInt(DOLLARS);
      const schedule: ReleaseSchedule = calculateReleaseSchedule(amount);
      const currentBlock = await getBlockNumber();

      const scheduleName1 = new Uint8Array(32).fill(1);
      const scheduleName2 = new Uint8Array(32).fill(2);

      const scheduleTransferTx1 = ExtrinsicHelper.timeReleaseScheduleNamedTransfer(
        fundingSource,
        scheduleName1,
        vesterKeys,
        schedule,
        currentBlock + 20
      );
      const scheduleTransferTx2 = ExtrinsicHelper.timeReleaseScheduleNamedTransfer(
        fundingSource,
        scheduleName2,
        vesterKeys,
        schedule,
        currentBlock + 20
      );
      const { target: target1 } = await scheduleTransferTx1.signAndSend();
      const { target: target2 } = await scheduleTransferTx2.signAndSend();
      assert.notEqual(target1, undefined, 'should have returned Scheduled event');
      assert.notEqual(target2, undefined, 'should have returned Scheduled event');

      const reservedAmountBefore =
        await ExtrinsicHelper.apiPromise.query.timeRelease.scheduleReservedAmounts(scheduleName1);

      assert.equal('10,000,000,000,000', reservedAmountBefore.toHuman());

      await ExtrinsicHelper.runToBlock(currentBlock + 2);

      const cancelScheduleTransferTx1 = ExtrinsicHelper.timeReleaseCancelScheduledNamedTransfer(
        fundingSource,
        scheduleName1
      );
      const { target: cancelTarget1 } = await cancelScheduleTransferTx1.signAndSend();
      assert.notEqual(cancelTarget1, undefined, 'should have returned scheduler Canceled event');

      const reservedAmountAfter =
        await ExtrinsicHelper.apiPromise.query.timeRelease.scheduleReservedAmounts(scheduleName1);
      assert.equal(true, reservedAmountAfter.isEmpty, 'reserved amount should be empty');

      const cancelScheduleTransferTx2 = ExtrinsicHelper.timeReleaseCancelScheduledNamedTransfer(
        fundingSource,
        scheduleName2
      );

      const { target: cancelTarget2 } = await cancelScheduleTransferTx2.signAndSend();

      assert.notEqual(cancelTarget2, undefined, 'should have returned scheduler Canceled event');
    });
  });

  describe('create schedule', function () {
    it('create a schedule transfer', async function () {
      const amount = 100000n * BigInt(DOLLARS);
      const schedule: ReleaseSchedule = calculateReleaseSchedule(amount);
      const currentBlock = await getBlockNumber();

      const scheduleName1 = new Uint8Array(32).fill(1);

      const scheduleTransferTx1 = ExtrinsicHelper.timeReleaseScheduleNamedTransfer(
        fundingSource,
        scheduleName1,
        vesterKeys,
        schedule,
        currentBlock + 10
      );
      const { target: target1 } = await scheduleTransferTx1.signAndSend();
      assert.notEqual(target1, undefined, 'should have returned Scheduled event');

      const reservedAmountBefore =
        await ExtrinsicHelper.apiPromise.query.timeRelease.scheduleReservedAmounts(scheduleName1);

      assert.equal('10,000,000,000,000', reservedAmountBefore.toHuman());

      await ExtrinsicHelper.runToBlock(currentBlock + 15);

      const reservedAmountAfter =
        await ExtrinsicHelper.apiPromise.query.timeRelease.scheduleReservedAmounts(scheduleName1);
      assert.equal(true, reservedAmountAfter.isEmpty, 'reserved amount should be empty');
    });
  });
});
