use super::mock::*;
use frame_support::assert_ok;

#[test]
fn add_item_ok() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5u64;
		let payload = vec![1; 64];

		assert_ok!(StatefulMessageStoragePallet::add_item(
			RuntimeOrigin::signed(caller_1),
			payload
		));
	})
}
