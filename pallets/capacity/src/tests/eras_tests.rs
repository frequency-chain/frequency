use super::mock::*;
use crate::{
	tests::testing_utils::{run_to_block, system_run_to_block},
	CurrentEraInfo, RewardEraInfo,
};

#[test]
fn start_new_era_if_needed() {
	new_test_ext().execute_with(|| {
		CurrentEraInfo::<Test>::set(RewardEraInfo { era_index: 1, started_at: 0 });
		system_run_to_block(9);
		run_to_block(10);
		let mut current_era_info = CurrentEraInfo::<Test>::get();
		assert_eq!(current_era_info.era_index, 2u32);
		assert_eq!(current_era_info.started_at, 10u32);

		system_run_to_block(19);
		run_to_block(20);
		current_era_info = CurrentEraInfo::<Test>::get();
        assert_eq!(current_era_info.era_index, 3u32);
		assert_eq!(current_era_info.started_at, 20u32);
	})
}
