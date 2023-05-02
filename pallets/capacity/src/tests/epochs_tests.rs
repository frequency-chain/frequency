use super::mock::*;
use crate::{
	tests::testing_utils::{run_to_block, system_run_to_block},
	Config, EpochLength, Error, Event,
};
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
		let max_epoch_length: u32 = <Test as Config>::MaxEpochLength::get();

		assert_eq!(epoch_length, max_epoch_length);
	});
}
#[test]
fn get_epoch_length_should_return_storage_epoch_length() {
	new_test_ext().execute_with(|| {
		EpochLength::<Test>::set(101u32);
		let epoch_length: <Test as frame_system::Config>::BlockNumber =
			Capacity::get_epoch_length();

		assert_eq!(epoch_length, 101u32);
	});
}

#[test]
fn start_new_epoch_if_needed_when_not_starting_from_zero() {
	new_test_ext().execute_with(|| {
		EpochLength::<Test>::set(100u32);
		let epoch_info = Capacity::get_current_epoch_info();
		let cur_epoch = Capacity::get_current_epoch();
		// Run the system to block 999 before initializing Capacity to emulate
		// deploying to an existing network where blockheight is greater than 0.
		system_run_to_block(999);
		run_to_block(1000);
		let after_epoch = Capacity::get_current_epoch();
		let after_epoch_info = Capacity::get_current_epoch_info();
		assert_eq!(epoch_info.epoch_start, 0);
		assert_eq!(after_epoch_info.epoch_start, 1000);
		assert_eq!(after_epoch, cur_epoch + 1);
	})
}

#[test]
fn start_new_epoch_if_needed_when_epoch_length_changes() {
	new_test_ext().execute_with(|| {
		EpochLength::<Test>::set(100u32);
		// Starting block = 100
		system_run_to_block(100);
		let epoch_info = Capacity::get_current_epoch_info();
		let cur_epoch = Capacity::get_current_epoch();
		assert_eq!(epoch_info.epoch_start, 0);
		assert_eq!(cur_epoch, 0);
		run_to_block(150);
		let middle_epoch = Capacity::get_current_epoch();
		let middle_epoch_info = Capacity::get_current_epoch_info();
		assert_eq!(middle_epoch, 1);
		assert_eq!(middle_epoch_info.epoch_start, 101);
		// Change epoch length = 20
		EpochLength::<Test>::set(20u32);
		run_to_block(151);
		let after_epoch = Capacity::get_current_epoch();
		let after_epoch_info = Capacity::get_current_epoch_info();
		// epoch 1 starts at 101
		// epoch 2 starts at 151 (First block after epoch length change)
		assert_eq!(after_epoch_info.epoch_start, 151);
		assert_eq!(after_epoch, 2);
		run_to_block(171);
		let last_epoch = Capacity::get_current_epoch();
		let last_epoch_info = Capacity::get_current_epoch_info();
		assert_eq!(last_epoch, 3);
		// epoch 3 starts at 171 (151 + 20)
		assert_eq!(last_epoch_info.epoch_start, 171);
	})
}
