use super::mock::*;
use crate::{
	tests::testing_utils::{run_to_block, system_run_to_block},
	Config, CurrentEraInfo, Error, Event, RewardEraInfo,
};

use frame_support::traits::Get;

#[test]
fn start_new_era_if_needed() {
	new_test_ext().execute_with(|| {
		CurrentEraInfo::<Test>::set(RewardEraInfo { current_era: 1, era_start: 0 });
		system_run_to_block(9);
		run_to_block(10);
		let mut current_era_info = CurrentEraInfo::<Test>::get();
		assert_eq!(current_era_info.current_era, 2u32);
		assert_eq!(current_era_info.era_start, 10u32);

		system_run_to_block(19);
		run_to_block(20);
		current_era_info = CurrentEraInfo::<Test>::get();
		assert_eq!(current_era_info.current_era, 3u32);
		assert_eq!(current_era_info.era_start, 20u32);
	})
}
