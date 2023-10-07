use super::{mock::*,
			testing_utils::{run_to_block, system_run_to_block, setup_provider}};
use crate::{CurrentEraInfo, RewardEraInfo, StakingRewardPool, RewardPoolInfo};

#[test]
fn start_new_era_if_needed_updates_reward_pool_and_era_info() {
	new_test_ext().execute_with(|| {
		CurrentEraInfo::<Test>::set(RewardEraInfo { era_index: 1, started_at: 0 });
		StakingRewardPool::<Test>::insert(1, RewardPoolInfo {
			total_staked_token: 10_000,
			total_reward_pool: 1_000,
			unclaimed_balance: 1_000,
		});
		system_run_to_block(8);

		// TODO: Provider boost, after staking updates reward pool info
		// let staker = 10_000;
		// let provider_msa: MessageSourceId = 1;
		// let stake_amount = 600u64;
		// setup_provider(&staker, &provider_msa, &stake_amount, ProviderBoost);

		system_run_to_block(9);
		run_to_block(10);
		let mut current_era_info = CurrentEraInfo::<Test>::get();
		assert_eq!(current_era_info.era_index, 2u32);
		assert_eq!(current_era_info.started_at, 10u32);

		assert_eq!(StakingRewardPool::<Test>::count(), 2);
		let current_reward_pool_info = StakingRewardPool::<Test>::get(2).unwrap();
		assert_eq!(current_reward_pool_info, RewardPoolInfo {
			total_staked_token: 10_000,
			total_reward_pool: 1_000,
			unclaimed_balance: 1_000,
		});

		system_run_to_block(19);
		run_to_block(20);
		current_era_info = CurrentEraInfo::<Test>::get();
        assert_eq!(current_era_info.era_index, 3u32);
		assert_eq!(current_era_info.started_at, 20u32);
	})
}
