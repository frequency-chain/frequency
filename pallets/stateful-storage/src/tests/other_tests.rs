use crate::{
	stateful_child_tree::StatefulChildTree,
	tests::{
		mock::*,
		test_common::{constants::*, test_constants::*},
	},
	types::*,
	Config, Error,
};
use codec::{Encode, MaxEncodedLen};
use common_primitives::{
	schema::{ModelType, PayloadLocation, SchemaId},
	stateful_storage::PageNonce,
	utils::wrap_binary_data,
};
use frame_support::{assert_err, assert_ok, traits::Len, BoundedVec};
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use sp_core::{ConstU32, Pair};
use sp_runtime::MultiSignature;
use sp_std::collections::btree_set::BTreeSet;

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
fn is_empty_false_for_non_empty_page() {
	let page: ItemizedPage<Test> =
		create_itemized_page_from::<Test>(None, &[generate_payload_bytes(None)]);

	assert_eq!(page.is_empty(), false);
}

#[test]
fn is_empty_true_for_empty_page() {
	let page: ItemizedPage<Test> = create_itemized_page_from::<Test>(None, &[]);

	assert_eq!(page.is_empty(), true);
}

#[test]
fn child_tree_write_read() {
	new_test_ext().execute_with(|| {
		// arrange
		let pallet_name: &[u8] = b"test-pallet";
		let storage_name_1: &[u8] = b"storage1";
		let msa_id = 1;
		let schema_id: SchemaId = 2;
		let page_id: u8 = 3;
		let keys = &(schema_id, page_id);
		let val = TestStruct {
			model_type: ModelType::AvroBinary,
			payload_location: PayloadLocation::OnChain,
			number: 8276387272,
		};

		// act
		<StatefulChildTree>::write(&msa_id, pallet_name, storage_name_1, keys, &val);

		// assert
		let read =
			<StatefulChildTree>::try_read(&msa_id, pallet_name, storage_name_1, keys).unwrap();
		assert_eq!(Some(val), read);
	});
}

type TestKeyString = BoundedVec<u8, ConstU32<16>>;
type TestKey = (TestKeyString, TestKeyString, u64);

#[test]
fn child_tree_iterator() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let mut arr: Vec<(TestKey, TestKey)> = Vec::new();
		let pallet_name: &[u8] = b"test-pallet";
		let storage_name_1: &[u8] = b"storage1";
		let storage_name_2: &[u8] = b"storage2";
		let prefix_1 = TestKeyString::try_from(b"part_1".to_vec()).unwrap();
		let prefix_2a = TestKeyString::try_from(b"part_2a".to_vec()).unwrap();
		let prefix_2b = TestKeyString::try_from(b"part_2b".to_vec()).unwrap();

		for i in 1u64..=10u64 {
			let k: TestKey = (
				prefix_1.clone(),
				match i % 2 {
					0 => prefix_2a.clone(),
					_ => prefix_2b.clone(),
				},
				i.clone(),
			);
			let s = k.clone();
			arr.push((k.clone(), s.clone()));
			<StatefulChildTree>::write(&msa_id, pallet_name, storage_name_1, &k, s);
		}

		// Try empty prefix
		let all_nodes = <StatefulChildTree>::prefix_iterator::<TestKey, TestKey, _>(
			&msa_id,
			pallet_name,
			storage_name_1,
			&(),
		);
		let r: BTreeSet<u64> = all_nodes.map(|(_k, s)| s.2).collect::<BTreeSet<u64>>();

		// Should return all items
		assert_eq!(
			r,
			arr.iter().map(|(_k, s)| s.2).collect(),
			"iterator with empty prefix should have returned all items with full key"
		);

		// Try 1-level prefix
		let prefix_key = (prefix_1.clone(),);
		let mut nodes = <StatefulChildTree>::prefix_iterator::<TestKey, TestKey, _>(
			&msa_id,
			pallet_name,
			storage_name_1,
			&prefix_key.clone(),
		);
		let r0: BTreeSet<u64> = nodes.by_ref().map(|(_k, v)| v.2).collect();

		assert_eq!(
			r0,
			arr.iter().map(|(_k, s)| s.2).collect(),
			"iterator over topmost key should have returned all items"
		);

		for (k, s) in nodes {
			assert_eq!(k, (s.0, s.1, s.2), "iterated keys should have been decoded properly");
		}

		// Try 2-level prefix
		let prefix_key = (prefix_1.clone(), prefix_2a.clone());
		let nodes2 = <StatefulChildTree>::prefix_iterator::<TestKey, TestKey, _>(
			&msa_id,
			pallet_name,
			storage_name_1,
			&prefix_key,
		);
		let r1: BTreeSet<u64> = nodes2.map(|(_, v)| v.2).collect();

		// Should return only even-numbered items
		assert_eq!(
			r1,
			arr.iter().filter(|(k, _s)| k.2 % 2 == 0).map(|(k, _s)| k.2).collect(),
			"iterator over second-level key should have returned only even-numbered items"
		);

		// Try on another storage
		let nodes3 = <StatefulChildTree>::prefix_iterator::<TestKey, TestKey, _>(
			&msa_id,
			pallet_name,
			storage_name_2,
			&prefix_key,
		);
		let r3: BTreeSet<u64> = nodes3.map(|(_, v)| v.2).collect();

		// Should return empty
		assert_eq!(r3.len(), 0, "iterator over another storage shoudl return empty items");
	});
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

#[test]
fn apply_actions_on_signature_schema_fails() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let msa_id = 1;
		let schema_id = ITEMIZED_SIGNATURE_REQUIRED_SCHEMA;
		let payload = vec![1; 5];
		let actions1 = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				NONEXISTENT_PAGE_HASH,
				BoundedVec::try_from(actions1).unwrap(),
			),
			Error::<Test>::UnsupportedOperationForSchema
		);
	});
}

#[test]
fn insert_page_fails_for_signature_schema() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = test_public(1);
		let msa_id = 1;
		let schema_id = PAGINATED_SIGNED_SCHEMA;
		let page_id = 11;
		let payload = generate_payload_bytes::<PaginatedPageSize>(None);

		// assert
		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
				NONEXISTENT_PAGE_HASH,
				payload.into(),
			),
			Error::<Test>::UnsupportedOperationForSchema
		);
	});
}

#[test]
fn signature_replay_on_existing_page_errors() {
	new_test_ext().execute_with(|| {
		// Setup
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let keys = (schema_id, page_id);
		let page_a: PaginatedPage<Test> = generate_page(Some(1), Some(1));
		let page_b: PaginatedPage<Test> = generate_page(Some(2), Some(2));
		let payload_a_to_b = PaginatedUpsertSignaturePayload {
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
			target_hash: page_a.get_hash(),
			payload: page_b.data.clone(),
		};

		// Set up initial state A
		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
			&page_a,
		);

		// Make sure we successfully apply state transition A -> B
		let encoded_payload = wrap_binary_data(payload_a_to_b.encode());
		let owner_a_to_b_signature: MultiSignature = pair.sign(&encoded_payload).into();
		assert_ok!(StatefulStoragePallet::upsert_page_with_signature(
			RuntimeOrigin::signed(caller_1.clone()),
			delegator_key.into(),
			owner_a_to_b_signature.clone(),
			payload_a_to_b.clone()
		));

		// Read back page state & get hash
		let current_page: PaginatedPage<Test> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				PAGINATED_STORAGE_PREFIX,
				&keys,
			)
			.unwrap()
			.expect("no page read");

		// Make sure we successfully apply state transition B -> A
		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1.clone()),
			msa_id,
			schema_id,
			page_id,
			current_page.get_hash(),
			page_a.data
		));

		// Signature replay A -> B should fail
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_a_to_b_signature,
				payload_a_to_b
			),
			Error::<Test>::StalePageState
		);
	})
}

// NOTE: This is a known issue. When it's fixed, this test will start failing & we can change the test assertion.
#[test]
fn signature_replay_on_deleted_page_check() {
	new_test_ext().execute_with(|| {
		// Setup
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let keys = (schema_id, page_id);
		let page_a: PaginatedPage<Test> = generate_page(Some(1), Some(1));
		let payload_null_to_a = PaginatedUpsertSignaturePayload {
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
			target_hash: NONEXISTENT_PAGE_HASH,
			payload: page_a.data.clone(),
		};

		// Make sure we successfully apply state transition Null -> A
		let encoded_payload = wrap_binary_data(payload_null_to_a.encode());
		let owner_null_to_a_signature: MultiSignature = pair.sign(&encoded_payload).into();
		assert_ok!(StatefulStoragePallet::upsert_page_with_signature(
			RuntimeOrigin::signed(caller_1.clone()),
			delegator_key.into(),
			owner_null_to_a_signature.clone(),
			payload_null_to_a.clone()
		));

		// Read back page state & get hash
		let current_page: PaginatedPage<Test> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				PAGINATED_STORAGE_PREFIX,
				&keys,
			)
			.unwrap()
			.expect("no page read");

		// Make sure we successfully delete the page
		assert_ok!(StatefulStoragePallet::delete_page(
			RuntimeOrigin::signed(caller_1.clone()),
			msa_id,
			schema_id,
			page_id,
			current_page.get_hash(),
		));

		// Signature replay A -> B (change assertion when this vulnerability is fixed)
		assert_ok!(StatefulStoragePallet::upsert_page_with_signature(
			RuntimeOrigin::signed(caller_1.clone()),
			delegator_key.into(),
			owner_null_to_a_signature,
			payload_null_to_a
		));
	})
}
