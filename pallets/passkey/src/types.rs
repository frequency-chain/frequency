use crate::Config;
use frame_support::{
	pallet_prelude::{ConstU32, Decode, Encode, MaxEncodedLen, TypeInfo},
	BoundedVec, RuntimeDebugNoBound,
};
use sp_runtime::MultiSignature;

/// This is the placeholder value that should be replaced by calculated challenge for
/// evaluation of a Passkey signature.
pub const CHALLENGE_PLACEHOLDER: &str = "#rplc#";
/// PassKey Public Key type in compressed encoded point format
/// the first byte is the tag indicating compressed format
pub type PasskeyPublicKey = [u8; 33];
/// PassKey Signature type
pub type PasskeySignature = BoundedVec<u8, ConstU32<96>>;
/// Passkey AuthenticatorData type. The length is 37 bytes or more
/// https://w3c.github.io/webauthn/#authenticator-data
pub type PasskeyAuthenticatorData = BoundedVec<u8, ConstU32<128>>;
/// Passkey ClientDataJson type
/// Note: The `challenge` field inside this json MUST be replaced with `CHALLENGE_PLACEHOLDER`
/// before submission to the chain
/// https://w3c.github.io/webauthn/#dictdef-collectedclientdata
pub type PasskeyClientDataJson = BoundedVec<u8, ConstU32<256>>;

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
