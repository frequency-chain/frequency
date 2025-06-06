use super::mock::*;
use crate as pallet_capacity;
use crate::{
	BalanceOf, StakingConfigProvider, StakingDetails, StakingType, StakingType::FlexibleBoost,
};
use common_primitives::{capacity::StakingType::CommittedBoost, msa::MessageSourceId};
use frame_support::traits::Get;
use pallet_capacity::Config;
use sp_runtime::{traits::Zero, Perbill};

#[test]
fn amount_releasable_in_pre_commit_phase_should_be_zero() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let staking_account =
			StakingDetails { active: 1_000u64, staking_type: StakingType::CommittedBoost };

		// Indicate we are in the pre-commit phase
		set_pte_block::<Test>(None);

		let (staking_type, amount_releasable) =
			Capacity::get_staking_type_and_releasable_amount_in_force(
				&account,
				&staking_account,
				&<Test as Config>::StakingConfigProvider::get(CommittedBoost),
			);
		assert_eq!(staking_type, CommittedBoost);
		assert_eq!(amount_releasable, 0);
	})
}

#[test]
fn amount_releasable_in_initial_commit_phase_should_be_zero() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let staking_account =
			StakingDetails { active: 1_000u64, staking_type: StakingType::CommittedBoost };

		// Indicate we are in the initial-commit phase
		set_pte_block::<Test>(Some(1));

		let (staking_type, amount_releasable) =
			Capacity::get_staking_type_and_releasable_amount_in_force(
				&account,
				&staking_account,
				&<Test as Config>::StakingConfigProvider::get(CommittedBoost),
			);
		assert_eq!(staking_type, CommittedBoost);
		assert_eq!(amount_releasable, 0);
	})
}

#[test]
fn amount_releasable_after_failsafe_block_should_be_one_hundred() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let staking_account =
			StakingDetails { active: 1_000u64, staking_type: StakingType::CommittedBoost };

		// Indicate we are in the initial-commit phase
		set_pte_block::<Test>(None);
		System::set_block_number(<Test as Config>::CommittedBoostFailsafeUnlockBlockNumber::get());

		let (staking_type, amount_releasable) =
			Capacity::get_staking_type_and_releasable_amount_in_force(
				&account,
				&staking_account,
				&<Test as Config>::StakingConfigProvider::get(CommittedBoost),
			);
		assert_eq!(staking_type, FlexibleBoost);
		assert_eq!(amount_releasable, staking_account.active);
	})
}

#[test]
fn amount_releasable_at_end_of_staged_release_should_be_one_hundred() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let staking_account =
			StakingDetails { active: 1_000u64, staking_type: StakingType::CommittedBoost };

		// Indicate we are in the initiai-commit phase
		set_pte_block::<Test>(Some(1));
		let staking_config = <Test as Config>::StakingConfigProvider::get(CommittedBoost);
		let block_number = 1 +
			staking_config.initial_commitment_blocks +
			(staking_config.commitment_release_stage_blocks *
				staking_config.commitment_release_stages);
		System::set_block_number(block_number);

		let (staking_type, amount_releasable) =
			Capacity::get_staking_type_and_releasable_amount_in_force(
				&account,
				&staking_account,
				&<Test as Config>::StakingConfigProvider::get(CommittedBoost),
			);
		assert_eq!(staking_type, FlexibleBoost);
		assert_eq!(amount_releasable, staking_account.active);
	})
}

#[test]
fn amount_releasable_during_staged_release_should_be_consistent_if_always_unstaked() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let staking_account =
			StakingDetails { active: 1_000u64, staking_type: StakingType::CommittedBoost };
		let initial_commitment = staking_account.active;

		// Indicate we are in the initial-commit phase
		set_pte_block::<Test>(Some(1));
		set_initial_commitment::<Test>(&account, Some(initial_commitment));
		let staking_config = <Test as Config>::StakingConfigProvider::get(CommittedBoost);
		let start_block_number = 1 + staking_config.initial_commitment_blocks;
		System::set_block_number(start_block_number);

		for stage in 0..staking_config.commitment_release_stages {
			let offset_blocks = stage * staking_config.commitment_release_stage_blocks;
			let block_number = start_block_number + offset_blocks;
			System::set_block_number(block_number);
			let (staking_type, amount_releasable) =
				Capacity::get_staking_type_and_releasable_amount_in_force(
					&account,
					&staking_account,
					&<Test as Config>::StakingConfigProvider::get(CommittedBoost),
				);
			assert_eq!(staking_type, CommittedBoost);
			assert_eq!(
				amount_releasable,
				initial_commitment / u64::from(staking_config.commitment_release_stages)
			);
		}
	})
}
