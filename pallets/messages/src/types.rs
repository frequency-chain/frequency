use codec::{Decode, Encode};
use common_primitives::{messages::MessageResponse, msa::MessageSenderId};
use frame_support::{traits::Get, BoundedVec};
use scale_info::TypeInfo;
use sp_std::prelude::*;

// #[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq)]
// pub struct MessageBulkPaylod<AccountId, MaxDataSize>
// where
// 	MaxDataSize: Get<u32> + Clone,
// {
// 	pub msg: Message<AccountId, MaxDataSize>,
// }

/// A single message type definition.
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
#[scale_info(skip_type_params(MaxDataSize))]
pub struct Message<AccountId, MaxDataSize>
where
	MaxDataSize: Get<u32> + Clone,
{
	///  Serialized data in a user-defined schema format
	pub payload: BoundedVec<u8, MaxDataSize>,
	///  Signature of the signer
	pub signer: AccountId,
	///  Message source account id (the original sender)
	pub msa_id: MessageSenderId,
	///  Stores index of message in block to keep total order
	pub index: u16,
}

impl<AccountId, MaxDataSize> Message<AccountId, MaxDataSize>
where
	AccountId: Clone,
	MaxDataSize: Get<u32> + Clone,
{
	/// Helper function to handle response type [`MessageResponse`] for RPC.
	pub fn map_to_response<BlockNumber>(
		&self,
		block_number: BlockNumber,
	) -> MessageResponse<AccountId, BlockNumber> {
		MessageResponse {
			signer: self.signer.clone(),
			index: self.index,
			msa_id: self.msa_id,
			block_number,
			payload: self.payload.clone().into_inner(),
		}
	}
}
