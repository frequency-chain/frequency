use super::{mock::*, testing_utils::run_to_block};
use crate::{self as pallet_capacity, FreezeReason, StakingAccountDetails};
use frame_support::{assert_noop, assert_ok, traits::fungible::InspectFreeze};
use pallet_capacity::{BalanceOf, Config, Error, Event};

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
		assert_eq!(
			10u64,
			<Test as Config>::Currency::balance_frozen(&FreezeReason::Staked.into(), &staker)
		);

		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));

		// Epoch length = 10, we want to run to epoch 3
		run_to_block(31);
		assert_ok!(Capacity::withdraw_unstaked(RuntimeOrigin::signed(staker)));

		assert_eq!(
			7u64,
			<Test as Config>::Currency::balance_frozen(&FreezeReason::Staked.into(), &staker)
		);
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
