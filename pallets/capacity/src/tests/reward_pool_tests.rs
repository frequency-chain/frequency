use crate::{
	tests::{mock::*, testing_utils::set_era_and_reward_pool},
	BalanceOf, Config, ProviderBoostRewardPools, RewardPoolHistoryChunk,
};
use common_primitives::capacity::RewardEra;
use frame_support::{assert_ok, traits::Get};
use std::ops::Add;

// Check eras_tests for how reward pool chunks are expected to be filled during runtime.

// Fill up a reward pool history chunk by adding 100 in each new era from the starting value.
fn fill_reward_pool_history_chunk(
	chunk_index: u32,
	starting_era: RewardEra,
	number_of_items: u32,
	starting_total_stake: BalanceOf<Test>,
) {
	let mut chunk: RewardPoolHistoryChunk<Test> = RewardPoolHistoryChunk::new();
	for i in 0u32..number_of_items {
		let additional_stake: BalanceOf<Test> = (i * 100u32).into();
		let total_stake: BalanceOf<Test> = starting_total_stake.add(additional_stake);
		let era = starting_era + i;
		assert_ok!(chunk.try_insert(era, total_stake));
	}
	ProviderBoostRewardPools::set(chunk_index, Some(chunk));
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

// Check boundary behavior for the first reward eras before hitting maximum history,
// then check behavior once the reward pool chunks are always full.
#[test]
fn get_total_stake_for_past_era_works_with_partly_filled_single_chunk() {
	new_test_ext().execute_with(|| {
		set_era_and_reward_pool(3, 21, 1000);
		System::set_block_number(22);
		fill_reward_pool_history_chunk(0, 1, 2, 100);
		assert_eq!(Capacity::get_total_stake_for_past_era(1, 3), Ok(100));
		assert_eq!(Capacity::get_total_stake_for_past_era(2, 3), Ok(200));
		assert!(Capacity::get_total_stake_for_past_era(3, 3).is_err());
		assert!(Capacity::get_total_stake_for_past_era(99, 3).is_err());
	})
}

#[test]
fn get_total_stake_for_past_era_works_with_1_full_chunk() {
	new_test_ext().execute_with(|| {
		System::set_block_number(52);
		set_era_and_reward_pool(6, 51, 1000);
		fill_reward_pool_history_chunk(0, 0, 3, 0); // eras 1-3
		fill_reward_pool_history_chunk(1, 3, 2, 300); // eras 4,5
		for i in 2u32..=4u32 {
			let expected_total: BalanceOf<Test> = (i * 100u32).into();
			let actual = Capacity::get_total_stake_for_past_era(i, 5);
			assert_eq!(actual, Ok(expected_total));
		}
		assert!(Capacity::get_total_stake_for_past_era(6, 6).is_err());
	})
}

#[test]
fn get_total_stake_for_past_era_works_with_2_full_chunks() {
	new_test_ext().execute_with(|| {
		System::set_block_number(72);
		set_era_and_reward_pool(7, 71, 1000);
		fill_reward_pool_history_chunk(0, 0, 3, 0);
		fill_reward_pool_history_chunk(1, 3, 3, 300);
		fill_reward_pool_history_chunk(2, 6, 1, 600);
		for i in 0u32..=6u32 {
			let expected_total: BalanceOf<Test> = (i * 100u32).into();
			assert_eq!(Capacity::get_total_stake_for_past_era(i, 7), Ok(expected_total));
		}
		assert!(Capacity::get_total_stake_for_past_era(7, 7).is_err());
	})
}

#[test]
fn get_total_stake_for_past_era_works_with_full_reward_pool() {
	new_test_ext().execute_with(|| {
		System::set_block_number(121);
		let history_limit: u32 = <Test as Config>::ProviderBoostHistoryLimit::get();
		set_era_and_reward_pool(12, 121, (2000u32).into());

		fill_reward_pool_history_chunk(0, 0, 3, 1);
		fill_reward_pool_history_chunk(1, 3, 3, 301);
		fill_reward_pool_history_chunk(2, 6, 3, 601);
		fill_reward_pool_history_chunk(3, 9, 3, 901);

		(0u32..history_limit).for_each(|era| {
			let expected_total: BalanceOf<Test> = ((era * 100u32) + 1u32).into();
			assert_eq!(Capacity::get_total_stake_for_past_era(era, 12), Ok(expected_total));
		});
		assert!(Capacity::get_total_stake_for_past_era(12, 12).is_err());
	})
}
