use super::mock::*;
use crate::*;
use frame_support::assert_err;
use sp_core::bounded::BoundedVec;

type UnlockBVec<T> = BoundedVec<
	UnlockChunk<BalanceOf<T>, <T as Config>::EpochNumber>,
	<T as Config>::MaxUnlockingChunks,
>;

#[test]
fn staking_account_details_withdraw_reduces_active_staking_balance_and_creates_unlock_chunk() {
	let mut staking_account_details = StakingAccountDetailsV2::<Test> {
		active: BalanceOf::<Test>::from(15u64),
		staking_type: StakingType::MaximumCapacity,
	};
	assert_eq!(Ok(3u64), staking_account_details.withdraw(3, 3));
	let expected_chunks: UnlockBVec<Test> =
		BoundedVec::try_from(vec![UnlockChunk { value: 3u64, thaw_at: 3u32 }]).unwrap();

	assert_eq!(
		staking_account_details,
		StakingAccountDetailsV2::<Test> {
			active: BalanceOf::<Test>::from(12u64),
			staking_type: StakingType::MaximumCapacity,
		}
	)
}

#[test]
fn staking_account_details_withdraw_goes_to_zero_when_result_below_minimum() {
	let mut staking_account_details = StakingAccountDetailsV2::<Test> {
		active: BalanceOf::<Test>::from(10u64),
		staking_type: StakingType::MaximumCapacity,
	};
	assert_eq!(Ok(10u64), staking_account_details.withdraw(6, 3));
	assert_eq!(0u64, staking_account_details.active);
	assert_eq!(10u64, staking_account_details.total);

	staking_account_details.deposit(10);
	assert_eq!(Ok(10u64), staking_account_details.withdraw(9, 3));
	assert_eq!(0u64, staking_account_details.active);

	staking_account_details.deposit(10);
	assert_eq!(Ok(10u64), staking_account_details.withdraw(11, 3));
	assert_eq!(0u64, staking_account_details.active);
}

#[test]
fn staking_account_details_withdraw_returns_err_when_too_many_chunks() {
	let maximum_chunks: UnlockBVec<Test> = BoundedVec::try_from(vec![
		UnlockChunk { value: 1u64, thaw_at: 3u32 },
		UnlockChunk { value: 1u64, thaw_at: 3u32 },
		UnlockChunk { value: 1u64, thaw_at: 3u32 },
		UnlockChunk { value: 1u64, thaw_at: 3u32 },
	])
	.unwrap();

	let mut staking_account_details = StakingAccountDetailsV2::<Test> {
		active: BalanceOf::<Test>::from(10u64),
		staking_type: StakingType::MaximumCapacity,
	};

	assert_err!(staking_account_details.withdraw(6, 3), Error::<Test>::MaxUnlockingChunksExceeded);
	assert_eq!(10u64, staking_account_details.active);
}

// TODO: move to UnlockChunks tests
// #[test]
// fn staking_account_details_reap_thawed_happy_path() {
// 	let mut staking_account = StakingAccountDetailsV2::<Test>::default();
// 	staking_account.deposit(10);
//
// 	// 10 token total, 6 token unstaked
// 	let new_unlocks: Vec<(u32, u32)> = vec![(1u32, 2u32), (2u32, 3u32), (3u32, 4u32)];
// 	assert_eq!(true, staking_account.set_unlock_chunks(&new_unlocks));
// 	assert_eq!(10, staking_account.total);
// 	assert_eq!(3, staking_account.unlocking.len());
//
// 	// At epoch 3, the first two chunks should be thawed.
// 	assert_eq!(3u64, staking_account.reap_thawed(3u32));
// 	assert_eq!(1, staking_account.unlocking.len());
// 	// ...leaving 10-3 = 7 total in staking
// 	assert_eq!(7, staking_account.total);
//
// 	// At epoch 5, all unstaking is done.
// 	assert_eq!(3u64, staking_account.reap_thawed(5u32));
// 	assert_eq!(0, staking_account.unlocking.len());
// 	// ...leaving 7-3 = 4 total
// 	// assert_eq!(4, staking_account.total);
// }

#[test]
fn impl_staking_account_details_increase_by() {
	let mut staking_account = StakingAccountDetailsV2::<Test>::default();
	assert_eq!(staking_account.deposit(10), Some(()));

	assert_eq!(
		staking_account,
		StakingAccountDetailsV2::<Test> {
			active: BalanceOf::<Test>::from(10u64),
			staking_type: StakingType::MaximumCapacity,
		}
	)
}

#[test]
fn impl_staking_account_details_default() {
	assert_eq!(
		StakingAccountDetailsV2::<Test>::default(),
		StakingAccountDetailsV2::<Test> {
			active: BalanceOf::<Test>::zero(),
			staking_type: StakingType::MaximumCapacity,
		},
	);
}

// #[test]
// fn impl_staking_account_details_get_stakable_amount_for() {
// 	new_test_ext().execute_with(|| {
// 		let account = 200;
// 		let staking_account = StakingAccountDetailsV2::<Test>::default();
//
// 		// When staking all of free balance.
// 		assert_eq!(staking_account.get_stakable_amount_for(&account, 10), 10);
//
// 		// When staking an amount below free balance.
// 		assert_eq!(staking_account.get_stakable_amount_for(&account, 5), 5);
//
// 		// When staking an amount above account free balance. It stakes all of the free balance.
// 		assert_eq!(staking_account.get_stakable_amount_for(&account, 200), 190);
// 	});
// }
