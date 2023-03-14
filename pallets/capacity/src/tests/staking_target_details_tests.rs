use super::mock::*;
use crate::*;
use frame_support::{assert_err, assert_ok};

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
fn staking_target_details_withdraw_reduces_staking_and_capacity_amounts() {
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
	let mut capacity_details = CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
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
}

#[test]
fn staking_target_details_replenish_by_amount_sets_new_capacity_and_touches_last_replenished_epoch()
{
	let mut capacity_details = CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
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
}

#[test]
fn staking_target_details_deduct_capacity_by_amount_can_do_math() {
	let mut capacity_details = CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
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
}
