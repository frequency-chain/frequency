use super::{mock::*, testing_utils::*};
use common_primitives::capacity::Replenishable;
use frame_support::{assert_noop, assert_ok};

use crate::{BalanceOf, CapacityDetails, CapacityLedger, Config, CurrentEpoch, Error};

#[test]
fn impl_replenish_all_for_account_is_successful() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let remaining_amount = 3u64;
		let total_available_amount = 10u64;
		let _ = create_capacity_account_and_fund(
			target_msa_id,
			remaining_amount,
			total_available_amount,
			1u32,
		);

		CurrentEpoch::<Test>::set(1u32);

		assert_ok!(Capacity::replenish_all_for(target_msa_id));

		let mut capacity_details =
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber>::default();

		capacity_details.remaining_capacity = 10u32.into();
		capacity_details.total_tokens_staked = 10u32.into();
		capacity_details.total_capacity_issued = 10u32.into();
		capacity_details.last_replenished_epoch = 1u32;

		assert_eq!(CapacityLedger::<Test>::get(target_msa_id).unwrap(), capacity_details);
	});
}

#[test]
fn impl_replenish_all_for_account_errors_target_capacity_not_found() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		assert_noop!(
			Capacity::replenish_all_for(target_msa_id),
			Error::<Test>::TargetCapacityNotFound
		);
	});
}

#[test]
fn impl_can_replenish_is_false_when_last_replenished_at_is_greater_or_equal_current_epoch() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let remaining_amount = 3u64;
		let total_available_amount = 10u64;
		let last_replenished_at = 1u32;

		let _ = create_capacity_account_and_fund(
			target_msa_id,
			remaining_amount,
			total_available_amount,
			last_replenished_at,
		);

		assert_eq!(CurrentEpoch::<Test>::get(), 0);

		assert!(!Capacity::can_replenish(target_msa_id));

		CurrentEpoch::<Test>::set(1);

		assert!(!Capacity::can_replenish(target_msa_id));
	});
}

#[test]
fn impl_can_replenish_is_true_when_last_replenished_at_is_less_than_current_epoch() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let remaining_amount = 3u64;
		let total_available_amount = 10u64;
		let last_replenished_at = 2u32;

		let _ = create_capacity_account_and_fund(
			target_msa_id,
			remaining_amount,
			total_available_amount,
			last_replenished_at,
		);

		CurrentEpoch::<Test>::set(3);

		assert!(Capacity::can_replenish(target_msa_id));
	});
}
