use super::mock::*;
use crate::{Config, Error};
use frame_support::{assert_err, assert_ok};

#[test]
fn add_item_ok() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5u64;
		let payload = vec![1; 64];

		assert_ok!(StatefulStoragePallet::add_item(RuntimeOrigin::signed(caller_1), payload));
	})
}

#[test]
fn add_item_with_large_payload_errors() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5u64;
		let payload =
			vec![
				1;
				TryInto::<usize>::try_into(<Test as Config>::MaxItemizedBlobSizeBytes::get())
					.unwrap() + 1
			];

		assert_err!(
			StatefulStoragePallet::add_item(RuntimeOrigin::signed(caller_1), payload),
			Error::<Test>::ItemExceedsMaxBlobSizeBytes
		)
	})
}

#[test]
fn upsert_page_too_large_errors() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5u64;
		let payload =
			vec![
				1;
				TryInto::<usize>::try_into(<Test as Config>::MaxPaginatedPageSizeBytes::get())
					.unwrap() + 1
			];

		assert_err!(
			StatefulStoragePallet::upsert_page(RuntimeOrigin::signed(caller_1), payload),
			Error::<Test>::PageExceedsMaxPageSizeBytes
		)
	})
}
