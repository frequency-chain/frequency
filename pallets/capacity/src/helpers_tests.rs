use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};

#[test]
fn start_new_epoch_works() {
	new_test_ext().execute_with(|| {
		CurrentEpoch::<Test>::set(Epoch {
			current_epoch: 1, // assuming zero-indexed.
			epoch_start: 10,  // assuming it was epoch 0 at genesis
		});
		// assume we start a new epoch every 10 blocks
		Capacity::start_new_epoch_if_needed(20);
		let current_epoch: Epoch<u64> = Capacity::get_current_epoch();
		assert_eq!(20u64, current_epoch.epoch_start);
		assert_eq!(2u64, current_epoch.current_epoch);
	})
}
