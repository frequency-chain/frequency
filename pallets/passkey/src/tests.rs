//! Unit tests for the passkey module.
use super::*;
use common_primitives::{node::AccountId, utils::wrap_binary_data};
use frame_support::assert_ok;
use mock::*;
use sp_core::{sr25519, Pair};
use sp_runtime::MultiSignature;

#[test]
fn test_works() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5;
		assert_ok!(Passkey::proxy(RuntimeOrigin::signed(caller_1),));
	});
}
