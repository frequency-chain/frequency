use crate::Config;
use frame_support::{
	pallet_prelude::{ConstU32, Decode, Encode, MaxEncodedLen, TypeInfo},
	BoundedVec, RuntimeDebugNoBound,
};
use p256::{ecdsa::signature::Verifier, EncodedPoint};
use sp_io::hashing::sha2_256;
use sp_runtime::MultiSignature;
#[allow(unused)]
use sp_std::boxed::Box;
use sp_std::vec::Vec;

/// This is the placeholder value that should be replaced by calculated challenge for
/// evaluation of a Passkey signature.
pub const CHALLENGE_PLACEHOLDER: &str = "#rplc#";
/// Passkey AuthenticatorData type. The length is 37 bytes or more
/// <https://w3c.github.io/webauthn/#authenticator-data>
pub type PasskeyAuthenticatorData = BoundedVec<u8, ConstU32<128>>;
/// Passkey ClientDataJson type
/// Note: The `challenge` field inside this json MUST be replaced with `CHALLENGE_PLACEHOLDER`
/// before submission to the chain
/// <https://w3c.github.io/webauthn/#dictdef-collectedclientdata>
pub type PasskeyClientDataJson = BoundedVec<u8, ConstU32<256>>;
/// PassKey Public Key type in compressed encoded point format
/// the first byte is the tag indicating compressed format
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, RuntimeDebugNoBound, Clone)]
pub struct PasskeyPublicKey(pub [u8; 33]);
/// PassKey Signature type
#[derive(
	Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, RuntimeDebugNoBound, Clone, Default,
)]
pub struct PasskeySignature(pub BoundedVec<u8, ConstU32<96>>);

/// Passkey Payload
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, RuntimeDebugNoBound, Clone)]
#[scale_info(skip_type_params(T))]
pub struct PasskeyPayload<T: Config> {
	/// passkey public key
	pub passkey_public_key: PasskeyPublicKey,
	/// a self-contained verifiable passkey signature with all required metadata
	pub verifiable_passkey_signature: VerifiablePasskeySignature,
	/// PassKey Call
	pub passkey_call: PasskeyCall<T>,
}

/// Passkey Payload V2
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, RuntimeDebugNoBound, Clone)]
#[scale_info(skip_type_params(T))]
pub struct PasskeyPayloadV2<T: Config> {
	/// passkey public key
	pub passkey_public_key: PasskeyPublicKey,
	/// a self-contained verifiable passkey signature with all required metadata
	pub verifiable_passkey_signature: VerifiablePasskeySignature,
	/// passkey_public_key signed by account_id's private key
	pub account_ownership_proof: MultiSignature,
	/// PassKey Call
	pub passkey_call: PasskeyCallV2<T>,
}

/// A verifiable Pass key contains all the required information to verify a passkey signature
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, RuntimeDebugNoBound, Clone)]
pub struct VerifiablePasskeySignature {
	/// passkey signature of `passkey_call`
	pub signature: PasskeySignature,
	/// passkey authenticator data
	pub authenticator_data: PasskeyAuthenticatorData,
	/// passkey client data in json format
	pub client_data_json: PasskeyClientDataJson,
}

/// Inner Passkey call
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, RuntimeDebugNoBound, Clone)]
#[scale_info(skip_type_params(T))]
pub struct PasskeyCall<T: Config> {
	/// account id which is the origin of this call
	pub account_id: T::AccountId,
	/// account nonce
	pub account_nonce: T::Nonce,
	/// passkey_public_key signed by account_id's private key
	pub account_ownership_proof: MultiSignature,
	/// Extrinsic call
	pub call: Box<<T as Config>::RuntimeCall>,
}

/// Inner Passkey call V2
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, RuntimeDebugNoBound, Clone)]
#[scale_info(skip_type_params(T))]
pub struct PasskeyCallV2<T: Config> {
	/// account id which is the origin of this call
	pub account_id: T::AccountId,
	/// account nonce
	pub account_nonce: T::Nonce,
	/// Extrinsic call
	pub call: Box<<T as Config>::RuntimeCall>,
}

impl<T: Config> From<PasskeyPayload<T>> for PasskeyPayloadV2<T> {
	fn from(payload: PasskeyPayload<T>) -> Self {
		PasskeyPayloadV2 {
			passkey_public_key: payload.passkey_public_key,
			verifiable_passkey_signature: payload.verifiable_passkey_signature,
			account_ownership_proof: payload.passkey_call.account_ownership_proof,
			passkey_call: PasskeyCallV2 {
				account_id: payload.passkey_call.account_id,
				account_nonce: payload.passkey_call.account_nonce,
				call: payload.passkey_call.call,
			},
		}
	}
}

impl<T: Config> From<PasskeyCall<T>> for PasskeyCallV2<T> {
	fn from(call: PasskeyCall<T>) -> Self {
		PasskeyCallV2 {
			account_id: call.account_id,
			account_nonce: call.account_nonce,
			call: call.call,
		}
	}
}

impl PasskeySignature {
	/// returns the inner raw data as a vector
	pub fn to_vec(&self) -> Vec<u8> {
		self.0.to_vec()
	}
}

impl TryFrom<PasskeySignature> for p256::ecdsa::DerSignature {
	type Error = ();

	fn try_from(value: PasskeySignature) -> Result<Self, Self::Error> {
		let result = p256::ecdsa::DerSignature::from_bytes(&value.to_vec()[..]).map_err(|_| ())?;
		Ok(result)
	}
}

impl PasskeyPublicKey {
	/// returns the inner raw data
	pub fn inner(&self) -> [u8; 33] {
		self.0
	}
}

impl TryFrom<EncodedPoint> for PasskeyPublicKey {
	type Error = ();

	fn try_from(value: EncodedPoint) -> Result<Self, Self::Error> {
		let bytes = value.as_bytes().to_vec();
		let inner: [u8; 33] = bytes.try_into().map_err(|_| ())?;
		Ok(PasskeyPublicKey(inner))
	}
}

impl TryFrom<&PasskeyPublicKey> for p256::ecdsa::VerifyingKey {
	type Error = ();

	fn try_from(value: &PasskeyPublicKey) -> Result<Self, Self::Error> {
		let encoded_point = EncodedPoint::from_bytes(&value.inner()[..]).map_err(|_| ())?;

		let result =
			p256::ecdsa::VerifyingKey::from_encoded_point(&encoded_point).map_err(|_| ())?;

		Ok(result)
	}
}

impl TryFrom<Vec<u8>> for PasskeySignature {
	type Error = ();

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		let inner: BoundedVec<u8, ConstU32<96>> = value.try_into().map_err(|_| ())?;
		Ok(PasskeySignature(inner))
	}
}

/// Passkey verification error types
pub enum PasskeyVerificationError {
	/// Invalid Passkey signature
	InvalidSignature,
	/// Invalid Passkey public key
	InvalidPublicKey,
	/// Invalid Client data json
	InvalidClientDataJson,
	/// Invalid proof
	InvalidProof,
	/// Invalid authenticator data
	InvalidAuthenticatorData,
}

impl From<PasskeyVerificationError> for u8 {
	fn from(value: PasskeyVerificationError) -> Self {
		match value {
			PasskeyVerificationError::InvalidSignature => 0u8,
			PasskeyVerificationError::InvalidPublicKey => 1u8,
			PasskeyVerificationError::InvalidClientDataJson => 2u8,
			PasskeyVerificationError::InvalidProof => 3u8,
			PasskeyVerificationError::InvalidAuthenticatorData => 4u8,
		}
	}
}

impl VerifiablePasskeySignature {
	/// verifying a P256 Passkey signature
	pub fn try_verify(
		&self,
		msg: &[u8],
		signer: &PasskeyPublicKey,
	) -> Result<(), PasskeyVerificationError> {
		let verifying_key: p256::ecdsa::VerifyingKey =
			signer.try_into().map_err(|_| PasskeyVerificationError::InvalidPublicKey)?;
		let passkey_signature: p256::ecdsa::DerSignature = self
			.signature
			.clone()
			.try_into()
			.map_err(|_| PasskeyVerificationError::InvalidSignature)?;
		let calculated_challenge = sha2_256(msg);
		let calculated_challenge_base64url = base64_url::encode(&calculated_challenge);

		// inject challenge inside clientJsonData
		let str_of_json = core::str::from_utf8(&self.client_data_json)
			.map_err(|_| PasskeyVerificationError::InvalidClientDataJson)?;
		let original_client_data_json =
			str_of_json.replace(CHALLENGE_PLACEHOLDER, &calculated_challenge_base64url);

		// prepare signing payload which is [authenticator || sha256(client_data_json)]
		let mut passkey_signature_payload = self.authenticator_data.to_vec();
		if passkey_signature_payload.len() < 37 {
			return Err(PasskeyVerificationError::InvalidAuthenticatorData);
		}
		passkey_signature_payload
			.extend_from_slice(&sha2_256(&original_client_data_json.as_bytes()));

		// finally verify the passkey signature against the payload
		verifying_key
			.verify(&passkey_signature_payload, &passkey_signature)
			.map_err(|_| PasskeyVerificationError::InvalidProof)
	}
}
