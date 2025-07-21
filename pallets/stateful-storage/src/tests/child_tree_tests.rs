use crate::{stateful_child_tree::StatefulChildTree, test_common::test_utility::*, tests::mock::*};
use common_primitives::schema::{ModelType, PayloadLocation, SchemaId};
use frame_support::BoundedVec;
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use sp_core::ConstU32;
extern crate alloc;
use alloc::collections::btree_set::BTreeSet;

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
