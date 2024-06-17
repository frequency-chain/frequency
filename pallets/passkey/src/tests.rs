//! Unit tests for the passkey module.
use super::*;
use frame_support::assert_ok;
use mock::*;

#[test]
fn test_works() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5;
		assert_ok!(Passkey::proxy(RuntimeOrigin::signed(caller_1),));
	});
}
