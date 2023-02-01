use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_core::{bounded::BoundedVec, Get};
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
	payload_len: u16,
}

/// This header is used to specify how much data is in a page and is inserted into the buffer
/// before every page in a Paginated model.
#[derive(Clone, Encode, Decode, PartialEq, MaxEncodedLen, Debug)]
pub struct PageHeader {
	/// The length of this item, not including the size of this header.
	payload_len: u16,
}

#[derive(Debug, PartialEq)]
pub enum PageError {
	ErrorParsing(&'static str),
	InvalidAction(&'static str),
	ArithmeticOverflow,
	PageSizeOverflow,
}

// NOTE: Would prefer to simply have an enum:
// ```no_compile
// enum PageType { Itemized, Paginated }
// ```
// However, currently can't use enum variants as const generic parameters.
pub trait PageType {}

#[derive(Debug)]
struct Itemized;

#[derive(Debug)]
struct Paginated;

impl PageType for Itemized {}
impl PageType for Paginated {}

/// A page of data
#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Debug)]
#[scale_info(skip_type_params(PageDataSize))]
#[codec(mel_bound(PageDataSize: MaxEncodedLen))]
pub struct Page<P: PageType, PageDataSize: Get<u32>> {
	data: BoundedVec<u8, PageDataSize>,
	page_type: PhantomData<P>,
}

/// an internal struct which contains the parsed items in a page
#[derive(Debug, PartialEq)]
struct ParsedItemPage<'a> {
	/// page current size
	page_size: usize,
	/// a map of item index to a slice of blob (header included)
	items: BTreeMap<u16, &'a [u8]>,
}

/// an internal struct which contains

/// an internal struct which contains the parsed items in a page
#[derive(Clone, Debug, PartialEq)]
struct ParsedPaginatedPage<'a> {
	/// the page header
	header: PageHeader,
	/// the page data
	data: &'a [u8],
}

impl<PageDataSize: Get<u32>, P: PageType> Page<P, PageDataSize> {
	pub fn is_empty(&self) -> bool {
		self.data.is_empty()
	}
}

impl<PageDataSize: Get<u32>, P: PageType> From<BoundedVec<u8, PageDataSize>>
	for Page<P, PageDataSize>
{
	fn from(bounded: BoundedVec<u8, PageDataSize>) -> Self {
		Self { data: bounded, page_type: PhantomData }
	}
}

impl<PageDataSize: Get<u32>, P: PageType> TryFrom<Vec<u8>> for Page<P, PageDataSize> {
	type Error = ();

	fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
		let bounded: BoundedVec<u8, PageDataSize> = BoundedVec::try_from(data).map_err(|_| ())?;
		Ok(Page::from(bounded))
	}
}

impl<PageDataSize: Get<u32>> TryFrom<ParsedPaginatedPage<'_>> for Page<Paginated, PageDataSize> {
	type Error = ();

	fn try_from(parsed_page: ParsedPaginatedPage<'_>) -> Result<Self, Self::Error> {
		let mut buffer: Vec<u8> = Vec::new();
		buffer.extend_from_slice(&parsed_page.header.encode()[..]);
		buffer.extend_from_slice(parsed_page.data);

		Self::try_from(buffer)
	}
}

impl<PageDataSize: Get<u32>> Page<Itemized, PageDataSize> {
	/// applies all actions to specified page and returns the updated page
	pub fn apply_actions(
		&self,
		actions: &[ItemAction],
	) -> Result<Page<Itemized, PageDataSize>, PageError> {
		let mut parsed = self.parse()?;

		let mut updated_page_buffer = Vec::with_capacity(parsed.page_size);
		let mut add_buffer = Vec::new();

		for action in actions {
			match action {
				ItemAction::Remove { index } => {
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

		// since BTreemap is sorted by key, all items will be kept in their old order
		for (_, slice) in parsed.items.iter() {
			updated_page_buffer.extend_from_slice(slice);
		}
		updated_page_buffer.append(&mut add_buffer);

		Page::<Itemized, PageDataSize>::try_from(updated_page_buffer)
			.map_err(|_| PageError::InvalidAction("page size exceeded"))
	}

	/// Parses all the items inside an ItemPage
	fn parse(&self) -> Result<ParsedItemPage, PageError> {
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

			items.insert(count, &self.data[offset..(offset + item_total_length)]);
			offset += item_total_length;
			count = count.checked_add(1).ok_or(PageError::ArithmeticOverflow)?;
		}

		Ok(ParsedItemPage { page_size: self.data.len(), items })
	}
}

impl<PageDataSize: Get<u32>> Page<Paginated, PageDataSize> {
	/// Parses a Paginated Page into its header and payload
	fn parse(&self) -> Result<ParsedPaginatedPage, PageError> {
		ensure!(
			PageHeader::max_encoded_len() <= self.data.len(),
			PageError::ErrorParsing("page smaller than header")
		);
		let header = <PageHeader>::decode(&mut &self.data[..])
			.map_err(|_| PageError::ErrorParsing("decoding header"))?;
		let page_total_length = PageHeader::max_encoded_len() + header.payload_len as usize;
		ensure!(
			page_total_length == self.data.len(),
			PageError::ErrorParsing("wrong payload size")
		);

		Ok(ParsedPaginatedPage { header, data: &self.data[PageHeader::max_encoded_len()..] })
	}
}

impl<'a> ParsedPaginatedPage<'a> {
	/// constructor
	fn new() -> ParsedPaginatedPage<'a> {
		Self { header: PageHeader { payload_len: 0 }, data: &[] }
	}

	/// Stores a new data payload and updates the header information
	fn set_data(&mut self, new_data: &'a [u8]) -> &Self {
		self.data = new_data;
		self.header.payload_len = self.data.len().try_into().unwrap();
		self
	}
}

impl<'a> From<(PageHeader, &'a [u8])> for ParsedPaginatedPage<'a> {
	fn from(args: (PageHeader, &'a [u8])) -> Self {
		Self { header: args.0, data: args.1 }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::assert_ok;
	use pretty_assertions::assert_eq;

	type TestPageSize = ConstU32<2048>;

	fn generate_payload_bytes(id: u8) -> Vec<u8> {
		format!("{{'type':{id}, 'description':'another test description {id}'}}")
			.as_bytes()
			.to_vec()
	}

	fn create_itemized_page_from(payloads: &[Vec<u8>]) -> Page<Itemized, TestPageSize> {
		let mut buffer: Vec<u8> = vec![];
		for p in payloads {
			buffer.extend_from_slice(&ItemHeader { payload_len: p.len() as u16 }.encode()[..]);
			buffer.extend_from_slice(p);
		}
		Page::<Itemized, TestPageSize>::try_from(buffer).unwrap()
	}

	fn create_paginated_page_from(
		header: &PageHeader,
		payload: &Vec<u8>,
	) -> Page<Paginated, TestPageSize> {
		let mut buffer: Vec<u8> = vec![];
		buffer.extend_from_slice(&header.encode()[..]);
		buffer.extend_from_slice(payload);
		Page::<Paginated, TestPageSize>::try_from(buffer).unwrap()
	}

	fn generate_page_header(data: &[u8]) -> PageHeader {
		PageHeader { payload_len: data.len() as u16 }
	}

	#[test]
	fn parsing_a_well_formed_item_page_should_work() {
		// arrange
		let payloads = vec![generate_payload_bytes(1), generate_payload_bytes(2)];
		let page = create_itemized_page_from(payloads.as_slice());

		// act
		let parsed = page.parse();

		// assert
		assert_ok!(&parsed);
		assert_eq!(
			parsed.as_ref().unwrap().page_size,
			payloads.len() * ItemHeader::max_encoded_len() +
				payloads.iter().map(|p| p.len()).sum::<usize>()
		);

		let items = parsed.unwrap().items;
		for index in 0..payloads.len() {
			assert_eq!(
				items.get(&(index as u16)).unwrap()[ItemHeader::max_encoded_len()..],
				payloads[index][..]
			);
		}
	}

	#[test]
	fn parsing_item_with_wrong_payload_size_should_return_parsing_error() {
		// arrange
		let payload = generate_payload_bytes(1);
		let mut buffer: Vec<u8> = vec![];
		buffer.extend_from_slice(
			&ItemHeader { payload_len: (payload.len() + 1) as u16 }.encode()[..],
		);
		buffer.extend_from_slice(&payload);
		let page: Page<Itemized, TestPageSize> = Page::try_from(buffer).unwrap();

		// act
		let parsed = page.parse();

		// assert
		assert_eq!(parsed, Err(PageError::ErrorParsing("wrong payload size")));
	}

	#[test]
	fn parsing_wrong_item_header_size_page_should_return_parsing_error() {
		// arrange
		let payload = generate_payload_bytes(2);
		let mut buffer: Vec<u8> = vec![];
		buffer.extend_from_slice(
			&ItemHeader { payload_len: (payload.len() - 1) as u16 }.encode()[..],
		);
		buffer.extend_from_slice(&payload);
		let page = Page::<Itemized, TestPageSize>::try_from(buffer).unwrap();

		// act
		let parsed = page.parse();

		// assert
		assert_eq!(parsed, Err(PageError::ErrorParsing("wrong header size")));
	}

	#[test]
	fn applying_remove_action_with_existing_index_should_remove_item() {
		// arrange
		let payloads = vec![generate_payload_bytes(2), generate_payload_bytes(4)];
		let page = create_itemized_page_from(payloads.as_slice());
		let expecting_page = create_itemized_page_from(&payloads[1..]);
		let actions = vec![ItemAction::Remove { index: 0 }];

		// act
		let result = page.apply_actions(&actions[..]);

		// assert
		assert_ok!(&result);
		let updated = result.unwrap();
		assert_eq!(expecting_page.data, updated.data);
	}

	#[test]
	fn applying_add_action_should_add_item_to_the_end_of_the_page() {
		// arrange
		let payload1 = vec![generate_payload_bytes(2)];
		let page = create_itemized_page_from(payload1.as_slice());
		let payload2 = vec![generate_payload_bytes(4)];
		let expecting_page =
			create_itemized_page_from(&vec![payload1[0].clone(), payload2[0].clone()][..]);
		let actions = vec![ItemAction::Add { data: payload2[0].clone() }];

		// act
		let result = page.apply_actions(&actions[..]);

		// assert
		assert_ok!(&result);
		let updated = result.unwrap();
		assert_eq!(expecting_page.data, updated.data);
	}

	#[test]
	fn applying_remove_action_with_non_existing_index_should_fail() {
		// arrange
		let payloads = vec![generate_payload_bytes(2), generate_payload_bytes(4)];
		let page = create_itemized_page_from(payloads.as_slice());
		let actions = vec![ItemAction::Remove { index: 2 }];

		// act
		let result = page.apply_actions(&actions[..]);

		// assert
		assert_eq!(result.is_err(), true);
	}

	#[test]
	fn applying_add_action_with_full_page_should_fail() {
		// arrange
		let mut arr: Vec<Vec<u8>> = vec![];
		let payload = generate_payload_bytes(2);
		while (arr.len() + 1) * (&payload.len() + ItemHeader::max_encoded_len()) <
			<TestPageSize as sp_core::Get<u32>>::get() as usize
		{
			arr.push(payload.clone());
		}
		let page = create_itemized_page_from(arr.as_slice());
		let actions = vec![ItemAction::Add { data: payload.clone() }];

		// act
		let result = page.apply_actions(&actions[..]);

		// assert
		assert_eq!(result.is_err(), true);
	}

	#[test]
	fn parse_too_small_paginated_page_should_error() {
		let data: Vec<u8> = vec![0];
		let page: Page<Paginated, TestPageSize> = Page::try_from(data).unwrap();

		let result = page.parse();

		assert_eq!(result, Err(PageError::ErrorParsing("page smaller than header")));
	}

	#[test]
	fn parse_payload_size_mismatch_should_error() {
		const SIZE: u16 = 5;
		// Try one more and one less than header-indicated size
		for offset in [1i16, -1i16] {
			let page = create_paginated_page_from(
				&PageHeader { payload_len: SIZE },
				&vec![0xff; (SIZE as i16 + offset).try_into().unwrap()].to_vec(),
			);

			let result = page.parse();

			assert_eq!(result, Err(PageError::ErrorParsing("wrong payload size")));
		}
	}

	#[test]
	fn parse_payload_too_large_should_error() {
		let page =
			create_paginated_page_from(&PageHeader { payload_len: 5 }, &vec![0xff; 6].to_vec());

		let result = page.parse();

		assert_eq!(result, Err(PageError::ErrorParsing("wrong payload size")));
	}

	#[test]
	fn good_page_should_parse_ok() {
		let payload = generate_payload_bytes(1);
		let header = PageHeader { payload_len: payload.len().try_into().unwrap() };
		let page = create_paginated_page_from(&header, &payload);

		assert_ok!(page.parse());
	}

	#[test]
	fn new_parsed_paginated_page_should_be_empty() {
		let page = ParsedPaginatedPage::new();

		assert_eq!(page.header, PageHeader { payload_len: 0 });
		assert_eq!(&page.data[..], Vec::<u8>::new().as_slice());
	}

	#[test]
	fn raw_to_parsed_to_raw_should_be_equal() {
		let payload = generate_payload_bytes(1);
		let header = generate_page_header(&payload);
		let original_raw = create_paginated_page_from(&header, &payload);
		let parsed = original_raw.parse().unwrap();
		let new_raw: Page<Paginated, TestPageSize> = Page::try_from(parsed).unwrap();

		assert_eq!(&original_raw.data[..], &new_raw.data[..]);
	}

	#[test]
	fn parsed_to_raw_to_parsed_should_be_equal() {
		let mut orig_parsed = ParsedPaginatedPage::new();
		let payload = generate_payload_bytes(1);
		orig_parsed.set_data(&payload);
		let raw: Page<Paginated, TestPageSize> = Page::try_from(orig_parsed.clone()).unwrap();
		let new_parsed = raw.parse().unwrap();

		assert_eq!(orig_parsed, new_parsed);
	}

	#[test]
	fn is_empty_false_for_non_empty_page() {
		let payload = generate_payload_bytes(1);
		let header = generate_page_header(&payload);
		let page = create_paginated_page_from(&header, &payload);

		assert_eq!(page.is_empty(), false);
	}

	#[test]
	fn is_empty_true_for_empty_page() {
		let payload = Vec::<u8>::new();
		let page: Page<Paginated, TestPageSize> = Page::try_from(payload).unwrap();

		assert_eq!(page.is_empty(), true);
	}
}
