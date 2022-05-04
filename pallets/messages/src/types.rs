use codec::{Decode, Encode};
use common_primitives::{messages::MessageResponse, msa::MessageSenderId};
use scale_info::TypeInfo;
use sp_std::prelude::*;

#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct Message<AccountId> {
	pub data: Vec<u8>,           //  Serialized data in a user-defined schema format
	pub signer: AccountId,       //  Signature of the signer
	pub msa_id: MessageSenderId, //  Message source account id (the original sender)
	pub index: u16,              //  Stores index of message in block to keep total order
}

impl<AccountId> Message<AccountId>
where
	AccountId: Clone,
{
	pub fn map_to_response<BlockNumber>(
		&self,
		block_number: BlockNumber,
	) -> MessageResponse<AccountId, BlockNumber> {
		MessageResponse {
			signer: self.signer.clone(),
			index: self.index,
			msa_id: self.msa_id,
			block_number,
			data: self.data.clone(),
		}
	}
}
