use super::mock::*;
use crate::{
	tests::testing_utils::{run_to_block, system_run_to_block},
	Config, CurrentEraInfo, Error, Event, RewardEraInfo, RewardPoolInfo, StakingAccountDetails,
	StakingRewardClaim, StakingRewardPool, StakingRewardsProvider,
};
use frame_support::{assert_err, assert_ok};

use common_primitives::{
	capacity::StakingType::MaximumCapacity,
	node::{Hash, RewardEra},
};
use frame_support::traits::Get;
use sp_core::H256;

#[test]
fn test_basic_rewards_provider_trivial_test() {
	new_test_ext().execute_with(|| {
		// this calls the implementation in the pallet
		assert_eq!(Ok(100_000u64), Capacity::reward_pool_size());
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
			to_era: 2,
		};
		assert!(Capacity::validate_staking_reward_claim(1, proof, payload));
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
	new_test_ext().execute_with(|| {
		let era = 20u32;
		CurrentEraInfo::<Test>::set(RewardEraInfo { era_index: era, started_at: 200u32 });
		StakingRewardPool::<Test>::insert(
			era,
			RewardPoolInfo {
				total_staked_token: 1_000u64,
				total_reward_pool: 100u64,
				unclaimed_balance: 1_000u64,
			},
		);
		assert_eq!(Ok(100u64), Capacity::reward_pool_size());
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
