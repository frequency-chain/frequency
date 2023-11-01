use super::{
	mock::*,
	testing_utils::{run_to_block, set_era_and_reward_pool_at_block, system_run_to_block},
};
use crate::{
	tests::testing_utils::setup_provider, Config, CurrentEraInfo, RewardEraInfo, RewardPoolInfo,
	StakingRewardPool,
};
use common_primitives::{capacity::StakingType::ProviderBoost, msa::MessageSourceId};
use sp_core::Get;

#[test]
fn start_new_era_if_needed_updates_era_info_and_limits_reward_pool_size() {
	new_test_ext().execute_with(|| {
		set_era_and_reward_pool_at_block(1, 0, 10_000);
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
		set_era_and_reward_pool_at_block(1, 0, 10_000);
		system_run_to_block(8);
		let staker = 10_000;
		let provider_msa: MessageSourceId = 1;
		let stake_amount = 600u64;
		setup_provider(&staker, &provider_msa, &stake_amount, ProviderBoost);

		system_run_to_block(9);
		run_to_block(10);
		assert_eq!(StakingRewardPool::<Test>::count(), 2);
		assert_eq!(
			StakingRewardPool::<Test>::get(2).unwrap(),
			RewardPoolInfo {
				total_staked_token: 10_600,
				total_reward_pool: 0,
				unclaimed_balance: 0,
			}
		);

		// TODO: after staking updates reward pool info #1699
		system_run_to_block(19);
		run_to_block(20);
		assert_eq!(StakingRewardPool::<Test>::count(), 3);
		assert_eq!(
			StakingRewardPool::<Test>::get(2).unwrap(),
			RewardPoolInfo {
				total_staked_token: 10_600,
				total_reward_pool: 1060,
				unclaimed_balance: 1060,
			}
		);
		assert_eq!(
			StakingRewardPool::<Test>::get(3).unwrap(),
			RewardPoolInfo {
				total_staked_token: 10_600,
				total_reward_pool: 0,
				unclaimed_balance: 0,
			}
		);
	});
}
