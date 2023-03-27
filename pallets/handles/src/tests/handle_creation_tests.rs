use crate::tests::mock::*;
use common_primitives::{handles::*, utils::wrap_binary_data};
use frame_support::{assert_ok, BoundedVec};
use sp_core::{sr25519, ConstU32, Encode, Pair};
use sp_runtime::MultiSignature;

#[test]
fn claim_handle_happy_path() {
	new_test_ext().execute_with(|| {
		// Provider
		let provider_key_pair = sr25519::Pair::generate().0;
		let provider_account = provider_key_pair.public();

		// Delegator
		let delegator_key_pair = sr25519::Pair::generate().0;
		let delegator_account = delegator_key_pair.public();
		let delegator_msa_id: u64 = 1;

		// Payload
		let base_handle: BoundedVec<u8, ConstU32<32>> = "test1".as_bytes().try_into().unwrap();
		let payload = ClaimHandlePayload::new(base_handle);
		let encoded_payload = wrap_binary_data(payload.encode());

		let proof: MultiSignature = delegator_key_pair.sign(&encoded_payload).into();

		assert_ok!(Handles::claim_handle(test_public(1), delegator_msa_id, proof, payload));
	});
}
