use super::mock::*;
use crate as pallet_capacity;
use crate::{BalanceOf, StakingConfigProvider, StakingType, StakingType::FlexibleBoost};
use common_primitives::capacity::StakingType::CommittedBoost;
use frame_support::traits::Get;
use pallet_capacity::Config;
use sp_runtime::Perbill;

#[test]
fn percent_releasable_in_pre_commit_phase_should_be_zero() {
	new_test_ext().execute_with(|| {
		// Indicate we are in the pre-commit phase
		set_pte_block::<Test>(None);

		let (staking_type, percent_releasable) =
			Capacity::get_staking_type_and_releasable_percent_in_force(
				StakingType::CommittedBoost,
				&<Test as Config>::StakingConfigProvider::get(CommittedBoost),
			);
		assert_eq!(staking_type, CommittedBoost);
		assert_eq!(percent_releasable, Perbill::zero());
	})
}

#[test]
fn percent_releasable_in_initial_commit_phase_should_be_zero() {
	new_test_ext().execute_with(|| {
		// Indicate we are in the initial-commit phase
		set_pte_block::<Test>(Some(1));

		let (staking_type, percent_releasable) =
			Capacity::get_staking_type_and_releasable_percent_in_force(
				StakingType::CommittedBoost,
				&<Test as Config>::StakingConfigProvider::get(CommittedBoost),
			);
		assert_eq!(staking_type, CommittedBoost);
		assert_eq!(percent_releasable, Perbill::zero());
	})
}

#[test]
fn percent_releasable_after_failsafe_block_should_be_one_hundred() {
	new_test_ext().execute_with(|| {
		// Indicate we are in the initiai-commit phase
		set_pte_block::<Test>(None);
		System::set_block_number(<Test as Config>::CommittedBoostFailsafeUnlockBlockNumber::get());

		let (staking_type, percent_releasable) =
			Capacity::get_staking_type_and_releasable_percent_in_force(
				StakingType::CommittedBoost,
				&<Test as Config>::StakingConfigProvider::get(CommittedBoost),
			);
		assert_eq!(staking_type, FlexibleBoost);
		assert_eq!(percent_releasable, Perbill::from_percent(100));
	})
}

#[test]
fn percent_releasable_at_end_of_staged_release_should_be_one_hundred() {
	new_test_ext().execute_with(|| {
		// Indicate we are in the initiai-commit phase
		set_pte_block::<Test>(Some(1));
		let staking_config = <Test as Config>::StakingConfigProvider::get(CommittedBoost);
		let block_number = 1 +
			staking_config.initial_commitment_blocks +
			(staking_config.commitment_release_stage_blocks *
				staking_config.commitment_release_stages);
		System::set_block_number(block_number);

		let (staking_type, percent_releasable) =
			Capacity::get_staking_type_and_releasable_percent_in_force(
				StakingType::CommittedBoost,
				&<Test as Config>::StakingConfigProvider::get(CommittedBoost),
			);
		assert_eq!(staking_type, FlexibleBoost);
		assert_eq!(percent_releasable, Perbill::from_percent(100));
	})
}

#[test]
fn amount_releasable_during_staged_release_should_be_consistent_if_always_unstaked() {
	new_test_ext().execute_with(|| {
		// Indicate we are in the initial-commit phase
		set_pte_block::<Test>(Some(1));
		let staking_config = <Test as Config>::StakingConfigProvider::get(CommittedBoost);
		let start_block_number = 1 + staking_config.initial_commitment_blocks;
		System::set_block_number(start_block_number);
		let mut prev_amount: BalanceOf<Test> = 0;
		let mut total_balance: BalanceOf<Test> = 1_000_000;

		for stage in 0..staking_config.commitment_release_stages {
			let offset_blocks = stage * staking_config.commitment_release_stage_blocks;
			let block_number = start_block_number + offset_blocks;
			System::set_block_number(block_number);
			let (staking_type, percent_releasable) =
				Capacity::get_staking_type_and_releasable_percent_in_force(
					StakingType::CommittedBoost,
					&<Test as Config>::StakingConfigProvider::get(CommittedBoost),
				);
			assert_eq!(staking_type, CommittedBoost);
			let amount = percent_releasable.mul_floor(total_balance);
			if prev_amount > 0 && amount > 0 {
				assert!((prev_amount as i32 - amount as i32).abs() <= 1);
			}
			prev_amount = amount;
			total_balance = total_balance.saturating_sub(amount);
		}
	})
}
