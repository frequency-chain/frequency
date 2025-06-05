use super::mock::*;
use crate as pallet_capacity;
use crate::{StakingConfigProvider, StakingType, StakingType::FlexibleBoost};
use common_primitives::capacity::StakingType::CommittedBoost;
use frame_support::traits::Get;
use pallet_capacity::Config;
use sp_runtime::Percent;

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
		assert_eq!(percent_releasable, Percent::zero());
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
		assert_eq!(percent_releasable, Percent::zero());
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
		assert_eq!(percent_releasable, Percent::from_percent(100));
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
		assert_eq!(percent_releasable, Percent::from_percent(100));
	})
}

#[test]
fn percent_releasable_during_staged_release_should_be_correct() {
	new_test_ext().execute_with(|| {
		// Indicate we are in the initial-commit phase
		set_pte_block::<Test>(Some(1));
		let staking_config = <Test as Config>::StakingConfigProvider::get(CommittedBoost);
		let start_block_number = 1 + staking_config.initial_commitment_blocks;
		let end_block_number = start_block_number +
			(staking_config.commitment_release_stage_blocks *
				staking_config.commitment_release_stages);
		System::set_block_number(start_block_number);

		println!("Start block number: {:?}", start_block_number);
		for value in (start_block_number..end_block_number)
			.step_by(usize::try_from(staking_config.commitment_release_stage_blocks).unwrap())
		{
			System::set_block_number(value);
			let (staking_type, percent_releasable) =
				Capacity::get_staking_type_and_releasable_percent_in_force(
					StakingType::CommittedBoost,
					&<Test as Config>::StakingConfigProvider::get(CommittedBoost),
				);
			// assert_eq!(staking_type, CommittedBoost);
			// assert_eq!(percent_releasable, Percent::from_percent(100));
			println!("Block: {:?} - Percent releasable: {:?}", value, percent_releasable);
		}
	})
}
