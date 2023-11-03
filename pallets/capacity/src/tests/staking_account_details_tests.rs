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
	assert_eq!(Ok(3u64), staking_account_details.withdraw(3));
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
	assert_eq!(Ok(10u64), staking_account_details.withdraw(6));
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
