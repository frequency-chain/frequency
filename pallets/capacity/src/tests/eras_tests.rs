use super::{
	mock::*,
	testing_utils::{run_to_block, system_run_to_block},
};
use crate::{Config, CurrentEraInfo, RewardEraInfo, RewardPoolInfo, StakingRewardPool};
use sp_core::Get;

#[test]
fn start_new_era_if_needed_updates_era_info_and_limits_reward_pool_size() {
	new_test_ext().execute_with(|| {
		CurrentEraInfo::<Test>::set(RewardEraInfo { era_index: 1, started_at: 0 });
		StakingRewardPool::<Test>::insert(
			1,
			RewardPoolInfo {
				total_staked_token: 10_000,
				total_reward_pool: 1_000,
				unclaimed_balance: 1_000,
			},
		);
		system_run_to_block(9);
		for i in 1..4 {
			let block_decade = i * 10;
			run_to_block(block_decade);

			let current_era_info = CurrentEraInfo::<Test>::get();

			let expected_era = i + 1;
			assert_eq!(current_era_info.era_index, expected_era);
			assert_eq!(current_era_info.started_at, block_decade);
			let past_eras_max: u32 = <Test as Config>::StakingRewardsPastErasMax::get();
			assert!(StakingRewardPool::<Test>::count().le(&past_eras_max));
			system_run_to_block(block_decade + 9);
		}
	})
}

#[test]
fn start_new_era_if_needed_updates_reward_pool() {
	new_test_ext().execute_with(|| {
		CurrentEraInfo::<Test>::set(RewardEraInfo { era_index: 1, started_at: 0 });
		StakingRewardPool::<Test>::insert(
			1,
			RewardPoolInfo {
				total_staked_token: 10_000,
				total_reward_pool: 1_000,
				unclaimed_balance: 1_000,
			},
		);
		system_run_to_block(8);

		// TODO: Provider boost, after staking updates reward pool info
		// let staker = 10_000;
		// let provider_msa: MessageSourceId = 1;
		// let stake_amount = 600u64;
		// setup_provider(&staker, &provider_msa, &stake_amount, ProviderBoost);

		system_run_to_block(9);
		run_to_block(10);
		assert_eq!(StakingRewardPool::<Test>::count(), 2);
		let current_reward_pool_info = StakingRewardPool::<Test>::get(2).unwrap();
		assert_eq!(
			current_reward_pool_info,
			RewardPoolInfo {
				total_staked_token: 10_000,
				total_reward_pool: 1_000,
				unclaimed_balance: 1_000,
			}
		);

		// TODO: after staking updates reward pool info
		// system_run_to_block(19);
		// run_to_block(20);
	});
}
