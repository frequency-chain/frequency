//! Types for the Stateful Storage Pallet
use crate::Config;
use alloc::boxed::Box;
use common_primitives::{
	node::EIP712Encode,
	schema::SchemaId,
	stateful_storage::{PageHash, PageId, PageNonce},
};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use lazy_static::lazy_static;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::bounded::BoundedVec;
extern crate alloc;
use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use core::{
	cmp::*,
	fmt::Debug,
	hash::{Hash, Hasher},
};
use sp_core::U256;

use common_primitives::{signatures::get_eip712_encoding_prefix, utils::to_abi_compatible_number};
use twox_hash::XxHash64;

/// Migration page size
pub const MIGRATION_PAGE_SIZE: u32 = 20;
/// Current storage version of the pallet.
pub const STATEFUL_STORAGE_VERSION: StorageVersion = StorageVersion::new(1);
/// pallet storage prefix
pub const PALLET_STORAGE_PREFIX: &[u8] = b"stateful-storage";
/// itemized storage prefix
pub const ITEMIZED_STORAGE_PREFIX: &[u8] = b"itemized";
/// paginated storage prefix
pub const PAGINATED_STORAGE_PREFIX: &[u8] = b"paginated";

/// MultipartKey type for Itemized storage
pub type ItemizedKey = (SchemaId,);
/// MultipartKey type for Paginated storage (full key)
pub type PaginatedKey = (SchemaId, PageId);
/// MultipartKey type for Paginated storage (prefix lookup)
pub type PaginatedPrefixKey = (SchemaId,);
/// Itemized page type
pub type ItemizedPage<T> = Page<<T as Config>::MaxItemizedPageSizeBytes>;
/// Paginated Page type
pub type PaginatedPage<T> = Page<<T as Config>::MaxPaginatedPageSizeBytes>;

/// Operations on Itemized storage
pub trait ItemizedOperations<T: Config> {
	/// Applies all actions to specified page and returns the updated page
	fn apply_item_actions(
		&self,
		actions: &[ItemAction<T::MaxItemizedBlobSizeBytes>],
	) -> Result<ItemizedPage<T>, PageError>;

	/// Parses all the items inside an ItemPage
	fn try_parse(&self, include_header: bool) -> Result<ParsedItemPage, PageError>;
}
/// Defines the actions that can be applied to an Itemized storage
#[derive(
	Clone, Encode, Decode, DecodeWithMemTracking, Debug, TypeInfo, MaxEncodedLen, PartialEq,
)]
#[scale_info(skip_type_params(DataSize))]
#[codec(mel_bound(DataSize: MaxEncodedLen))]
pub enum ItemAction<DataSize: Get<u32> + Clone + core::fmt::Debug + PartialEq> {
	/// Adding new Item into page
	Add {
		/// The data to add
		data: BoundedVec<u8, DataSize>,
	},
	/// Removing an existing item by index number. Index number starts from 0
	Delete {
		/// Index (0+) to delete
		index: u16,
	},
}

/// This header is used to specify the byte size of an item stored inside the buffer
/// All items will require this header to be inserted before the item data
#[derive(Encode, Decode, PartialEq, MaxEncodedLen, Debug)]
pub struct ItemHeader {
	/// The length of this item, not including the size of this header.
	pub payload_len: u16,
}

/// Errors dedicated to parsing or modifying pages
#[derive(Debug, PartialEq)]
pub enum PageError {
	/// Unable to decode the data in the item
	ErrorParsing(&'static str),
	/// Add or Delete Operation was not possible
	InvalidAction(&'static str),
	/// ItemPage count overflow catch
	ArithmeticOverflow,
	/// Page byte length over the max size
	PageSizeOverflow,
}

// REMOVED ItemizedSignaturePayload

/// Payload containing all necessary fields to verify Itemized related signatures
#[derive(
	Encode,
	Decode,
	DecodeWithMemTracking,
	TypeInfo,
	MaxEncodedLen,
	PartialEq,
	RuntimeDebugNoBound,
	Clone,
)]
#[scale_info(skip_type_params(T))]
pub struct ItemizedSignaturePayloadV2<T: Config> {
	/// Schema id of this storage
	#[codec(compact)]
	pub schema_id: SchemaId,

	/// Hash of targeted page to avoid race conditions
	#[codec(compact)]
	pub target_hash: PageHash,

	/// The block number at which the signed proof will expire
	pub expiration: BlockNumberFor<T>,

	/// Actions to apply to storage from possible: [`ItemAction`]
	pub actions: BoundedVec<
		ItemAction<<T as Config>::MaxItemizedBlobSizeBytes>,
		<T as Config>::MaxItemizedActionsCount,
	>,
}

impl<T: Config> EIP712Encode for ItemizedSignaturePayloadV2<T> {
	fn encode_eip_712(&self, chain_id: u32) -> Box<[u8]> {
		lazy_static! {
			// signed payload
			static ref MAIN_TYPE_HASH: [u8; 32] =
				sp_io::hashing::keccak_256(b"ItemizedSignaturePayloadV2(uint16 schemaId,uint32 targetHash,uint32 expiration,ItemAction[] actions)ItemAction(string actionType,bytes data,uint16 index)");

			static ref SUB_TYPE_HASH: [u8; 32] =
				sp_io::hashing::keccak_256(b"ItemAction(string actionType,bytes data,uint16 index)");

			static ref ITEM_ACTION_ADD: [u8; 32] = sp_io::hashing::keccak_256(b"Add");
			static ref ITEM_ACTION_DELETE: [u8; 32] = sp_io::hashing::keccak_256(b"Delete");

			static ref EMPTY_BYTES_HASH: [u8; 32] = sp_io::hashing::keccak_256([].as_slice());
		}
		// get prefix and domain separator
		let prefix_domain_separator: Box<[u8]> =
			get_eip712_encoding_prefix("0xcccccccccccccccccccccccccccccccccccccccc", chain_id);
		let coded_schema_id = to_abi_compatible_number(self.schema_id);
		let coded_target_hash = to_abi_compatible_number(self.target_hash);
		let expiration: U256 = self.expiration.into();
		let coded_expiration = to_abi_compatible_number(expiration.as_u128());
		let coded_actions = {
			let values: Vec<u8> = self
				.actions
				.iter()
				.flat_map(|a| match a {
					ItemAction::Add { data } => sp_io::hashing::keccak_256(
						&[
							SUB_TYPE_HASH.as_slice(),
							ITEM_ACTION_ADD.as_slice(),
							&sp_io::hashing::keccak_256(data.as_slice()),
							[0u8; 32].as_slice(),
						]
						.concat(),
					),
					ItemAction::Delete { index } => sp_io::hashing::keccak_256(
						&[
							SUB_TYPE_HASH.as_slice(),
							ITEM_ACTION_DELETE.as_slice(),
							EMPTY_BYTES_HASH.as_slice(),
							to_abi_compatible_number(*index).as_slice(),
						]
						.concat(),
					),
				})
				.collect();
			sp_io::hashing::keccak_256(&values)
		};
		let message = sp_io::hashing::keccak_256(
			&[
				MAIN_TYPE_HASH.as_slice(),
				&coded_schema_id,
				&coded_target_hash,
				&coded_expiration,
				&coded_actions,
			]
			.concat(),
		);
		let combined = [prefix_domain_separator.as_ref(), &message].concat();
		combined.into_boxed_slice()
	}
}

// REMOVED PaginatedSignaturePayload

/// Payload containing all necessary fields to verify signatures to upsert a Paginated storage
#[derive(
	Encode,
	Decode,
	DecodeWithMemTracking,
	TypeInfo,
	MaxEncodedLen,
	PartialEq,
	RuntimeDebugNoBound,
	Clone,
)]
#[scale_info(skip_type_params(T))]
pub struct PaginatedUpsertSignaturePayloadV2<T: Config> {
	/// Schema id of this storage
	#[codec(compact)]
	pub schema_id: SchemaId,

	/// Page id of this storage
	#[codec(compact)]
	pub page_id: PageId,

	/// Hash of targeted page to avoid race conditions
	#[codec(compact)]
	pub target_hash: PageHash,

	/// The block number at which the signed proof will expire
	pub expiration: BlockNumberFor<T>,

	/// payload to update the page with
	pub payload: BoundedVec<u8, <T as Config>::MaxPaginatedPageSizeBytes>,
}

impl<T: Config> EIP712Encode for PaginatedUpsertSignaturePayloadV2<T> {
	fn encode_eip_712(&self, chain_id: u32) -> Box<[u8]> {
		lazy_static! {
			// signed payload
			static ref MAIN_TYPE_HASH: [u8; 32] =
				sp_io::hashing::keccak_256(b"PaginatedUpsertSignaturePayloadV2(uint16 schemaId,uint16 pageId,uint32 targetHash,uint32 expiration,bytes payload)");
		}
		// get prefix and domain separator
		let prefix_domain_separator: Box<[u8]> =
			get_eip712_encoding_prefix("0xcccccccccccccccccccccccccccccccccccccccc", chain_id);
		let coded_schema_id = to_abi_compatible_number(self.schema_id);
		let coded_page_id = to_abi_compatible_number(self.page_id);
		let coded_target_hash = to_abi_compatible_number(self.target_hash);
		let expiration: U256 = self.expiration.into();
		let coded_expiration = to_abi_compatible_number(expiration.as_u128());
		let coded_payload = sp_io::hashing::keccak_256(self.payload.as_slice());
		let message = sp_io::hashing::keccak_256(
			&[
				MAIN_TYPE_HASH.as_slice(),
				&coded_schema_id,
				&coded_page_id,
				&coded_target_hash,
				&coded_expiration,
				&coded_payload,
			]
			.concat(),
		);
		let combined = [prefix_domain_separator.as_ref(), &message].concat();
		combined.into_boxed_slice()
	}
}

// REMOVED PaginatedDeleteSignaturePayload

/// Payload containing all necessary fields to verify signatures to delete a Paginated storage
#[derive(
	Encode,
	Decode,
	DecodeWithMemTracking,
	TypeInfo,
	MaxEncodedLen,
	PartialEq,
	RuntimeDebugNoBound,
	Clone,
)]
#[scale_info(skip_type_params(T))]
pub struct PaginatedDeleteSignaturePayloadV2<T: Config> {
	/// Schema id of this storage
	#[codec(compact)]
	pub schema_id: SchemaId,

	/// Page id of this storage
	#[codec(compact)]
	pub page_id: PageId,

	/// Hash of targeted page to avoid race conditions
	#[codec(compact)]
	pub target_hash: PageHash,

	/// The block number at which the signed proof will expire
	pub expiration: BlockNumberFor<T>,
}

impl<T: Config> EIP712Encode for PaginatedDeleteSignaturePayloadV2<T> {
	fn encode_eip_712(&self, chain_id: u32) -> Box<[u8]> {
		lazy_static! {
			// signed payload
			static ref MAIN_TYPE_HASH: [u8; 32] =
				sp_io::hashing::keccak_256(b"PaginatedDeleteSignaturePayloadV2(uint16 schemaId,uint16 pageId,uint32 targetHash,uint32 expiration)");
		}
		// get prefix and domain separator
		let prefix_domain_separator: Box<[u8]> =
			get_eip712_encoding_prefix("0xcccccccccccccccccccccccccccccccccccccccc", chain_id);
		let coded_schema_id = to_abi_compatible_number(self.schema_id);
		let coded_page_id = to_abi_compatible_number(self.page_id);
		let coded_target_hash = to_abi_compatible_number(self.target_hash);
		let expiration: U256 = self.expiration.into();
		let coded_expiration = to_abi_compatible_number(expiration.as_u128());
		let message = sp_io::hashing::keccak_256(
			&[
				MAIN_TYPE_HASH.as_slice(),
				&coded_schema_id,
				&coded_page_id,
				&coded_target_hash,
				&coded_expiration,
			]
			.concat(),
		);
		let combined = [prefix_domain_separator.as_ref(), &message].concat();
		combined.into_boxed_slice()
	}
}

/// A generic page of data which supports both Itemized and Paginated
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Debug, Default)]
#[scale_info(skip_type_params(PageDataSize))]
#[codec(mel_bound(PageDataSize: MaxEncodedLen))]
pub struct Page<PageDataSize: Get<u32>> {
	/// Incremental nonce to eliminate of signature replay attacks
	pub nonce: PageNonce,
	/// Data for the page
	/// - Itemized is limited by [`Config::MaxItemizedPageSizeBytes`]
	/// - Paginated is limited by [`Config::MaxPaginatedPageSizeBytes`]
	pub data: BoundedVec<u8, PageDataSize>,
}

/// An internal struct which contains the parsed items in a page
#[derive(Debug, PartialEq)]
pub struct ParsedItemPage<'a> {
	/// Page current size
	pub page_size: usize,
	/// A map of item index to a slice of blob (including header is optional)
	pub items: BTreeMap<u16, &'a [u8]>,
}

impl<PageDataSize: Get<u32>> Page<PageDataSize> {
	/// Check if the page is empty
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}

	/// Retrieve the hash of the page
	pub fn get_hash(&self) -> PageHash {
		if self.is_empty() {
			return PageHash::default();
		}
		let mut hasher = XxHash64::with_seed(0);
		self.hash(&mut hasher);
		let value_bytes: [u8; 4] =
			hasher.finish().to_be_bytes()[..4].try_into().expect("incorrect hash size");
		PageHash::from_be_bytes(value_bytes)
	}
}

/// PartialEq and Hash should be both derived or implemented manually based on clippy rules
impl<PageDataSize: Get<u32>> Hash for Page<PageDataSize> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		state.write(&self.nonce.encode());
		state.write(&self.data[..]);
	}
}

/// PartialEq and Hash should be both derived or implemented manually based on clippy rules
impl<PageDataSize: Get<u32>> PartialEq for Page<PageDataSize> {
	fn eq(&self, other: &Self) -> bool {
		self.nonce.eq(&other.nonce) && self.data.eq(&other.data)
	}
}

/// Deserializing a Page from a BoundedVec is used for the input payload--
/// so there is no nonce to be read, just the raw data.
impl<PageDataSize: Get<u32>> From<BoundedVec<u8, PageDataSize>> for Page<PageDataSize> {
	fn from(bounded: BoundedVec<u8, PageDataSize>) -> Self {
		Self { nonce: PageNonce::default(), data: bounded }
	}
}

/// Deserializing a Page from a `Vec<u8>` is used for reading from storage--
/// so we must first read the nonce, then the data payload.
impl<PageDataSize: Get<u32>> TryFrom<Vec<u8>> for Page<PageDataSize> {
	type Error = ();

	fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
		let nonce: PageNonce =
			PageNonce::decode(&mut &data[..PageNonce::max_encoded_len()]).map_err(|_| ())?;
		let bounded: BoundedVec<u8, PageDataSize> =
			BoundedVec::try_from(data[PageNonce::max_encoded_len()..].to_vec()).map_err(|_| ())?;
		Ok(Self { nonce, data: bounded })
	}
}

impl<T: Config> ItemizedOperations<T> for ItemizedPage<T> {
	/// Applies all actions to specified page and returns the updated page
	/// This has O(n) complexity when n is the number of all the bytes in that itemized storage
	fn apply_item_actions(
		&self,
		actions: &[ItemAction<T::MaxItemizedBlobSizeBytes>],
	) -> Result<Self, PageError> {
		let mut parsed = ItemizedOperations::<T>::try_parse(self, true)?;

		let mut updated_page_buffer = Vec::with_capacity(parsed.page_size);
		let mut add_buffer = Vec::new();

		for action in actions {
			match action {
				ItemAction::Delete { index } => {
					ensure!(
						parsed.items.contains_key(index),
						PageError::InvalidAction("item index is invalid")
					);
					parsed.items.remove(index);
				},
				ItemAction::Add { data } => {
					let header = ItemHeader {
						payload_len: data
							.len()
							.try_into()
							.map_err(|_| PageError::InvalidAction("invalid payload size"))?,
					};
					add_buffer.extend_from_slice(&header.encode()[..]);
					add_buffer.extend_from_slice(&data[..]);
				},
			}
		}

		// since BTreeMap is sorted by key, all items will be kept in their existing order
		for (_, slice) in parsed.items.iter() {
			updated_page_buffer.extend_from_slice(slice);
		}
		updated_page_buffer.append(&mut add_buffer);

		Ok(ItemizedPage::<T>::from(
			BoundedVec::try_from(updated_page_buffer).map_err(|_| PageError::PageSizeOverflow)?,
		))
	}

	/// Parses all the items inside an ItemPage
	/// This has O(n) complexity when n is the number of all the bytes in that itemized storage
	fn try_parse(&self, include_header: bool) -> Result<ParsedItemPage, PageError> {
		let mut count = 0u16;
		let mut items = BTreeMap::new();
		let mut offset = 0;
		while offset < self.data.len() {
			ensure!(
				offset + ItemHeader::max_encoded_len() <= self.data.len(),
				PageError::ErrorParsing("wrong header size")
			);
			let header = <ItemHeader>::decode(&mut &self.data[offset..])
				.map_err(|_| PageError::ErrorParsing("decoding header"))?;
			let item_total_length = ItemHeader::max_encoded_len() + header.payload_len as usize;
			ensure!(
				offset + item_total_length <= self.data.len(),
				PageError::ErrorParsing("wrong payload size")
			);

			items.insert(
				count,
				match include_header {
					true => &self.data[offset..(offset + item_total_length)],
					false =>
						&self.data
							[(offset + ItemHeader::max_encoded_len())..(offset + item_total_length)],
				},
			);
			offset += item_total_length;
			count = count.checked_add(1).ok_or(PageError::ArithmeticOverflow)?;
		}

		Ok(ParsedItemPage { page_size: self.data.len(), items })
	}
}
