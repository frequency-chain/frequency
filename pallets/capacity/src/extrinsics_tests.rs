use super::*;
use crate as pallet_capacity;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok, BoundedVec};
use sp_runtime::DispatchError::BadOrigin;
use testing_utils::{register_provider, run_to_block, staking_events};

#[test]
fn withdraw_unstaked_happy_path() {
	new_test_ext().execute_with(|| {
		// set up staker and staking account
		let staker = 500;
		// set new unlock chunks using tuples of (value, thaw_at in number of Epochs)
		let unlocks: Vec<(u32, u32)> = vec![(1u32, 2u32), (2u32, 3u32), (3u32, 4u32)];

		// setup_staking_account_for::<Test>(staker, staking_amount, &unlocks);
		let mut staking_account = StakingAccountDetails::<Test>::default();

		// we have 10 total staked, and 6 of those are unstaking.
		staking_account.deposit(10);
		assert_eq!(true, staking_account.set_unlock_chunks(&unlocks));
		assert_eq!(10u64, staking_account.total);
		Capacity::set_staking_account(&staker, &staking_account.into());

		let starting_account = Capacity::get_staking_account_for(&staker).unwrap();

		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));

		// We want to advance to epoch 3 to unlock the first two sets.
		run_to_block(31);
		assert_eq!(3u32, Capacity::get_current_epoch());
		assert_ok!(Capacity::withdraw_unstaked(RuntimeOrigin::signed(staker)));

		let current_account: StakingAccountDetails<Test> =
			Capacity::get_staking_account_for(&staker).unwrap();
		let expected_reaped_value = 3u64;
		assert_eq!(starting_account.total - expected_reaped_value, current_account.total);
		System::assert_last_event(
			Event::StakeWithdrawn { account: staker, amount: expected_reaped_value }.into(),
		);
	})
}

#[test]
fn withdraw_unstaked_correctly_sets_new_lock_state() {
	new_test_ext().execute_with(|| {
		let staker = 500;
		let mut staking_account = StakingAccountDetails::<Test>::default();
		staking_account.deposit(10);

		// set new unlock chunks using tuples of (value, thaw_at)
		let new_unlocks: Vec<(u32, u32)> = vec![(1u32, 2u32), (2u32, 3u32), (3u32, 4u32)];
		assert_eq!(true, staking_account.set_unlock_chunks(&new_unlocks));

		Capacity::set_staking_account(&staker, &staking_account);
		assert_eq!(1, Balances::locks(&staker).len());
		assert_eq!(10u64, Balances::locks(&staker)[0].amount);

		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));

		// Epoch length = 10, we want to run to epoch 3
		run_to_block(31);
		assert_ok!(Capacity::withdraw_unstaked(RuntimeOrigin::signed(staker)));

		assert_eq!(1, Balances::locks(&staker).len());
		assert_eq!(7u64, Balances::locks(&staker)[0].amount);
	})
}

#[test]
fn withdraw_unstaked_cleans_up_storage_and_removes_all_locks_if_no_stake_left() {
	new_test_ext().execute_with(|| {
		let mut staking_account = StakingAccountDetails::<Test>::default();
		let staking_amount: BalanceOf<Test> = 10;
		staking_account.deposit(staking_amount);

		// set new unlock chunks using tuples of (value, thaw_at)
		let new_unlocks: Vec<(u32, u32)> = vec![(10u32, 2u32)];
		assert_eq!(true, staking_account.set_unlock_chunks(&new_unlocks));

		let staker = 500;
		Capacity::set_staking_account(&staker, &staking_account);
		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));

		// Epoch Length = 10 and UnstakingThawPeriod = 2 (epochs)
		run_to_block(30);
		assert_ok!(Capacity::withdraw_unstaked(RuntimeOrigin::signed(staker)));
		assert!(Capacity::get_staking_account_for(&staker).is_none());

		assert_eq!(0, Balances::locks(&staker).len());
	})
}

#[test]
fn withdraw_unstaked_cannot_withdraw_if_no_unstaking_chunks() {
	new_test_ext().execute_with(|| {
		let staker = 500;
		let mut staking_account = StakingAccountDetails::<Test>::default();
		staking_account.deposit(10);
		Capacity::set_staking_account(&staker, &staking_account);
		assert_noop!(
			Capacity::withdraw_unstaked(RuntimeOrigin::signed(500)),
			Error::<Test>::NoUnstakedTokensAvailable
		);
	})
}
#[test]
fn withdraw_unstaked_cannot_withdraw_if_unstaking_chunks_not_thawed() {
	new_test_ext().execute_with(|| {
		let staker = 500;
		let mut staking_account = StakingAccountDetails::<Test>::default();
		staking_account.deposit(10);

		// set new unlock chunks using tuples of (value, thaw_at)
		let new_unlocks: Vec<(u32, u32)> = vec![(1u32, 3u32), (2u32, 40u32), (3u32, 9u32)];
		assert_eq!(true, staking_account.set_unlock_chunks(&new_unlocks));

		Capacity::set_staking_account(&staker, &staking_account);

		run_to_block(2);
		assert_noop!(
			Capacity::withdraw_unstaked(RuntimeOrigin::signed(500)),
			Error::<Test>::NoUnstakedTokensAvailable
		);
	})
}

#[test]
fn withdraw_unstaked_error_if_not_a_staking_account() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Capacity::withdraw_unstaked(RuntimeOrigin::signed(999)),
			Error::<Test>::NotAStakingAccount
		);
	})
}

#[test]
fn stake_works() {
	new_test_ext().execute_with(|| {
		let account = 200;
		let target: MessageSourceId = 1;
		let amount = 5;
		let capacity = 5;
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
		assert_eq!(Capacity::get_capacity_for(target).unwrap().last_replenished_epoch, 0);

		let events = staking_events();
		assert_eq!(events.first().unwrap(), &Event::Staked { account, target, amount, capacity });

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
		let capacity = 5;
		register_provider(target, String::from("Foo"));

		// First Stake
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account), target, initial_amount));
		let events = staking_events();
		assert_eq!(
			events.first().unwrap(),
			&Event::Staked { account, target, amount: initial_amount, capacity }
		);

		assert_eq!(Balances::locks(&account)[0].amount, 5);
		assert_eq!(Balances::locks(&account)[0].reasons, WithdrawReasons::all().into());

		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));

		// run to epoch 2
		run_to_block(21);

		let additional_amount = 10;
		let capacity = 10;
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
			events.last().unwrap(),
			&Event::Staked { account, target, amount: additional_amount, capacity }
		);

		assert_eq!(Balances::locks(&account)[0].amount, 15);
		assert_eq!(Balances::locks(&account)[0].reasons, WithdrawReasons::all().into());
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
			assert_eq!(Capacity::get_capacity_for(target).unwrap().last_replenished_epoch, 0);

			assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));

			// run to epoch 2
			run_to_block(21);

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

		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));

		// run to epoch 2
		run_to_block(21);
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
		assert_eq!(Capacity::get_capacity_for(target_1).unwrap().last_replenished_epoch, 0);

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
		assert_eq!(Capacity::get_capacity_for(target).unwrap().last_replenished_epoch, 0);
	});
}

#[test]
fn unstake_happy_path() {
	new_test_ext().execute_with(|| {
		let token_account = 200;
		let target: MessageSourceId = 1;
		let staking_amount = 10;
		let unstaking_amount = 4;

		register_provider(target, String::from("Test Target"));

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(token_account), target, staking_amount));
		assert_ok!(Capacity::unstake(
			RuntimeOrigin::signed(token_account),
			target,
			unstaking_amount
		));

		// Assert that staking account detail values are decremented correctly after unstaking
		let staking_account_details = Capacity::get_staking_account_for(token_account).unwrap();

		assert_eq!(staking_account_details.unlocking.len(), 1);
		let expected_unlocking_chunks: BoundedVec<
			UnlockChunk<BalanceOf<Test>, <Test as Config>::EpochNumber>,
			<Test as Config>::MaxUnlockingChunks,
		> = BoundedVec::try_from(vec![UnlockChunk { value: unstaking_amount, thaw_at: 2u32 }])
			.unwrap();

		assert_eq!(
			StakingAccountDetails::<Test> {
				active: BalanceOf::<Test>::from(6u64),
				total: BalanceOf::<Test>::from(staking_amount),
				unlocking: expected_unlocking_chunks,
			},
			staking_account_details,
		);

		// Assert that staking target detail values are decremented correctly after unstaking
		let staking_target_details = Capacity::get_target_for(token_account, target).unwrap();

		assert_eq!(
			staking_target_details,
			StakingTargetDetails::<BalanceOf<Test>> {
				amount: BalanceOf::<Test>::from(6u64),
				capacity: BalanceOf::<Test>::from(6u64),
			}
		);

		// Assert that the capacity detail values for the target are decremented properly after unstaking
		let capacity_details = Capacity::get_capacity_for(target).unwrap();

		assert_eq!(
			capacity_details,
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
				remaining: BalanceOf::<Test>::from(10u64),
				total_tokens_staked: BalanceOf::<Test>::from(6u64),
				total_available: BalanceOf::<Test>::from(6u64),
				last_replenished_epoch: <Test as Config>::EpochNumber::from(0u32),
			}
		);

		let events = staking_events();
		assert_eq!(
			events.last().unwrap(),
			&Event::UnStaked {
				account: token_account,
				target,
				amount: unstaking_amount,
				capacity: BalanceOf::<Test>::from(4u64)
			}
		);
	});
}

#[test]
fn unstake_errors_unstaking_amount_is_zero() {
	new_test_ext().execute_with(|| {
		let token_account = 200;
		let target: MessageSourceId = 1;
		let staking_amount = 10;
		let unstaking_amount = 0;

		register_provider(target, String::from("Test Target"));

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(token_account), target, staking_amount));
		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(token_account), target, unstaking_amount),
			Error::<Test>::UnstakedAmountIsZero
		);
	});
}

#[test]
fn unstake_errors_max_unlocking_chunks_exceeded() {
	new_test_ext().execute_with(|| {
		let token_account = 200;
		let target: MessageSourceId = 1;
		let staking_amount = 10;
		let unstaking_amount = 1;

		register_provider(target, String::from("Test Target"));

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(token_account), target, staking_amount));

		for _n in 0..<Test as pallet_capacity::Config>::MaxUnlockingChunks::get() {
			assert_ok!(Capacity::unstake(
				RuntimeOrigin::signed(token_account),
				target,
				unstaking_amount
			));
		}

		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(token_account), target, unstaking_amount),
			Error::<Test>::MaxUnlockingChunksExceeded
		);
	});
}

#[test]
fn unstake_errors_amount_to_unstake_exceeds_amount_staked() {
	new_test_ext().execute_with(|| {
		let token_account = 200;
		let target: MessageSourceId = 1;
		let staking_amount = 10;
		let unstaking_amount = 11;

		register_provider(target, String::from("Test Target"));

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(token_account), target, staking_amount));
		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(token_account), target, unstaking_amount),
			Error::<Test>::AmountToUnstakeExceedsAmountStaked
		);
	});
}

#[test]
fn unstake_errors_not_a_staking_account() {
	new_test_ext().execute_with(|| {
		let token_account = 200;
		let target: MessageSourceId = 1;

		let unstaking_amount = 11;

		register_provider(target, String::from("Test Target"));

		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(token_account), target, unstaking_amount),
			Error::<Test>::StakingAccountNotFound
		);
	});
}

#[test]
fn set_epoch_length_happy_path() {
	new_test_ext().execute_with(|| {
		let epoch_length: <Test as frame_system::Config>::BlockNumber = 9;
		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), epoch_length));

		let storage_epoch_length: <Test as frame_system::Config>::BlockNumber =
			Capacity::get_epoch_length();
		assert_eq!(epoch_length, storage_epoch_length);

		System::assert_last_event(Event::EpochLengthUpdated { blocks: epoch_length }.into());
	});
}

#[test]
fn set_epoch_length_errors_when_greater_than_max_epoch_length() {
	new_test_ext().execute_with(|| {
		let epoch_length: <Test as frame_system::Config>::BlockNumber = 101;

		assert_noop!(
			Capacity::set_epoch_length(RuntimeOrigin::root(), epoch_length),
			Error::<Test>::MaxEpochLengthExceeded
		);
	});
}

#[test]
fn set_epoch_length_errors_when_not_submitted_as_root() {
	new_test_ext().execute_with(|| {
		let epoch_length: <Test as frame_system::Config>::BlockNumber = 11;

		assert_noop!(Capacity::set_epoch_length(RuntimeOrigin::signed(1), epoch_length), BadOrigin);
	});
}
