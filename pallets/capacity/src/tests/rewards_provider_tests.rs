use super::mock::*;
use crate::{
	tests::testing_utils::{run_to_block, system_run_to_block},
	Config, CurrentEraInfo, Error, Event, RewardEraInfo, StakingAccountDetails, StakingRewardClaim,
	StakingRewardsProvider,
};

use common_primitives::{capacity::StakingType::MaximumCapacity, node::Hash};
use frame_support::traits::Get;
use sp_core::H256;

#[test]
fn test_basic_rewards_provider_trivial_test() {
	new_test_ext().execute_with(|| {
		// this calls the implementation in the pallet
		assert_eq!(100_000u64, Capacity::reward_pool_size(1));
		assert_eq!(5u64, Capacity::staking_reward_total(1, 5u32, 10u32));
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
