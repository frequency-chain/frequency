use crate::tests::mock::*;
use common_primitives::{handles::*, utils::wrap_binary_data};
use frame_support::assert_ok;
use sp_core::{sr25519, Encode, Pair};
use sp_runtime::MultiSignature;

#[test]
fn claim_handle_happy_path() {
	new_test_ext().execute_with(|| {
		// Provider
		let provider_key_pair = sr25519::Pair::generate().0;
		let provider_account = provider_key_pair.public();
		println!("provider_account={}", provider_account);

		// Delegator
		let delegator_key_pair = sr25519::Pair::generate().0;
		let delegator_account = delegator_key_pair.public();
		println!("delegator_account={}", delegator_account);

		// Payload
		let base_handle = "test1".as_bytes().to_vec();

		println!("base_handle={:?}", base_handle);

		let payload = ClaimHandlePayload::new(base_handle.clone());
		let encoded_payload = wrap_binary_data(payload.encode());
		println!("encoded_payload={:?}", encoded_payload);

		let proof: MultiSignature = delegator_key_pair.sign(&encoded_payload).into();
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(provider_account.into()),
			delegator_account.into(),
			proof,
			payload
		));
		let events_occured = System::events();
		println!("#events = {}", events_occured.len());
		// System::assert_last_event(Event::HandleCreated { msa_id: 1, handle: base_handle }.into());
	});
}
