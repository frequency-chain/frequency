#[cfg(feature = "std")]
use crate::utils;
use crate::{msa::MessageSourceId, node::BlockNumber};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::One;
extern crate alloc;
use alloc::{vec, vec::Vec};
#[cfg(feature = "std")]
use utils::*;

/// A type for responding with an single Message in an RPC-call dependent on schema model
/// IPFS, Parquet: { index, block_number, provider_msa_id, cid, payload_length }
/// Avro, OnChain: { index, block_number, provider_msa_id, msa_id, payload }
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct MessageResponse {
	/// Message source account id of the Provider. This may be the same id as contained in `msa_id`,
	/// indicating that the original source MSA is acting as its own provider. An id differing from that
	/// of `msa_id` indicates that `provider_msa_id` was delegated by `msa_id` to send this message on
	/// its behalf .
	pub provider_msa_id: MessageSourceId,
	/// Index in block to get total order.
	pub index: u16,
	/// Block-number for which the message was stored.
	pub block_number: BlockNumber,
	///  Message source account id (the original source).
	#[cfg_attr(feature = "std", serde(skip_serializing_if = "Option::is_none", default))]
	pub msa_id: Option<MessageSourceId>,
	/// Serialized data in a the schemas.
	#[cfg_attr(
		feature = "std",
		serde(with = "as_hex_option", skip_serializing_if = "Option::is_none", default)
	)]
	pub payload: Option<Vec<u8>>,
	/// The content address for an IPFS payload in Base32. Will always be CIDv1.
	#[cfg_attr(
		feature = "std",
		serde(with = "as_string_option", skip_serializing_if = "Option::is_none", default)
	)]
	pub cid: Option<Vec<u8>>,
	///  Offchain payload length (IPFS).
	#[cfg_attr(feature = "std", serde(skip_serializing_if = "Option::is_none", default))]
	pub payload_length: Option<u32>,
}
/// A type for requesting paginated messages.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct BlockPaginationRequest {
	/// Starting block-number (inclusive).
	pub from_block: BlockNumber,
	/// Current page index starting from 0.
	pub from_index: u32,
	/// Ending block-number (exclusive).
	pub to_block: BlockNumber,
	/// The number of messages in a single page.
	pub page_size: u32,
}

impl BlockPaginationRequest {
	/// Hard limit on the number of items per page that can be returned
	pub const MAX_PAGE_SIZE: u32 = 10000;
	/// Hard limit on the block range for a request (~7 days at 12 sec per block)
	pub const MAX_BLOCK_RANGE: u32 = 50000; // ~3 days (6 sec per block)= ~7 days (12 sec per block)

	/// Helper function for request validation.
	/// * Page size should not exceed MAX_PAGE_SIZE.
	/// * Block range [from_block:to_block) should not exceed MAX_BLOCK_RANGE.
	pub fn validate(&self) -> bool {
		self.page_size > 0 &&
			self.page_size <= Self::MAX_PAGE_SIZE &&
			self.from_block < self.to_block &&
			self.to_block - self.from_block <= Self::MAX_BLOCK_RANGE
	}
}

/// A type for responding with a collection of paginated messages.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct BlockPaginationResponse<T> {
	/// Collection of messages for a given [`BlockPaginationRequest`].
	pub content: Vec<T>,
	/// Flag to indicate the end of paginated messages.
	pub has_next: bool,
	#[cfg_attr(feature = "std", serde(skip_serializing_if = "Option::is_none"))]
	/// Flag to indicate the starting block number for the next page.
	pub next_block: Option<BlockNumber>,
	#[cfg_attr(feature = "std", serde(skip_serializing_if = "Option::is_none"))]
	/// Flag to indicate the next index for the following request.
	pub next_index: Option<u32>,
}

impl<T> BlockPaginationResponse<T> {
	/// Generates a new empty Pagination request
	pub const fn new() -> BlockPaginationResponse<T> {
		BlockPaginationResponse {
			content: vec![],
			has_next: false,
			next_block: None,
			next_index: None,
		}
	}

	/// Checks if we are at the end of the pagination
	/// if we are, update the response with the correct next information
	pub fn check_end_condition_and_set_next_pagination(
		&mut self,
		block_number: BlockNumber,
		current_index: u32,
		list_size: u32,
		request: &BlockPaginationRequest,
	) -> bool {
		if self.content.len() as u32 == request.page_size {
			let mut next_block = block_number;
			let mut next_index = current_index + 1;

			// checking if it's end of current list
			if next_index == list_size {
				next_block = block_number + BlockNumber::one();
				next_index = 0;
			}

			if next_block < request.to_block {
				self.has_next = true;
				self.next_block = Some(next_block);
				self.next_index = Some(next_index);
			}
			return true
		}

		false
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		messages::{BlockPaginationRequest, BlockPaginationResponse, MessageResponse},
		node::BlockNumber,
	};

	struct TestCase<T> {
		input: BlockPaginationRequest,
		expected: T,
		message: String,
	}

	#[test]
	fn as_hex_option_msg_ipfs_serialize_deserialize_test() {
		// skip deserialize if Option::none works
		let msg = MessageResponse {
			payload: None,
			msa_id: None,
			provider_msa_id: 1,
			index: 1,
			block_number: 1,
			cid: Some(
				"bafkreidgvpkjawlxz6sffxzwgooowe5yt7i6wsyg236mfoks77nywkptdq"
					.as_bytes()
					.to_vec(),
			),
			payload_length: Some(42),
		};
		let serialized = serde_json::to_string(&msg).unwrap();
		assert_eq!(serialized, "{\"provider_msa_id\":1,\"index\":1,\"block_number\":1,\"cid\":\"bafkreidgvpkjawlxz6sffxzwgooowe5yt7i6wsyg236mfoks77nywkptdq\",\"payload_length\":42}");

		let deserialized: MessageResponse = serde_json::from_str(&serialized).unwrap();
		assert_eq!(deserialized, msg);
	}

	#[test]
	fn as_hex_option_empty_payload_deserialize_as_default_value() {
		let expected_msg = MessageResponse {
			payload: None,
			msa_id: Some(1),
			provider_msa_id: 1,
			index: 1,
			block_number: 1,
			cid: None,
			payload_length: None,
		};

		// Notice Payload field is missing
		let serialized_msg_without_payload =
			"{\"provider_msa_id\":1,\"index\":1,\"block_number\":1,\"msa_id\":1}";

		let deserialized_result: MessageResponse =
			serde_json::from_str(serialized_msg_without_payload).unwrap();
		assert_eq!(deserialized_result, expected_msg);
	}

	#[test]
	fn block_pagination_request_validation_test() {
		let test_cases: Vec<TestCase<bool>> = vec![
			TestCase {
				input: BlockPaginationRequest { from_block: 10, from_index: 0, to_block: 12, page_size: 1 },
				expected: true,
				message: "Should be valid".to_string(),
			},
			TestCase {
				input: BlockPaginationRequest { from_block: 10, from_index: 0, to_block: 12, page_size: 0 },
				expected: false,
				message: "Page with size 0 is invalid".to_string(),
			},
			TestCase {
				input: BlockPaginationRequest { from_block: 10, from_index: 0, to_block: 8, page_size: 1 },
				expected: false,
				message: "from_block should be less than to_block".to_string(),
			},
			TestCase {
				input: BlockPaginationRequest { from_block: 10, from_index: 0, to_block: 8, page_size: 10000 + 1 },
				expected: false,
				message: "page_size should be less than MAX_PAGE_SIZE".to_string(),
			},
			TestCase {
				input: BlockPaginationRequest { from_block: 1, from_index: 0, to_block: 50000 + 2, page_size: 1 },
				expected: false,
				message: "the difference between from_block and to_block should be less than MAX_BLOCK_RANGE".to_string(),
			},
		];

		for tc in test_cases {
			assert_eq!(tc.expected, tc.input.validate(), "{}", tc.message);
		}
	}

	#[test]
	fn check_end_condition_does_not_mutate_when_at_the_end() {
		let mut resp = BlockPaginationResponse::<u32> {
			content: vec![1, 2, 3],
			has_next: false,
			next_block: None,
			next_index: None,
		};

		let total_data_length: u32 = resp.content.len() as u32;

		let request = BlockPaginationRequest {
			from_block: 1 as BlockNumber,
			from_index: 0,
			to_block: 5,
			// Page is LARGER
			page_size: total_data_length + 10,
		};
		// We are at the LAST block
		let current_block = 5;
		// Index after content
		let current_index = total_data_length - 1;
		// Critical Bit: NO more data than index
		let list_size = current_index;
		let is_full = resp.check_end_condition_and_set_next_pagination(
			current_block,
			current_index,
			list_size,
			&request,
		);
		// NOT FULL
		assert!(!is_full);
		// NOTHING MORE
		assert!(!resp.has_next);
		// None
		assert_eq!(None, resp.next_block);
		assert_eq!(None, resp.next_index);
	}

	#[test]
	fn check_end_condition_mutates_when_more_in_list_than_page() {
		let mut resp = BlockPaginationResponse::<u32> {
			content: vec![1, 2, 3],
			has_next: false,
			next_block: None,
			next_index: None,
		};

		let total_data_length: u32 = resp.content.len() as u32;

		let request = BlockPaginationRequest {
			from_block: 1 as BlockNumber,
			from_index: 0,
			to_block: 5,
			page_size: total_data_length,
		};
		// We have not completed the block yet
		let current_block = 1;
		// End of the Block
		let current_index = total_data_length - 1;
		// Critical Bit: MORE Data to go in length than page_size
		let list_size = total_data_length + 1;
		let is_full = resp.check_end_condition_and_set_next_pagination(
			current_block,
			current_index,
			list_size,
			&request,
		);
		assert!(is_full);
		assert!(resp.has_next);
		// SAME block
		assert_eq!(Some(1), resp.next_block);
		// NEXT index
		assert_eq!(Some(current_index + 1), resp.next_index);
	}

	#[test]
	fn check_end_condition_mutates_when_more_than_page_but_none_left_in_block() {
		let mut resp = BlockPaginationResponse::<u32> {
			content: vec![1, 2, 3],
			has_next: false,
			next_block: None,
			next_index: None,
		};

		let total_data_length: u32 = resp.content.len() as u32;

		let request = BlockPaginationRequest {
			from_block: 1 as BlockNumber,
			from_index: 0,
			to_block: 5,
			page_size: total_data_length,
		};
		// We have not completed the block yet
		let current_block = 1;
		// End of the Block
		let current_index = total_data_length - 1;
		// SAME in length than page_size
		let list_size = total_data_length;
		let is_full = resp.check_end_condition_and_set_next_pagination(
			current_block,
			current_index,
			list_size,
			&request,
		);
		assert!(is_full);
		assert!(resp.has_next);
		// NEXT block
		assert_eq!(Some(current_block + 1), resp.next_block);
		// ZERO index
		assert_eq!(Some(0), resp.next_index);
	}
}
