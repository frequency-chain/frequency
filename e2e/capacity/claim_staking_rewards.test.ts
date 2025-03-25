import '@frequency-chain/api-augment';
import assert from 'assert';
import {getFundingSource} from '../scaffolding/funding';
import {
    createKeys,
    createMsaAndProvider,
    CENTS,
    DOLLARS,
    createAndFundKeypair,
    boostProvider,
    stakeToProvider, getNextRewardEraBlock, getBlockNumber,
} from '../scaffolding/helpers';
import {KeyringPair} from "@polkadot/keyring/types";
import {ExtrinsicHelper} from "../scaffolding/extrinsicHelpers";
import {getUnifiedAddress} from "../scaffolding/ethereum";

const fundingSource = getFundingSource(import.meta.url);

describe('Capacity: claim_staking_rewards', function () {
    const setUpForBoosting = async (boosterName: string, providerName: string, amountInDollars: bigint): Promise<[number, KeyringPair]> => {
        const stakeKeys = createKeys('booster');
        const providerBalance = 2n * DOLLARS;
        const provider = await createMsaAndProvider(fundingSource, stakeKeys, 'Provider1', providerBalance);
        const booster = await createAndFundKeypair(fundingSource, 10n * DOLLARS, 'booster');
        await assert.doesNotReject(boostProvider(fundingSource, booster, provider, amountInDollars * DOLLARS));

        return [provider.toNumber(), booster];
    };

    it("boost, wait 2 eras, claim, then unstake does not fail", async function () {
        //Provider boost 4mil
        const [provider, booster] = await setUpForBoosting('booster2', 'provider2', 4n);
        const boosterAddr = getUnifiedAddress(booster);

        let result = await ExtrinsicHelper.apiPromise.query.capacity.stakingAccountLedger(boosterAddr);
        const startingAmount = result.unwrap().active.toNumber();
        assert.equal(startingAmount, 4n * DOLLARS);

        // Advance to a block where there are rewards available
        await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());
        await ExtrinsicHelper.runToBlock(await getNextRewardEraBlock());

        // Claim rewards
        const claimRewardsOp = ExtrinsicHelper.claimStakingRewards(booster);
        const {target: claimRewardsEvent} = await claimRewardsOp.fundAndSend(fundingSource);
        assert.notEqual(claimRewardsEvent, undefined, 'claimStakingRewards: should have returned a ProviderBoostRewardsClaimed event');
        const rewardAmount = claimRewardsEvent?.data.rewardAmount || 0;
        assert.notEqual(0, rewardAmount);

        // Unboost 1mil from provider—this succeeds
        const unstakeOp = ExtrinsicHelper.unstake(booster, provider, 1n * DOLLARS);
        assert.doesNotReject(unstakeOp.fundAndSend(fundingSource));
        result = await ExtrinsicHelper.apiPromise.query.capacity.stakingAccountLedger(boosterAddr);

        const afterUnstakeAmount = result.unwrap().active.toNumber();
        assert.equal(3n * DOLLARS, afterUnstakeAmount);
    });
});
