use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use testing_utils::run_to_block;

#[test]
fn staking_account_details_reap_thawed_happy_path() {
	new_test_ext().execute_with(|| {
		let mut staking_account = StakingAccountDetails::<Test>::default();
		staking_account.increase_by(10);

		let new_unlocks: Vec<(u32, u32)> = vec![(1u32, 2u32), (2u32, 3u32), (3u32, 4u32)];
		assert_eq!(true, staking_account.set_unlock_chunks(&new_unlocks));
		assert_eq!(16, staking_account.total);
		assert_eq!(3, staking_account.unlocking.len());

		assert_eq!(Ok(1u64), staking_account.reap_thawed(2));
		assert_eq!(2, staking_account.unlocking.len());
		assert_eq!(15, staking_account.total);

		assert_eq!(Ok(5u64), staking_account.reap_thawed(5));
		assert_eq!(0, staking_account.unlocking.len());
		assert_eq!(10, staking_account.total);
	})
}

#[test]
fn happy_path() {
	new_test_ext().execute_with(|| {
		let staker = 500;
		let mut staking_account = StakingAccountDetails::<Test>::default();
		staking_account.increase_by(10);

		// set new unlock chunks using tuples of (value, thaw_at)
		let new_unlocks: Vec<(u32, u32)> = vec![(1u32, 2u32), (2u32, 3u32), (3u32, 4u32)];

		assert_eq!(true, staking_account.set_unlock_chunks(&new_unlocks));
		assert_eq!(16u64, staking_account.total);

		Capacity::set_staking_account(&staker, &staking_account);
		run_to_block(3);
		assert_ok!(Capacity::withdraw_unstaked(RuntimeOrigin::signed(staker)));

		let current_account: StakingAccountDetails<Test> =
			Capacity::get_staking_account_for(&staker).unwrap();
		let expected_reaped_value = 3u64;
		let expected_total = staking_account.total - expected_reaped_value;
		assert_eq!(expected_total, current_account.total);

		System::assert_last_event(
			Event::StakeWithdrawn { account: staker, amount: expected_reaped_value }.into(),
		);
	})
}

#[test]
fn cleans_up_storage_if_no_stake_left() {
	new_test_ext().execute_with(|| {
		assert!(false, "TODO");
	})
}

#[test]
fn cannot_withdraw_if_no_unstaking_chunks() {
	new_test_ext().execute_with(|| {
		// - Returns `Error::NoUnstakedTokensAvailable` if there are no unstaking tokens available to withdraw.
		let staker = 500;
		let mut staking_account = StakingAccountDetails::<Test>::default();
		staking_account.increase_by(10);
		Capacity::set_staking_account(&staker, &staking_account);
		assert_noop!(
			Capacity::withdraw_unstaked(RuntimeOrigin::signed(500)),
			Error::<Test>::NoUnstakedTokensAvailable
		);
	})
}
#[test]
fn cannot_withdraw_if_unstaking_chunks_not_thawed() {
	new_test_ext().execute_with(|| {
		let staker = 500;
		let mut staking_account = StakingAccountDetails::<Test>::default();
		staking_account.increase_by(10);

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
fn error_if_not_a_staking_account() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Capacity::withdraw_unstaked(RuntimeOrigin::signed(999)),
			Error::<Test>::NotAStakingAccount
		);
	})
}
