use crate::Config;
use frame_support::{
	pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
	RuntimeDebugNoBound,
};
use sp_core::ConstU32;
use sp_runtime::{BoundedVec, MultiSignature};
use sp_std::{prelude::*, vec::Vec};

/// Gets stable weights for a capacity Call
pub trait GetStableWeight<RuntimeCall, Weight> {
	/// Get stable weights for Call
	fn get_stable_weight(call: &RuntimeCall) -> Option<Weight>;

	/// Get inner calls from a Call if any exist,
	/// e.g. in case of `pay_with_capacity` and `pay_with_capacity_batch_all`
	fn get_inner_calls(outer_call: &RuntimeCall) -> Option<Vec<&RuntimeCall>>;
}

/// PassKey Public Key type
pub type PasskeyPublicKey = BoundedVec<u8, ConstU32<96>>;
/// PassKey Signature type
pub type PasskeySignature = BoundedVec<u8, ConstU32<96>>;
/// Passkey Authenticator type
pub type PasskeyAuthenticator = BoundedVec<u8, ConstU32<96>>;
/// Passkey Authenticator type
pub type PasskeyClientDataJson = BoundedVec<u8, ConstU32<1024>>; // TODO: remove the challenge field to reduce the size

/// Passkey Payload
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, RuntimeDebugNoBound, Clone)]
#[scale_info(skip_type_params(T))]
pub struct PasskeyPayload<T: Config> {
	/// passkey public key
	pub passkey_public_key: PasskeyPublicKey,
	/// passkey signature of `passkey_call`
	pub passkey_signature: PasskeySignature,
	/// passkey authenticator data
	pub passkey_authenticator: PasskeyAuthenticator,
	// passkey client data in json format
	pub passkey_client_data_json: PasskeyClientDataJson,
	/// PassKey Call
	pub passkey_call: PasskeyCall<T>,
}

/// Inner Passkey call
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, RuntimeDebugNoBound, Clone)]
#[scale_info(skip_type_params(T))]
pub struct PasskeyCall<T: Config> {
	/// account id which is the origin of this call
	pub account_id: T::AccountId,
	/// passkey_public_key signed by account_id's private key
	pub account_ownership_proof: MultiSignature,
	/// Extrinsic call
	pub call: Box<<T as Config>::RuntimeCall>,
}
