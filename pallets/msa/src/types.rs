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

#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct KeyInfo<BlockNumber> {
	pub msa_id: MessageSenderId,
	pub nonce: u32,
	pub expired: BlockNumber,
}

impl<BlockNumber: Clone> KeyInfo<BlockNumber> {
	pub fn map_to_response<AccountId: Clone>(
		&self,
		key: AccountId,
	) -> KeyInfoResponse<AccountId, BlockNumber> {
		KeyInfoResponse {
			key: key.clone(),
			msa_id: self.msa_id,
			nonce: self.nonce,
			expired: self.expired.clone(),
		}
	}
}

#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq)]
pub struct AddProvider {
	pub authorized_msa_id: MessageSenderId,
	pub permission: u8,
}
