use super::mock::*;
use crate::{
	CurrentEraInfo, Error, RewardEraInfo, StakingDetails, StakingRewardClaim,
	StakingRewardsProvider, StakingType::*,
};
use frame_support::assert_err;

use sp_core::H256;

#[test]
fn staking_reward_total_happy_path() {
	new_test_ext().execute_with(|| {
		CurrentEraInfo::<Test>::set(RewardEraInfo { era_index: 11, started_at: 100 });
		// this calls the implementation in the pallet
		assert_eq!(Ok(5u64), Capacity::staking_reward_total(1, 5u32, 10u32));
		let proof = H256::random();
		let payload: StakingRewardClaim<Test> = StakingRewardClaim {
			claimed_reward: 1,
			staking_account_end_state: StakingDetails { active: 1, staking_type: MaximumCapacity },
			from_era: 1,
			to_era: 5,
		};
		assert!(Capacity::validate_staking_reward_claim(4, proof, payload));
	})
}

#[test]
fn payout_eligible_happy_path() {
	new_test_ext().execute_with(|| {
		let is_eligible = Capacity::payout_eligible(1);
		assert!(!is_eligible);
	})
}

#[test]
fn test_staking_reward_total_era_out_of_range() {
	new_test_ext().execute_with(|| {
		CurrentEraInfo::<Test>::set(RewardEraInfo { era_index: 10, started_at: 100 });
		assert_err!(Capacity::staking_reward_total(1, 5u32, 4u32), Error::<Test>::EraOutOfRange);
		assert_err!(Capacity::staking_reward_total(1, 15u32, 5u32), Error::<Test>::EraOutOfRange);
		assert_err!(Capacity::staking_reward_total(1, 5u32, 13u32), Error::<Test>::EraOutOfRange);
	});
}

// This tests Capacity implementation of the trait, but uses the mock's constants,
// to ensure that it's correctly specified in the pallet.
#[test]
fn staking_reward_for_era_implementation() {
	struct TestCase {
		total_staked: u64,    // by all boosters
		amount_staked: u64,   // by a single booster
		expected_reward: u64, // for a single era
	}
	let test_cases: Vec<TestCase> = vec![
		TestCase { total_staked: 1_000_000, amount_staked: 0, expected_reward: 0 }, // shouldn't happen, but JIC
		TestCase { total_staked: 1_000_000, amount_staked: 30, expected_reward: 0 }, // rounded result
		TestCase { total_staked: 1_000_000, amount_staked: 150, expected_reward: 1 }, // it rounds the result
		TestCase { total_staked: 1_000_000, amount_staked: 1000, expected_reward: 4 }, // hits the cap starting with this example
		TestCase { total_staked: 1_000_000, amount_staked: 11000, expected_reward: 42 }, // > 0.0038% of total, reward = 11000*.0038
	];
	for tc in test_cases {
		assert_eq!(
			Capacity::staking_reward_for_era(tc.amount_staked, tc.total_staked, 10_000),
			tc.expected_reward
		);
	}
}
