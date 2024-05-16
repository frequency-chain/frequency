use super::mock::*;
use crate::{
	CurrentEraInfo, Error, ProviderBoostHistories, ProviderBoostHistory, RewardEraInfo,
	StakingDetails, StakingRewardClaim, StakingRewardsProvider, StakingType::*,
	UnclaimedRewardInfo,
};
use frame_support::{assert_err, assert_ok, traits::Len};

use crate::tests::testing_utils::{
	run_to_block, set_era_and_reward_pool, setup_provider, system_run_to_block,
};
use common_primitives::msa::MessageSourceId;
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
		let rewards = Capacity::list_unclaimed_rewards(&account).unwrap();
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
		let rewards = Capacity::list_unclaimed_rewards(&account).unwrap();
		assert!(rewards.is_empty())
	})
}

#[test]
fn check_for_unclaimed_rewards_has_eligible_rewards() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let target: MessageSourceId = 10;
		let amount = 1_000u64;

		// staking 1k as of block 1, era 1
		setup_provider(&account, &target, &amount, ProviderBoost);

		// staking 2k as of block 11, era 2
		run_to_block(11);
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account), target, amount));

		//  staking 3k as of era 4, block 31
		run_to_block(31);
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account), target, amount));

		run_to_block(51);
		assert_eq!(Capacity::get_current_era().era_index, 6u32);
		assert_eq!(Capacity::get_current_era().started_at, 51u32);

		assert!(Capacity::get_reward_pool_for_era(0u32).is_none());
		assert!(Capacity::get_reward_pool_for_era(1u32).is_some());
		assert!(Capacity::get_reward_pool_for_era(6u32).is_some());

		// rewards for eras 1 and 6 should not be returned; era 1 has been deleted, era 6 is current
		// era and therefore ineligible.
		// rewards should be for eras 1=1k, 2=2k, 3=2k, 4=3k, 5=3k
		let rewards = Capacity::list_unclaimed_rewards(&account).unwrap();
		assert_eq!(rewards.len(), 5usize);
		let expected_info: [UnclaimedRewardInfo<Test>; 5] = [
			UnclaimedRewardInfo {
				reward_era: 1u32,
				expires_at_block: 61,
				staked_amount: 1_000,
				earned_amount: 4,
			},
			UnclaimedRewardInfo {
				reward_era: 2u32,
				expires_at_block: 71,
				staked_amount: 2_000,
				earned_amount: 8,
			},
			UnclaimedRewardInfo {
				reward_era: 3u32,
				expires_at_block: 81,
				staked_amount: 2_000,
				earned_amount: 8,
			},
			UnclaimedRewardInfo {
				reward_era: 4u32,
				expires_at_block: 91,
				staked_amount: 3_000,
				earned_amount: 11,
			},
			UnclaimedRewardInfo {
				reward_era: 5u32,
				expires_at_block: 101,
				staked_amount: 3_000,
				earned_amount: 11,
			},
		];
		for i in 0..=4 {
			assert_eq!(rewards.get(i).unwrap(), &expected_info[i]);
		}
	})
}

#[test]
fn has_unclaimed_rewards_works() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let target: MessageSourceId = 10;
		let amount = 1_000u64;

		assert!(!Capacity::has_unclaimed_rewards(&account));

		// staking 1k as of block 1, era 1
		setup_provider(&account, &target, &amount, ProviderBoost);
		assert!(!Capacity::has_unclaimed_rewards(&account));

		// staking 2k as of block 11, era 2
		run_to_block(11);
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account), target, amount));
		assert!(Capacity::has_unclaimed_rewards(&account));

		//  staking 3k as of era 4, block 31
		run_to_block(31);
		assert!(Capacity::has_unclaimed_rewards(&account));

		run_to_block(61);
		assert!(Capacity::has_unclaimed_rewards(&account));
	})
}
