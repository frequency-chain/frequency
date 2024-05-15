use super::mock::*;
use crate::{
	CurrentEraInfo, Error, ProviderBoostHistories, ProviderBoostHistory, RewardEraInfo,
	StakingDetails, StakingRewardClaim, StakingRewardsProvider, StakingType::*,
	UnclaimedRewardInfo,
};
use frame_support::{assert_err, traits::Len};

use crate::tests::testing_utils::{run_to_block, set_era_and_reward_pool, system_run_to_block};
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
fn era_staking_reward_implementation() {
	struct TestCase {
		total_staked: u64,    // by all boosters
		amount_staked: u64,   // by a single booster
		expected_reward: u64, // for a single era
		reward_pool: u64,     // reward pool size
	}
	let test_cases: Vec<TestCase> = vec![
		TestCase {
			total_staked: 1_000_000,
			amount_staked: 0,
			expected_reward: 0,
			reward_pool: 10_000,
		}, // shouldn't happen, but JIC
		TestCase {
			total_staked: 1_000_000,
			amount_staked: 30,
			expected_reward: 0,
			reward_pool: 10_000,
		}, // truncated result
		TestCase {
			total_staked: 1_000_000,
			amount_staked: 150,
			expected_reward: 1,
			reward_pool: 10_000,
		}, // truncated result
		TestCase {
			total_staked: 1_000_000,
			amount_staked: 1000,
			expected_reward: 4,
			reward_pool: 10_000,
		}, // hits the cap starting with this example
		TestCase {
			total_staked: 1_000_000,
			amount_staked: 11000,
			expected_reward: 42,
			reward_pool: 10_000,
		}, // > 0.0038% of total, reward = 11000*.0038
		TestCase {
			total_staked: 20_000_000,
			amount_staked: 888_889,
			expected_reward: 3378,
			reward_pool: 11_000_000,
		}, //  testnet/mainnet values
	];
	for tc in test_cases {
		assert_eq!(
			Capacity::era_staking_reward(tc.amount_staked, tc.total_staked, tc.reward_pool),
			tc.expected_reward
		);
	}
}

#[test]
fn check_for_unclaimed_rewards_returns_empty_set_when_no_staking() {
	new_test_ext().execute_with(|| {
		let account = 500u64;
		let history: ProviderBoostHistory<Test> = ProviderBoostHistory::new();
		ProviderBoostHistories::<Test>::set(account, Some(history));
		let rewards = Capacity::check_for_unclaimed_rewards(&account).unwrap();
		assert!(rewards.is_empty())
	})
}
#[test]
fn check_for_unclaimed_rewards_returns_empty_set_when_only_staked_this_era() {
	new_test_ext().execute_with(|| {
		system_run_to_block(5);
		set_era_and_reward_pool(5u32, 1u32, 1000u64);
		let account = 500u64;
		let mut history: ProviderBoostHistory<Test> = ProviderBoostHistory::new();
		history.add_era_balance(&5u32, &100u64);
		ProviderBoostHistories::<Test>::set(account, Some(history));
		let rewards = Capacity::check_for_unclaimed_rewards(&account).unwrap();
		assert!(rewards.is_empty())
	})
}

#[test]
fn check_for_unclaimed_rewards_has_eligible_rewards() {
	new_test_ext().execute_with(|| {
		system_run_to_block(99);
		run_to_block(102);
		let account = 10_000u64;
		let mut history: ProviderBoostHistory<Test> = ProviderBoostHistory::new();
		for era in 4u32..15u32 {
			set_era_and_reward_pool(era, (era * 10u32).into(), 1_000_000u64);
		}
		history.add_era_balance(&4u32, &1_000u64);
		history.add_era_balance(&12u32, &1_000u64);
		history.add_era_balance(&14u32, &1_000u64); // ineligible, in current era

		ProviderBoostHistories::<Test>::set(account, Some(history));
		let rewards = Capacity::check_for_unclaimed_rewards(&account).unwrap();
		assert_eq!(rewards.len(), 4usize);
		assert_eq!(
			rewards.get(3).unwrap(),
			&UnclaimedRewardInfo {
				reward_era: 13,
				expires_at_block: 130,
				staked_amount: 2_000,
				earned_amount: 8,
			}
		)
	})
}
