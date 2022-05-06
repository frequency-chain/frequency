use codec::{Decode, Encode};
use common_primitives::{messages::MessageResponse, msa::MessageSenderId};
use frame_support::{traits::Get, BoundedVec};
use scale_info::TypeInfo;
use sp_std::prelude::*;

#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
#[scale_info(skip_type_params(MaxDataSize))]
pub struct Message<AccountId, MaxDataSize>
where
	MaxDataSize: Get<u32>,
{
	pub data: BoundedVec<u8, MaxDataSize>, //  Serialized data in a user-defined schema format
	pub signer: AccountId,                 //  Signature of the signer
	pub msa_id: MessageSenderId,           //  Message source account id (the original sender)
	pub index: u16,                        //  Stores index of message in block to keep total order
}

impl<AccountId, MaxDataSize> Message<AccountId, MaxDataSize>
where
	AccountId: Clone,
	MaxDataSize: Get<u32>,
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
			data: self.data.clone().into_inner(),
		}
	}
}
