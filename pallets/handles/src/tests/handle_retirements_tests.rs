use crate::{tests::mock::*, Error};
use common_primitives::{handles::*, utils::wrap_binary_data};
use frame_support::assert_noop;
use sp_core::{sr25519, Encode, Pair};
use sp_runtime::MultiSignature;

#[test]
fn retire_handle_no_handle() {
	new_test_ext().execute_with(|| {
		let provider_account = test_public(1);
		let delegator_key_pair = sr25519::Pair::generate().0;
		let delegator_account = delegator_key_pair.public();

		// Payload
		let full_handle = "test1.1".as_bytes().to_vec();
		let payload = RetireHandlePayload::new(full_handle.clone());
		let encoded_payload = wrap_binary_data(payload.encode());
		let proof: MultiSignature = delegator_key_pair.sign(&encoded_payload).into();
		assert_noop!(
			Handles::retire_handle(
				RuntimeOrigin::signed(provider_account.into()),
				delegator_account.into(),
				proof,
				payload
			),
			Error::<Test>::MSAHandleDoesNotExist
		);

	});
}
