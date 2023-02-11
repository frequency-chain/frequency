use super::{mock::*, Event as StatefulEvent};
use crate::{
	pallet,
	stateful_child_tree::{StatefulChildTree, StatefulPageKeyPart},
	types::*,
	Config, Error,
};
use codec::{Decode, Encode, MaxEncodedLen};
use common_primitives::schema::{ModelType, PayloadLocation, SchemaId};
use frame_support::{assert_err, assert_ok, BoundedVec};
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use scale_info::TypeInfo;
use sp_core::Get;

type ItemizedPageSize = <Test as Config>::MaxItemizedPageSizeBytes;
type PaginatedPageSize = <Test as Config>::MaxPaginatedPageSizeBytes;

fn generate_payload_bytes<T: Get<u32>>(id: Option<u8>) -> BoundedVec<u8, T> {
	let value = id.unwrap_or(1);
	format!("{{'type':{value}, 'description':'another test description {value}'}}")
		.as_bytes()
		.to_vec()
		.try_into()
		.unwrap()
}

fn create_itemized_page_from<T: pallet::Config>(
	payloads: &[BoundedVec<u8, ItemizedPageSize>],
) -> ItemizedPage<T> {
	let mut buffer: Vec<u8> = vec![];
	for p in payloads {
		buffer.extend_from_slice(&ItemHeader { payload_len: p.len() as u16 }.encode()[..]);
		buffer.extend_from_slice(p.as_slice());
	}
	ItemizedPage::<T>::try_from(buffer).unwrap()
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
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let page: PaginatedPage<Test> = payload.clone().into();

		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id,
			payload.into()
		));

		let keys = [schema_id.encode().to_vec(), page_id.encode().to_vec()];
		let new_page: PaginatedPage<Test> =
			StatefulChildTree::try_read(&msa_id, &keys).unwrap().unwrap();
		assert_eq!(page, new_page);
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
		let old_content = generate_payload_bytes::<PaginatedPageSize>(Some(200));
		let new_content = generate_payload_bytes::<PaginatedPageSize>(Some(201));
		let old_page: PaginatedPage<Test> = old_content.clone().into();

		let keys = [schema_id.encode().to_vec(), page_id.encode().to_vec()];
		StatefulChildTree::write(&msa_id, &keys, old_page);
		let old_page: PaginatedPage<Test> =
			StatefulChildTree::try_read(&msa_id, &keys).unwrap().unwrap();

		assert_eq!(old_content, old_page.data);
		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id,
			new_content.clone().into()
		));

		let new_page: PaginatedPage<Test> =
			StatefulChildTree::try_read(&msa_id, &keys).unwrap().unwrap();
		assert_eq!(new_content, new_page.data);
	})
}

#[test]
fn delete_page_id_out_of_bounds_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1;
		let msa_id = 2;
		let schema_id = PAGINATED_SCHEMA;
		let page_id = <Test as Config>::MaxPaginatedPageId::get() + 1;

		assert_err!(
			StatefulStoragePallet::delete_page(
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
fn delete_page_with_invalid_msa_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1000; // hard-coded in mocks to return None for MSA
		let msa_id = 1;
		let schema_id = 1;
		let page_id = 1;

		assert_err!(
			StatefulStoragePallet::delete_page(
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
fn delete_page_with_invalid_schema_id_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = INVALID_SCHEMA_ID;
		let page_id = 1;

		assert_err!(
			StatefulStoragePallet::delete_page(
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
fn delete_page_with_invalid_schema_payload_location_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = ITEMIZED_SCHEMA;
		let page_id = 1;

		assert_err!(
			StatefulStoragePallet::delete_page(
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
fn delete_page_with_no_delegation_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;

		assert_err!(
			StatefulStoragePallet::delete_page(
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
fn delete_nonexistent_page_succeeds_noop() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 10;

		assert_ok!(StatefulStoragePallet::delete_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id
		));
	})
}

#[test]
fn delete_existing_page_succeeds() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 11;
		let payload = generate_payload_bytes::<PaginatedPageSize>(None);

		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id,
			payload.into()
		));

		assert_ok!(StatefulStoragePallet::delete_page(
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
	let payloads = vec![
		generate_payload_bytes::<ItemizedPageSize>(Some(1)),
		generate_payload_bytes::<ItemizedPageSize>(Some(2)),
	];
	let page = create_itemized_page_from::<Test>(&payloads);

	// act
	let parsed = page.parse_as_itemized(true);

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
	let parsed = page.parse_as_itemized(true);

	// assert
	assert_eq!(parsed, Err(PageError::ErrorParsing("wrong payload size")));
}

#[test]
fn applying_delete_action_with_existing_index_should_delete_item() {
	// arrange
	let payloads = vec![
		generate_payload_bytes::<ItemizedPageSize>(Some(2)),
		generate_payload_bytes::<ItemizedPageSize>(Some(4)),
	];
	let page = create_itemized_page_from::<Test>(payloads.as_slice());
	let expecting_page = create_itemized_page_from::<Test>(&payloads[1..]);
	let actions = vec![ItemAction::Delete { index: 0 }];

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
	let payload1 = vec![generate_payload_bytes::<ItemizedPageSize>(Some(2))];
	let page = create_itemized_page_from::<Test>(payload1.as_slice());
	let payload2 = vec![generate_payload_bytes::<ItemizedPageSize>(Some(4))];
	let expecting_page =
		create_itemized_page_from::<Test>(&vec![payload1[0].clone(), payload2[0].clone()][..]);
	let actions = vec![ItemAction::Add { data: payload2[0].clone().into() }];

	// act
	let result = page.apply_item_actions(&actions[..]);

	// assert
	assert_ok!(&result);
	let updated = result.unwrap();
	assert_eq!(expecting_page.data, updated.data);
}

#[test]
fn applying_delete_action_with_non_existing_index_should_fail() {
	// arrange
	let payloads = vec![
		generate_payload_bytes::<ItemizedPageSize>(Some(2)),
		generate_payload_bytes::<ItemizedPageSize>(Some(4)),
	];
	let page = create_itemized_page_from::<Test>(payloads.as_slice());
	let actions = vec![ItemAction::Delete { index: 2 }];

	// act
	let result = page.apply_item_actions(&actions[..]);

	// assert
	assert_eq!(result.is_err(), true);
}

#[test]
fn applying_add_action_with_full_page_should_fail() {
	// arrange
	let new_payload = generate_payload_bytes::<ItemizedPageSize>(Some(2));
	let page_len =
		ItemizedPageSize::get() as usize - ItemHeader::max_encoded_len() - new_payload.len();
	let existing_data = vec![1u8; page_len];
	let existing_payload: BoundedVec<u8, ItemizedPageSize> =
		BoundedVec::try_from(existing_data).unwrap();
	let page = create_itemized_page_from::<Test>(&[existing_payload]);
	let actions = vec![ItemAction::Add { data: new_payload.clone().into() }];

	// act
	let result = page.apply_item_actions(&actions[..]);

	// assert
	assert_eq!(result, Err(PageError::PageSizeOverflow));
}

#[test]
fn is_empty_false_for_non_empty_page() {
	let page: ItemizedPage<Test> = vec![1].try_into().unwrap();

	assert_eq!(page.is_empty(), false);
}

#[test]
fn is_empty_true_for_empty_page() {
	let page: ItemizedPage<Test> = Vec::<u8>::new().try_into().unwrap();

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
			let k = i.encode();
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
			let keys = vec![k.to_owned()];
			StatefulChildTree::write(&msa_id, &keys[..], t);
		}

		// act
		let nodes = StatefulChildTree::prefix_iterator::<TestStruct>(&msa_id, &[]);

		let mut count = 0;
		for n in nodes {
			assert!(arr.contains(&n));
			count += 1;
		}
		assert_eq!(count, arr.len());
	});
}

#[test]
fn apply_item_actions_with_add_item_action_bigger_than_expected_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = ITEMIZED_SCHEMA;
		let payload =
			vec![1; (<Test as Config>::MaxItemizedBlobSizeBytes::get() + 1).try_into().unwrap()];
		let actions = vec![ItemAction::Add { data: payload }];

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				BoundedVec::try_from(actions).unwrap(),
			),
			Error::<Test>::ItemExceedsMaxBlobSizeBytes
		)
	});
}

#[test]
fn apply_item_actions_with_invalid_msa_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 1000; // hard-coded in mocks to return None for MSA
		let msa_id = 1;
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload }];

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
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
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = INVALID_SCHEMA_ID;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload }];

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
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
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = PAGINATED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload }];

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
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
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = UNDELEGATED_ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload }];

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				BoundedVec::try_from(actions).unwrap(),
			),
			Error::<Test>::UnAuthorizedDelegate
		)
	});
}

#[test]
fn apply_item_actions_with_corrupted_state_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions1 = vec![ItemAction::Add { data: payload.clone() }];
		let schema_key = schema_id.encode().to_vec();
		StatefulChildTree::write::<Vec<u8>>(&msa_id, &[schema_key], payload);

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				BoundedVec::try_from(actions1).unwrap(),
			),
			Error::<Test>::CorruptedState
		)
	});
}

#[test]
fn apply_item_actions_with_valid_input_should_update_storage() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload }];

		// act
		assert_ok!(StatefulStoragePallet::apply_item_actions(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			BoundedVec::try_from(actions).unwrap(),
		));

		// assert
		let schema_key = schema_id.encode().to_vec();
		let items = StatefulChildTree::try_read::<Vec<u8>>(&msa_id, &[schema_key]).unwrap();
		assert!(items.is_some());
		System::assert_last_event(StatefulEvent::ItemizedPageUpdated { msa_id, schema_id }.into());
	});
}

#[test]
fn apply_item_actions_with_valid_input_and_empty_items_should_remove_storage() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 1;
		let msa_id = 1;
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions1 = vec![ItemAction::Add { data: payload }];
		let actions2 = vec![ItemAction::Delete { index: 0 }];
		assert_ok!(StatefulStoragePallet::apply_item_actions(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			BoundedVec::try_from(actions1).unwrap(),
		));

		// act
		assert_ok!(StatefulStoragePallet::apply_item_actions(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			BoundedVec::try_from(actions2).unwrap(),
		));

		// assert
		let schema_key = schema_id.encode().to_vec();
		let items = StatefulChildTree::try_read::<Vec<u8>>(&msa_id, &[schema_key]).unwrap();
		assert!(items.is_none());
		System::assert_last_event(StatefulEvent::ItemizedPageDeleted { msa_id, schema_id }.into());
	});
}
