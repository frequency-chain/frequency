use super::{mock::*, testing_utils::*};
use crate::{
	BalanceOf, CapacityDetails, Config, Error, Event, FreezeReason, StakingDetails,
	StakingType::MaximumCapacity,
};
use common_primitives::{capacity::Nontransferable, msa::MessageSourceId};
use frame_support::{assert_noop, assert_ok, traits::fungible::InspectFreeze};
use sp_runtime::ArithmeticError;

#[test]
fn stake_works() {
	new_test_ext().execute_with(|| {
		let account = 200;
		let target: MessageSourceId = 1;
		let amount = 50;
		let capacity = 5;
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account), target, amount));

		// Check that StakingAccountLedger is updated.
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().active, 50);

		// Check that StakingTargetLedger is updated.
		assert_eq!(Capacity::get_target_for(account, target).unwrap().amount, 50);
		assert_eq!(Capacity::get_target_for(account, target).unwrap().capacity, 5);

		// Check that CapacityLedger is updated.
		let capacity_details = Capacity::get_capacity_for(target).unwrap();

		assert_eq!(
			CapacityDetails {
				remaining_capacity: 5,
				total_tokens_staked: 50,
				total_capacity_issued: 5,
				last_replenished_epoch: 0,
			},
			capacity_details
		);

		let events = staking_events();
		assert_eq!(events.first().unwrap(), &Event::Staked { account, target, amount, capacity });

		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::CapacityStaking.into(),
				&account
			),
			amount
		);
	});
}

#[test]
fn stake_errors_invalid_target_when_target_is_not_registered_provider() {
	new_test_ext().execute_with(|| {
		let account = 100;
		let target: MessageSourceId = 1;
		let amount = 1;
		assert_noop!(
			Capacity::stake(RuntimeOrigin::signed(account), target, amount),
			Error::<Test>::InvalidTarget
		);
	});
}

#[test]
fn stake_errors_insufficient_staking_amount_when_staking_below_minimum_staking_amount() {
	new_test_ext().execute_with(|| {
		let account = 200;
		let target: MessageSourceId = 1;
		let amount = 1;
		register_provider(target, String::from("Foo"));
		assert_noop!(
			Capacity::stake(RuntimeOrigin::signed(account), target, amount),
			Error::<Test>::StakingAmountBelowMinimum
		);
	});
}

#[test]
fn stake_errors_zero_amount_not_allowed() {
	new_test_ext().execute_with(|| {
		let account = 100;
		let target: MessageSourceId = 1;
		let amount = 0;
		assert_noop!(
			Capacity::stake(RuntimeOrigin::signed(account), target, amount),
			Error::<Test>::ZeroAmountNotAllowed
		);
	});
}

#[test]
fn stake_increase_stake_amount_works() {
	new_test_ext().execute_with(|| {
		let account = 300;
		let target: MessageSourceId = 1;
		let initial_amount = 50;
		let capacity = 5;
		register_provider(target, String::from("Foo"));

		// First Stake
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account), target, initial_amount));
		let events = staking_events();
		assert_eq!(
			events.first().unwrap(),
			&Event::Staked { account, target, amount: initial_amount, capacity }
		);
		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::CapacityStaking.into(),
				&account
			),
			50
		);

		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));

		// run to epoch 2
		run_to_block(21);

		let additional_amount = 100;
		let capacity = 10;
		// Additional Stake
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account), target, additional_amount));

		// Check that StakingAccountLedger is updated.
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().active, 150);

		// Check that StakingTargetLedger is updated.
		assert_eq!(Capacity::get_target_for(account, target).unwrap().amount, 150);
		assert_eq!(Capacity::get_target_for(account, target).unwrap().capacity, 15);

		// Check that CapacityLedger is updated.
		assert_eq!(Capacity::get_capacity_for(target).unwrap().remaining_capacity, 15);
		assert_eq!(Capacity::get_capacity_for(target).unwrap().total_capacity_issued, 15);
		assert_eq!(Capacity::get_capacity_for(target).unwrap().last_replenished_epoch, 0);

		let events = staking_events();
		assert_eq!(
			events.last().unwrap(),
			&Event::Staked { account, target, amount: additional_amount, capacity }
		);

		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::CapacityStaking.into(),
				&account
			),
			150
		);
	});
}

#[test]
fn stake_multiple_accounts_can_stake_to_the_same_target() {
	new_test_ext().execute_with(|| {
		new_test_ext().execute_with(|| {
			let target: MessageSourceId = 1;
			register_provider(target, String::from("Foo"));
			let account_1 = 200;
			let stake_amount_1 = 50;

			assert_ok!(Capacity::stake(RuntimeOrigin::signed(account_1), target, stake_amount_1));

			// Check that StakingAccountLedger is updated.
			assert_eq!(Capacity::get_staking_account_for(account_1).unwrap().active, 50);

			// Check that StakingTargetLedger is updated.
			assert_eq!(Capacity::get_target_for(account_1, target).unwrap().amount, 50);
			assert_eq!(Capacity::get_target_for(account_1, target).unwrap().capacity, 5);

			// Check that CapacityLedger is updated.
			assert_eq!(Capacity::get_capacity_for(target).unwrap().remaining_capacity, 5);
			assert_eq!(Capacity::get_capacity_for(target).unwrap().total_capacity_issued, 5);
			assert_eq!(Capacity::get_capacity_for(target).unwrap().last_replenished_epoch, 0);

			assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));

			// run to epoch 2
			run_to_block(21);

			let account_2 = 300;
			let stake_amount_2 = 100;

			assert_ok!(Capacity::stake(RuntimeOrigin::signed(account_2), target, stake_amount_2));

			// Check that StakingAccountLedger is updated.
			assert_eq!(Capacity::get_staking_account_for(account_2).unwrap().active, 100);

			// Check that StakingTargetLedger is updated.
			assert_eq!(Capacity::get_target_for(account_2, target).unwrap().amount, 100);
			assert_eq!(Capacity::get_target_for(account_2, target).unwrap().capacity, 10);

			// Check that CapacityLedger is updated.
			assert_eq!(Capacity::get_capacity_for(target).unwrap().remaining_capacity, 15);
			assert_eq!(Capacity::get_capacity_for(target).unwrap().total_capacity_issued, 15);
			assert_eq!(Capacity::get_capacity_for(target).unwrap().last_replenished_epoch, 0);
		});
	});
}

#[test]
fn stake_an_account_can_stake_to_multiple_targets() {
	new_test_ext().execute_with(|| {
		let target_1: MessageSourceId = 1;
		let target_2: MessageSourceId = 2;
		register_provider(target_1, String::from("Foo"));
		register_provider(target_2, String::from("Boo"));

		let account = 400;
		let amount_1 = 100;
		let amount_2 = 200;

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account), target_1, amount_1));

		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));

		// run to epoch 2
		run_to_block(21);
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account), target_2, amount_2));

		// Check that StakingAccountLedger is updated.
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().active, 300);

		// Check that StakingTargetLedger is updated for target 1.
		assert_eq!(Capacity::get_target_for(account, target_1).unwrap().amount, 100);
		assert_eq!(Capacity::get_target_for(account, target_1).unwrap().capacity, 10);

		// Check that StakingTargetLedger is updated for target 2.
		assert_eq!(Capacity::get_target_for(account, target_2).unwrap().amount, 200);
		assert_eq!(Capacity::get_target_for(account, target_2).unwrap().capacity, 20);

		// Check that CapacityLedger is updated for target 1.
		assert_eq!(Capacity::get_capacity_for(target_1).unwrap().remaining_capacity, 10);
		assert_eq!(Capacity::get_capacity_for(target_1).unwrap().total_capacity_issued, 10);
		assert_eq!(Capacity::get_capacity_for(target_1).unwrap().last_replenished_epoch, 0);

		// Check that CapacityLedger is updated for target 2.
		assert_eq!(Capacity::get_capacity_for(target_2).unwrap().remaining_capacity, 20);
		assert_eq!(Capacity::get_capacity_for(target_2).unwrap().total_capacity_issued, 20);
		assert_eq!(Capacity::get_capacity_for(target_2).unwrap().last_replenished_epoch, 0);
	});
}

#[test]
fn stake_when_staking_amount_is_greater_than_free_balance_it_stakes_maximum() {
	new_test_ext().execute_with(|| {
		let target: MessageSourceId = 1;
		register_provider(target, String::from("Foo"));
		let account = 200;
		// An amount greater than the free balance
		let amount = 230;

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account), target, amount));

		// Check that StakingAccountLedger is updated.
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().active, 190);

		// Check that StakingTargetLedger is updated.
		assert_eq!(Capacity::get_target_for(account, target).unwrap().amount, 190);
		assert_eq!(Capacity::get_target_for(account, target).unwrap().capacity, 19);

		// Check that CapacityLedger is updated.
		assert_eq!(Capacity::get_capacity_for(target).unwrap().remaining_capacity, 19);
		assert_eq!(Capacity::get_capacity_for(target).unwrap().total_capacity_issued, 19);
		assert_eq!(Capacity::get_capacity_for(target).unwrap().last_replenished_epoch, 0);
	});
}

#[test]
fn get_stakable_amount_for_works() {
	new_test_ext().execute_with(|| {
		let account = 200;
		// An amount greater than the free balance
		let amount = 230;
		let res: u64 = Capacity::get_stakable_amount_for(&account, amount);
		assert_eq!(res, 190);
	})
}

#[test]
fn stake_when_staking_amount_is_less_than_min_token_balance_it_errors() {
	new_test_ext().execute_with(|| {
		let target: MessageSourceId = 1;
		register_provider(target, String::from("Foo"));
		let account = 50;
		// An amount that leaves less than the minimum token balance
		let amount = 4;

		assert_noop!(
			Capacity::stake(RuntimeOrigin::signed(account), target, amount),
			Error::<Test>::BalanceTooLowtoStake
		);
	});
}

#[test]
fn ensure_can_stake_errors_with_zero_amount_not_allowed() {
	new_test_ext().execute_with(|| {
		let account = 100;
		let target: MessageSourceId = 1;
		let amount = 0;
		assert_noop!(
			Capacity::ensure_can_stake(&account, target, amount, MaximumCapacity),
			Error::<Test>::ZeroAmountNotAllowed
		);
	});
}

#[test]
fn increase_stake_and_issue_capacity_errors_with_overflow() {
	new_test_ext().execute_with(|| {
		let target: MessageSourceId = 1;
		let staker = 200;
		let amount = 10;
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker), target, amount));
		let mut staking_account = Capacity::get_staking_account_for(staker).unwrap_or_default();

		let overflow_amount = u64::MAX;

		assert_noop!(
			Capacity::increase_stake_and_issue_capacity(
				&staker,
				&mut staking_account,
				target,
				overflow_amount
			),
			ArithmeticError::Overflow
		);
	});
}

#[test]
fn ensure_can_stake_errors_invalid_target() {
	new_test_ext().execute_with(|| {
		let account = 100;
		let target: MessageSourceId = 1;
		let amount = 1;

		assert_noop!(
			Capacity::ensure_can_stake(&account, target, amount, MaximumCapacity),
			Error::<Test>::InvalidTarget
		);
	});
}

#[test]
fn ensure_can_stake_errors_insufficient_staking_amount() {
	new_test_ext().execute_with(|| {
		let account = 200;
		let target: MessageSourceId = 1;
		let amount = 4;
		register_provider(target, String::from("Foo"));

		assert_noop!(
			Capacity::ensure_can_stake(&account, target, amount, MaximumCapacity),
			Error::<Test>::StakingAmountBelowMinimum
		);
	});
}

#[test]
fn ensure_can_stake_is_successful() {
	new_test_ext().execute_with(|| {
		let account = 200;
		let target: MessageSourceId = 1;
		let amount = 10;
		register_provider(target, String::from("Foo"));

		let staking_details = StakingDetails::<Test>::default();
		assert_ok!(
			Capacity::ensure_can_stake(&account, target, amount, MaximumCapacity),
			(staking_details, BalanceOf::<Test>::from(10u64))
		);
	});
}

#[test]
fn increase_stake_and_issue_capacity_is_successful() {
	new_test_ext().execute_with(|| {
		let staker = 10_000; // has 10_000 token
		let target: MessageSourceId = 1;
		let amount = 550;
		let mut staking_account = StakingDetails::<Test>::default();

		assert_ok!(Capacity::increase_stake_and_issue_capacity(
			&staker,
			&mut staking_account,
			target,
			amount
		));

		assert_eq!(staking_account.active, amount);

		let capacity_details = Capacity::get_capacity_for(&target).unwrap();

		assert_eq!(capacity_details.remaining_capacity, 55);
		assert_eq!(capacity_details.total_capacity_issued, 55);
		assert_eq!(capacity_details.last_replenished_epoch, 0);

		let target_details = Capacity::get_target_for(&staker, &target).unwrap();

		assert_eq!(target_details.amount, amount);
		assert_eq!(target_details.capacity, 55);
	});
}

#[test]
fn stake_when_there_are_unlocks_sets_lock_correctly() {
	new_test_ext().execute_with(|| {
		let staker = 600;
		let target1 = 2;
		let target2 = 3;
		register_provider(target1, String::from("target1"));
		register_provider(target2, String::from("target2"));
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker), target1, 20));

		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(staker), target1, 5));

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker), target2, 20));

		// should all still be locked.
		assert_eq!(Balances::balance_frozen(&FreezeReason::CapacityStaking.into(), &staker), 40);
	})
}

#[test]
fn impl_deposit_is_successful() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let remaining_amount = BalanceOf::<Test>::from(5u32);
		let total_available_amount = BalanceOf::<Test>::from(10u32);
		let _ = create_capacity_account_and_fund(
			target_msa_id,
			remaining_amount,
			total_available_amount,
			1u32,
		);
		let amount = BalanceOf::<Test>::from(5u32);
		let capacity = BalanceOf::<Test>::from(1u32);
		assert_ok!(Capacity::deposit(target_msa_id, amount, capacity));
	});
}

#[test]
fn impl_deposit_errors_target_capacity_not_found() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let amount = BalanceOf::<Test>::from(10u32);
		let capacity = BalanceOf::<Test>::from(5u32);

		assert_noop!(
			Capacity::deposit(target_msa_id, amount, capacity),
			Error::<Test>::TargetCapacityNotFound
		);
	});
}
