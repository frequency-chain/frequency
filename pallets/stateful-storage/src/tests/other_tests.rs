use crate::{
	pallet,
	stateful_child_tree::StatefulChildTree,
	tests::{mock::*, test_common::constants::*},
	types::*,
	Config, Error, Event as StatefulEvent,
};
use codec::{Decode, Encode, MaxEncodedLen};
use common_primitives::{
	schema::{ModelType, PayloadLocation, SchemaId},
	stateful_storage::{PageHash, PageId, PageNonce},
	utils::wrap_binary_data,
};
use frame_support::{assert_err, assert_ok, assert_storage_noop, traits::Len, BoundedVec};
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use scale_info::TypeInfo;
use sp_core::{sr25519, ConstU32, Get, Pair};
use sp_runtime::MultiSignature;
use sp_std::{collections::btree_set::BTreeSet, hash::Hasher};
use twox_hash::XxHash64;

type ItemizedPageSize = <Test as Config>::MaxItemizedPageSizeBytes;
type PaginatedPageSize = <Test as Config>::MaxPaginatedPageSizeBytes;
type ItemizedBlobSize = <Test as Config>::MaxItemizedBlobSizeBytes;

const NONEXISTENT_PAGE_HASH: u32 = 0;

fn generate_payload_bytes<T: Get<u32>>(id: Option<u8>) -> BoundedVec<u8, T> {
	let value = id.unwrap_or(1);
	format!("{{'type':{value}, 'description':'another test description {value}'}}")
		.as_bytes()
		.to_vec()
		.try_into()
		.unwrap()
}

fn generate_page<T: Get<u32>>(in_nonce: Option<PageNonce>, id: Option<u8>) -> Page<T> {
	let nonce = in_nonce.unwrap_or_default();
	Page::<T> { nonce, data: generate_payload_bytes(id) }
}

fn add_itemized_payload_to_buffer<T: Config>(buffer: &mut Vec<u8>, payload: &[u8]) {
	buffer.extend_from_slice(&ItemHeader { payload_len: payload.len() as u16 }.encode()[..]);
	buffer.extend_from_slice(payload);
}

fn create_itemized_page_from<T: pallet::Config>(
	nonce_in: Option<PageNonce>,
	payloads: &[BoundedVec<u8, ItemizedBlobSize>],
) -> ItemizedPage<T> {
	let nonce = nonce_in.unwrap_or_default();
	let mut buffer: Vec<u8> = vec![];
	for p in payloads {
		add_itemized_payload_to_buffer::<T>(&mut buffer, p.as_slice());
	}
	let data = BoundedVec::<u8, T::MaxItemizedPageSizeBytes>::try_from(buffer).unwrap();
	ItemizedPage::<T> { nonce, data }
}

fn hash_payload(data: &[u8]) -> PageHash {
	let mut hasher = XxHash64::with_seed(0);
	sp_std::hash::Hash::hash(&data, &mut hasher);
	let value_bytes: [u8; 4] =
		hasher.finish().to_be_bytes()[..4].try_into().expect("incorrect hash size");
	PageHash::from_be_bytes(value_bytes)
}

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, MaxEncodedLen)]
/// A structure defining a Schema
struct TestStruct {
	pub model_type: ModelType,
	pub payload_location: PayloadLocation,
	pub number: u64,
}

#[test]
fn upsert_page_id_out_of_bounds_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = 1;
		let page_id = <Test as Config>::MaxPaginatedPageId::get() + 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id.into(),
				schema_id,
				page_id,
				hash_payload(&payload),
				payload
			),
			Error::<Test>::PageIdExceedsMaxAllowed
		)
	})
}

#[test]
fn upsert_page_with_invalid_msa_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = INVALID_MSA_ID;
		let caller_1 = test_public(msa_id); // hard-coded in mocks to return None for MSA
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id.into(),
				schema_id,
				page_id,
				hash_payload(&payload),
				payload
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	})
}

#[test]
fn upsert_page_with_invalid_schema_id_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = INVALID_SCHEMA_ID;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id.into(),
				schema_id,
				page_id,
				hash_payload(&payload),
				payload
			),
			Error::<Test>::InvalidSchemaId
		)
	})
}

#[test]
fn upsert_page_with_invalid_schema_payload_location_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = ITEMIZED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id.into(),
				schema_id,
				page_id,
				hash_payload(&payload),
				payload
			),
			Error::<Test>::SchemaPayloadLocationMismatch
		)
	})
}

#[test]
fn upsert_page_with_no_delegation_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = test_public(1);
		let msa_id = 2;
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
				PageHash::default(),
				payload
			),
			Error::<Test>::UnauthorizedDelegate
		)
	})
}

#[test]
fn upsert_new_page_with_bad_state_hash_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id.into(),
				schema_id,
				page_id,
				hash_payload(&payload),
				payload
			),
			Error::<Test>::StalePageState
		)
	})
}

#[test]
fn upsert_existing_page_with_bad_state_hash_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let page = generate_page::<PaginatedPageSize>(None, Some(100));

		let key = (schema_id, page_id);
		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key,
			&page,
		);

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
				0u32,
				page.data,
			),
			Error::<Test>::StalePageState
		)
	})
}

#[test]
fn upsert_new_page_succeeds() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let page: PaginatedPage<Test> = payload.clone().into();

		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id,
			0u32,
			payload.into()
		));

		let keys = (schema_id, page_id);
		let new_page: PaginatedPage<Test> = <StatefulChildTree>::try_read(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
		)
		.unwrap()
		.expect("new page is empty");
		assert_eq!(page.data, new_page.data, "new page contents incorrect");
		assert_eq!(new_page.nonce, 1, "new page nonce incorrect");
	})
}

#[test]
fn upsert_existing_page_modifies_page() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id: SchemaId = PAGINATED_SCHEMA;
		let page_id: PageId = 1;
		let old_nonce = 3;
		let old_content = generate_payload_bytes(Some(200));
		let new_content = generate_payload_bytes::<PaginatedPageSize>(Some(201));
		let mut old_page: PaginatedPage<Test> = old_content.clone().into();
		old_page.nonce = old_nonce;

		let keys = (schema_id, page_id);
		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
			old_page,
		);
		let old_page: PaginatedPage<Test> = <StatefulChildTree>::try_read(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
		)
		.unwrap()
		.unwrap();

		assert_eq!(old_content, old_page.data);
		assert_eq!(old_nonce, old_page.nonce);
		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id,
			old_page.get_hash(),
			new_content.clone().into()
		));

		let new_page: PaginatedPage<Test> = <StatefulChildTree>::try_read(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
		)
		.unwrap()
		.unwrap();
		assert_eq!(new_content, new_page.data);
		assert_eq!(old_nonce + 1, new_page.nonce);
	})
}

#[test]
fn delete_page_id_out_of_bounds_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = PAGINATED_SCHEMA;
		let page_id = <Test as Config>::MaxPaginatedPageId::get() + 1;

		assert_err!(
			StatefulStoragePallet::delete_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
				NONEXISTENT_PAGE_HASH,
			),
			Error::<Test>::PageIdExceedsMaxAllowed
		);
	})
}

#[test]
fn delete_page_with_invalid_msa_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = test_public(INVALID_MSA_ID); // hard-coded in mocks to return None for MSA
		let msa_id = 1;
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;

		assert_err!(
			StatefulStoragePallet::delete_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
				NONEXISTENT_PAGE_HASH,
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	})
}

#[test]
fn delete_page_with_invalid_schema_id_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(1);
		let schema_id = INVALID_SCHEMA_ID;
		let page_id = 1;

		assert_err!(
			StatefulStoragePallet::delete_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
				NONEXISTENT_PAGE_HASH,
			),
			Error::<Test>::InvalidSchemaId
		)
	})
}

#[test]
fn delete_page_with_invalid_schema_payload_location_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = ITEMIZED_SCHEMA;
		let page_id = 1;

		assert_err!(
			StatefulStoragePallet::delete_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
				NONEXISTENT_PAGE_HASH,
			),
			Error::<Test>::SchemaPayloadLocationMismatch
		)
	})
}

#[test]
fn delete_page_with_no_delegation_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(2);
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;

		assert_err!(
			StatefulStoragePallet::delete_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
				NONEXISTENT_PAGE_HASH,
			),
			Error::<Test>::UnauthorizedDelegate
		)
	})
}

#[test]
fn delete_nonexistent_page_succeeds_noop() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 10;

		assert_ok!(StatefulStoragePallet::delete_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id,
			NONEXISTENT_PAGE_HASH,
		));
	})
}

#[test]
fn delete_existing_page_with_bad_hash_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 11;
		let page: PaginatedPage<Test> = generate_page(None, None);

		let keys = (schema_id, page_id);
		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
			&page,
		);

		assert_err!(
			StatefulStoragePallet::delete_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
				0u32,
			),
			Error::<Test>::StalePageState
		);
	})
}

#[test]
fn delete_existing_page_succeeds() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 11;
		let page: PaginatedPage<Test> = generate_page(None, None);
		let page_hash = page.get_hash();
		let keys = (schema_id, page_id);

		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
			&page,
		);

		assert_ok!(StatefulStoragePallet::delete_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id,
			page_hash
		));

		let page: Option<PaginatedPage<Test>> = <StatefulChildTree>::try_read(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
		)
		.unwrap();
		assert_eq!(page, None);
	})
}

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
fn apply_item_actions_with_invalid_msa_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(INVALID_MSA_ID); // hard-coded in mocks to return None for MSA
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				NONEXISTENT_PAGE_HASH,
				BoundedVec::try_from(actions).unwrap(),
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	});
}

#[test]
fn apply_item_actions_with_invalid_schema_id_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = INVALID_SCHEMA_ID;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				NONEXISTENT_PAGE_HASH,
				BoundedVec::try_from(actions).unwrap(),
			),
			Error::<Test>::InvalidSchemaId
		)
	});
}

#[test]
fn apply_item_actions_with_invalid_schema_location_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = PAGINATED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				NONEXISTENT_PAGE_HASH,
				BoundedVec::try_from(actions).unwrap(),
			),
			Error::<Test>::SchemaPayloadLocationMismatch
		)
	});
}

#[test]
fn apply_item_actions_with_no_delegation_and_different_caller_from_owner_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(2);
		let schema_id = UNDELEGATED_ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				NONEXISTENT_PAGE_HASH,
				BoundedVec::try_from(actions).unwrap(),
			),
			Error::<Test>::UnauthorizedDelegate
		)
	});
}

#[test]
fn apply_item_actions_with_corrupted_state_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let page: ItemizedPage<Test> = generate_page(None, None);
		let actions1 = vec![ItemAction::Add { data: payload.clone().try_into().unwrap() }];
		let key = (schema_id,);
		StatefulChildTree::<<Test as Config>::KeyHasher>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&key,
			&page,
		);
		let previous_page: ItemizedPage<Test> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&key,
			)
			.unwrap()
			.unwrap();
		let previous_hash = previous_page.get_hash();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				previous_hash,
				BoundedVec::try_from(actions1).unwrap(),
			),
			Error::<Test>::CorruptedState
		)
	});
}

#[test]
fn apply_item_actions_initial_state_with_stale_hash_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions1 = vec![ItemAction::Add { data: payload.try_into().unwrap() }];

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				1u32, // any non-zero value
				BoundedVec::try_from(actions1).unwrap(),
			),
			Error::<Test>::StalePageState
		)
	});
}

#[test]
fn apply_item_actions_existing_page_with_stale_hash_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions1 = vec![ItemAction::Add { data: payload.clone().try_into().unwrap() }];

		let page = ItemizedPage::<Test>::default();
		let page_hash = page.get_hash();
		let mut new_page =
			ItemizedOperations::<Test>::apply_item_actions(&page, &actions1).unwrap();
		new_page.nonce = 1;
		let key = (schema_id,);
		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&key,
			&new_page,
		);

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_hash,
				BoundedVec::try_from(actions1).unwrap(),
			),
			Error::<Test>::StalePageState
		)
	});
}

#[test]
fn apply_item_actions_initial_state_with_valid_input_should_update_storage() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let prev_content_hash: PageHash = 0;
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];

		// act
		assert_ok!(StatefulStoragePallet::apply_item_actions(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			prev_content_hash,
			BoundedVec::try_from(actions).unwrap(),
		));

		// assert
		let updated_page: Option<ItemizedPage<Test>> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&(schema_id,),
			)
			.unwrap();
		assert!(updated_page.is_some());
		let updated_page = updated_page.unwrap();
		let curr_content_hash = updated_page.get_hash();
		assert_eq!(updated_page.nonce, PageNonce::default() + 1);
		System::assert_last_event(
			StatefulEvent::ItemizedPageUpdated {
				msa_id,
				schema_id,
				prev_content_hash,
				curr_content_hash,
			}
			.into(),
		);
	});
}

#[test]
fn apply_item_actions_existing_page_with_valid_input_should_update_storage() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let nonce = 10;
		let actions = vec![ItemAction::Add { data: payload.clone().try_into().unwrap() }];
		let page = create_itemized_page_from::<Test>(Some(nonce), &[payload.try_into().unwrap()]);
		let prev_content_hash = page.get_hash();
		let key = (schema_id,);

		// act
		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&key,
			&page,
		);
		assert_ok!(StatefulStoragePallet::apply_item_actions(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			prev_content_hash,
			BoundedVec::try_from(actions).unwrap(),
		));

		// assert
		let updated_page: Option<ItemizedPage<Test>> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&(schema_id,),
			)
			.unwrap();
		assert!(updated_page.is_some());
		let updated_page = updated_page.unwrap();
		let curr_content_hash = updated_page.get_hash();
		assert_eq!(nonce + 1, updated_page.nonce);
		System::assert_last_event(
			StatefulEvent::ItemizedPageUpdated {
				msa_id,
				schema_id,
				prev_content_hash,
				curr_content_hash,
			}
			.into(),
		);
	});
}

#[test]
fn apply_item_actions_with_valid_input_and_empty_items_should_remove_storage() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = ITEMIZED_SCHEMA;
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

		// act
		assert_ok!(StatefulStoragePallet::apply_item_actions(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			content_hash,
			BoundedVec::try_from(actions2).unwrap(),
		));

		// assert
		let items2: Option<ItemizedPage<Test>> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&(schema_id,),
			)
			.unwrap();
		assert!(items2.is_none());
		System::assert_last_event(
			StatefulEvent::ItemizedPageDeleted {
				msa_id,
				schema_id,
				prev_content_hash: content_hash,
			}
			.into(),
		);
	});
}

#[test]
fn apply_item_actions_with_signature_having_wrong_signature_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let (signature_key, _) = sr25519::Pair::generate();
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let payload = ItemizedSignaturePayload {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = signature_key.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidSignature
		)
	});
}

#[test]
fn apply_item_actions_with_signature_having_too_far_expiration_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let payload = ItemizedSignaturePayload {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: PageHash::default(),
			msa_id,
			expiration: (<Test as Config>::MortalityWindowSize::get() + 1).into(),
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::ProofNotYetValid
		)
	});
}

#[test]
fn apply_item_actions_with_signature_having_expired_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let block_number = 10;
		let payload = ItemizedSignaturePayload {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: PageHash::default(),
			msa_id,
			expiration: block_number,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		System::set_block_number(block_number);
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::ProofHasExpired
		)
	});
}

#[test]
fn apply_item_actions_with_signature_having_correct_input_should_work() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let prev_content_hash = PageHash::default();
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let payload = ItemizedSignaturePayload {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: prev_content_hash,
			msa_id,
			expiration: 10,
			schema_id,
		};

		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_ok!(StatefulStoragePallet::apply_item_actions_with_signature(
			RuntimeOrigin::signed(caller_1),
			delegator_key.into(),
			owner_signature,
			payload
		));

		// assert
		let updated_page: Option<ItemizedPage<Test>> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&(schema_id,),
			)
			.unwrap();
		assert!(updated_page.is_some());
		let curr_content_hash = updated_page.unwrap().get_hash();
		System::assert_last_event(
			StatefulEvent::ItemizedPageUpdated {
				msa_id,
				schema_id,
				prev_content_hash,
				curr_content_hash,
			}
			.into(),
		);
	});
}

#[test]
fn apply_item_actions_with_signature_having_non_existing_msa_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let pair = get_invalid_msa_signature_account(); // hardcoded key that returns None Msa
		let delegator_key = pair.public();
		let msa_id = 1;
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let payload = ItemizedSignaturePayload {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	});
}

#[test]
fn apply_item_actions_with_signature_having_wrong_msa_in_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_correct_msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let wrong_msa_id = 3;
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let payload = ItemizedSignaturePayload {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: PageHash::default(),
			msa_id: wrong_msa_id,
			expiration: 10,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	});
}

#[test]
fn apply_item_actions_with_signature_having_invalid_schema_id_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = INVALID_SCHEMA_ID;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let payload = ItemizedSignaturePayload {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidSchemaId
		)
	});
}

#[test]
fn apply_item_actions_with_signature_having_invalid_schema_location_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let payload = ItemizedSignaturePayload {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::SchemaPayloadLocationMismatch
		)
	});
}

#[test]
fn apply_item_actions_with_signature_having_page_with_stale_hash_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.clone().try_into().unwrap() }];
		let page = ItemizedPage::<Test>::default();
		let page_hash = page.get_hash();
		let page = ItemizedOperations::<Test>::apply_item_actions(&page, &actions).unwrap();
		let key = (schema_id,);
		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&key,
			page,
		);
		let payload = ItemizedSignaturePayload {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: page_hash,
			msa_id,
			expiration: 10,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::StalePageState
		)
	});
}

#[test]
fn apply_item_actions_with_signature_having_valid_input_and_empty_items_should_remove_storage() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
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

		let payload = ItemizedSignaturePayload {
			actions: BoundedVec::try_from(actions2).unwrap(),
			target_hash: content_hash,
			msa_id,
			expiration: 10,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_ok!(StatefulStoragePallet::apply_item_actions_with_signature(
			RuntimeOrigin::signed(caller_1),
			delegator_key.into(),
			owner_signature,
			payload
		));

		// assert
		let items2: Option<ItemizedPage<Test>> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&(schema_id,),
			)
			.unwrap();
		assert!(items2.is_none());
		System::assert_last_event(
			StatefulEvent::ItemizedPageDeleted {
				msa_id,
				schema_id,
				prev_content_hash: content_hash,
			}
			.into(),
		);
	});
}

#[test]
fn apply_item_actions_with_signature_having_corrupted_state_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let page: ItemizedPage<Test> = generate_page(None, None);
		let actions = vec![ItemAction::Add { data: payload.clone().try_into().unwrap() }];
		let key = (schema_id,);
		StatefulChildTree::<<Test as Config>::KeyHasher>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&key,
			&page,
		);
		let previous_page: ItemizedPage<Test> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&key,
			)
			.unwrap()
			.unwrap();
		let previous_hash = previous_page.get_hash();

		let payload = ItemizedSignaturePayload {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: previous_hash,
			msa_id,
			expiration: 10,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::CorruptedState
		)
	});
}

#[test]
fn upsert_page_with_signature_having_page_id_out_of_bounds_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = (<Test as Config>::MaxPaginatedPageId::get() + 1).into();
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::PageIdExceedsMaxAllowed
		)
	})
}

#[test]
fn upsert_page_with_signature_having_expired_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let block_number = 10;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: block_number,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		System::set_block_number(block_number);
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::ProofHasExpired
		)
	})
}

#[test]
fn upsert_page_with_signature_having_out_of_window_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: (<Test as Config>::MortalityWindowSize::get() + 1).into(),
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::ProofNotYetValid
		)
	})
}

#[test]
fn upsert_page_with_signature_having_wrong_signature_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let (signature_key, _) = sr25519::Pair::generate();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = signature_key.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidSignature
		)
	})
}

#[test]
fn upsert_page_with_signature_having_non_existing_msa_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let pair = get_invalid_msa_signature_account(); // hardcoded key that returns None Msa
		let delegator_key = pair.public();
		let msa_id = 1;
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	})
}

#[test]
fn upsert_page_with_signature_having_wrong_msa_in_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_correct_msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let wrong_msa_id = 3;
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id: wrong_msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	})
}

#[test]
fn upsert_page_with_signature_having_invalid_schema_id_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = INVALID_SCHEMA_ID;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidSchemaId
		)
	})
}

#[test]
fn upsert_page_with_signature_having_invalid_schema_location_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::SchemaPayloadLocationMismatch
		)
	})
}

#[test]
fn upsert_page_with_signature_having_invalid_hash_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default() + 1, // any non default hash value
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::StalePageState
		)
	})
}

#[test]
fn upsert_page_with_signature_having_valid_inputs_should_work() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let page: PaginatedPage<Test> = payload.clone().into();
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_ok!(StatefulStoragePallet::upsert_page_with_signature(
			RuntimeOrigin::signed(caller_1),
			delegator_key.into(),
			owner_signature,
			payload
		));

		// assert
		let keys = (schema_id, page_id);
		let new_page: PaginatedPage<Test> = <StatefulChildTree>::try_read(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
		)
		.expect("new page could not be read")
		.expect("new page is empty");
		assert_eq!(page.data, new_page.data, "new page contents incorrect");
		assert_eq!(new_page.nonce, 1, "new page nonce incorrect");
		System::assert_last_event(
			StatefulEvent::PaginatedPageUpdated {
				msa_id,
				schema_id,
				page_id,
				prev_content_hash: PageHash::default(),
				curr_content_hash: new_page.get_hash(),
			}
			.into(),
		);
	})
}

#[test]
fn delete_page_with_signature_having_page_id_out_of_bounds_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = (<Test as Config>::MaxPaginatedPageId::get() + 1).into();
		let payload = PaginatedDeleteSignaturePayload {
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::delete_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::PageIdExceedsMaxAllowed
		)
	})
}

#[test]
fn delete_page_with_signature_having_expired_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let block_number = 10;
		let payload = PaginatedDeleteSignaturePayload {
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		System::set_block_number(block_number);
		assert_err!(
			StatefulStoragePallet::delete_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::ProofHasExpired
		)
	})
}

#[test]
fn delete_page_with_signature_having_out_of_window_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = PaginatedDeleteSignaturePayload {
			target_hash: PageHash::default(),
			msa_id,
			expiration: (<Test as Config>::MortalityWindowSize::get() + 1).into(),
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::delete_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::ProofNotYetValid
		)
	})
}

#[test]
fn delete_page_with_signature_having_wrong_signature_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let (signature_key, _) = sr25519::Pair::generate();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = PaginatedDeleteSignaturePayload {
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = signature_key.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::delete_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidSignature
		)
	})
}

#[test]
fn delete_page_with_signature_having_non_existing_msa_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let pair = get_invalid_msa_signature_account(); // hardcoded key that returns None Msa
		let delegator_key = pair.public();
		let msa_id = 1;
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = PaginatedDeleteSignaturePayload {
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::delete_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	})
}

#[test]
fn delete_page_with_signature_having_wrong_msa_in_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_correct_msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let wrong_msa_id = 3;
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = PaginatedDeleteSignaturePayload {
			target_hash: PageHash::default(),
			msa_id: wrong_msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::delete_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	})
}

#[test]
fn delete_page_with_signature_having_invalid_schema_id_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = INVALID_SCHEMA_ID;
		let page_id = 1;
		let payload = PaginatedDeleteSignaturePayload {
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::delete_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidSchemaId
		)
	})
}

#[test]
fn delete_page_with_signature_having_invalid_schema_location_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let page_id = 1;
		let payload = PaginatedDeleteSignaturePayload {
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::delete_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::SchemaPayloadLocationMismatch
		)
	})
}

#[test]
fn delete_page_with_signature_having_invalid_hash_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1.clone()),
			msa_id,
			schema_id,
			page_id,
			NONEXISTENT_PAGE_HASH,
			payload.into(),
		));

		let payload = PaginatedDeleteSignaturePayload {
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::delete_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::StalePageState
		)
	})
}

#[test]
fn delete_page_with_signature_with_non_existing_page_should_noop() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = PaginatedDeleteSignaturePayload {
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_storage_noop!(assert_eq!(
			StatefulStoragePallet::delete_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Ok(())
		));
	})
}

#[test]
fn delete_page_with_signature_having_valid_inputs_should_remove_page() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let page = generate_page::<PaginatedPageSize>(Some(1), Some(100));
		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&(schema_id, page_id),
			&page,
		);

		let payload = PaginatedDeleteSignaturePayload {
			target_hash: page.get_hash(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_ok!(StatefulStoragePallet::delete_page_with_signature(
			RuntimeOrigin::signed(caller_1),
			delegator_key.into(),
			owner_signature,
			payload
		));

		// assert
		let removed_page: Option<PaginatedPage<Test>> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				PAGINATED_STORAGE_PREFIX,
				&(schema_id, page_id),
			)
			.unwrap();
		assert!(removed_page.is_none());
		System::assert_last_event(
			StatefulEvent::PaginatedPageDeleted {
				msa_id,
				schema_id,
				page_id,
				prev_content_hash: page.get_hash(),
			}
			.into(),
		);
	})
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
fn delete_page_fails_for_append_only() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = test_public(1);
		let msa_id = 1;
		let schema_id = PAGINATED_APPEND_ONLY_SCHEMA;
		let page_id = 11;
		let payload = generate_payload_bytes::<PaginatedPageSize>(None);
		let page: PaginatedPage<Test> = payload.clone().into();
		let page_hash = page.get_hash();

		// assert
		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1.clone()),
			msa_id,
			schema_id,
			page_id,
			NONEXISTENT_PAGE_HASH,
			payload.into(),
		));

		assert_err!(
			StatefulStoragePallet::delete_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
				page_hash
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
