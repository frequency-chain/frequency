use super::mock::*;
use crate::{
	tests::testing_utils::{run_to_block, setup_provider, system_run_to_block},
	Config, CurrentEraInfo, RewardEraInfo, RewardPoolInfo, StakingRewardPool,
	StakingType::*,
};
use common_primitives::msa::MessageSourceId;
use frame_support::assert_ok;
use sp_core::Get;

#[test]
fn start_new_era_if_needed_updates_era_info() {
	new_test_ext().execute_with(|| {
		system_run_to_block(9);
		for i in 1..=4 {
			let block_decade = (i * 10) + 1;
			run_to_block(block_decade);

			let current_era_info = CurrentEraInfo::<Test>::get();

			let expected_era = i + 1;
			assert_eq!(
				current_era_info,
				RewardEraInfo { era_index: expected_era, started_at: block_decade }
			);

			let past_eras_max: u32 = <Test as Config>::StakingRewardsPastErasMax::get();
			assert!(StakingRewardPool::<Test>::count().le(&past_eras_max));
			system_run_to_block(block_decade + 9);
		}
	})
}

// Test that additional stake is carried over to the next era's RewardPoolInfo.
#[test]
fn start_new_era_if_needed_updates_reward_pool() {
	new_test_ext().execute_with(|| {
		system_run_to_block(8);
		let staker = 10_000;
		let provider_msa: MessageSourceId = 1;
		let stake_amount = 600u64;

		// set up initial stake.
		setup_provider(&staker, &provider_msa, &stake_amount, ProviderBoost);

		for i in 1u32..4 {
			let era = i + 1;
			let final_block = (i * 10) + 1;
			system_run_to_block(final_block - 1);
			run_to_block(final_block);
			assert_eq!(StakingRewardPool::<Test>::count(), era);
			assert_eq!(
				StakingRewardPool::<Test>::get(era).unwrap(),
				RewardPoolInfo {
					total_staked_token: (stake_amount * i as u64),
					total_reward_pool: 10_000,
					unclaimed_balance: 10_000,
				}
			);
			assert_ok!(Capacity::provider_boost(
				RuntimeOrigin::signed(staker),
				provider_msa,
				stake_amount
			));
		}
	});
}
