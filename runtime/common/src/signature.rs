use common_primitives::utils::wrap_binary_data;
use sp_runtime::{traits::Verify, AccountId32, MultiSignature};
use sp_std::vec::Vec;

pub fn check_signature(signature: &MultiSignature, signer: AccountId32, payload: Vec<u8>) -> bool {
	let verify_signature = |payload: &[u8]| signature.verify(payload, &signer.clone().into());

	if verify_signature(&payload) {
		return true;
	}

	let wrapped_payload = wrap_binary_data(payload);
	verify_signature(&wrapped_payload)
}

#[cfg(test)]
use sp_core::{sr25519, Pair};

#[test]
fn test_verify_signature_with_wrapped_bytes() {
	let (key_pair_delegator, _) = sr25519::Pair::generate();

	let payload = b"test_payload".to_vec();
	let encode_add_provider_data = wrap_binary_data(payload.clone());

	let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

	assert!(check_signature(&signature, key_pair_delegator.public().into(), payload.clone()));
}

#[test]
fn test_verify_signature_without_wrapped_bytes() {
	let (signer, _) = sr25519::Pair::generate();

	let payload = b"test_payload".to_vec();

	let signature: MultiSignature = signer.sign(&payload).into();

	assert!(check_signature(&signature, signer.public().into(), payload));
}

#[test]
fn test_check_signature_with_invalid_signature() {
	let (signer, _) = sr25519::Pair::generate();

	let payload = b"test_payload".to_vec();

	let signature: MultiSignature = signer.sign(&payload).into();

	let invalid_payload = b"invalid_payload".to_vec();

	assert!(!check_signature(&signature, signer.public().into(), invalid_payload));
}
