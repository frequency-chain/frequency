use super::*;
use crate as pallet_capacity;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok, traits::WithdrawReasons, BoundedVec};
use testing_utils::{register_provider, run_to_block, staking_events};

#[test]
fn capacity_reap_thawed_happy_path() {
	new_test_ext().execute_with(|| {
		let staker = 200;
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
		let expected_total = staking_account.total - 3u64;
		assert_eq!(expected_total, current_account.total)
	})
}

#[test]
fn cannot_withdraw_if_no_thawed_balance() {
	new_test_ext().execute_with(|| {
		/// - Returns `Error::NoUnstakedTokensAvailable` if there are no unstaking tokens available to withdraw.
		assert!(false, "is not true");
	})
}
#[test]
fn withdraws_only_thawed_balance() {
	new_test_ext().execute_with(|| {
		assert!(false, "is not true");
	})
}

#[test]
fn error_if_not_a_staking_account() {
	new_test_ext().execute_with(|| {
		/// - Returns `Error:: NotAStakingAccount` if `origin` has nothing staked
		assert!(false, "is not true");
	})
}
