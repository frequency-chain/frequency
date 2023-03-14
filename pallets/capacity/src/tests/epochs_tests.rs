use super::mock::*;
use crate::{Config, EpochLength, Error, Event};
use frame_support::{assert_noop, assert_ok, traits::Get};
use sp_runtime::DispatchError::BadOrigin;
#[test]
fn set_epoch_length_happy_path() {
	new_test_ext().execute_with(|| {
		let epoch_length: <Test as frame_system::Config>::BlockNumber = 9;
		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), epoch_length));

		let storage_epoch_length: <Test as frame_system::Config>::BlockNumber =
			Capacity::get_epoch_length();
		assert_eq!(epoch_length, storage_epoch_length);

		System::assert_last_event(Event::EpochLengthUpdated { blocks: epoch_length }.into());
	});
}

#[test]
fn set_epoch_length_errors_when_greater_than_max_epoch_length() {
	new_test_ext().execute_with(|| {
		let epoch_length: <Test as frame_system::Config>::BlockNumber = 101;

		assert_noop!(
			Capacity::set_epoch_length(RuntimeOrigin::root(), epoch_length),
			Error::<Test>::MaxEpochLengthExceeded
		);
	});
}

#[test]
fn set_epoch_length_errors_when_not_submitted_as_root() {
	new_test_ext().execute_with(|| {
		let epoch_length: <Test as frame_system::Config>::BlockNumber = 11;

		assert_noop!(Capacity::set_epoch_length(RuntimeOrigin::signed(1), epoch_length), BadOrigin);
	});
}

#[test]
fn get_epoch_length_should_return_max_epoch_length_when_unset() {
	new_test_ext().execute_with(|| {
		let epoch_length: <Test as frame_system::Config>::BlockNumber =
			Capacity::get_epoch_length();
		let max_epoch_length: u64 = <Test as Config>::MaxEpochLength::get();

		assert_eq!(epoch_length, max_epoch_length);
	});
}
#[test]
fn get_epoch_length_should_return_storage_epoch_length() {
	new_test_ext().execute_with(|| {
		EpochLength::<Test>::set(101u64);
		let epoch_length: <Test as frame_system::Config>::BlockNumber =
			Capacity::get_epoch_length();

		assert_eq!(epoch_length, 101u64);
	});
}
