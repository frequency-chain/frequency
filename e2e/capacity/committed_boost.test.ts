/* eslint-disable no-restricted-syntax */
import '@frequency-chain/api-augment';
import { KeyringPair } from '@polkadot/keyring/types';
import { u64 } from '@polkadot/types';
import assert from 'assert';
import { Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  createKeys,
  createMsaAndProvider,
  CENTS,
  DOLLARS,
  createAndFundKeypair,
  committedBoostProvider,
} from '../scaffolding/helpers';
import { getFundingSource, getSudo } from '../scaffolding/funding';
import { isTestnet } from '../scaffolding/env';
import { before as mochaBefore, it as mochaIt } from 'mocha';

const fundingSource = getFundingSource(import.meta.url);

// Hard-coded dev values from the runtime
const failsafeBlockNumber = 2000;
const initialCommitmentBlocks = 10;
const blocksPerStage = 10;
const numStages = 4n;

// eslint-disable-next-line @typescript-eslint/no-empty-function
const before = isTestnet() ? () => {} : mochaBefore;
const it = isTestnet() ? mochaIt.skip : mochaIt;

describe('Committed Boost Unstaking Tests', function () {
  const initialStake = 1000n * DOLLARS;
  let alwaysUnstakeKeys: KeyringPair;
  let neverUnstakeKeys: KeyringPair;
  let providerId: u64;

  before(async function () {
    const accountBalance: bigint = 1010n * DOLLARS;
    alwaysUnstakeKeys = await createAndFundKeypair(fundingSource, accountBalance, 'unstakeKeys');
    neverUnstakeKeys = await createAndFundKeypair(fundingSource, accountBalance, 'neverUnstakeKeys');
    const providerKeys = createKeys('providerKeys');
    providerId = await createMsaAndProvider(fundingSource, providerKeys, 'Committed Boost', 100n * CENTS);

    // Make sure that the PTE block is not set
    const pteKey = ExtrinsicHelper.apiPromise.query.capacity.precipitatingEventBlockNumber.key();
    const op = new Extrinsic(() => ExtrinsicHelper.api.tx.system.killStorage([pteKey]), getSudo().keys);
    await assert.doesNotReject(() => op.sudoSignAndSend(), 'should have unset PTE');
    await assert.doesNotReject(
      async () => committedBoostProvider(fundingSource, alwaysUnstakeKeys, providerId, initialStake),
      'should have staked to provider'
    );
    await assert.doesNotReject(
      async () => committedBoostProvider(fundingSource, neverUnstakeKeys, providerId, initialStake),
      'should have staked to provider'
    );
  });

  describe('PreCommitment phase', function () {
    it('should not be able to unstake in PreCommitment phase', async function () {
      await assert.rejects(
        async () => ExtrinsicHelper.unstake(alwaysUnstakeKeys, providerId, 1).fundAndSend(fundingSource),
        /InsufficientUnfrozenStakingBalance/,
        'should have rejected unstake in PreCommitment phase'
      );
    });
  });

  describe('During Committed Boost program', function () {
    let pteBlockNumber: number;

    before(async function () {
      pteBlockNumber = (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber();
      const op = new Extrinsic(
        () => ExtrinsicHelper.api.tx.capacity.setPteViaGovernance(pteBlockNumber),
        getSudo().keys
      );
      await assert.doesNotReject(async () => op.sudoSignAndSend(), 'should have set PTE');
    });

    it('should not be able to unstake during InitialCommitment phase', async function () {
      const unstakableAmount = await ExtrinsicHelper.apiPromise.call.capacityRuntimeApi.getUnstakableEligibleAmount(
        alwaysUnstakeKeys.address
      );
      assert.equal(unstakableAmount.toNumber(), 0);
      await assert.rejects(
        async () => ExtrinsicHelper.unstake(alwaysUnstakeKeys, providerId, 1).fundAndSend(fundingSource),
        /InsufficientUnfrozenStakingBalance/,
        'should have rejected unstake in InitialCommitment phase'
      );
    });

    describe('StagedRelease phase', function () {
      let currentBlockNumber: number;
      let runToBlockNumber: number;

      before(async function () {
        currentBlockNumber = (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber();
        runToBlockNumber = pteBlockNumber + initialCommitmentBlocks;
        // set block to the end of the InitialCommitment phase
        await ExtrinsicHelper.runToBlock(runToBlockNumber);
      });

      describe('Staking in StagedRelease phase', function () {
        it('should no longer be able to stake once in StagedRelease phase', async function () {
          await assert.rejects(
            () => committedBoostProvider(fundingSource, alwaysUnstakeKeys, providerId, 1n * DOLLARS),
            /capacity.CommittedBoostStakingPeriodPassed/,
            'should have rejected stake in StagedRelease phase'
          );
        });
      });

      describe('Unstaking in StagedRelease phase', function () {
        [
          {
            stage: 1,
            amountReleasable: initialStake / numStages,
            cumulativeAmountReleasable: initialStake / numStages,
          },
          {
            stage: 2,
            amountReleasable: initialStake / numStages,
            cumulativeAmountReleasable: (2n * initialStake) / numStages,
          },
          {
            stage: 3,
            amountReleasable: initialStake / numStages,
            cumulativeAmountReleasable: (3n * initialStake) / numStages,
          },
          { stage: 4, amountReleasable: initialStake / numStages, cumulativeAmountReleasable: initialStake },
        ].forEach(async function (stage) {
          it(`should be able to unstake the appropriate amount at stage ${stage.stage}`, async function () {
            const releasedAmount = (
              await ExtrinsicHelper.apiPromise.call.capacityRuntimeApi.getUnstakableEligibleAmount(
                alwaysUnstakeKeys.address
              )
            ).toBigInt();
            const cumulativeReleasedAmount = (
              await ExtrinsicHelper.apiPromise.call.capacityRuntimeApi.getUnstakableEligibleAmount(
                neverUnstakeKeys.address
              )
            ).toBigInt();
            assert.equal(
              releasedAmount,
              stage.amountReleasable,
              `should have released ${stage.amountReleasable} at stage ${stage.stage} `
            );
            assert.equal(
              cumulativeReleasedAmount,
              stage.cumulativeAmountReleasable,
              `should have cumulatively released ${stage.cumulativeAmountReleasable} at stage ${stage.stage} `
            );
            // Make sure unstakers do not have unclaimed rewards so we can unstake
            let unclaimedRewards = await ExtrinsicHelper.apiPromise.call.capacityRuntimeApi.listUnclaimedRewards(
              alwaysUnstakeKeys.address
            );
            if (!unclaimedRewards.isEmpty) {
              await assert.doesNotReject(() =>
                ExtrinsicHelper.claimRewards(alwaysUnstakeKeys).fundAndSend(fundingSource)
              );
            }
            unclaimedRewards = await ExtrinsicHelper.apiPromise.call.capacityRuntimeApi.listUnclaimedRewards(
              neverUnstakeKeys.address
            );
            if (!unclaimedRewards.isEmpty) {
              await assert.doesNotReject(() =>
                ExtrinsicHelper.claimRewards(neverUnstakeKeys).fundAndSend(fundingSource)
              );
            }
            await assert.doesNotReject(
              () =>
                ExtrinsicHelper.unstake(alwaysUnstakeKeys, providerId, stage.amountReleasable).fundAndSend(
                  fundingSource
                ),
              'should have successfully unstaked'
            );
            await assert.rejects(
              () =>
                ExtrinsicHelper.unstake(
                  neverUnstakeKeys,
                  providerId,
                  stage.cumulativeAmountReleasable + 1n
                ).fundAndSend(fundingSource),
              BigInt(stage.stage) % numStages === 0n
                ? /capacity.InsufficientStakingBalance/
                : /capacity.InsufficientUnfrozenStakingBalance/,
              'should have failed to unstake more than the cumulative amount released'
            );
          });
        });

        afterEach(async function () {
          // advance to the next release stage
          currentBlockNumber = (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber();
          const currentStage = Math.floor(
            (currentBlockNumber - pteBlockNumber - initialCommitmentBlocks) / blocksPerStage
          );
          runToBlockNumber = pteBlockNumber + initialCommitmentBlocks + (currentStage + 1) * blocksPerStage;
          // set block to the end of the InitialCommitment phase
          await ExtrinsicHelper.runToBlock(runToBlockNumber);
        });
      });
    });
  });

  describe('Failsafe', function () {
    before(async function () {
      // unset PTE block
      const pteKey = ExtrinsicHelper.apiPromise.query.capacity.precipitatingEventBlockNumber.key();
      const op = new Extrinsic(() => ExtrinsicHelper.api.tx.system.killStorage([pteKey]), getSudo().keys);
      await assert.doesNotReject(() => op.sudoSignAndSend(), 'should have unset PTE');
    });

    it('should allow all to be unstaked after failsafe', async function () {
      await ExtrinsicHelper.runToBlock(failsafeBlockNumber);
      const releasableAmount = await ExtrinsicHelper.apiPromise.call.capacityRuntimeApi.getUnstakableEligibleAmount(
        neverUnstakeKeys.address
      );
      assert.equal(releasableAmount, initialStake, 'entire initial stake should be releasable');
      const unclaimedRewards = await ExtrinsicHelper.apiPromise.call.capacityRuntimeApi.listUnclaimedRewards(
        neverUnstakeKeys.address
      );
      if (!unclaimedRewards.isEmpty) {
        await assert.doesNotReject(() => ExtrinsicHelper.claimRewards(neverUnstakeKeys).fundAndSend(fundingSource));
      }
      await assert.doesNotReject(
        () => ExtrinsicHelper.unstake(neverUnstakeKeys, providerId, initialStake).fundAndSend(fundingSource),
        'should have successfully unstaked'
      );
    });
  });
});
