use common_primitives::{
	signatures::UnifiedSignature,
	utils::{wrap_binary_data, EIP712Encode},
};
use sp_runtime::{traits::Verify, AccountId32, MultiSignature};
extern crate alloc;
use sp_core::Encode;

pub fn check_signature<P>(signature: &MultiSignature, signer: AccountId32, payload: &P) -> bool
where
	P: Encode + EIP712Encode,
{
	let unified_signature: UnifiedSignature = signature.clone().into();
	let scale_encoded = payload.encode();
	let verify_signature = |payload: &[u8]| unified_signature.verify(payload, &signer.clone());

	if verify_signature(&scale_encoded) {
		return true;
	}

	match unified_signature {
		// we don't need to check the wrapped bytes for ethereum signatures but we need to check EIP-712 ones
		UnifiedSignature::Ecdsa(_) => verify_signature(&payload.encode_eip_712()),
		_ => {
			let wrapped_payload = wrap_binary_data(scale_encoded);
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
	/// A wrapped vec that allow different encodings for signature checks
	#[derive(Clone, Debug, Encode)]
	pub struct TestArrayWrapper(pub [u8; 12]);

	impl EIP712Encode for TestArrayWrapper {
		fn encode_eip_712(&self) -> Box<[u8]> {
			// not used in test but required to be implemented
			Vec::new().into_boxed_slice()
		}
	}

	#[test]
	fn test_verify_signature_with_wrapped_bytes() {
		let (key_pair_delegator, _) = sr25519::Pair::generate();

		let payload = b"test_payload";
		let encode_add_provider_data = wrap_binary_data(payload.to_vec());

		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		assert!(check_signature(
			&signature,
			key_pair_delegator.public().into(),
			&TestArrayWrapper(payload.clone())
		));
	}

	#[test]
	fn test_verify_signature_without_wrapped_bytes() {
		let (signer, _) = sr25519::Pair::generate();

		let payload = b"test_payload";

		let signature: MultiSignature = signer.sign(payload.as_slice()).into();

		assert!(check_signature(
			&signature,
			signer.public().into(),
			&TestArrayWrapper(payload.clone())
		));
	}

	#[test]
	fn test_check_signature_with_invalid_signature() {
		let (signer, _) = sr25519::Pair::generate();

		let payload = b"test_payload";

		let signature: MultiSignature = signer.sign(payload.as_slice()).into();

		let invalid_payload = b"fake_payload";

		assert!(!check_signature(
			&signature,
			signer.public().into(),
			&TestArrayWrapper(invalid_payload.clone())
		));
	}

	#[test]
	fn test_ethereum_verify_signature_without_wrapped_bytes_should_work() {
		let (signer, _) = ecdsa::Pair::generate();

		let payload = b"test_payload";

		let signature: MultiSignature =
			signer.sign_prehashed(&keccak_256(&payload.to_vec())).into();
		let unified_signer = UnifiedSigner::from(signer.public());

		assert!(check_signature(
			&signature,
			unified_signer.into_account(),
			&TestArrayWrapper(payload.clone())
		));
	}

	#[test]
	fn test_ethereum_verify_signature_wrapped_bytes_should_fail() {
		let (signer, _) = ecdsa::Pair::generate();

		let payload = b"test_payload";
		let encode_add_provider_data = wrap_binary_data(payload.to_vec());
		let signature: MultiSignature =
			signer.sign_prehashed(&keccak_256(&encode_add_provider_data)).into();
		let unified_signer = UnifiedSigner::from(signer.public());

		assert!(!check_signature(
			&signature,
			unified_signer.into_account(),
			&TestArrayWrapper(payload.clone())
		));
	}
}
