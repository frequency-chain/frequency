import '@frequency-chain/api-augment';
import assert from 'assert';
import type { KeyringPair } from '@polkadot/keyring/types';
import { getFundingSource } from '../scaffolding/funding';
import {
  createKeys,
  createMsaAndProvider,
  CENTS,
  DOLLARS,
  createAndFundKeypair,
  boostProvider,
  stakeToProvider,
  addProxy,
  getBlockNumber,
  getFreeBalance,
  getNextRewardEraBlock,
} from '../scaffolding/helpers';
import { Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { createKeyMulti, encodeMultiAddress, sortAddresses } from '@polkadot/util-crypto';
import { getUnifiedAddress } from '@frequency-chain/ethereum-utils';
import { hexAddPrefix } from '@polkadot/util';
import { PalletCapacityStakingDetails } from '@polkadot/types/lookup';
import { firstValueFrom } from 'rxjs';
let fundingSource: KeyringPair;
const tokenMinStake: bigint = 1n * CENTS;

describe('Capacity: provider_boost extrinsic', function () {
  const providerBalance = 2n * DOLLARS;

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);
  });

  it('An account can do a simple provider boost call', async function () {
    const stakeKeys = createKeys('booster');
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);
    const booster = await createAndFundKeypair(fundingSource, 5n * DOLLARS, 'booster');
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
  });

  it('fails when staker is a Maximized Capacity staker', async function () {
    const stakeKeys = createKeys('booster');
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);
    await assert.doesNotReject(stakeToProvider(fundingSource, stakeKeys, provider, tokenMinStake));
    await assert.rejects(boostProvider(fundingSource, stakeKeys, provider, tokenMinStake), {
      name: 'CannotChangeStakingType',
    });
  });

  it("fails when staker doesn't have enough token", async function () {
    const stakeKeys = createKeys('booster');
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);
    const booster = await createAndFundKeypair(fundingSource, 1n * DOLLARS, 'booster');
    await assert.rejects(boostProvider(booster, booster, provider, 1n * DOLLARS), { name: 'BalanceTooLowtoStake' });
  });

  it('staker can boost multiple times', async function () {
    const stakeKeys = createKeys('booster');
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);
    const booster = await createAndFundKeypair(fundingSource, 10n * DOLLARS, 'booster');
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
    await assert.doesNotReject(boostProvider(fundingSource, booster, provider, 1n * DOLLARS));
  });

  it('can boost, claim, and unstake via proxy account', async function () {
    const boostAmount = 2n * DOLLARS;

    const alice = await createAndFundKeypair(fundingSource, 1000n * DOLLARS, 'alice');
    const bob = await createAndFundKeypair(fundingSource, 2000n * DOLLARS, 'bob');

    const stakeKeys = createKeys('booster');
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);

    await addProxy(bob, getUnifiedAddress(alice), 'Staking');
    const innerBoost = ExtrinsicHelper.api.tx.capacity.providerBoost(provider, boostAmount);
    const expectedEvent = ExtrinsicHelper.api.events.capacity.ProviderBoosted;
    await ExtrinsicHelper.proxySignAndSend(innerBoost, alice, bob, expectedEvent);

    // Verify that the stakingLedger reflects that bob is staking `boostAmount`
    const stakingLedgerRes = await ExtrinsicHelper.apiPromise.query.capacity.stakingAccountLedger(
      getUnifiedAddress(bob)
    );
    assert(stakingLedgerRes.isSome);
    const stakingLedger = stakingLedgerRes.unwrap();
    assert(stakingLedger.active.eq(boostAmount));

    // Move out of the era we boosted inside of
    await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());
    // Have three eras where we get rewards
    await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());
    await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());
    await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());

    const bobBalance = await getFreeBalance(bob);
    const aliceBalance = await getFreeBalance(alice);

    const innerClaim = ExtrinsicHelper.api.tx.capacity.claimStakingRewards();
    const expectedClaimEvent = ExtrinsicHelper.api.events.capacity.ProviderBoostRewardClaimed;
    await ExtrinsicHelper.proxySignAndSend(innerClaim, alice, bob, expectedClaimEvent);

    // Verify that alice's proxy call cost balance and alice account does not get rewards.
    const aliceNewBalance = await getFreeBalance(alice);
    assert(
      aliceNewBalance < aliceBalance,
      `expected alice not to get claimed rewards. Alice new - old = ${aliceNewBalance - aliceBalance}`
    );

    // assert that bob got the claimed rewards.
    const bobNewBalance = await getFreeBalance(bob);
    assert(bobNewBalance > bobBalance, `expected bob to get rewards. Bob new - old = ${bobNewBalance - bobBalance}`);

    const innerUnstake = ExtrinsicHelper.api.tx.capacity.unstake(1, boostAmount);
    const expectedUnstakeEvent = ExtrinsicHelper.api.events.capacity.UnStaked;
    await ExtrinsicHelper.proxySignAndSend(innerUnstake, alice, bob, expectedUnstakeEvent);
  });

  it('multisig call as a proxy works for capacity staking ops', async function () {
    // Input the addresses that will make up the multisig account.
    const alice = await createAndFundKeypair(fundingSource, 2000n * DOLLARS, 'alice');
    const bob = await createAndFundKeypair(fundingSource, 2000n * DOLLARS, 'bob');
    const charlie = await createAndFundKeypair(fundingSource, 2000n * DOLLARS, 'charlie');
    const fergie = await createAndFundKeypair(fundingSource, 2000n * DOLLARS, 'fergie');

    const stakeKeys = createKeys('booster');
    const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);

    const owners = [getUnifiedAddress(alice), getUnifiedAddress(bob), getUnifiedAddress(charlie)];

    // The number of accounts that must approve. Must be greater than 0 and less than
    // or equal to the total number of addresses.
    const threshold = 2;

    // Convert byte array to SS58 encoding.
    const ss58MultiAddress = encodeMultiAddress(owners, threshold);

    await addProxy(fergie, ss58MultiAddress, 'Any');
    const api = ExtrinsicHelper.api;

    // 2) Build inner call: Fergie boosts 'Provider1'
    const boostAmount = 2n * DOLLARS;
    const inner = api.tx.capacity.providerBoost(provider, boostAmount);

    // 3) Wrap with proxy: multisig will act as Fergie
    const proxyCall = api.tx.proxy.proxy(
      getUnifiedAddress(fergie), // real
      null, // forceProxyType = None (use Any granted above)
      inner
    );

    // Helpers
    // the weight for this is pretty high
    const weight = api.createType('Weight', { refTime: 1_584_641_224, proofSize: 27000 });
    const sort = (all: string[], who: string) => sortAddresses(all.filter((addr) => addr !== who));

    // 4) First approval by Alice (creates the multisig record)
    const othersForAlice = sort(owners, owners[0]);
    let timepoint: any | null = null;

    const multiSigExtAlice = new Extrinsic(
      () => api.tx.multisig.asMulti(threshold, othersForAlice, timepoint, proxyCall.method, weight),
      alice,
      api.events.multisig.MultisigExecuted
    );
    await assert.doesNotReject(multiSigExtAlice.signAndSend());

    const proxyCallHash = proxyCall.method.hash;

    const infoOpt = await ExtrinsicHelper.apiPromise.query.multisig.multisigs(ss58MultiAddress, proxyCallHash);
    assert(infoOpt.isSome);
    timepoint = infoOpt.unwrap().when; // { height, index }

    // 5) Second approval by Charlie (executes the call)
    const othersForCharlie = sort(owners, owners[2]);

    const multiSigExtCharlie = new Extrinsic(
      () => api.tx.multisig.asMulti(threshold, othersForCharlie, timepoint, proxyCall.method, weight),
      charlie,
      api.events.multisig.MultisigExecuted
    );
    await assert.doesNotReject(multiSigExtCharlie.signAndSend());

    // check that Fergie has a staking account with the expected amount in it.
    const ledgerResp = await ExtrinsicHelper.apiPromise.query.capacity.stakingAccountLedger(getUnifiedAddress(fergie));
    assert(ledgerResp.isSome);
    const ledger: PalletCapacityStakingDetails = ledgerResp.unwrap();
    assert(ledger.active.eq(boostAmount));
  });
});
