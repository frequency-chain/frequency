use super::*;
use common_primitives::msa::{KeyInfoResponse, MessageSenderId};
use scale_info::TypeInfo;

use codec::{Decode, Encode};

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

impl<BlockNumber> KeyInfo<BlockNumber>
where
	BlockNumber: Clone,
{
	pub fn map_to_response<AccountId>(
		&self,
		key: AccountId,
	) -> KeyInfoResponse<AccountId, BlockNumber>
	where
		AccountId: Clone,
	{
		KeyInfoResponse {
			key: key.clone(),
			msa_id: self.msa_id,
			nonce: self.nonce,
			expired: self.expired.clone(),
		}
	}
}
