use crate::msa::MessageSourceId;
#[cfg(feature = "std")]
use crate::utils;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::{prelude::*, vec};
#[cfg(feature = "std")]
use utils::*;

/// A type for responding with an single Message in an RPC-call.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct MessageResponse<AccountId, BlockNumber> {
	#[cfg_attr(feature = "std", serde(with = "as_hex"))]
	/// Serialized data in a user-defined schema format.
	pub payload: Vec<u8>,
	/// The public key of the provider and the signer of the transaction.
	pub provider_key: AccountId,
	/// Message source account id (the original source).
	pub msa_id: MessageSourceId,
	/// Index in block to get total order
	pub index: u16,
	/// Block-number for which the message was stored.
	pub block_number: BlockNumber,
}

/// A type for requesting paginated messages.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct BlockPaginationRequest<BlockNumber> {
	/// Starting block-number (inclusive).
	pub from_block: BlockNumber,
	/// Current page index starting from 0.
	pub from_index: u32,
	/// Ending block-number (exclusive).
	pub to_block: BlockNumber,
	/// The number of messages in a single page.
	pub page_size: u32,
}

impl<BlockNumber> BlockPaginationRequest<BlockNumber>
where
	BlockNumber: Copy + AtLeast32BitUnsigned,
{
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
			self.to_block.sub(self.from_block) <= BlockNumber::from(Self::MAX_BLOCK_RANGE)
	}
}

/// A type for responding with a collection of paginated messages.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct BlockPaginationResponse<BlockNumber, T> {
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

impl<BlockNumber, T> BlockPaginationResponse<BlockNumber, T> {
	/// Generates a new empty Pagination request
	/// # Returns
	/// * `BlockPaginationResponse<BlockNumber, T>`
	pub const fn new() -> BlockPaginationResponse<BlockNumber, T> {
		BlockPaginationResponse {
			content: vec![],
			has_next: false,
			next_block: None,
			next_index: None,
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::messages::BlockPaginationRequest;

	struct TestCase<T> {
		input: BlockPaginationRequest<u32>,
		expected: T,
		message: String,
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
}
