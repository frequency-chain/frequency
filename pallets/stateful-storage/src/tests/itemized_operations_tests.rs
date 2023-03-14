use crate::{
	stateful_child_tree::StatefulChildTree,
	test_common::{constants::*, test_constants::*},
	tests::mock::*,
	types::*,
	Config, Error,
};
use codec::{Encode, MaxEncodedLen};
use common_primitives::stateful_storage::PageNonce;
use frame_support::{assert_err, assert_ok, traits::Len, BoundedVec};
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
fn parsing_item_with_wrong_payload_size_should_return_parsing_error() {
	// arrange
	let payload = generate_payload_bytes::<ItemizedPageSize>(Some(1));
	let mut buffer: Vec<u8> = vec![];
	buffer.extend_from_slice(&ItemHeader { payload_len: (payload.len() + 1) as u16 }.encode()[..]);
	buffer.extend_from_slice(&payload);
	let page: ItemizedPage<Test> = ItemizedPage::<Test>::try_from(buffer).unwrap();

	// act
	let parsed = ItemizedOperations::<Test>::try_parse(&page, true);

	// assert
	assert_eq!(parsed, Err(PageError::ErrorParsing("wrong payload size")));
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
	let actions = vec![ItemAction::Delete { index: 0 }];

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
	let actions = vec![ItemAction::Add { data: payload2[0].clone().into() }];

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
	let actions = vec![ItemAction::Delete { index: 2 }];

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
		sp_std::mem::size_of::<PageNonce>();
	let mut item_buffer = Vec::<u8>::with_capacity(size_to_fill);
	while (size_to_fill as i32).saturating_sub(item_buffer.len().try_into().unwrap()) >
		ItemizedBlobSize::get().try_into().unwrap()
	{
		add_itemized_payload_to_buffer::<Test>(&mut item_buffer, &payload.as_slice());
	}
	let page: ItemizedPage<Test> =
		ItemizedPage::<Test> { nonce: 0, data: item_buffer.try_into().unwrap() };
	let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];

	// act
	let result = ItemizedOperations::<Test>::apply_item_actions(&page, &actions[..]);

	// assert
	assert_eq!(result, Err(PageError::PageSizeOverflow));
}

#[test]
fn apply_delete_item_on_append_only_fails() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let msa_id = 1;
		let schema_id = ITEMIZED_APPEND_ONLY_SCHEMA;
		let payload = vec![1; 5];
		let actions1 = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let actions2 = vec![ItemAction::Delete { index: 0 }];
		let keys = (schema_id,);
		assert_ok!(StatefulStoragePallet::apply_item_actions(
			RuntimeOrigin::signed(caller_1.clone()),
			msa_id,
			schema_id,
			NONEXISTENT_PAGE_HASH,
			BoundedVec::try_from(actions1).unwrap(),
		));

		let items1: Option<ItemizedPage<Test>> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&keys,
			)
			.unwrap();
		assert!(items1.is_some());
		let content_hash = items1.unwrap().get_hash();

		// assert
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				content_hash,
				BoundedVec::try_from(actions2).unwrap(),
			),
			Error::<Test>::UnsupportedOperationForSchema
		);
	});
}
