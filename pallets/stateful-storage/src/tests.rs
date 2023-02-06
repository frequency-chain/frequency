use super::mock::*;
use crate::{
	child_tree_storage::ChildTreeStorage,
	stateful_child_tree::{StatefulChildTree, StatefulPageKeyPart},
	types::{ItemAction, ItemHeader, ItemPage, PageError},
	Config, Error,
};
use codec::{Decode, Encode, MaxEncodedLen};
use common_primitives::schema::{ModelType, PayloadLocation, SchemaId};
use frame_support::{assert_err, assert_ok};
use scale_info::TypeInfo;
use sp_core::bounded::BoundedVec;
use sp_runtime::traits::One;

#[test]
fn upsert_page_too_large_errors() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5u64;
		let payload =
			vec![
				1;
				TryInto::<usize>::try_into(<Test as Config>::MaxPaginatedPageSizeBytes::get())
					.unwrap() + 1
			];

		assert_err!(
			StatefulStoragePallet::upsert_page(RuntimeOrigin::signed(caller_1), payload),
			Error::<Test>::PageExceedsMaxPageSizeBytes
		)
	})
}

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, MaxEncodedLen)]
/// A structure defining a Schema
struct TestStruct {
	pub model_version: u32,
	pub number: u64,
}

#[test]
fn child_tree_write_read() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let k1 = b"key1";
		let val = TestStruct { model_version: 1, number: 8276387272 };

		// act
		ChildTreeStorage::write(&msa_id, k1, &val);

		// assert
		let read = ChildTreeStorage::try_read::<TestStruct>(&msa_id, k1);
		assert_eq!(Ok(Some(val)), read);
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
			arr.push((k, TestStruct { model_version: 2, number: i }));
		}
		for (k, t) in arr.as_slice() {
			ChildTreeStorage::write(&msa_id, k, t);
		}

		// act
		let mut v = Vec::new();
		let nodes = ChildTreeStorage::prefix_iterator::<TestStruct>(&msa_id, &[]);
		for n in nodes {
			v.push(n);
		}

		// assert
		assert_eq!(v, arr);
	});
}

fn create_page_from<T: Config>(payloads: &[Vec<u8>]) -> ItemPage<T> {
	let mut buffer: Vec<u8> = vec![];
	for p in payloads {
		buffer.extend_from_slice(&ItemHeader { payload_len: p.len() as u16 }.encode()[..]);
		buffer.extend_from_slice(p);
	}
	let bounded: BoundedVec<u8, T::MaxItemizedPageSizeBytes> =
		BoundedVec::try_from(buffer).unwrap();
	ItemPage::new(T::BlockNumber::one(), payloads.len() as u16, bounded)
}

#[test]
fn parsing_a_well_formed_item_page_should_work() {
	// arrange
	let payloads = vec![
		"{'type':2, 'description':'another test description 1'}".as_bytes().to_vec(),
		"{'type':4, 'description':'another test description 2'}".as_bytes().to_vec(),
	];
	let page = create_page_from::<Test>(payloads.as_slice());

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
fn parsing_wrong_payload_size_page_should_return_parsing_error() {
	// arrange
	let payload = "{'type':2, 'description':'another test description 1'}".as_bytes().to_vec();
	let mut buffer: Vec<u8> = vec![];
	buffer.extend_from_slice(&ItemHeader { payload_len: (payload.len() + 1) as u16 }.encode()[..]);
	buffer.extend_from_slice(&payload);
	let bounded: BoundedVec<u8, _> = BoundedVec::try_from(buffer).unwrap();
	let page = ItemPage::<Test>::new(1, 1u16, bounded);

	// act
	let parsed = page.parse();

	// assert
	assert_eq!(parsed, Err(PageError::ErrorParsing("wrong payload size")));
}

#[test]
fn parsing_wrong_header_size_page_should_return_parsing_error() {
	// arrange
	let payload = "{'type':2, 'description':'another test description 1'}".as_bytes().to_vec();
	let mut buffer: Vec<u8> = vec![];
	buffer.extend_from_slice(&ItemHeader { payload_len: (payload.len() - 1) as u16 }.encode()[..]);
	buffer.extend_from_slice(&payload);
	let bounded: BoundedVec<u8, _> = BoundedVec::try_from(buffer).unwrap();
	let page = ItemPage::<Test>::new(1, 1u16, bounded);

	// act
	let parsed = page.parse();

	// assert
	assert_eq!(parsed, Err(PageError::ErrorParsing("wrong header size")));
}

#[test]
fn applying_remove_action_with_exisitng_index_should_remove_item() {
	// arrange
	let payloads = vec![
		"{'type':2, 'description':'another test description 1'}".as_bytes().to_vec(),
		"{'type':4, 'description':'another test description 2'}".as_bytes().to_vec(),
	];
	let page = create_page_from::<Test>(payloads.as_slice());
	let expecting_page = create_page_from::<Test>(&payloads[1..]);
	let actions = vec![ItemAction::Remove { index: 0 }];

	// act
	let result = page.apply_actions(2, &actions[..]);

	// assert
	assert_ok!(&result);
	let updated = result.unwrap();
	assert_eq!(expecting_page.data, updated.data);
}

#[test]
fn applying_add_action_should_add_item_to_the_end_of_the_page() {
	// arrange
	let payload1 =
		vec!["{'type':2, 'description':'another test description 1'}".as_bytes().to_vec()];
	let page = create_page_from::<Test>(payload1.as_slice());
	let payload2 =
		vec!["{'type':4, 'description':'another test description 2'}".as_bytes().to_vec()];
	let expecting_page =
		create_page_from::<Test>(&vec![payload1[0].clone(), payload2[0].clone()][..]);
	let actions = vec![ItemAction::Add { data: payload2[0].clone() }];

	// act
	let result = page.apply_actions(2, &actions[..]);

	// assert
	assert_ok!(&result);
	let updated = result.unwrap();
	assert_eq!(expecting_page.data, updated.data);
}

#[test]
fn applying_remove_action_with_non_existing_index_should_fail() {
	// arrange
	let payloads = vec![
		"{'type':2, 'description':'another test description 1'}".as_bytes().to_vec(),
		"{'type':4, 'description':'another test description 2'}".as_bytes().to_vec(),
	];
	let page = create_page_from::<Test>(payloads.as_slice());
	let actions = vec![ItemAction::Remove { index: 2 }];

	// act
	let result = page.apply_actions(2, &actions[..]);

	// assert
	assert_eq!(result.is_err(), true);
}

#[test]
fn applying_add_action_with_full_page_should_fail() {
	// arrange
	let mut arr: Vec<Vec<u8>> = vec![];
	let payload = "{'type':2, 'description':'another test description 1'}".as_bytes().to_vec();
	while (arr.len() + 1) * (&payload.len() + ItemHeader::max_encoded_len()) <
		<Test as Config>::MaxItemizedPageSizeBytes::get() as usize
	{
		arr.push(payload.clone());
	}
	let page = create_page_from::<Test>(arr.as_slice());
	let actions = vec![ItemAction::Add { data: payload.clone() }];

	// act
	let result = page.apply_actions(2, &actions[..]);

	// assert
	assert_eq!(result.is_err(), true);
}

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, MaxEncodedLen)]
/// A structure defining a Schema
struct TestStruct {
	pub model_type: ModelType,
	pub payload_location: PayloadLocation,
	pub number: u64,
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
