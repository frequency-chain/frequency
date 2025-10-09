use common_primitives::{
	messages::MessageResponse,
	msa::MessageSourceId,
	node::BlockNumber,
	schema::{PayloadLocation, SchemaId},
};
use core::fmt::Debug;
use frame_support::{traits::Get, BoundedVec};
use multibase::Base;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
extern crate alloc;
use alloc::vec::Vec;
use common_primitives::messages::MessageResponseV2;

/// Multihash hash function code for SHA2-256
pub const SHA2_256: u64 = 0x12;
/// Multihash hash function code for Blake3
pub const BLAKE3: u64 = 0x1e;

/// Payloads stored offchain contain a tuple of (bytes(the payload reference), payload length).
pub type OffchainPayloadType = (Vec<u8>, u32);
/// Index of message in the block
pub type MessageIndex = u16;

/// A single message type definition.
#[derive(Default, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxDataSize))]
#[codec(mel_bound(MaxDataSize: MaxEncodedLen))]
pub struct Message<MaxDataSize>
where
	MaxDataSize: Get<u32> + Debug,
{
	///  Data structured by the associated schema's model.
	pub payload: BoundedVec<u8, MaxDataSize>,
	/// Message source account id of the Provider. This may be the same id as contained in `msa_id`,
	/// indicating that the original source MSA is acting as its own provider. An id differing from that
	/// of `msa_id` indicates that `provider_msa_id` was delegated by `msa_id` to send this message on
	/// its behalf.
	pub provider_msa_id: MessageSourceId,
	///  Message source account id (the original source).
	pub msa_id: Option<MessageSourceId>,
	///  The SchemaId of the schema that defines the format of the payload
	pub schema_id: SchemaId,
}

/// Trait for converting message storage to response type
pub trait MapToResponse<I, R> {
	/// Maps a stored message to an RPC response
	fn map_to_response(&self, index_values: I) -> Option<R>;
}

impl<MaxDataSize: Get<u32> + Debug>
	MapToResponse<(BlockNumber, PayloadLocation, u16), MessageResponse> for Message<MaxDataSize>
{
	fn map_to_response(
		&self,
		index_values: (BlockNumber, PayloadLocation, u16),
	) -> Option<MessageResponse> {
		let (block_number, payload_location, index) = index_values;
		let base_response = MessageResponse {
			provider_msa_id: self.provider_msa_id,
			index,
			block_number,
			msa_id: self.msa_id,
			..Default::default()
		};

		match payload_location {
			PayloadLocation::OnChain => Some(MessageResponse {
				payload: Some(self.payload.to_vec()),
				cid: None,
				payload_length: None,
				..base_response
			}),
			PayloadLocation::IPFS => {
				let (binary_cid, payload_length) =
					OffchainPayloadType::decode(&mut &self.payload[..]).unwrap_or_default();
				Some(MessageResponse {
					cid: Some(multibase::encode(Base::Base32Lower, binary_cid).as_bytes().to_vec()),
					payload_length: Some(payload_length),
					payload: None,
					..base_response
				})
			}, // Message types of Itemized and Paginated are retrieved differently
			_ => None,
		}
	}
}

impl<MaxDataSize: Get<u32> + Debug>
	MapToResponse<(BlockNumber, SchemaId, PayloadLocation, u16), MessageResponseV2>
	for Message<MaxDataSize>
{
	/// Helper function to handle response type [`MessageResponseV2`] depending on the Payload Location (on chain or IPFS)
	fn map_to_response(
		&self,
		index_values: (BlockNumber, SchemaId, PayloadLocation, u16),
	) -> Option<MessageResponseV2> {
		let (block_number, schema_id, payload_location, index) = index_values;
		let base_response = MessageResponseV2 {
			provider_msa_id: self.provider_msa_id,
			index,
			block_number,
			msa_id: self.msa_id,
			schema_id,
			..Default::default()
		};

		match payload_location {
			PayloadLocation::OnChain => Some(MessageResponseV2 {
				payload: Some(self.payload.to_vec()),
				cid: None,
				payload_length: None,
				..base_response
			}),
			PayloadLocation::IPFS => {
				let (binary_cid, payload_length) =
					OffchainPayloadType::decode(&mut &self.payload[..]).unwrap_or_default();
				Some(MessageResponseV2 {
					cid: Some(multibase::encode(Base::Base32Lower, binary_cid).as_bytes().to_vec()),
					payload_length: Some(payload_length),
					payload: None,
					..base_response
				})
			}, // Message types of Itemized and Paginated are retrieved differently
			_ => None,
		}
	}
}

impl<MaxDataSize: Get<u32> + Debug>
	MapToResponse<(BlockNumber, PayloadLocation, u16), MessageResponseV2> for Message<MaxDataSize>
{
	/// Helper function to handle response type [`MessageResponseV2`] depending on the Payload Location (on chain or IPFS)
	fn map_to_response(
		&self,
		index_values: (BlockNumber, PayloadLocation, u16),
	) -> Option<MessageResponseV2> {
		let (block_number, payload_location, index) = index_values;
		let base_response = MessageResponseV2 {
			provider_msa_id: self.provider_msa_id,
			index,
			block_number,
			msa_id: self.msa_id,
			schema_id: self.schema_id,
			..Default::default()
		};

		match payload_location {
			PayloadLocation::OnChain => Some(MessageResponseV2 {
				payload: Some(self.payload.to_vec()),
				cid: None,
				payload_length: None,
				..base_response
			}),
			PayloadLocation::IPFS => {
				let (binary_cid, payload_length) =
					OffchainPayloadType::decode(&mut &self.payload[..]).unwrap_or_default();
				Some(MessageResponseV2 {
					cid: Some(multibase::encode(Base::Base32Lower, binary_cid).as_bytes().to_vec()),
					payload_length: Some(payload_length),
					payload: None,
					..base_response
				})
			}, // Message types of Itemized and Paginated are retrieved differently
			_ => None,
		}
	}
}
