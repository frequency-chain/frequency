use common_primitives::{signatures::UnifiedSignature, utils::wrap_binary_data};
use sp_runtime::{traits::Verify, AccountId32, MultiSignature};
extern crate alloc;
use alloc::vec::Vec;

pub fn check_signature(signature: &MultiSignature, signer: AccountId32, payload: Vec<u8>) -> bool {
	let unified_signature: UnifiedSignature = signature.clone().into();
	let verify_signature =
		|payload: &[u8]| unified_signature.verify(payload, &signer.clone().into());

	if verify_signature(&payload) {
		return true;
	}

	match unified_signature {
		// we don't need to check the wrapped bytes for ethereum signatures
		UnifiedSignature::Ecdsa(_) => false,
		_ => {
			let wrapped_payload = wrap_binary_data(payload);
			verify_signature(&wrapped_payload)
		},
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common_primitives::signatures::UnifiedSigner;
	use sp_core::{ecdsa, keccak_256, sr25519, Pair};
	use sp_runtime::traits::IdentifyAccount;

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

	#[test]
	fn test_ethereum_verify_signature_without_wrapped_bytes_should_work() {
		let (signer, _) = ecdsa::Pair::generate();

		let payload = b"test_payload".to_vec();

		let signature: MultiSignature = signer.sign_prehashed(&keccak_256(&payload)).into();
		let unified_signer = UnifiedSigner::from(signer.public());

		assert!(check_signature(&signature, unified_signer.into_account(), payload));
	}

	#[test]
	fn test_ethereum_verify_signature_wrapped_bytes_should_fail() {
		let (signer, _) = ecdsa::Pair::generate();

		let payload = b"test_payload".to_vec();
		let encode_add_provider_data = wrap_binary_data(payload.clone());
		let signature: MultiSignature =
			signer.sign_prehashed(&keccak_256(&encode_add_provider_data)).into();
		let unified_signer = UnifiedSigner::from(signer.public());

		assert_eq!(check_signature(&signature, unified_signer.into_account(), payload), false);
	}
}
