use crate::{homoglyphs::canonical::HandleConverter, tests::mock::*};
use common_primitives::{handles::*, utils::wrap_binary_data};
use frame_support::assert_ok;
use sp_core::{sr25519, Encode, Pair};
use sp_runtime::MultiSignature;

#[test]
fn claim_handle_happy_path() {
	new_test_ext().execute_with(|| {
		// Provider
		let provider_account = test_public(1);
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

		let base_handle_str = core::str::from_utf8(&base_handle).ok().unwrap();
		println!("base_handle_str={}", base_handle_str);

		let handle_converter = HandleConverter::new();
		let canonical_handle_vec =
			handle_converter.convert_to_canonical(base_handle_str).as_bytes().to_vec();
		let canonical_handle: Handle = canonical_handle_vec.try_into().unwrap();

		Handles::get_current_suffix_index_for_canonical_handle(canonical_handle);
		println!("#events = {}", events_occured.len());
		// System::assert_last_event(Event::HandleCreated { msa_id: 1, handle: base_handle }.into());
	});
}
