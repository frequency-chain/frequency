use core::ops::Add;

use super::mock::*;
use crate as pallet_capacity;
use crate::{
	tests::testing_utils::{capacity_events, register_provider},
	BalanceOf, BlockNumberFor, CapacityLedger, Error, Event, FreezeReason, StakingAccountLedger,
	StakingConfigProvider, StakingDetails, StakingTargetDetails, StakingTargetLedger, StakingType,
	StakingType::FlexibleBoost,
	UnlockChunk, UnstakeUnlocks,
};
use common_primitives::{capacity::StakingType::CommittedBoost, msa::MessageSourceId};
use frame_support::{
	assert_err, assert_noop, assert_ok,
	dispatch::RawOrigin,
	traits::{fungible::InspectFreeze, Get},
	BoundedVec,
};
use pallet_capacity::Config;
use sp_runtime::Perbill;

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
fn amount_releasable_at_end_of_staged_release_should_be_total_initial_amount() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let staking_account =
			StakingDetails { active: 1_000u64, staking_type: StakingType::CommittedBoost };

		set_pte_block::<Test>(Some(1));
		let staking_config = <Test as Config>::StakingConfigProvider::get(CommittedBoost);
		let block_number = 2 +
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
		assert_eq!(staking_type, CommittedBoost);
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

#[test]
fn committed_boost_works() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 200;
		let capacity = 10; // Maximized stake (10% of staked amount) * 50% (in trait impl)
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::committed_boost(RuntimeOrigin::signed(account), target, amount));

		// Check that StakingAccountLedger is updated.
		let boost_account: StakingDetails<Test> =
			StakingAccountLedger::<Test>::get(account).unwrap();

		// Check that the staking account has the correct staking type.
		assert_eq!(boost_account.active, 200);
		assert_eq!(boost_account.staking_type, StakingType::CommittedBoost);

		// Check that the capacity generated is correct. (5% of amount staked, since 10% is what's in the mock)
		let capacity_details = CapacityLedger::<Test>::get(target).unwrap();
		assert_eq!(capacity_details.total_capacity_issued, capacity);

		let events = capacity_events();
		assert_eq!(
			events.first().unwrap(),
			&Event::StakedV2 {
				account,
				target,
				amount,
				capacity,
				staking_type: StakingType::CommittedBoost
			}
		);

		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::CapacityStaking.into(),
				&account
			),
			200u64
		);

		let target_details = StakingTargetLedger::<Test>::get(account, target).unwrap();
		assert_eq!(target_details.amount, amount);
	});
}

#[test]
fn committed_boost_unstake_should_fail_before_pte() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 200;
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::committed_boost(RuntimeOrigin::signed(account), target, amount));

		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(account), target, amount),
			Error::<Test>::InsufficientUnfrozenStakingBalance
		);
	});
}

#[test]
fn committed_boost_unstake_should_fail_after_pte_and_before_release_stage() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 200;
		let pte_block: BlockNumberFor<Test> = 500u32.into();
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::committed_boost(RuntimeOrigin::signed(account), target, amount));

		System::set_block_number(pte_block + 1);
		assert_ok!(Capacity::set_pte_via_governance(RawOrigin::Root.into(), pte_block,));
		System::set_block_number(2000);

		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(account), target, amount),
			Error::<Test>::InsufficientUnfrozenStakingBalance
		);
	});
}

#[test]
fn committed_boost_unstake_should_unstake_some_amount_in_release_stage() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 200;
		let pte_block: BlockNumberFor<Test> = 500u32.into();
		let cfg = <Test as crate::Config>::StakingConfigProvider::get(StakingType::CommittedBoost);
		let release_middle: u32 =
			cfg.commitment_release_stage_blocks * cfg.commitment_release_stages / 2;
		let unstaking_amount = 10;
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::committed_boost(RuntimeOrigin::signed(account), target, amount));

		System::set_block_number(pte_block + 1);
		assert_ok!(Capacity::set_pte_via_governance(RawOrigin::Root.into(), pte_block,));
		System::set_block_number(pte_block as u32 + cfg.initial_commitment_blocks + release_middle);

		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(account), target, unstaking_amount),);

		// Assert that staking account detail values are decremented correctly after unstaking
		let staking_account_details = StakingAccountLedger::<Test>::get(account).unwrap();

		let expected_unlocking_chunks: BoundedVec<
			UnlockChunk<BalanceOf<Test>, <Test as Config>::EpochNumber>,
			<Test as Config>::MaxUnlockingChunks,
		> = BoundedVec::try_from(vec![UnlockChunk { value: unstaking_amount, thaw_at: 2u32 }])
			.unwrap();

		let unlocking = UnstakeUnlocks::<Test>::get(account).unwrap();
		assert_eq!(unlocking, expected_unlocking_chunks);

		assert_eq!(
			StakingDetails::<Test> {
				active: amount - unstaking_amount,
				staking_type: StakingType::CommittedBoost
			},
			staking_account_details,
		);

		// Assert that staking target detail values are decremented correctly after unstaking
		let staking_target_details = StakingTargetLedger::<Test>::get(account, target).unwrap();

		assert_eq!(
			staking_target_details,
			StakingTargetDetails::<BalanceOf<Test>> {
				amount: amount - unstaking_amount,
				capacity: 9u64
			}
		);

		let events = capacity_events();
		assert_eq!(
			events.last().unwrap(),
			&Event::UnStaked { account, target, amount: unstaking_amount, capacity: 1u64 }
		);
	});
}

#[test]
fn committed_boost_unstake_should_unstake_all_amount_after_release_stage() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 200;
		let pte_block: BlockNumberFor<Test> = 500u32.into();
		let cfg = <Test as crate::Config>::StakingConfigProvider::get(StakingType::CommittedBoost);
		let after_release: u32 =
			cfg.commitment_release_stage_blocks * cfg.commitment_release_stages + 1u32;
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::committed_boost(RuntimeOrigin::signed(account), target, amount));

		System::set_block_number(pte_block + 1);
		assert_ok!(Capacity::set_pte_via_governance(RawOrigin::Root.into(), pte_block,));
		System::set_block_number(
			pte_block as u32 + cfg.initial_commitment_blocks + after_release as u32,
		);

		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(account), target, amount),);

		// Assert that staking account detail values are decremented correctly after unstaking
		let staking_account_details = StakingAccountLedger::<Test>::get(account);

		let expected_unlocking_chunks: BoundedVec<
			UnlockChunk<BalanceOf<Test>, <Test as Config>::EpochNumber>,
			<Test as Config>::MaxUnlockingChunks,
		> = BoundedVec::try_from(vec![UnlockChunk { value: amount, thaw_at: 2u32 }]).unwrap();

		let unlocking = UnstakeUnlocks::<Test>::get(account).unwrap();
		assert_eq!(unlocking, expected_unlocking_chunks);

		assert_eq!(None, staking_account_details,);

		// Assert that staking target detail values are decremented correctly after unstaking
		let staking_target_details = StakingTargetLedger::<Test>::get(account, target);

		assert_eq!(staking_target_details, None,);

		let events = capacity_events();
		assert_eq!(
			events.last().unwrap(),
			&Event::UnStaked { account, target, amount, capacity: 10u64 }
		);
	});
}

#[test]
fn committed_boost_unstake_should_unstake_all_amount_after_pte_expiration() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 200;
		let expiration_block: u32 =
			<Test as crate::Config>::CommittedBoostFailsafeUnlockBlockNumber::get();
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::committed_boost(RuntimeOrigin::signed(account), target, amount));

		System::set_block_number(expiration_block + 1);

		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(account), target, amount),);

		// Assert that staking account detail values are decremented correctly after unstaking
		let staking_account_details = StakingAccountLedger::<Test>::get(account);

		let expected_unlocking_chunks: BoundedVec<
			UnlockChunk<BalanceOf<Test>, <Test as Config>::EpochNumber>,
			<Test as Config>::MaxUnlockingChunks,
		> = BoundedVec::try_from(vec![UnlockChunk { value: amount, thaw_at: 2u32 }]).unwrap();

		let unlocking = UnstakeUnlocks::<Test>::get(account).unwrap();
		assert_eq!(unlocking, expected_unlocking_chunks);

		assert_eq!(None, staking_account_details,);

		// Assert that staking target detail values are decremented correctly after unstaking
		let staking_target_details = StakingTargetLedger::<Test>::get(account, target);

		assert_eq!(staking_target_details, None,);

		let events = capacity_events();
		assert_eq!(
			events.last().unwrap(),
			&Event::UnStaked { account, target, amount, capacity: 10u64 }
		);
	});
}

#[test]
fn committed_boost_get_unstakable_amount_for_should_return_zero_before_pte() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 200;
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::committed_boost(RuntimeOrigin::signed(account), target, amount));

		let unstakable = Capacity::get_unstakable_amount_for(&account);

		assert_eq!(unstakable, 0);
	});
}

#[test]
fn committed_boost_get_unstakable_amount_for_should_return_zero_in_committment_stage() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 200;
		let pte_block: BlockNumberFor<Test> = 500u32.into();
		let cfg = <Test as crate::Config>::StakingConfigProvider::get(StakingType::CommittedBoost);

		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::committed_boost(RuntimeOrigin::signed(account), target, amount));

		System::set_block_number(pte_block + 1);
		assert_ok!(Capacity::set_pte_via_governance(RawOrigin::Root.into(), pte_block,));
		System::set_block_number(pte_block + cfg.initial_commitment_blocks - 1);
		let unstakable = Capacity::get_unstakable_amount_for(&account);

		assert_eq!(unstakable, 0);
	});
}

#[test]
fn committed_boost_get_unstakable_amount_for_should_return_all_after_pte_expiration() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 200;
		let expiration_block: u32 =
			<Test as crate::Config>::CommittedBoostFailsafeUnlockBlockNumber::get();
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::committed_boost(RuntimeOrigin::signed(account), target, amount));

		System::set_block_number(expiration_block + 1);

		let unstakable = Capacity::get_unstakable_amount_for(&account);

		assert_eq!(amount, unstakable);
	});
}
