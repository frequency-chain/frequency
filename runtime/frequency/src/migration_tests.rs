use super::*;
use crate::xcm::tests::mock::{new_test_ext_with_balances, TestRuntime};
use parity_scale_codec::Decode;
use staging_xcm::opaque::latest::prelude::XCM_VERSION;
use common_runtime::constants::xcm_version::SAFE_XCM_VERSION;

#[test]
fn pre_upgrade_returns_current_value() {
	new_test_ext_with_balances(vec![]).execute_with(|| {
		let encoded_result = SetSafeXcmVersion::<TestRuntime>::pre_upgrade().unwrap();
		let decoded_result = Option::<u32>::decode(&mut &encoded_result[..]).unwrap();
		assert_eq!(decoded_result, None);
	});
}

#[test]
fn post_upgrade_sets_safe_version() {
	new_test_ext_with_balances(vec![]).execute_with(|| {
		let storage_key = frame_support::storage::storage_prefix(b"PolkadotXcm", b"SafeXcmVersion");

		// Clear the storage
		frame_support::storage::unhashed::kill(&storage_key);
		let pre = SetSafeXcmVersion::<TestRuntime>::pre_upgrade().unwrap();
		SetSafeXcmVersion::<TestRuntime>::on_runtime_upgrade();
		SetSafeXcmVersion::<TestRuntime>::post_upgrade(pre).unwrap();
		assert_eq!(
			frame_support::storage::unhashed::get::<u32>(&storage_key).unwrap(),
			XCM_VERSION
		);
	});
}

#[test]
fn migration_is_idempotent() {
	new_test_ext_with_balances(vec![]).execute_with(|| {
		let storage_key = frame_support::storage::storage_prefix(b"PolkadotXcm", b"SafeXcmVersion");

		// Clear the storage initially
		frame_support::storage::unhashed::kill(&storage_key);

		// Run migration first time
		let weight1 = SetSafeXcmVersion::<TestRuntime>::on_runtime_upgrade();
		assert_eq!(
			frame_support::storage::unhashed::get::<u32>(&storage_key).unwrap(),
			SAFE_XCM_VERSION
		);

		// Run migration second time - should be a no-op
		let weight2 = SetSafeXcmVersion::<TestRuntime>::on_runtime_upgrade();
		assert_eq!(
			frame_support::storage::unhashed::get::<u32>(&storage_key).unwrap(),
			SAFE_XCM_VERSION
		);

		// Second run should only do a read (less weight)
		assert!(weight2.ref_time() < weight1.ref_time());
	});
}

#[test]
fn test_weight_calculation() {
	new_test_ext_with_balances(vec![]).execute_with(|| {
		let storage_key = frame_support::storage::storage_prefix(b"PolkadotXcm", b"SafeXcmVersion");

		// Test weight when migration is needed
		frame_support::storage::unhashed::kill(&storage_key);
		let weight_with_write = SetSafeXcmVersion::<TestRuntime>::on_runtime_upgrade();

		// Test weight when migration is not needed (value already exists)
		let weight_read_only = SetSafeXcmVersion::<TestRuntime>::on_runtime_upgrade();

		// Weight with write should be greater than read-only weight
		assert!(weight_with_write.ref_time() > weight_read_only.ref_time());
	});
}
