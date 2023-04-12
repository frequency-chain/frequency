use crate::{handles_signed_extension::HandlesSignedExtension, tests::mock::*};
use frame_support::{assert_ok, dispatch::DispatchInfo};
use sp_core::{sr25519, Pair};
use sp_runtime::traits::SignedExtension;

/// Assert that retiring a handle passes the signed extension HandlesSignedExtension
#[test]
fn signed_extension_retire_handle_success() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		// Claim the handle
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec());
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload
		));

		// Retire the handle
		let call_retire_handle: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Handles(HandlesCall::retire_handle {});
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = HandlesSignedExtension::<Test>::new().validate(
			&alice.public().into(),
			call_retire_handle,
			&info,
			len,
		);
		assert_ok!(result);
	});
}
