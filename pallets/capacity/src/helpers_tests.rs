use super::*;
use crate::mock::*;

struct TestCase {
	name: &'static str,
	starting_epoch: u64,
	epoch_start_block: u64,
	expected_epoch: u64,
	expected_epoch_start_block: u64,
	expected_capacity: u64,
	at_block: u64,
}

#[test]
fn start_new_epoch_works() {
	new_test_ext().execute_with(|| {
		// assumes the mock epoch length is 10 blocks.
		let test_cases: Vec<TestCase> = vec![
			TestCase {
				name: "epoch changes at the right time",
				starting_epoch: 2,
				epoch_start_block: 299,
				expected_epoch: 3,
				expected_epoch_start_block: 309,
				expected_capacity: 0,
				at_block: 309,
			},
			TestCase {
				//
				name: "epoch does not change",
				starting_epoch: 2,
				epoch_start_block: 211,
				expected_epoch: 2,
				expected_epoch_start_block: 211,
				expected_capacity: 45,
				at_block: 215,
			},
		];
		for tc in test_cases {
			CurrentEpoch::<Test>::set(tc.starting_epoch.clone());
			CurrentEpochInfo::<Test>::set(EpochInfo { epoch_start: tc.epoch_start_block });
			CurrentEpochUsedCapacity::<Test>::set(45); // just some non-zero value
			Capacity::start_new_epoch_if_needed(tc.at_block);
			assert_eq!(
				tc.expected_epoch,
				Capacity::get_current_epoch(),
				"{}: had wrong current epoch",
				tc.name
			);
			assert_eq!(
				tc.expected_epoch_start_block,
				CurrentEpochInfo::<Test>::get().epoch_start,
				"{}: had wrong epoch start block",
				tc.name
			);
			assert_eq!(
				tc.expected_capacity,
				Capacity::get_current_epoch_used_capacity(),
				"{}: had wrong used capacity",
				tc.name
			);
		}
	})
}
