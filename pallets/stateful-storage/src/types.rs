use super::*;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::*, DefaultNoBound};
use scale_info::TypeInfo;
use sp_core::bounded::BoundedVec;
use sp_std::{cmp::*, collections::btree_map::BTreeMap, fmt::Debug, prelude::*};

/// Defines the actions that can be applied to an Itemized storage
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, PartialOrd, Ord)]
pub enum ItemAction {
	Add { data: Vec<u8> },
	Remove { index: u16 },
}

/// This header is used to specify how long an item is inside the buffer and inserted into buffer
/// before every item
#[derive(Encode, Decode, PartialEq, MaxEncodedLen, Debug)]
pub struct ItemHeader {
	/// The length of this item, not including the size of this header.
	pub payload_len: u16,
}

#[derive(Debug, PartialEq)]
pub enum ItemPageError {
	ErrorParsing(&'static str),
	InvalidAction(&'static str),
	ArithmeticOverflow,
}

/// A page of items
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Debug, DefaultNoBound)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound(T::MaxItemizedPageSizeBytes: MaxEncodedLen))]
pub struct ItemPage<T: Config> {
	/// last updated block number to avoid race conditions
	pub last_update: T::BlockNumber,
	/// number of items in page
	pub item_count: u16,
	/// the items data stored in the page
	pub data: BoundedVec<u8, T::MaxItemizedPageSizeBytes>,
}

/// an internal struct which contains the parsed items in a page
#[derive(Debug, PartialEq)]
pub struct ParsedPage<'a> {
	/// page current size
	pub page_size: usize,
	/// number of items
	pub item_count: u16,
	/// a map of item index to a slice of blob (header included)
	pub items: BTreeMap<u16, &'a [u8]>,
}

impl<T: Config> ItemPage<T> {
	/// creates new itemPage from BoundedVec
	pub fn new(
		current_block: T::BlockNumber,
		item_count: u16,
		data: BoundedVec<u8, T::MaxItemizedPageSizeBytes>,
	) -> Self {
		Self { last_update: current_block, item_count, data }
	}

	/// applies all actions to specified page and returns the updated page
	pub fn apply_actions(
		&self,
		current_block: T::BlockNumber,
		actions: &[ItemAction],
	) -> Result<ItemPage<T>, ItemPageError> {
		ensure!(self.last_update < current_block, ItemPageError::InvalidAction("action against obsolete page"));
		let mut parsed = self.parse()?;

		let mut updated_page_buffer = Vec::with_capacity(parsed.page_size);
		let mut add_buffer = Vec::new();

		for action in actions {
			match action {
				ItemAction::Remove { index } => {
					ensure!(
						parsed.items.contains_key(&index),
						ItemPageError::InvalidAction("item index is invalid")
					);
					parsed.items.remove(&index);
					parsed.item_count = parsed.item_count.saturating_sub(1);
				},
				ItemAction::Add { data } => {
					let header = ItemHeader {
						payload_len: data
							.len()
							.try_into()
							.map_err(|_| ItemPageError::InvalidAction("invalid payload size"))?,
					};
					add_buffer.extend_from_slice(&header.encode()[..]);
					add_buffer.extend_from_slice(&data[..]);
					parsed.item_count = parsed
						.item_count
						.checked_add(1)
						.ok_or(ItemPageError::ArithmeticOverflow)?;
				},
			}
		}

		// since BTreemap is sorted by key, all items will be kept in their old order
		for (_, slice) in parsed.items.iter() {
			updated_page_buffer.extend_from_slice(slice);
		}
		updated_page_buffer.append(&mut add_buffer);

		Ok(ItemPage::<T>::new(
			current_block,
			parsed.item_count,
			updated_page_buffer
				.try_into()
				.map_err(|_| ItemPageError::InvalidAction("page size exceeded"))?,
		))
	}

	/// Parses all the items inside an ItemPage
	pub fn parse(&self) -> Result<ParsedPage, ItemPageError> {
		let mut item_count = 0u16;
		let mut items = BTreeMap::new();
		let mut offset = 0;
		while offset < self.data.len() {
			ensure!(
				offset + ItemHeader::max_encoded_len() <= self.data.len(),
				ItemPageError::ErrorParsing("wrong header size")
			);
			let header = <ItemHeader>::decode(&mut &self.data[offset..])
				.map_err(|_| ItemPageError::ErrorParsing("decoding header"))?;
			let item_total_length = ItemHeader::max_encoded_len() + header.payload_len as usize;
			ensure!(
				offset + item_total_length <= self.data.len(),
				ItemPageError::ErrorParsing("wrong payload size")
			);

			items.insert(item_count, &self.data[offset..(offset + item_total_length)]);
			offset += item_total_length;
			item_count = item_count.checked_add(1).ok_or(ItemPageError::ArithmeticOverflow)?;
		}

		Ok(ParsedPage { page_size: self.data.len(), item_count, items })
	}

	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}
}
