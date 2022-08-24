use codec::{Decode, Encode};
use common_primitives::{messages::MessageResponse, msa::MessageSourceId};
use frame_support::{traits::Get, BoundedVec};
use scale_info::TypeInfo;
use sp_std::prelude::*;

/// A single message type definition.
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
#[scale_info(skip_type_params(MaxDataSize))]
pub struct Message<MaxDataSize>
where
	MaxDataSize: Get<u32> + Clone,
{
	///  Data structured by the associated schema's model.
	pub payload: BoundedVec<u8, MaxDataSize>,
	/// Message source account id of the Provider. This may be the same id as contained in `msa_id`,
	/// indicating that the original source MSA is acting as its own provider. An id differing from that
	/// of `msa_id` indicates that `provider_msa_id` was delegated by `msa_id` to send this message on
	/// its behalf.
	pub provider_msa_id: MessageSourceId,
	///  Message source account id (the original source).
	pub msa_id: MessageSourceId,
	///  Stores index of message in block to keep total order.
	pub index: u16,
}

impl<MaxDataSize> Message<MaxDataSize>
where
	MaxDataSize: Get<u32> + Clone,
{
	/// Helper function to handle response type [`MessageResponse`] for RPC.
	pub fn map_to_response<BlockNumber>(
		&self,
		block_number: BlockNumber,
		payload_length: u32,
	) -> MessageResponse<BlockNumber> {
		MessageResponse {
			provider_msa_id: self.provider_msa_id,
			index: self.index,
			msa_id: self.msa_id,
			block_number,
			payload: self.payload.clone().into_inner(),
			payload_length,
		}
	}
}
