use super::*;
use crate as pallet_capacity;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok, traits::WithdrawReasons, BoundedVec};
use testing_utils::{register_provider, run_to_block, staking_events};

#[test]
fn stake_works() {
	new_test_ext().execute_with(|| {
		let account = 200;
		let target: MessageSourceId = 1;
		let amount = 5;
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account), target, amount));

		// Check that StakingAccountLedger is updated.
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().total, amount);
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().active, amount);
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().unlocking.len(), 0);

		// Check that StakingTargetLedger is updated.
		assert_eq!(Capacity::get_target_for(account, target).unwrap().amount, amount);
		assert_eq!(Capacity::get_target_for(account, target).unwrap().capacity, amount);

		// Check that CapacityLedger is updated.
		assert_eq!(Capacity::get_capacity_for(target).unwrap().remaining, amount);
		assert_eq!(Capacity::get_capacity_for(target).unwrap().total_available, amount);
		assert_eq!(Capacity::get_capacity_for(target).unwrap().last_replenished_epoch, 1);

		let events = staking_events();
		assert_eq!(events.first().unwrap(), &Event::Staked { account, target, amount });

		assert_eq!(Balances::locks(&account)[0].amount, amount);
		assert_eq!(Balances::locks(&account)[0].reasons, WithdrawReasons::all().into());
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
		let account = 100;
		let target: MessageSourceId = 1;
		let amount = 1;
		register_provider(target, String::from("Foo"));
		assert_noop!(
			Capacity::stake(RuntimeOrigin::signed(account), target, amount),
			Error::<Test>::InsufficientStakingAmount
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
		let account = 200;
		let target: MessageSourceId = 1;
		let initial_amount = 5;
		register_provider(target, String::from("Foo"));

		// First Stake
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account), target, initial_amount));
		let events = staking_events();
		assert_eq!(
			events.first().unwrap(),
			&Event::Staked { account, target, amount: initial_amount }
		);

		assert_eq!(Balances::locks(&account)[0].amount, 5);
		assert_eq!(Balances::locks(&account)[0].reasons, WithdrawReasons::all().into());

		run_to_block(2);

		let additional_amount = 10;
		// Additional Stake
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account), target, additional_amount));

		// Check that StakingAccountLedger is updated.
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().total, 15);
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().active, 15);
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().unlocking.len(), 0);

		// Check that StakingTargetLedger is updated.
		assert_eq!(Capacity::get_target_for(account, target).unwrap().amount, 15);
		assert_eq!(Capacity::get_target_for(account, target).unwrap().capacity, 15);

		// Check that CapacityLedger is updated.
		assert_eq!(Capacity::get_capacity_for(target).unwrap().remaining, 15);
		assert_eq!(Capacity::get_capacity_for(target).unwrap().total_available, 15);
		assert_eq!(Capacity::get_capacity_for(target).unwrap().last_replenished_epoch, 2);

		let events = staking_events();
		assert_eq!(
			events.first().unwrap(),
			&Event::Staked { account, target, amount: additional_amount }
		);

		assert_eq!(Balances::locks(&account)[0].amount, 15);
		assert_eq!(Balances::locks(&account)[0].reasons, WithdrawReasons::all().into());
	});
}

#[test]
fn it_configures_staking_minimum_greater_than_or_equal_to_existential_deposit() {
	new_test_ext().execute_with(|| {
		let minimum_staking_balance_config: BalanceOf<Test> =
			<Test as pallet_capacity::Config>::MinimumStakingAmount::get();
		assert!(
			minimum_staking_balance_config >=
				<Test as pallet_capacity::Config>::Currency::minimum_balance()
		)
	});
}

#[test]
fn stake_multiple_accounts_can_stake_to_the_same_target() {
	new_test_ext().execute_with(|| {
		new_test_ext().execute_with(|| {
			let target: MessageSourceId = 1;
			register_provider(target, String::from("Foo"));
			let account_1 = 200;
			let stake_amount_1 = 5;

			assert_ok!(Capacity::stake(RuntimeOrigin::signed(account_1), target, stake_amount_1));

			// Check that StakingAccountLedger is updated.
			assert_eq!(Capacity::get_staking_account_for(account_1).unwrap().total, 5);
			assert_eq!(Capacity::get_staking_account_for(account_1).unwrap().active, 5);
			assert_eq!(Capacity::get_staking_account_for(account_1).unwrap().unlocking.len(), 0);

			// Check that StakingTargetLedger is updated.
			assert_eq!(Capacity::get_target_for(account_1, target).unwrap().amount, 5);
			assert_eq!(Capacity::get_target_for(account_1, target).unwrap().capacity, 5);

			// Check that CapacityLedger is updated.
			assert_eq!(Capacity::get_capacity_for(target).unwrap().remaining, 5);
			assert_eq!(Capacity::get_capacity_for(target).unwrap().total_available, 5);
			assert_eq!(Capacity::get_capacity_for(target).unwrap().last_replenished_epoch, 1);

			run_to_block(2);

			let account_2 = 300;
			let stake_amount_2 = 10;

			assert_ok!(Capacity::stake(RuntimeOrigin::signed(account_2), target, stake_amount_2));

			// Check that StakingAccountLedger is updated.
			assert_eq!(Capacity::get_staking_account_for(account_2).unwrap().total, 10);
			assert_eq!(Capacity::get_staking_account_for(account_2).unwrap().active, 10);
			assert_eq!(Capacity::get_staking_account_for(account_2).unwrap().unlocking.len(), 0);

			// Check that StakingTargetLedger is updated.
			assert_eq!(Capacity::get_target_for(account_2, target).unwrap().amount, 10);
			assert_eq!(Capacity::get_target_for(account_2, target).unwrap().capacity, 10);

			// Check that CapacityLedger is updated.
			assert_eq!(Capacity::get_capacity_for(target).unwrap().remaining, 15);
			assert_eq!(Capacity::get_capacity_for(target).unwrap().total_available, 15);
			assert_eq!(Capacity::get_capacity_for(target).unwrap().last_replenished_epoch, 2);
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

		let account = 200;
		let amount_1 = 10;
		let amount_2 = 7;

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account), target_1, amount_1));
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().total, 10);

		run_to_block(2);
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account), target_2, amount_2));

		// Check that StakingAccountLedger is updated.
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().total, 17);
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().active, 17);
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().unlocking.len(), 0);

		// Check that StakingTargetLedger is updated for target 1.
		assert_eq!(Capacity::get_target_for(account, target_1).unwrap().amount, 10);
		assert_eq!(Capacity::get_target_for(account, target_1).unwrap().capacity, 10);

		// Check that StakingTargetLedger is updated for target 2.
		assert_eq!(Capacity::get_target_for(account, target_2).unwrap().amount, 7);
		assert_eq!(Capacity::get_target_for(account, target_2).unwrap().capacity, 7);

		// Check that CapacityLedger is updated for target 1.
		assert_eq!(Capacity::get_capacity_for(target_1).unwrap().remaining, 10);
		assert_eq!(Capacity::get_capacity_for(target_1).unwrap().total_available, 10);
		assert_eq!(Capacity::get_capacity_for(target_1).unwrap().last_replenished_epoch, 1);

		// Check that CapacityLedger is updated for target 2.

		assert_eq!(Capacity::get_capacity_for(target_2).unwrap().remaining, 7);
		assert_eq!(Capacity::get_capacity_for(target_2).unwrap().total_available, 7);
		assert_eq!(Capacity::get_capacity_for(target_2).unwrap().last_replenished_epoch, 2);
	});
}

#[test]
fn stake_when_staking_amount_is_greater_than_free_balance_it_stakes_maximum() {
	new_test_ext().execute_with(|| {
		let target: MessageSourceId = 1;
		register_provider(target, String::from("Foo"));
		let account = 100;
		// An amount greater than the free balance
		let amount = 13;

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account), target, amount));

		// Check that StakingAccountLedger is updated.
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().total, 10);
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().active, 10);
		assert_eq!(Capacity::get_staking_account_for(account).unwrap().unlocking.len(), 0);

		// Check that StakingTargetLedger is updated.
		assert_eq!(Capacity::get_target_for(account, target).unwrap().amount, 10);
		assert_eq!(Capacity::get_target_for(account, target).unwrap().capacity, 10);

		// Check that CapacityLedger is updated.
		assert_eq!(Capacity::get_capacity_for(target).unwrap().remaining, 10);
		assert_eq!(Capacity::get_capacity_for(target).unwrap().total_available, 10);
		assert_eq!(Capacity::get_capacity_for(target).unwrap().last_replenished_epoch, 1);
	});
}

#[test]
fn ensure_can_stake_errors_with_zero_amount_not_allowed() {
	new_test_ext().execute_with(|| {
		let account = 100;
		let target: MessageSourceId = 1;
		let amount = 0;
		assert_noop!(
			Capacity::ensure_can_stake(&account, target, amount),
			Error::<Test>::ZeroAmountNotAllowed
		);
	});
}

#[test]
fn increase_stake_and_issue_capacity_errors_with_overflow() {
	new_test_ext().execute_with(|| {
		let target: MessageSourceId = 1;
		let staker = 100;
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
			Capacity::ensure_can_stake(&account, target, amount),
			Error::<Test>::InvalidTarget
		);
	});
}

#[test]
fn ensure_can_stake_errors_insufficient_staking_amount() {
	new_test_ext().execute_with(|| {
		let account = 100;
		let target: MessageSourceId = 1;
		let amount = 4;
		register_provider(target, String::from("Foo"));

		assert_noop!(
			Capacity::ensure_can_stake(&account, target, amount),
			Error::<Test>::InsufficientStakingAmount
		);
	});
}

#[test]
fn ensure_can_stake_is_successful() {
	new_test_ext().execute_with(|| {
		let account = 100;
		let target: MessageSourceId = 1;
		let amount = 10;
		register_provider(target, String::from("Foo"));

		let staking_details = StakingAccountDetails::<Test>::default();
		assert_ok!(
			Capacity::ensure_can_stake(&account, target, amount),
			(staking_details, BalanceOf::<Test>::from(10u64))
		);
	});
}

#[test]
fn increase_stake_and_issue_capacity_is_successful() {
	new_test_ext().execute_with(|| {
		let staker = 100;
		let target: MessageSourceId = 1;
		let amount = 55;
		let mut staking_account = StakingAccountDetails::<Test>::default();

		assert_ok!(Capacity::increase_stake_and_issue_capacity(
			&staker,
			&mut staking_account,
			target,
			amount
		));

		assert_eq!(staking_account.total, 55);
		assert_eq!(staking_account.active, 55);
		assert_eq!(staking_account.unlocking.len(), 0);

		let capacity_details = Capacity::get_capacity_for(&target).unwrap();

		assert_eq!(capacity_details.remaining, 55);
		assert_eq!(capacity_details.total_available, 55);
		assert_eq!(capacity_details.last_replenished_epoch, 1);

		let target_details = Capacity::get_target_for(&staker, &target).unwrap();

		assert_eq!(target_details.amount, 55);
		assert_eq!(target_details.capacity, 55);
	});
}

#[test]
fn set_staking_account_is_succesful() {
	new_test_ext().execute_with(|| {
		let staker = 100;
		let mut staking_account = StakingAccountDetails::<Test>::default();
		staking_account.increase_by(55);

		Capacity::set_staking_account(&staker, &staking_account);

		assert_eq!(Balances::locks(&staker)[0].amount, 55);
	});
}

#[test]
fn set_target_details_is_successful() {
	new_test_ext().execute_with(|| {
		let staker = 100;
		let target: MessageSourceId = 1;

		assert_eq!(Capacity::get_target_for(&staker, target), None);

		let mut target_details = StakingTargetDetails::<BalanceOf<Test>>::default();
		target_details.amount = 10;
		target_details.capacity = 10;

		Capacity::set_target_details_for(&staker, target, target_details);

		let stored_target_details = Capacity::get_target_for(&staker, target).unwrap();

		assert_eq!(stored_target_details.amount, 10);
		assert_eq!(stored_target_details.capacity, 10);
	});
}

#[test]
fn set_capacity_details_is_successful() {
	new_test_ext().execute_with(|| {
		let target: MessageSourceId = 1;

		assert_eq!(Capacity::get_capacity_for(target), None);

		let mut capacity_details = CapacityDetails::<
			BalanceOf<Test>,
			<Test as frame_system::Config>::BlockNumber,
		>::default();

		capacity_details.remaining = 10;
		capacity_details.total_available = 10;
		capacity_details.last_replenished_epoch = 1;

		Capacity::set_capacity_for(target, capacity_details);

		let stored_capacity_details = Capacity::get_capacity_for(target).unwrap();

		assert_eq!(stored_capacity_details.remaining, 10);
		assert_eq!(stored_capacity_details.total_available, 10);
		assert_eq!(stored_capacity_details.last_replenished_epoch, 1);
	});
}

#[test]
fn impl_staking_account_details_increase_by() {
	new_test_ext().execute_with(|| {
		let mut staking_account = StakingAccountDetails::<Test>::default();
		assert_eq!(staking_account.increase_by(10), Some(()));

		assert_eq!(
			staking_account,
			StakingAccountDetails::<Test> {
				active: BalanceOf::<Test>::from(10u64),
				total: BalanceOf::<Test>::from(10u64),
				unlocking: BoundedVec::default(),
			}
		)
	});
}

#[test]
fn impl_staking_account_details_default() {
	new_test_ext().execute_with(|| {
		assert_eq!(
			StakingAccountDetails::<Test>::default(),
			StakingAccountDetails::<Test> {
				active: BalanceOf::<Test>::zero(),
				total: BalanceOf::<Test>::zero(),
				unlocking: BoundedVec::default(),
			},
		);
	});
}

#[test]
fn impl_staking_account_details_get_stakable_amount_for() {
	new_test_ext().execute_with(|| {
		let account = 100;
		let staking_account = StakingAccountDetails::<Test>::default();

		// When staking all of free balance.
		assert_eq!(staking_account.get_stakable_amount_for(&account, 10), 10);

		// When staking an amount below free balance.
		assert_eq!(staking_account.get_stakable_amount_for(&account, 5), 5);

		// When staking an amount above account free balance. It stakes all of the free balance.
		assert_eq!(staking_account.get_stakable_amount_for(&account, 25), 10);
	});
}

#[test]
fn impl_staking_target_details_increase_by() {
	new_test_ext().execute_with(|| {
		let mut staking_target = StakingTargetDetails::<BalanceOf<Test>>::default();
		assert_eq!(staking_target.increase_by(10, 10), Some(()));

		assert_eq!(
			staking_target,
			StakingTargetDetails::<BalanceOf<Test>> {
				amount: BalanceOf::<Test>::from(10u64),
				capacity: 10
			}
		)
	});
}

#[test]
fn impl_staking_capacity_details_increase_by() {
	new_test_ext().execute_with(|| {
		let mut capacity_details = CapacityDetails::<
			BalanceOf<Test>,
			<Test as frame_system::Config>::BlockNumber,
		>::default();
		assert_eq!(capacity_details.increase_by(10, 1), Some(()));

		assert_eq!(
			capacity_details,
			CapacityDetails::<BalanceOf<Test>, <Test as frame_system::Config>::BlockNumber> {
				remaining: BalanceOf::<Test>::from(10u64),
				total_available: BalanceOf::<Test>::from(10u64),
				last_replenished_epoch: <Test as frame_system::Config>::BlockNumber::from(1u32)
			}
		)
	});
}
