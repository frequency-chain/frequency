use crate::{test_common::test_utility::*, tests::mock::*, types::*, Config};
use common_primitives::stateful_storage::PageNonce;
use frame_support::{assert_ok, traits::Len};
use parity_scale_codec::{Encode, MaxEncodedLen};
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

#[test]
fn parsing_a_well_formed_item_page_should_work() {
	// arrange
	let payloads = vec![
		generate_payload_bytes::<ItemizedBlobSize>(Some(1)),
		generate_payload_bytes::<ItemizedBlobSize>(Some(2)),
	];
	let page = create_itemized_page_from::<Test>(None, &payloads);

	// act
	let parsed = ItemizedOperations::<Test>::try_parse(&page, true);

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
fn parsing_itemized_page_with_item_payload_size_too_big_should_fail() {
	// arrange
	let payload = generate_payload_bytes::<ItemizedPageSize>(Some(1));
	let mut buffer: Vec<u8> = vec![];
	buffer.extend_from_slice(
		&ItemHeader::V2 { schema_id: 0, payload_len: (payload.len() + 1) as u16 }.encode()[..],
	);
	buffer.extend_from_slice(&payload);
	let page: ItemizedPage<Test> = ItemizedPage::<Test> {
		page_version: Default::default(),
		schema_id: None,
		nonce: 0,
		data: buffer.try_into().unwrap(),
	};

	// act
	let parsed = ItemizedOperations::<Test>::try_parse(&page, true);

	// assert
	assert_eq!(parsed, Err(PageError::ErrorParsing("item payload exceeds page data")));
}

#[test]
fn parsing_itemized_page_with_incomplete_trailing_item_header_should_fail() {
	// arrange
	let payload = generate_payload_bytes::<ItemizedPageSize>(Some(1));
	let mut buffer: Vec<u8> = vec![];
	buffer.extend_from_slice(
		&ItemHeader::V2 { schema_id: 0, payload_len: payload.len() as u16 }.encode()[..],
	);
	buffer.extend_from_slice(&payload);
	// Make page buffer extend past the last item
	buffer.extend_from_slice(&[0u8; 1]);
	let page: ItemizedPage<Test> = ItemizedPage::<Test> {
		page_version: Default::default(),
		schema_id: None,
		nonce: 0,
		data: buffer.try_into().unwrap(),
	};

	// act
	let parsed = ItemizedOperations::<Test>::try_parse(&page, true);

	// assert
	assert_eq!(parsed, Err(PageError::ErrorParsing("incomplete item header")));
}

#[test]
fn parsing_itemized_page_with_corrupt_item_header_should_fail() {
	// arrange
	let buffer: Vec<u8> = "this is some garbage data".encode();
	let page: ItemizedPage<Test> = ItemizedPage::<Test> {
		page_version: Default::default(),
		schema_id: None,
		nonce: 0,
		data: buffer.try_into().unwrap(),
	};

	// act
	let parsed = ItemizedOperations::<Test>::try_parse(&page, true);

	// assert
	assert_eq!(parsed, Err(PageError::ErrorParsing("decoding item header")));
}

#[test]
fn applying_delete_action_with_existing_index_should_delete_item() {
	// arrange
	let payloads = vec![
		generate_payload_bytes::<ItemizedBlobSize>(Some(2)),
		generate_payload_bytes::<ItemizedBlobSize>(Some(4)),
	];
	let page = create_itemized_page_from::<Test>(None, payloads.as_slice());
	let expecting_page = create_itemized_page_from::<Test>(None, &payloads[1..]);
	let actions = vec![ItemActionV2::Delete { index: 0 }];

	// act
	let result = ItemizedOperations::<Test>::apply_item_actions(&page, &actions);

	// assert
	assert_ok!(&result);
	let updated = result.unwrap();
	assert_eq!(expecting_page.data, updated.data);
}

#[test]
fn applying_add_action_should_add_item_to_the_end_of_the_page() {
	// arrange
	let payload1 =
		vec![generate_payload_bytes::<<Test as Config>::MaxItemizedBlobSizeBytes>(Some(2))];
	let page = create_itemized_page_from::<Test>(None, payload1.as_slice());
	let payload2 =
		vec![generate_payload_bytes::<<Test as Config>::MaxItemizedBlobSizeBytes>(Some(4))];
	let expecting_page = create_itemized_page_from::<Test>(
		None,
		&vec![payload1[0].clone(), payload2[0].clone()][..],
	);
	let actions = vec![ItemActionV2::Add { schema_id: 0, data: payload2[0].clone().into() }];

	// act
	let result = ItemizedOperations::<Test>::apply_item_actions(&page, &actions[..]);

	// assert
	assert_ok!(&result);
	let updated = result.unwrap();
	assert_eq!(expecting_page.data, updated.data);
}

#[test]
fn applying_delete_action_with_non_existing_index_should_fail() {
	// arrange
	let payloads = vec![
		generate_payload_bytes::<ItemizedBlobSize>(Some(2)),
		generate_payload_bytes::<ItemizedBlobSize>(Some(4)),
	];
	let page = create_itemized_page_from::<Test>(None, payloads.as_slice());
	let actions = vec![ItemActionV2::Delete { index: 2 }];

	// act
	let result = ItemizedOperations::<Test>::apply_item_actions(&page, &actions[..]);

	// assert
	assert_eq!(result.is_err(), true);
}

#[test]
fn applying_add_action_with_full_page_should_fail() {
	// arrange
	let payload: Vec<u8> = vec![1u8; ItemizedBlobSize::get() as usize];
	let size_to_fill = (<Test as Config>::MaxItemizedPageSizeBytes::get() as usize) -
		core::mem::size_of::<PageNonce>();
	let mut item_buffer = Vec::<u8>::with_capacity(size_to_fill);
	while (size_to_fill as i32).saturating_sub(item_buffer.len().try_into().unwrap()) >
		ItemizedBlobSize::get().try_into().unwrap()
	{
		add_itemized_payload_to_buffer::<Test>(&mut item_buffer, &payload.as_slice());
	}
	let page: ItemizedPage<Test> = ItemizedPage::<Test> {
		page_version: Default::default(),
		schema_id: None,
		nonce: 0,
		data: item_buffer.try_into().unwrap(),
	};
	let actions = vec![ItemActionV2::Add { schema_id: 0, data: payload.try_into().unwrap() }];

	// act
	let result = ItemizedOperations::<Test>::apply_item_actions(&page, &actions[..]);

	// assert
	assert_eq!(result, Err(PageError::PageSizeOverflow));
}
