use super::mock::*;
use crate::{
	tests::testing_utils::{run_to_block, setup_provider, system_run_to_block},
	Config, CurrentEraInfo, ProviderBoostRewardPool, RewardEraInfo, RewardPoolHistoryChunk,
	StakingType::*,
};
use common_primitives::msa::MessageSourceId;
use frame_support::{assert_ok, traits::Len};
use log::Level::Error;
use sp_core::Get;

#[test]
fn start_new_era_if_needed_updates_era_info() {
	todo!();
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

			let past_eras_max: u32 = <Test as Config>::ProviderBoostHistoryLimit::get();
			// assert!(ProviderBoostRewardPool::<Test>::count().le(&past_eras_max));
			system_run_to_block(block_decade + 9);
		}
	})
}

// Test that additional stake is carried over to the next era's RewardPoolInfo.
#[test]
fn start_new_era_if_needed_updates_reward_pool() {
	new_test_ext().execute_with(|| {
		todo!();

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

			assert_ok!(Capacity::provider_boost(
				RuntimeOrigin::signed(staker),
				provider_msa,
				stake_amount
			));
		}
	});
}

#[test]
fn reward_pool_history_chunk_default_tests() {
	let chunk: RewardPoolHistoryChunk<Test> = RewardPoolHistoryChunk::new();
	assert!(!chunk.is_full());
	assert!(chunk.total_for_era(&0u32).is_none());
	let default: RewardPoolHistoryChunk<Test> = RewardPoolHistoryChunk::default();
	assert!(default.total_for_era(&032).is_none());
	assert_eq!(default.era_range(), (0u32, 0u32));
}

#[test]
fn reward_pool_history_chunk_insert_range_remove() {
	let mut chunk: RewardPoolHistoryChunk<Test> = RewardPoolHistoryChunk::new();
	assert_eq!(chunk.try_insert(22u32, 100u64), Ok(None));
	assert_eq!(chunk.try_insert(22u32, 110u64), Ok(Some(100u64)));
	assert!(chunk.try_insert(23u32, 123u64).is_ok());
	assert!(chunk.try_insert(24u32, 99u64).is_ok());
	// For <Test> the limit is 3
	assert_eq!(chunk.try_insert(25u32, 99u64), Err((25u32, 99u64)));
	assert_eq!(chunk.total_for_era(&23u32), Some(&123u64));
	assert_eq!(chunk.era_range(), (22u32, 24u32));
}
