use super::mock::*;
use crate::{
	tests::testing_utils::*, BalanceOf, Config, CurrentEraInfo, CurrentEraProviderBoostTotal,
	RewardEraInfo,
};
use common_primitives::msa::MessageSourceId;
use frame_support::assert_ok;

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
	assert_eq!(chunk.is_full(), is_full, "{:?}", chunk);
	assert_eq!(chunk.earliest_era(), Some(&era), "{:?}", chunk);
	assert_eq!(chunk.total_for_era(&era), Some(&total), "{:?}", chunk);
}

// gets the last (i.e. latest non-current) stored reward pool era, which is in chunk 0.
// asserts that it is the same as `era`, and that it has amount `total`
fn assert_last_era_total(era: <Test as Config>::RewardEra, total: BalanceOf<Test>) {
	let chunk_idx = Capacity::get_chunk_index_for_era(era);
	let chunk_opt = Capacity::get_reward_pool_chunk(chunk_idx);
	assert!(chunk_opt.is_some(), "No pool for Era: {:?} with chunk index: {:?}", era, chunk_idx);
	let chunk = chunk_opt.unwrap();
	let (_earliest, latest) = chunk.era_range();
	assert_eq!(latest, era);
	assert_eq!(chunk.total_for_era(&era), Some(&total));
}

fn assert_chunk_is_empty(chunk_index: u32) {
	let chunk_opt = Capacity::get_reward_pool_chunk(chunk_index);
	if chunk_opt.is_some() {
		assert!(chunk_opt.unwrap().earliest_era().is_none())
	} else {
		assert!(chunk_opt.is_none())
	}
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
		// No change
		assert_chunk_is_full_and_has_earliest_era_total(0, true, 1, 100);
		// New Chunk
		assert_chunk_is_full_and_has_earliest_era_total(1, true, 4, 400);
		assert_chunk_is_empty(2);
		assert_last_era_total(6, 600);

		for i in [7u32, 8u32, 9u32, 10u32, 11u32, 12u32, 13u32, 14u32, 15u32] {
			boost_provider_and_run_to_end_of_era(staker, provider_msa, stake_amount, i);
		}
		// No changes
		assert_chunk_is_full_and_has_earliest_era_total(0, true, 1, 100);
		assert_chunk_is_full_and_has_earliest_era_total(1, true, 4, 400);
		// New
		assert_chunk_is_full_and_has_earliest_era_total(2, true, 7, 700);
		assert_chunk_is_full_and_has_earliest_era_total(3, true, 10, 1000);
		assert_chunk_is_full_and_has_earliest_era_total(4, true, 13, 1300);
		assert_last_era_total(15, 1500);

		// check that it all rolls over properly.
		for i in [16u32, 17u32] {
			boost_provider_and_run_to_end_of_era(staker, provider_msa, stake_amount, i);
		}
		// No Changes
		assert_chunk_is_full_and_has_earliest_era_total(1, true, 4, 400);
		assert_chunk_is_full_and_has_earliest_era_total(2, true, 7, 700);
		assert_chunk_is_full_and_has_earliest_era_total(3, true, 10, 1000);
		assert_chunk_is_full_and_has_earliest_era_total(4, true, 13, 1300);
		// New
		assert_chunk_is_full_and_has_earliest_era_total(0, false, 16, 1600);
		assert_last_era_total(17, 1700);
		// There shouldn't be a chunk 5 ever with this config
		assert_chunk_is_empty(5);
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
		#[derive(Debug)]
		struct TestCase {
			era: u32,
			expected: u32,
		}
		// assuming history limit is 12, chunk length is 3
		// [1,2,3],[4,5,6],[7,8,9],[10,11,12],[13,14,15]
		// [16],[4,5,6],[7,8,9],[10,11,12],[13,14,15]
		for test in {
			vec![
				TestCase { era: 1, expected: 0 },
				TestCase { era: 2, expected: 0 },
				TestCase { era: 3, expected: 0 },
				TestCase { era: 4, expected: 1 },
				TestCase { era: 5, expected: 1 },
				TestCase { era: 6, expected: 1 },
				TestCase { era: 7, expected: 2 },
				TestCase { era: 11, expected: 3 },
				TestCase { era: 15, expected: 4 },
				TestCase { era: 16, expected: 0 },
				TestCase { era: 22, expected: 2 },
				TestCase { era: 55, expected: 3 },
			]
		} {
			assert_eq!(Capacity::get_chunk_index_for_era(test.era), test.expected, "{:?}", test);
		}
	})
}
