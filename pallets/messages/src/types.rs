use codec::{Decode, Encode};
use common_primitives::{messages::MessageResponse, msa::MessageSourceId};
use frame_support::{traits::Get, BoundedVec};
use scale_info::TypeInfo;
use sp_std::prelude::*;

/// A single message type definition.
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
#[scale_info(skip_type_params(MaxDataSize))]
pub struct Message<AccountId, MaxDataSize>
where
	MaxDataSize: Get<u32> + Clone,
{
	///  Data structured by the associated schema's model
	pub payload: BoundedVec<u8, MaxDataSize>,
	///  Public key of the provider that signed the transaction
	pub provider_key: AccountId,
	///  Message source account id (the original source)
	pub msa_id: MessageSourceId,
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
			provider_key: self.provider_key.clone(),
			index: self.index,
			msa_id: self.msa_id,
			block_number,
			payload: self.payload.clone().into_inner(),
		}
	}
}
