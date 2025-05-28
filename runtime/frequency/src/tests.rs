use super::*;
use crate::xcm::tests::mock::{new_test_ext_with_balances, TestRuntime};
use common_runtime::constants::xcm_version::SAFE_XCM_VERSION;

#[test]
fn pre_upgrade_returns_current_value() {
	new_test_ext_with_balances(vec![]).execute_with(|| {
		pallet_xcm::SafeXcmVersion::<TestRuntime>::kill();
		assert_eq!(SetSafeXcmVersion::<TestRuntime>::pre_upgrade().unwrap(), None);

		pallet_xcm::SafeXcmVersion::<TestRuntime>::set(Some(3));
		assert_eq!(SetSafeXcmVersion::<TestRuntime>::pre_upgrade().unwrap(), Some(3));
	});
}

#[test]
fn post_upgrade_sets_safe_version() {
	new_test_ext_with_balances(vec![]).execute_with(|| {
		pallet_xcm::SafeXcmVersion::<TestRuntime>::kill();
		let pre = SetSafeXcmVersion::<TestRuntime>::pre_upgrade().unwrap();
		SetSafeXcmVersion::<TestRuntime>::on_runtime_upgrade();
		SetSafeXcmVersion::<TestRuntime>::post_upgrade(pre).unwrap();
		assert_eq!(pallet_xcm::SafeXcmVersion::<TestRuntime>::get(), Some(SAFE_XCM_VERSION));
	});
}
