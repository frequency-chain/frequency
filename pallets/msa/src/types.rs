use super::*;
pub use common_primitives::msa::{Delegator, KeyInfoResponse, MessageSourceId, Provider};
use scale_info::TypeInfo;

use codec::{Decode, Encode};

pub const EMPTY_FUNCTION: fn(MessageSourceId) -> DispatchResult = |_| Ok(());

/// A type definition for the payload of adding an MSA key - [add_key_to_msa](msa::add_key_to_msa)
#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq)]
pub struct AddKeyData {
	/// Message Source Account identifier
	pub msa_id: MessageSourceId,
	/// A cryptographic nonce.
	pub nonce: u32,
}

#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq)]
pub struct AddProvider {
	pub authorized_msa_id: MessageSourceId,
	pub permission: u8,
}
