use super::*;
pub use common_primitives::msa::{Delegator, KeyInfoResponse, MessageSenderId, Provider};
use scale_info::TypeInfo;

use codec::{Decode, Encode};

pub const EMPTY_FUNCTION: fn(MessageSenderId) -> DispatchResult = |_| Ok(());

#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq)]
pub struct AddKeyData {
	pub msa_id: MessageSenderId,
	pub nonce: u32,
}

#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq)]
pub struct AddProvider {
	pub authorized_msa_id: MessageSenderId,
	pub permission: u8,
}
