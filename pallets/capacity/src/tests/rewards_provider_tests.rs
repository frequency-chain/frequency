use super::mock::*;
use crate::{
	CurrentEraInfo, Error, RewardEraInfo, RewardPoolInfo, StakingAccountDetails,
	StakingRewardClaim, StakingRewardPool, StakingRewardsProvider,
};
use frame_support::assert_err;

use common_primitives::capacity::StakingType::MaximumCapacity;
use sp_core::H256;

#[test]
fn test_staking_reward_total_happy_path() {
	new_test_ext().execute_with(|| {
		CurrentEraInfo::<Test>::set(RewardEraInfo { era_index: 11, started_at: 100 });
		// this calls the implementation in the pallet
		assert_eq!(Ok(5u64), Capacity::staking_reward_total(1, 5u32, 10u32));
		let proof = H256::random();
		let payload: StakingRewardClaim<Test> = StakingRewardClaim {
			claimed_reward: 1,
			staking_account_end_state: StakingAccountDetails {
				active: 1,
				total: 1,
				unlocking: Default::default(),
				staking_type: MaximumCapacity,
				last_rewards_claimed_at: None,
				stake_change_unlocking: Default::default(),
			},
			from_era: 1,
			to_era: 5,
		};
		assert!(Capacity::validate_staking_reward_claim(4, proof, payload));
	})
}

#[test]
fn test_payout_eligible() {
	new_test_ext().execute_with(|| {
		let is_eligible = Capacity::payout_eligible(1);
		assert!(!is_eligible);
	})
}

#[test]
fn test_reward_pool_size_happy_path() {
	struct TestCase {
		total_staked: u64,
		expected_reward_pool: u64,
	}
	new_test_ext().execute_with(|| {
		let test_cases: Vec<TestCase> = vec![
			TestCase { total_staked: 0, expected_reward_pool: 0 },
			TestCase { total_staked: 4, expected_reward_pool: 0 },
			TestCase { total_staked: 10, expected_reward_pool: 1 },
			TestCase { total_staked: 333, expected_reward_pool: 33 },
		];
		let era = 20u32;
		CurrentEraInfo::<Test>::set(RewardEraInfo { era_index: era, started_at: 200u32 });
		for tc in test_cases {
			StakingRewardPool::<Test>::insert(
				era,
				RewardPoolInfo {
					total_staked_token: tc.total_staked,
					total_reward_pool: 0u64,
					unclaimed_balance: 0u64,
				},
			);
			assert_eq!(Ok(tc.expected_reward_pool), Capacity::reward_pool_size());
		}
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
