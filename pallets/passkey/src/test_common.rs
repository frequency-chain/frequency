///
/// Constants used both in benchmarks and tests
///
#[allow(unused)]
pub mod constants {
	use super::*;
	/// client data json in base64-url format, the challenged is replaced with `#rplc#`
	pub const REPLACED_CLIENT_DATA_JSON: &'static str = "eyJ0eXBlIjoid2ViYXV0aG4uZ2V0IiwiY2hhbGxlbmdlIjoiI3JwbGMjIiwib3JpZ2luIjoiaHR0cHM6Ly9wYXNza2V5LmFtcGxpY2EuaW86ODA4MCIsImNyb3NzT3JpZ2luIjpmYWxzZSwiYWxnIjoiSFMyNTYifQ";
	/// authenticator data in base64-url format
	pub const AUTHENTICATOR_DATA: &'static str =
		"WJ8JTNbivTWn-433ubs148A7EgWowi4SAcYBjLWfo1EdAAAAAA";
}

/// Utility functions to be used across tests and benchmarks
#[allow(unused)]
pub mod utilities {
	use crate::{PasskeyPublicKey, PasskeySignature, CHALLENGE_PLACEHOLDER};
	use p256::{
		ecdsa::{signature::Signer, SigningKey},
		elliptic_curve::sec1::ToEncodedPoint,
	};
	use sp_io::hashing::sha2_256;

	/// get PasskeyPublicKey from Secret key
	pub fn get_p256_public_key(secret: &p256::SecretKey) -> PasskeyPublicKey {
		let encoded = secret.public_key().to_encoded_point(true);
		let passkey_public_key: PasskeyPublicKey = encoded.try_into().unwrap();
		passkey_public_key
	}
	/// getting a passkey specific signature
	pub fn passkey_sign(
		secret: &p256::SecretKey,
		payload: &[u8],
		client_data_json: &[u8],
		authenticator_data: &[u8],
	) -> PasskeySignature {
		let signing_key: SigningKey = secret.into();
		let calculated_challenge = sha2_256(payload);
		let calculated_challenge_base64url = base64_url::encode(&calculated_challenge);

		// inject challenge inside clientJsonData
		let str_of_json = core::str::from_utf8(client_data_json).unwrap();
		let original_client_data_json =
			str_of_json.replace(CHALLENGE_PLACEHOLDER, &calculated_challenge_base64url);

		// prepare signing payload which is [authenticator || sha256(client_data_json)]
		let mut passkey_signature_payload = authenticator_data.to_vec();
		passkey_signature_payload
			.extend_from_slice(&sha2_256(&original_client_data_json.as_bytes()));

		let (signature, _) = signing_key.try_sign(&passkey_signature_payload).unwrap();
		let der_sig = p256::ecdsa::DerSignature::from(signature);
		let passkey_signature: PasskeySignature = der_sig.as_bytes().to_vec().try_into().unwrap();
		passkey_signature
	}
}
