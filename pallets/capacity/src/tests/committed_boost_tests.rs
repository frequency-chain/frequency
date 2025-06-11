use super::mock::*;

use crate as pallet_capacity;
use crate::{CommitmentPhase, StakingConfigProvider, StakingDetails, StakingType};
use frame_support::traits::Get;
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_capacity::Config;

#[test]
fn commitment_phase_should_be_pre_commitment_if_pte_block_is_none() {
	new_test_ext().execute_with(|| {
		// Indicate we are in the pre-commit phase
		set_pte_block::<Test>(None);

		// At beginning of pre-commitment phase
		assert_eq!(
			Capacity::get_committed_boosting_phase(System::block_number()),
			CommitmentPhase::PreCommitment
		);

		// Just before the failsafe block
		let failsafe_block: BlockNumberFor<Test> =
			<Test as Config>::CommittedBoostFailsafeUnlockBlockNumber::get();
		System::set_block_number(failsafe_block - 1);
		assert_eq!(
			Capacity::get_committed_boosting_phase(System::block_number()),
			CommitmentPhase::PreCommitment
		);
	})
}

#[test]
fn commitment_phase_should_be_initial_commitment_if_pte_block_is_set() {
	new_test_ext().execute_with(|| {
		// Indicate we are in the pre-commit phase
		let pte_block = 1;
		set_pte_block::<Test>(Some(pte_block));

		// Beginning of initial commitment phase
		assert_eq!(
			Capacity::get_committed_boosting_phase(pte_block,),
			CommitmentPhase::InitialCommitment
		);

		// End of initial commitment phase
		System::set_block_number(
			<Test as Config>::StakingConfigProvider::get(StakingType::CommittedBoost)
				.initial_commitment_blocks,
		);
		assert_eq!(
			Capacity::get_committed_boosting_phase(System::block_number()),
			CommitmentPhase::InitialCommitment
		);
	})
}

#[test]
fn commitment_phase_should_be_staged_release_if_past_initial_commitment_phase() {
	new_test_ext().execute_with(|| {
		// Indicate we are in the pre-commit phase
		let pte_block = 1;
		set_pte_block::<Test>(Some(pte_block));
		let staking_config =
			<Test as Config>::StakingConfigProvider::get(StakingType::CommittedBoost);

		// End of initial commitment phase
		System::set_block_number(1 + staking_config.initial_commitment_blocks);
		assert_eq!(
			Capacity::get_committed_boosting_phase(System::block_number()),
			CommitmentPhase::StagedRelease
		);

		// End of staged release (not past)
		System::set_block_number(
			pte_block +
				staking_config.initial_commitment_blocks +
				(staking_config.commitment_release_stage_blocks *
					staking_config.commitment_release_stages) -
				1,
		);
		assert_eq!(
			Capacity::get_committed_boosting_phase(System::block_number()),
			CommitmentPhase::StagedRelease
		);
	})
}

#[test]
fn commitment_phase_should_be_program_ended_if_past_staged_release_phase() {
	new_test_ext().execute_with(|| {
		// Indicate we are in the pre-commit phase
		let pte_block = 1;
		set_pte_block::<Test>(Some(pte_block));
		let staking_config =
			<Test as Config>::StakingConfigProvider::get(StakingType::CommittedBoost);

		// End of initial commitment phase
		System::set_block_number(
			pte_block +
				staking_config.initial_commitment_blocks +
				(staking_config.commitment_release_stage_blocks *
					staking_config.commitment_release_stages),
		);
		assert_eq!(
			Capacity::get_committed_boosting_phase(System::block_number()),
			CommitmentPhase::RewardProgramEnded
		);
	})
}

#[test]
fn commitment_phase_should_be_failsafe_if_pte_block_not_set_and_past_failsafe_block() {
	new_test_ext().execute_with(|| {
		// Indicate we are in the pre-commit phase
		set_pte_block::<Test>(None);

		// End of initial commitment phase
		System::set_block_number(<Test as Config>::CommittedBoostFailsafeUnlockBlockNumber::get());
		assert_eq!(
			Capacity::get_committed_boosting_phase(System::block_number()),
			CommitmentPhase::Failsafe
		);
	})
}

#[test]
fn amount_releasable_in_pre_commit_phase_should_be_zero() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let staking_account =
			StakingDetails { active: 1_000u64, staking_type: StakingType::CommittedBoost };

		// Indicate we are in the pre-commit phase
		set_pte_block::<Test>(None);

		let staking_type = Capacity::get_staking_type_in_force(StakingType::CommittedBoost);
		let amount_releasable = Capacity::get_releasable_amount(&account, &staking_account);
		assert_eq!(staking_type, StakingType::CommittedBoost);
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

		let staking_type = Capacity::get_staking_type_in_force(StakingType::CommittedBoost);
		let amount_releasable = Capacity::get_releasable_amount(&account, &staking_account);
		assert_eq!(staking_type, StakingType::CommittedBoost);
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

		let staking_type = Capacity::get_staking_type_in_force(StakingType::CommittedBoost);
		let amount_releasable = Capacity::get_releasable_amount(&account, &staking_account);
		assert_eq!(staking_type, StakingType::FlexibleBoost);
		assert_eq!(amount_releasable, staking_account.active);
	})
}

#[test]
fn full_amount_should_be_releasable_at_end_of_staged_release() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let staking_account =
			StakingDetails { active: 1_000u64, staking_type: StakingType::CommittedBoost };

		set_pte_block::<Test>(Some(1));
		let staking_config =
			<Test as Config>::StakingConfigProvider::get(StakingType::CommittedBoost);
		let block_number = 2 +
			staking_config.initial_commitment_blocks +
			(staking_config.commitment_release_stage_blocks *
				staking_config.commitment_release_stages);
		System::set_block_number(block_number);

		let staking_type = Capacity::get_staking_type_in_force(StakingType::CommittedBoost);
		let amount_releasable = Capacity::get_releasable_amount(&account, &staking_account);
		assert_eq!(staking_type, StakingType::FlexibleBoost);
		assert_eq!(amount_releasable, staking_account.active);
	})
}

#[test]
fn amount_releasable_during_staged_release_should_be_consistent_if_always_unstaked() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let staking_config =
			<Test as Config>::StakingConfigProvider::get(StakingType::CommittedBoost);
		// Design staking amount to have the largest possible remainder after each stage
		let mut staking_account = StakingDetails {
			active: (1_000 * staking_config.commitment_release_stages as u64) +
				staking_config.commitment_release_stages as u64 -
				1,
			staking_type: StakingType::CommittedBoost,
		};
		let initial_commitment = staking_account.active;

		// Indicate we are in the initial-commit phase
		set_pte_block::<Test>(Some(1));
		set_initial_commitment::<Test>(&account, Some(initial_commitment));
		let start_block_number = 1 + staking_config.initial_commitment_blocks;
		let amount_releasable_per_stage =
			initial_commitment / u64::from(staking_config.commitment_release_stages);
		System::set_block_number(start_block_number);

		for stage in 0..staking_config.commitment_release_stages {
			let offset_blocks = stage * staking_config.commitment_release_stage_blocks;
			let block_number = start_block_number + offset_blocks;
			System::set_block_number(block_number);
			let staking_type = Capacity::get_staking_type_in_force(StakingType::CommittedBoost);
			let amount_releasable = Capacity::get_releasable_amount(&account, &staking_account);
			assert_eq!(staking_type, StakingType::CommittedBoost);
			if stage < (staking_config.commitment_release_stages - 1) {
				assert_eq!(amount_releasable, amount_releasable_per_stage);
			} else {
				// Allow for a "balloon" unstake in the final stage due to the initial stake not being evenly divisible by the stage size
				assert_eq!(
					amount_releasable,
					amount_releasable_per_stage + staking_config.commitment_release_stages as u64 -
						1
				);
			}

			staking_account.active -= amount_releasable;
		}
	})
}

#[test]
fn amount_releasable_during_staged_release_should_be_cumulatively_correct_if_not_unstaked_at_each_of_stage(
) {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let staking_config =
			<Test as Config>::StakingConfigProvider::get(StakingType::CommittedBoost);
		// Design staking amount to have the largest possible remainder after each stage
		let staking_account = StakingDetails {
			active: (1_000 * staking_config.commitment_release_stages as u64) +
				staking_config.commitment_release_stages as u64 -
				1,
			staking_type: StakingType::CommittedBoost,
		};
		let initial_commitment = staking_account.active;

		// Indicate we are in the initial-commit phase
		set_pte_block::<Test>(Some(1));
		set_initial_commitment::<Test>(&account, Some(initial_commitment));
		let start_block_number = 1 + staking_config.initial_commitment_blocks;
		let amount_releasable_per_stage =
			initial_commitment / u64::from(staking_config.commitment_release_stages);
		System::set_block_number(start_block_number);

		for stage in 0..staking_config.commitment_release_stages {
			let offset_blocks = stage * staking_config.commitment_release_stage_blocks;
			let block_number = start_block_number + offset_blocks;
			System::set_block_number(block_number);
			let staking_type = Capacity::get_staking_type_in_force(StakingType::CommittedBoost);
			let amount_releasable = Capacity::get_releasable_amount(&account, &staking_account);
			assert_eq!(staking_type, StakingType::CommittedBoost);
			let expected_amount_releasable = amount_releasable_per_stage * (stage as u64 + 1);
			if stage < (staking_config.commitment_release_stages - 1) {
				assert_eq!(amount_releasable, expected_amount_releasable);
			} else {
				// Allow for a "balloon" unstake in the final stage due to the initial stake not being evenly divisible by the stage size
				assert_eq!(amount_releasable, staking_account.active);
			}
		}
	})
}

#[test]
fn flexible_boost_should_result_in_full_releasable_amount() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let staking_account =
			StakingDetails { active: 1_000u64, staking_type: StakingType::FlexibleBoost };
		let releasable_amount = Capacity::get_releasable_amount(&account, &staking_account);
		let staking_type = Capacity::get_staking_type_in_force(StakingType::FlexibleBoost);

		assert_eq!(staking_type, StakingType::FlexibleBoost);
		assert_eq!(releasable_amount, staking_account.active);
	})
}

#[test]
fn max_capacity_stake_should_result_in_full_releasable_amount() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let staking_account =
			StakingDetails { active: 1_000u64, staking_type: StakingType::MaximumCapacity };
		let releasable_amount = Capacity::get_releasable_amount(&account, &staking_account);
		let staking_type = Capacity::get_staking_type_in_force(StakingType::MaximumCapacity);

		assert_eq!(staking_type, StakingType::MaximumCapacity);
		assert_eq!(releasable_amount, staking_account.active);
	})
}
