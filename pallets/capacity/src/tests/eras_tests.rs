use super::mock::*;
use crate::{
	tests::testing_utils::*, BalanceOf, Config, CurrentEraInfo, CurrentEraProviderBoostTotal,
	RewardEraInfo,
};
use common_primitives::msa::MessageSourceId;
use frame_support::assert_ok;
use sp_core::Get;

pub fn boost_provider_and_run_to_end_of_era(
	staker: u64,
	provider_msa: MessageSourceId,
	stake_amount: u64,
	era: u32,
) {
	assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(staker), provider_msa, stake_amount));
	let block_decade: u32 = (era * 10) + 1;
	run_to_block(block_decade);
	assert_eq!(CurrentEraProviderBoostTotal::<Test>::get(), stake_amount * (era as u64));
	system_run_to_block(block_decade + 9);
}

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
			system_run_to_block(block_decade + 9);
		}
	})
}

// checks that the chunk indicated has an earliest era matching `era`,
// asserts whether it is full (or not) and asserts that its total stored matches `total`
fn assert_chunk_is_full_and_has_earliest_era_total(
	chunk_index: u32,
	is_full: bool,
	era: <Test as Config>::RewardEra,
	total: BalanceOf<Test>,
) {
	let chunk = Capacity::get_reward_pool_chunk(chunk_index).unwrap();
	assert_eq!(chunk.is_full(), is_full);
	assert_eq!(chunk.earliest_era(), Some(&era));
	assert_eq!(chunk.total_for_era(&era), Some(&total));
}

// gets the last (i.e. latest non-current) stored reward pool era, which is in chunk 0.
// asserts that it is the same as `era`, and that it has amount `total`
fn assert_last_era_total(era: <Test as Config>::RewardEra, total: BalanceOf<Test>) {
	let chunk = Capacity::get_reward_pool_chunk(0).unwrap();
	let (_earliest, latest) = chunk.era_range();
	assert_eq!(latest, era);
	assert_eq!(chunk.total_for_era(&era), Some(&total));
}

fn assert_chunk_is_empty(chunk_index: u32) {
	let chunk = Capacity::get_reward_pool_chunk(chunk_index).unwrap();
	assert!(chunk.earliest_era().is_none());
}

// Test that additional stake is carried over to the next era's RewardPoolInfo.
#[test]
fn start_new_era_if_needed_updates_reward_pool() {
	new_test_ext().execute_with(|| {
		system_run_to_block(8);
		let staker = 10_000;
		let provider_msa: MessageSourceId = 1;
		let stake_amount = 100u64;

		register_provider(provider_msa, "Binky".to_string());

		for i in [1u32, 2u32, 3u32] {
			boost_provider_and_run_to_end_of_era(staker, provider_msa, stake_amount, i);
		}
		// check that first chunk is filled correctly.
		assert_chunk_is_full_and_has_earliest_era_total(0, true, 1, 100);
		assert_chunk_is_empty(1);
		assert_chunk_is_empty(2);
		assert_last_era_total(3, 300);

		for i in [4u32, 5u32, 6u32] {
			boost_provider_and_run_to_end_of_era(staker, provider_msa, stake_amount, i);
		}
		assert_chunk_is_full_and_has_earliest_era_total(0, true, 4, 400);
		assert_chunk_is_full_and_has_earliest_era_total(1, true, 1, 100);
		assert_chunk_is_empty(2);
		assert_last_era_total(6, 600);

		for i in [7u32, 8u32, 9u32] {
			boost_provider_and_run_to_end_of_era(staker, provider_msa, stake_amount, i);
		}
		assert_chunk_is_full_and_has_earliest_era_total(0, true, 7, 700);
		assert_chunk_is_full_and_has_earliest_era_total(1, true, 4, 400);
		assert_chunk_is_full_and_has_earliest_era_total(2, true, 1, 100);
		assert_last_era_total(9, 900);

		// check that it all rolls over properly.
		for i in [10u32, 11u32] {
			boost_provider_and_run_to_end_of_era(staker, provider_msa, stake_amount, i);
		}
		assert_chunk_is_full_and_has_earliest_era_total(0, true, 9, 900);
		assert_chunk_is_full_and_has_earliest_era_total(1, true, 6, 600);
		assert_chunk_is_full_and_has_earliest_era_total(2, true, 3, 300);
		assert_last_era_total(11, 1100);
	});
}

#[test]
fn get_expiration_block_for_era_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Capacity::get_current_era().era_index, 1u32);
		assert_eq!(Capacity::block_at_end_of_era(10u32), 100);

		set_era_and_reward_pool(7, 61, 0);
		assert_eq!(Capacity::block_at_end_of_era(7u32), 70);
		assert_eq!(Capacity::block_at_end_of_era(10u32), 100);

		set_era_and_reward_pool(10, 91, 0);
		assert_eq!(Capacity::block_at_end_of_era(10u32), 100);

		set_era_and_reward_pool(99, 2342, 0);
		assert_eq!(Capacity::block_at_end_of_era(99), 2351);

		assert_eq!(Capacity::block_at_end_of_era(108), 2441);
	})
}

#[test]
fn get_chunk_index_for_era_works() {
	new_test_ext().execute_with(|| {
		struct TestCase {
			era: u32,
			current_era: u32,
			expected: Option<u32>,
		}
		// assuming history limit is 12, chunk length is 3
		for test in {
			vec![
				TestCase { era: 3, current_era: 6, expected: Some(0) },
				TestCase { era: 2, current_era: 1, expected: None },
				TestCase { era: 3, current_era: 16, expected: None },
				TestCase { era: 1, current_era: 1, expected: None },
				TestCase { era: 1, current_era: 2, expected: Some(0) },
				TestCase { era: 1, current_era: 3, expected: Some(0) },
				TestCase { era: 1, current_era: 4, expected: Some(0) },
				TestCase { era: 1, current_era: 5, expected: Some(1) },
				TestCase { era: 1, current_era: 6, expected: Some(1) },
				TestCase { era: 1, current_era: 7, expected: Some(1) },
				TestCase { era: 12, current_era: 13, expected: Some(0) },
				TestCase { era: 11, current_era: 13, expected: Some(0) },
				TestCase { era: 10, current_era: 13, expected: Some(0) },
				TestCase { era: 9, current_era: 13, expected: Some(1) },
				TestCase { era: 8, current_era: 13, expected: Some(1) },
				TestCase { era: 7, current_era: 13, expected: Some(1) },
				TestCase { era: 6, current_era: 13, expected: Some(2) },
				TestCase { era: 5, current_era: 13, expected: Some(2) },
				TestCase { era: 4, current_era: 13, expected: Some(2) },
				TestCase { era: 3, current_era: 13, expected: Some(3) },
				TestCase { era: 55, current_era: 60, expected: Some(1) },
				TestCase { era: 55, current_era: 62, expected: Some(2) },
			]
		} {
			assert_eq!(
				Capacity::get_chunk_index_for_era(test.era, test.current_era),
				test.expected
			);
		}
	})
}
