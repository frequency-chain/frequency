use super::{
	mock::*,
	testing_utils::{register_provider, run_to_block},
};
use crate::{
	unlock_chunks_from_vec, unlock_chunks_reap_thawed, CurrentEpoch, CurrentEpochInfo, EpochInfo,
	Error, Event, UnstakeUnlocks,
};
use frame_support::{assert_noop, assert_ok};

#[test]
fn withdraw_unstaked_happy_path() {
	new_test_ext().execute_with(|| {
		let staker = 500;
		CurrentEpoch::<Test>::set(0);
		CurrentEpochInfo::<Test>::set(EpochInfo { epoch_start: 0 });
		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));

		// set new unlock chunks using tuples of (value, thaw_at in number of Epochs)
		let new_unlocks: Vec<(u32, u32)> = vec![(1u32, 2u32), (2u32, 3u32), (3u32, 4u32)];
		let unlocking = unlock_chunks_from_vec::<Test>(&new_unlocks);
		UnstakeUnlocks::<Test>::set(&staker, Some(unlocking));

		// We want to advance to epoch 3 to unlock the first two sets.
		run_to_block(31);
		assert_eq!(3u32, Capacity::get_current_epoch());
		assert_ok!(Capacity::withdraw_unstaked(RuntimeOrigin::signed(staker)));

		let expected_reaped_value = 3u64;
		System::assert_last_event(
			Event::StakeWithdrawn { account: staker, amount: expected_reaped_value }.into(),
		);
	})
}

#[test]
fn withdraw_unstaked_correctly_sets_new_lock_state() {
	new_test_ext().execute_with(|| {
		let staker = 500;
		let target = 1;
		let amount = 20;
		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));

		register_provider(target, String::from("WithdrawUnst"));
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker), target, amount));

		run_to_block(1);
		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(staker), target, 1));
		assert_eq!(1, Balances::locks(&staker).len());
		assert_eq!(20u64, Balances::locks(&staker)[0].amount);

		// thaw period in mock is 2 Epochs * 10 blocks = 20 blocks.
		run_to_block(21);
		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(staker), target, 2));
		assert_ok!(Capacity::withdraw_unstaked(RuntimeOrigin::signed(staker)));
		assert_eq!(1, Balances::locks(&staker).len());
		assert_eq!(19u64, Balances::locks(&staker)[0].amount);

		run_to_block(41);
		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(staker), target, 3));
		assert_ok!(Capacity::withdraw_unstaked(RuntimeOrigin::signed(staker)));
		assert_eq!(1, Balances::locks(&staker).len());
		assert_eq!(17u64, Balances::locks(&staker)[0].amount);

		run_to_block(61);
		assert_ok!(Capacity::withdraw_unstaked(RuntimeOrigin::signed(staker)));
		assert_eq!(1, Balances::locks(&staker).len());
		assert_eq!(14u64, Balances::locks(&staker)[0].amount);
	})
}

#[test]
fn withdraw_unstaked_cleans_up_storage_and_removes_all_locks_if_no_stake_left() {
	new_test_ext().execute_with(|| {
		let staker = 500;
		let target = 1;
		let amount = 20;
		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));

		register_provider(target, String::from("WithdrawUnst"));
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker), target, amount));

		run_to_block(1);
		// unstake everything
		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(staker), target, 20));
		// wait for thaw
		run_to_block(21);
		assert_ok!(Capacity::withdraw_unstaked(RuntimeOrigin::signed(staker)));
		assert_eq!(0, Balances::locks(&staker).len());
		assert!(Capacity::get_unstake_unlocking_for(&staker).is_none());
	})
}

#[test]
fn withdraw_unstaked_cannot_withdraw_if_no_unstaking_chunks() {
	new_test_ext().execute_with(|| {
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
		let target = 1;
		let amount = 10;
		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));
		register_provider(target, String::from("WithdrawUnst"));
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker), target, amount));

		run_to_block(11);
		assert_noop!(
			Capacity::withdraw_unstaked(RuntimeOrigin::signed(500)),
			Error::<Test>::NoUnstakedTokensAvailable
		);
	})
}
