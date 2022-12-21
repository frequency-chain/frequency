use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn it_stakes() {
	new_test_ext().execute_with(|| {
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(1), 1, 1));
	});
}
