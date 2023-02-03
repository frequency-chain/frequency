use super::mock::*;
use crate::{
	stateful_child_tree::{StatefulChildTree, StatefulPageKeyPart},
	Config, Error,
};
use common_primitives::schema::{ModelType, PayloadLocation, SchemaId};
use frame_support::{assert_err, assert_ok};

#[test]
fn add_item_ok() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5u64;
		let payload = vec![1; 64];

		assert_ok!(StatefulStoragePallet::add_item(RuntimeOrigin::signed(caller_1), payload));
	})
}

#[test]
fn add_item_with_large_payload_errors() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5u64;
		let payload =
			vec![
				1;
				TryInto::<usize>::try_into(<Test as Config>::MaxItemizedBlobSizeBytes::get())
					.unwrap() + 1
			];

		assert_err!(
			StatefulStoragePallet::add_item(RuntimeOrigin::signed(caller_1), payload),
			Error::<Test>::ItemExceedsMaxBlobSizeBytes
		)
	})
}

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
