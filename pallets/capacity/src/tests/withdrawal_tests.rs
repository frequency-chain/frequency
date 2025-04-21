use super::{mock::*, testing_utils::*};
use crate::{BalanceOf, CapacityDetails, CapacityLedger, Config, Error, Event};
use common_primitives::capacity::Nontransferable;
use frame_support::{assert_noop, assert_ok};

#[test]
fn impl_withdraw_is_successful() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let remaining_amount = 10u64;
		let total_available_amount = 10u64;
		let _ = create_capacity_account_and_fund(
			target_msa_id,
			remaining_amount,
			total_available_amount,
			1u32,
		);

		assert_ok!(Capacity::deduct(target_msa_id, 5u64));
		let events = capacity_events();

		assert_eq!(
			events.last().unwrap(),
			&Event::CapacityWithdrawn { msa_id: target_msa_id, amount: 5u64 }
		);

		let mut capacity_details =
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber>::default();

		capacity_details.remaining_capacity = 5u64;
		capacity_details.total_tokens_staked = 10u64;
		capacity_details.total_capacity_issued = 10u64;
		capacity_details.last_replenished_epoch = 1u32;

		assert_eq!(CapacityLedger::<Test>::get(target_msa_id).unwrap(), capacity_details);
	});
}

#[test]
fn impl_withdraw_errors_target_capacity_not_found() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let amount = 10u64;
		assert_noop!(
			Capacity::deduct(target_msa_id, amount),
			Error::<Test>::TargetCapacityNotFound
		);
	});
}

#[test]
fn impl_withdraw_errors_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let remaining_amount = 10u64;
		let total_available_amount = 10u64;
		let _ = create_capacity_account_and_fund(
			target_msa_id,
			remaining_amount,
			total_available_amount,
			1u32,
		);

		assert_noop!(
			Capacity::deduct(target_msa_id, 20u32.into()),
			Error::<Test>::InsufficientCapacityBalance
		);

		let mut capacity_details =
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber>::default();

		capacity_details.remaining_capacity = 10u64;
		capacity_details.total_tokens_staked = 10u64;
		capacity_details.total_capacity_issued = 10u64;
		capacity_details.last_replenished_epoch = 1u32;

		assert_eq!(CapacityLedger::<Test>::get(target_msa_id).unwrap(), capacity_details);
	});
}
