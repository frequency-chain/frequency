use super::mock::*;
use crate::{
	stateful_child_tree::{StatefulChildTree, StatefulPageKeyPart},
	types::*,
	Config, Error,
};
use codec::{Decode, Encode, MaxEncodedLen};
use common_primitives::schema::{ModelType, PayloadLocation, SchemaId};
use frame_support::{assert_err, assert_ok};
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use scale_info::TypeInfo;

type TestPageSize = <Test as Config>::MaxItemizedPageSizeBytes;
type TestPage = Page<TestPageSize>;

fn generate_payload_bytes(id: Option<u8>) -> Vec<u8> {
	let value = id.unwrap_or(1);
	format!("{{'type':{value}, 'description':'another test description {value}'}}")
		.as_bytes()
		.to_vec()
}

fn create_page_from(payloads: &[Vec<u8>]) -> TestPage {
	let mut buffer: Vec<u8> = vec![];
	for p in payloads {
		buffer.extend_from_slice(&ItemHeader { payload_len: p.len() as u16 }.encode()[..]);
		buffer.extend_from_slice(p);
	}
	TestPage::try_from(buffer).unwrap()
}

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, MaxEncodedLen)]
/// A structure defining a Schema
struct TestStruct {
	pub model_type: ModelType,
	pub payload_location: PayloadLocation,
	pub number: u64,
}

#[test]
fn upsert_page_too_large_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 5;
		let msa_id = 1;
		let schema_id = 1;
		let page_id = 0;
		let payload =
			vec![
				1;
				TryInto::<usize>::try_into(<Test as Config>::MaxPaginatedPageSizeBytes::get())
					.unwrap() + 1
			];

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
				payload
			),
			Error::<Test>::PageExceedsMaxPageSizeBytes
		)
	})
}

#[test]
fn upsert_page_id_out_of_bounds_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 5;
		let msa_id = 1;
		let schema_id = 1;
		let page_id = <Test as Config>::MaxPaginatedPageId::get() + 1;
		let payload = vec![1; 1];

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
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
		let caller_1 = 1000; // hard-coded in mocks to return None for MSA
		let msa_id = 1;
		let schema_id = 1;
		let page_id = 1;
		let payload = vec![1; 1];

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
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
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = INVALID_SCHEMA_ID;
		let page_id = 1;
		let payload = vec![1; 1];

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
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
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = ITEMIZED_SCHEMA;
		let page_id = 1;
		let payload = vec![1; 1];

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
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
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = vec![1; 1];

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
				payload
			),
			Error::<Test>::UnAuthorizedDelegate
		)
	})
}

#[test]
fn upsert_new_page_succeeds() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes(Some(100));

		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id,
			payload.clone()
		));

		let keys = [schema_id.encode().to_vec(), page_id.encode().to_vec()];
		let page = StatefulChildTree::try_read::<Vec<u8>>(&msa_id, &keys).unwrap().unwrap();
		assert_eq!(payload, page);
	})
}

#[test]
fn upsert_existing_page_modifies_page() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1;
		let msa_id = 2;
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let old_content = generate_payload_bytes(Some(200));
		let new_content = generate_payload_bytes(Some(201));

		let keys = [schema_id.encode().to_vec(), page_id.encode().to_vec()];
		StatefulChildTree::write(&msa_id, &keys, old_content.clone());
		let old_page = StatefulChildTree::try_read::<Vec<u8>>(&msa_id, &keys).unwrap().unwrap();

		assert_eq!(old_content, old_page);
		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id,
			new_content.clone()
		));

		let new_page = StatefulChildTree::try_read::<Vec<u8>>(&msa_id, &keys).unwrap().unwrap();
		assert_eq!(new_content, new_page);
	})
}

#[test]
fn remove_page_id_out_of_bounds_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1;
		let msa_id = 2;
		let schema_id = PAGINATED_SCHEMA;
		let page_id = <Test as Config>::MaxPaginatedPageId::get() + 1;

		assert_err!(
			StatefulStoragePallet::remove_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
			),
			Error::<Test>::PageIdExceedsMaxAllowed
		);
	})
}

#[test]
fn remove_page_with_invalid_msa_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1000; // hard-coded in mocks to return None for MSA
		let msa_id = 1;
		let schema_id = 1;
		let page_id = 1;

		assert_err!(
			StatefulStoragePallet::remove_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	})
}

#[test]
fn remove_page_with_invalid_schema_id_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = INVALID_SCHEMA_ID;
		let page_id = 1;

		assert_err!(
			StatefulStoragePallet::remove_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
			),
			Error::<Test>::InvalidSchemaId
		)
	})
}

#[test]
fn remove_page_with_invalid_schema_payload_location_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = ITEMIZED_SCHEMA;
		let page_id = 1;

		assert_err!(
			StatefulStoragePallet::remove_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
			),
			Error::<Test>::SchemaPayloadLocationMismatch
		)
	})
}

#[test]
fn remove_page_with_no_delegation_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;

		assert_err!(
			StatefulStoragePallet::remove_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
			),
			Error::<Test>::UnAuthorizedDelegate
		)
	})
}

#[test]
fn remove_nonexistent_page_succeeds_noop() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1`1	234l;'
		 0 ;

		// TODO: Get list of existing pages to verify the page doesn't already exist
		// PROBLEM: Child Trie iterator doesn't appear to yield keys
		// let key = schema_id.encode().to_vec();
		// let i = StatefulChildTree::prefix_iterator::<Vec<u8>>(&msa_id, &[key]);
		// let existing_node = i.filter(|k,v| {})

		assert_ok!(StatefulStoragePallet::remove_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id
		));
	})
}

#[test]
fn remove_existing_page_succeeds() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 11;
		let payload = generate_payload_bytes(None);

		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id,
			payload
		));

		assert_ok!(StatefulStoragePallet::remove_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id
		));

		let schema_key = schema_id.encode().to_vec();
		let page_key = page_id.encode().to_vec();
		let page =
			StatefulChildTree::try_read::<Vec<u8>>(&msa_id, &[schema_key, page_key]).unwrap();
		assert_eq!(page, None);
	})
}

#[test]
fn parsing_a_well_formed_item_page_should_work() {
	// arrange
	let payloads = vec![generate_payload_bytes(Some(1)), generate_payload_bytes(Some(2))];
	let page = create_page_from(payloads.as_slice());

	// act
	let parsed = page.parse_as_itemized();

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
	let payload = generate_payload_bytes(Some(1));
	let mut buffer: Vec<u8> = vec![];
	buffer.extend_from_slice(&ItemHeader { payload_len: (payload.len() + 1) as u16 }.encode()[..]);
	buffer.extend_from_slice(&payload);
	let page: TestPage = Page::try_from(buffer).unwrap();

	// act
	let parsed = page.parse_as_itemized();

	// assert
	assert_eq!(parsed, Err(PageError::ErrorParsing("wrong payload size")));
}

#[test]
fn applying_remove_action_with_existing_index_should_remove_item() {
	// arrange
	let payloads = vec![generate_payload_bytes(Some(2)), generate_payload_bytes(Some(4))];
	let page = create_page_from(payloads.as_slice());
	let expecting_page = create_page_from(&payloads[1..]);
	let actions = vec![ItemAction::Remove { index: 0 }];

	// act
	let result = page.apply_item_actions(&actions);

	// assert
	assert_ok!(&result);
	let updated = result.unwrap();
	assert_eq!(expecting_page.data, updated.data);
}

#[test]
fn applying_add_action_should_add_item_to_the_end_of_the_page() {
	// arrange
	let payload1 = vec![generate_payload_bytes(Some(2))];
	let page = create_page_from(payload1.as_slice());
	let payload2 = vec![generate_payload_bytes(Some(4))];
	let expecting_page = create_page_from(&vec![payload1[0].clone(), payload2[0].clone()][..]);
	let actions = vec![ItemAction::Add { data: payload2[0].clone() }];

	// act
	let result = page.apply_item_actions(&actions[..]);

	// assert
	assert_ok!(&result);
	let updated = result.unwrap();
	assert_eq!(expecting_page.data, updated.data);
}

#[test]
fn applying_remove_action_with_non_existing_index_should_fail() {
	// arrange
	let payloads = vec![generate_payload_bytes(Some(2)), generate_payload_bytes(Some(4))];
	let page = create_page_from(payloads.as_slice());
	let actions = vec![ItemAction::Remove { index: 2 }];

	// act
	let result = page.apply_item_actions(&actions[..]);

	// assert
	assert_eq!(result.is_err(), true);
}

#[test]
fn applying_add_action_with_full_page_should_fail() {
	// arrange
	let mut arr: Vec<Vec<u8>> = vec![];
	let payload = generate_payload_bytes(Some(2));
	while (arr.len() + 1) * (&payload.len() + ItemHeader::max_encoded_len()) <
		<TestPageSize as sp_core::Get<u32>>::get() as usize
	{
		arr.push(payload.clone());
	}
	let page = create_page_from(arr.as_slice());
	let actions = vec![ItemAction::Add { data: payload.clone() }];

	// act
	let result = page.apply_item_actions(&actions[..]);

	// assert
	assert_eq!(result.is_err(), true);
}

#[test]
fn is_empty_false_for_non_empty_page() {
	let page: TestPage = vec![1].try_into().unwrap();

	assert_eq!(page.is_empty(), false);
}

#[test]
fn is_empty_true_for_empty_page() {
	let page: TestPage = Vec::<u8>::new().try_into().unwrap();

	assert_eq!(page.is_empty(), true);
}

#[test]
fn child_tree_write_read() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let schema_id: SchemaId = 2;
		let page_id: u8 = 3;
		let k1: StatefulPageKeyPart = schema_id.to_be_bytes().to_vec();
		let k2: StatefulPageKeyPart = page_id.to_be_bytes().to_vec();
		let keys = &[k1, k2];
		let val = TestStruct {
			model_type: ModelType::AvroBinary,
			payload_location: PayloadLocation::OnChain,
			number: 8276387272,
		};

		// act
		StatefulChildTree::write(&msa_id, keys, &val);

		// assert
		let read = StatefulChildTree::try_read::<TestStruct>(&msa_id, keys).unwrap();
		assert_eq!(Some(val), read);
	});
}

#[test]
fn child_tree_iterator() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let mut arr: Vec<(Vec<u8>, TestStruct)> = Vec::new();
		for i in 1..=10 {
			let k = [b"key", &i.encode()[..]].concat();
			arr.push((
				k,
				TestStruct {
					model_type: ModelType::AvroBinary,
					payload_location: PayloadLocation::OnChain,
					number: i,
				},
			));
		}
		for (k, t) in arr.as_slice() {
			let keys = &[k.to_owned()];
			StatefulChildTree::write(&msa_id, keys, t);
		}

		// act
		let mut v = Vec::new();
		let nodes = StatefulChildTree::prefix_iterator::<TestStruct>(&msa_id, &[]);
		for n in nodes {
			v.push(n);
		}

		// assert
		assert_eq!(v, arr);
	});
}
