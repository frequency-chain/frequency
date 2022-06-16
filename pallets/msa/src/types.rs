use super::*;
pub use common_primitives::msa::{Delegator, KeyInfoResponse, MessageSourceId, Provider};
use scale_info::TypeInfo;

use codec::{Decode, Encode};

pub const EMPTY_FUNCTION: fn(MessageSourceId) -> DispatchResult = |_| Ok(());

#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq)]
pub struct AddKeyData {
	pub msa_id: MessageSourceId,
	pub nonce: u32,
}

#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq)]
pub struct AddProvider {
	pub authorized_msa_id: MessageSourceId,
	pub permission: u8,
}
