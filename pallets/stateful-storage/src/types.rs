use crate::Config;
use codec::{Decode, Encode, MaxEncodedLen};
use common_primitives::{
	msa::MessageSourceId,
	schema::SchemaId,
	stateful_storage::{PageHash, PageId},
};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_core::bounded::BoundedVec;
use sp_std::{
	cmp::*,
	collections::btree_map::BTreeMap,
	fmt::Debug,
	hash::{Hash, Hasher},
	prelude::*,
};
use twox_hash::XxHash32;

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

/// Defines the actions that can be applied to an Itemized storage
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, PartialOrd, Ord)]
pub enum ItemAction {
	/// Adding new Item into page
	Add { data: Vec<u8> },
	/// removing a new item by index number. Index number starts from 0
	Delete { index: u16 },
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
	ErrorParsing(&'static str),
	InvalidAction(&'static str),
	ArithmeticOverflow,
	PageSizeOverflow,
}

/// Payload containing all necessary fields to verify Itemized related signatures
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, RuntimeDebugNoBound, Clone)]
#[scale_info(skip_type_params(T))]
pub struct ItemizedSignaturePayload<T: Config> {
	#[codec(compact)]
	pub msa_id: MessageSourceId,
	#[codec(compact)]
	pub schema_id: SchemaId,
	pub target_hash: PageHash,
	pub expiration: T::BlockNumber,
	pub actions: BoundedVec<ItemAction, <T as Config>::MaxItemizedActionsCount>,
}

/// Payload containing all necessary fields to verify signatures to upsert a Paginated storage
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, RuntimeDebugNoBound, Clone)]
#[scale_info(skip_type_params(T))]
pub struct PaginatedUpsertSignaturePayload<T: Config> {
	#[codec(compact)]
	pub msa_id: MessageSourceId,
	#[codec(compact)]
	pub schema_id: SchemaId,
	#[codec(compact)]
	pub page_id: PageId,
	pub target_hash: PageHash,
	pub expiration: T::BlockNumber,
	pub payload: BoundedVec<u8, <T as Config>::MaxPaginatedPageSizeBytes>,
}

/// Payload containing all necessary fields to verify signatures to delete a Paginated storage
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, RuntimeDebugNoBound, Clone)]
#[scale_info(skip_type_params(T))]
pub struct PaginatedDeleteSignaturePayload<T: Config> {
	#[codec(compact)]
	pub msa_id: MessageSourceId,
	#[codec(compact)]
	pub schema_id: SchemaId,
	#[codec(compact)]
	pub page_id: PageId,
	pub target_hash: PageHash,
	pub expiration: T::BlockNumber,
}

/// A generic page of data which supports both Itemized and Paginated
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Debug, Default)]
#[scale_info(skip_type_params(PageDataSize))]
#[codec(mel_bound(PageDataSize: MaxEncodedLen))]
pub struct Page<PageDataSize: Get<u32>> {
	pub data: BoundedVec<u8, PageDataSize>,
}

/// An internal struct which contains the parsed items in a page
#[derive(Debug, PartialEq)]
pub struct ParsedItemPage<'a> {
	/// page current size
	pub page_size: usize,
	/// a map of item index to a slice of blob (including header is optional)
	pub items: BTreeMap<u16, &'a [u8]>,
}

impl<PageDataSize: Get<u32>> Page<PageDataSize> {
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}

	pub fn get_hash(&self) -> PageHash {
		if self.is_empty() {
			return PageHash::default()
		}
		let mut hasher = XxHash32::with_seed(0);
		self.hash(&mut hasher);
		hasher.finish() as PageHash
	}
}

impl<PageDataSize: Get<u32>> Hash for Page<PageDataSize> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		state.write(&self.data[..]);
	}
}

impl<PageDataSize: Get<u32>> From<BoundedVec<u8, PageDataSize>> for Page<PageDataSize> {
	fn from(bounded: BoundedVec<u8, PageDataSize>) -> Self {
		Self { data: bounded }
	}
}

impl<PageDataSize: Get<u32>> TryFrom<Vec<u8>> for Page<PageDataSize> {
	type Error = ();

	fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
		let bounded: BoundedVec<u8, PageDataSize> = BoundedVec::try_from(data).map_err(|_| ())?;
		Ok(Page::from(bounded))
	}
}

impl<PageDataSize: Get<u32>> Page<PageDataSize> {
	/// applies all actions to specified page and returns the updated page
	pub fn apply_item_actions(&self, actions: &[ItemAction]) -> Result<Self, PageError> {
		let mut parsed = self.parse_as_itemized(true)?;

		let mut updated_page_buffer = Vec::with_capacity(parsed.page_size);
		let mut add_buffer = Vec::new();

		for action in actions {
			match action {
				ItemAction::Delete { index } => {
					ensure!(
						parsed.items.contains_key(&index),
						PageError::InvalidAction("item index is invalid")
					);
					parsed.items.remove(&index);
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

		// since BTreeMap is sorted by key, all items will be kept in their old order
		for (_, slice) in parsed.items.iter() {
			updated_page_buffer.extend_from_slice(slice);
		}
		updated_page_buffer.append(&mut add_buffer);

		Page::<PageDataSize>::try_from(updated_page_buffer).map_err(|_| PageError::PageSizeOverflow)
	}

	/// Parses all the items inside an ItemPage
	pub fn parse_as_itemized(&self, include_header: bool) -> Result<ParsedItemPage, PageError> {
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
