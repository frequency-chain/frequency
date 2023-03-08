use super::*;
use crate::mock::*;
use frame_support::{assert_err, assert_noop, assert_ok, BoundedVec};

type UnlockBVec<T> = BoundedVec<
	UnlockChunk<BalanceOf<T>, <T as Config>::EpochNumber>,
	<T as Config>::MaxUnlockingChunks,
>;

#[test]
fn staking_account_details_reap_thawed_happy_path() {
	let mut staking_account = StakingAccountDetails::<Test>::default();
	staking_account.deposit(10);

	// 10 token total, 6 token unstaked
	let new_unlocks: Vec<(u32, u32)> = vec![(1u32, 2u32), (2u32, 3u32), (3u32, 4u32)];
	assert_eq!(true, staking_account.set_unlock_chunks(&new_unlocks));
	assert_eq!(10, staking_account.total);
	assert_eq!(3, staking_account.unlocking.len());

	// At epoch 3, the first two chunks should be thawed.
	assert_eq!(3u64, staking_account.reap_thawed(3u32));
	assert_eq!(1, staking_account.unlocking.len());
	// ...leaving 10-3 = 7 total in staking
	assert_eq!(7, staking_account.total);

	// At epoch 5, all unstaking is done.
	assert_eq!(3u64, staking_account.reap_thawed(5u32));
	assert_eq!(0, staking_account.unlocking.len());
	// ...leaving 7-3 = 4 total
	assert_eq!(4, staking_account.total);
}

#[test]
fn impl_staking_account_details_increase_by() {
	new_test_ext().execute_with(|| {
		let mut staking_account = StakingAccountDetails::<Test>::default();
		assert_eq!(staking_account.deposit(10), Some(()));

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
		assert_eq!(staking_target.deposit(10, 10), Some(()));

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
fn impl_staking_capacity_details_deposit() {
	new_test_ext().execute_with(|| {
		let mut capacity_details =
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber>::default();

		assert_eq!(capacity_details.deposit(&10), Some(()));

		assert_eq!(
			capacity_details,
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
				remaining_capacity: BalanceOf::<Test>::from(10u64),
				total_tokens_staked: BalanceOf::<Test>::from(10u64),
				total_capacity_issued: BalanceOf::<Test>::from(10u64),
				last_replenished_epoch: <Test as Config>::EpochNumber::from(0u32)
			}
		)
	});
}

#[test]
fn staking_account_details_withdraw_reduces_active_staking_balance_and_creates_unlock_chunk() {
	new_test_ext().execute_with(|| {
		let mut staking_account_details = StakingAccountDetails::<Test> {
			active: BalanceOf::<Test>::from(10u64),
			total: BalanceOf::<Test>::from(10u64),
			unlocking: BoundedVec::default(),
		};
		assert_eq!(Ok(3u64), staking_account_details.withdraw(3, 3));
		let expected_chunks: UnlockBVec<Test> =
			BoundedVec::try_from(vec![UnlockChunk { value: 3u64, thaw_at: 3u32 }]).unwrap();

		assert_eq!(
			staking_account_details,
			StakingAccountDetails::<Test> {
				active: BalanceOf::<Test>::from(7u64),
				total: BalanceOf::<Test>::from(10u64),
				unlocking: expected_chunks,
			}
		)
	});
}

#[test]
fn staking_account_details_withdraw_goes_to_zero_when_result_below_minimum() {
	new_test_ext().execute_with(|| {
		let mut staking_account_details = StakingAccountDetails::<Test> {
			active: BalanceOf::<Test>::from(10u64),
			total: BalanceOf::<Test>::from(10u64),
			unlocking: BoundedVec::default(),
		};
		assert_eq!(Ok(10u64), staking_account_details.withdraw(6, 3));
		assert_eq!(0u64, staking_account_details.active);
		assert_eq!(10u64, staking_account_details.total);
	});
}

#[test]
fn staking_account_details_withdraw_returns_err_when_too_many_chunks() {
	new_test_ext().execute_with(|| {
		let maximum_chunks: UnlockBVec<Test> = BoundedVec::try_from(vec![
			UnlockChunk { value: 1u64, thaw_at: 3u32 },
			UnlockChunk { value: 1u64, thaw_at: 3u32 },
			UnlockChunk { value: 1u64, thaw_at: 3u32 },
			UnlockChunk { value: 1u64, thaw_at: 3u32 },
		])
		.unwrap();

		let mut staking_account_details = StakingAccountDetails::<Test> {
			active: BalanceOf::<Test>::from(10u64),
			total: BalanceOf::<Test>::from(10u64),
			unlocking: maximum_chunks,
		};

		assert_noop!(
			staking_account_details.withdraw(6, 3),
			Error::<Test>::MaxUnlockingChunksExceeded
		);
		assert_eq!(10u64, staking_account_details.active);
		assert_eq!(10u64, staking_account_details.total);
	})
}

#[test]
fn staking_target_details_withdraw_reduces_staking_and_capacity_amounts() {
	new_test_ext().execute_with(|| {
		let mut staking_target_details = StakingTargetDetails::<BalanceOf<Test>> {
			amount: BalanceOf::<Test>::from(15u64),
			capacity: BalanceOf::<Test>::from(20u64),
		};
		staking_target_details.withdraw(10, 10);

		assert_eq!(
			staking_target_details,
			StakingTargetDetails::<BalanceOf<Test>> {
				amount: BalanceOf::<Test>::from(5u64),
				capacity: BalanceOf::<Test>::from(10u64),
			}
		)
	});
}

#[test]
fn staking_target_details_withdraw_reduces_total_tokens_staked_and_total_tokens_available() {
	new_test_ext().execute_with(|| {
		let mut capacity_details =
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
				remaining_capacity: BalanceOf::<Test>::from(10u64),
				total_tokens_staked: BalanceOf::<Test>::from(10u64),
				total_capacity_issued: BalanceOf::<Test>::from(10u64),
				last_replenished_epoch: <Test as Config>::EpochNumber::from(0u32),
			};
		capacity_details.withdraw(4, 5);

		assert_eq!(
			capacity_details,
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
				remaining_capacity: BalanceOf::<Test>::from(6u64),
				total_tokens_staked: BalanceOf::<Test>::from(5u64),
				total_capacity_issued: BalanceOf::<Test>::from(6u64),
				last_replenished_epoch: <Test as Config>::EpochNumber::from(0u32)
			}
		)
	});
}

#[test]
fn staking_target_details_replenish_all_resets_remaining_capacity_to_total_capacity_issued() {
	new_test_ext().execute_with(|| {
		let mut capacity_details =
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
				total_tokens_staked: BalanceOf::<Test>::from(22u64),
				total_capacity_issued: BalanceOf::<Test>::from(12u64),
				remaining_capacity: BalanceOf::<Test>::from(10u64),
				last_replenished_epoch: <Test as Config>::EpochNumber::from(3u32),
			};
		let current_epoch: u32 = 5;
		capacity_details.replenish_all(&current_epoch);

		assert_eq!(12u64, capacity_details.total_capacity_issued);
		assert_eq!(capacity_details.total_capacity_issued, capacity_details.remaining_capacity);
		assert_eq!(current_epoch, capacity_details.last_replenished_epoch);
		assert_eq!(22u64, capacity_details.total_tokens_staked);
	})
}

#[test]
fn staking_target_details_replenish_by_amount_sets_new_capacity_and_touches_last_replenished_epoch()
{
	new_test_ext().execute_with(|| {
		let mut capacity_details =
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
				total_tokens_staked: BalanceOf::<Test>::from(65_000u64),
				total_capacity_issued: BalanceOf::<Test>::from(32_500u64),
				remaining_capacity: BalanceOf::<Test>::from(1_000u64),
				last_replenished_epoch: <Test as Config>::EpochNumber::from(3_000u32),
			};
		let current_epoch: u32 = 3434;
		capacity_details.replenish_by_amount(2001, &current_epoch);

		assert_eq!(32_500u64, capacity_details.total_capacity_issued);
		assert_eq!(3_001u64, capacity_details.remaining_capacity);
		assert_eq!(current_epoch, capacity_details.last_replenished_epoch);
		assert_eq!(65_000u64, capacity_details.total_tokens_staked);
	})
}

#[test]
fn staking_target_details_deduct_capacity_by_amount_can_do_math() {
	new_test_ext().execute_with(|| {
		let mut capacity_details =
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
				total_tokens_staked: BalanceOf::<Test>::from(5u64),
				total_capacity_issued: BalanceOf::<Test>::from(7u64),
				remaining_capacity: BalanceOf::<Test>::from(6u64),
				last_replenished_epoch: <Test as Config>::EpochNumber::from(2u32),
			};
		assert_ok!(capacity_details.deduct_capacity_by_amount(2u64));

		assert_eq!(5u64, capacity_details.total_tokens_staked);
		assert_eq!(7u64, capacity_details.total_capacity_issued);
		assert_eq!(4u64, capacity_details.remaining_capacity);
		assert_eq!(2u32, capacity_details.last_replenished_epoch);

		assert_err!(capacity_details.deduct_capacity_by_amount(99u64), ArithmeticError::Underflow);
	});
}
