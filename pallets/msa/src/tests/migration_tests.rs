use crate::{
	migration::{v1, v2::*},
	tests::mock::*,
	Config, Pallet, ProviderToRegistryEntry, ProviderToRegistryEntryV2,
};
use common_primitives::msa::ProviderId;
use frame_support::{pallet_prelude::*, BoundedVec};
use sp_core::Encode;
use sp_runtime::traits::Zero;

#[test]
fn test_migration_status_encoding() {
	let hashed_raw_key = ProviderToRegistryEntry::<Test>::hashed_key_for(ProviderId(5));
	let status = MigrationStatus {
		migrated_count: 10,
		completed: false,
		last_raw_key: Some(hashed_raw_key),
	};

	let encoded = status.encode();
	let decoded = MigrationStatus::decode(&mut &encoded[..]).unwrap();

	assert_eq!(status.migrated_count, decoded.migrated_count);
	assert_eq!(status.completed, decoded.completed);
	assert_eq!(status.last_raw_key, decoded.last_raw_key);
}

#[test]
fn migrate_provider_entries_batch_empty() {
	new_test_ext().execute_with(|| {
		let (weight, count, last_raw_key) = migrate_provider_entries_batch::<Test>(10, None);

		assert_eq!(count, 0);
		assert!(weight.is_zero());
		assert!(last_raw_key.is_none());
	});
}

#[test]
fn migrate_provider_entries_batch_single_item() {
	new_test_ext().execute_with(|| {
		let provider_id = ProviderId(1);
		let old_entry = v1::ProviderRegistryEntry {
			provider_name: BoundedVec::try_from(b"TestProvider".to_vec()).unwrap(),
		};

		ProviderToRegistryEntry::<Test>::insert(provider_id, old_entry.clone());

		let (weight, count, last_raw_key) = migrate_provider_entries_batch::<Test>(10, None);

		assert_eq!(count, 1);
		assert!(!weight.is_zero());
		let hashed_raw_key = ProviderToRegistryEntry::<Test>::hashed_key_for(provider_id);
		assert_eq!(last_raw_key, Some(hashed_raw_key));

		let new_entry = ProviderToRegistryEntryV2::<Test>::get(provider_id).unwrap();
		assert_eq!(new_entry.default_name, old_entry.provider_name);
		assert!(new_entry.default_logo_250_100_png_cid.is_empty());
		assert!(new_entry.localized_names.is_empty());
		assert!(new_entry.localized_logo_250_100_png_cids.is_empty());

		// Old storage is still present
		assert!(ProviderToRegistryEntry::<Test>::get(provider_id).is_some());
	});
}

#[test]
fn migrate_provider_entries_batch_multiple_items() {
	new_test_ext().execute_with(|| {
		for i in 1..=5 {
			let provider_id = ProviderId(i);
			let name = format!("Provider{}", i);
			let old_entry = v1::ProviderRegistryEntry {
				provider_name: BoundedVec::try_from(name.as_bytes().to_vec()).unwrap(),
			};
			ProviderToRegistryEntry::<Test>::insert(provider_id, old_entry);
		}

		// First batch - should migrate 3 items
		let (weight1, count1, last_key1) = migrate_provider_entries_batch::<Test>(3, None);
		assert_eq!(count1, 3);
		assert!(!weight1.is_zero());
		assert!(last_key1.is_some()); // Just check it exists, don't check specific value

		// Second batch - should migrate remaining 2 items
		let (weight2, count2, last_key2) = migrate_provider_entries_batch::<Test>(3, last_key1);
		assert_eq!(count2, 2);
		assert!(!weight2.is_zero());
		assert!(last_key2.is_some());

		// Verify ALL items were migrated (this is what matters)
		for i in 1..=5 {
			let provider_id = ProviderId(i);
			let new_entry = ProviderToRegistryEntryV2::<Test>::get(provider_id).unwrap();
			let name: BoundedVec<u8, <Test as Config>::MaxProviderNameSize> =
				BoundedVec::try_from(format!("Provider{}", i).as_bytes().to_vec()).unwrap();
			assert_eq!(new_entry.default_name, name);
		}
	});
}

#[test]
fn on_initialize_migration_test() {
	new_test_ext().execute_with(|| {
		// Should return zero weight since version is already >= 2
		StorageVersion::new(2).put::<Pallet<Test>>();
		let weight = on_initialize_migration::<Test>();
		assert_eq!(weight, frame_support::weights::Weight::zero());
	});
}

#[test]
fn batch_migration_respects_boundaries() {
	new_test_ext().execute_with(|| {
		let total_items = 51;

		for i in 1..=total_items {
			let old_entry = v1::ProviderRegistryEntry {
				provider_name: BoundedVec::try_from(format!("Provider{}", i).as_bytes().to_vec())
					.unwrap(),
			};
			ProviderToRegistryEntry::<Test>::insert(ProviderId(i), old_entry);
		}

		// First batch
		let (_, count1, last_key1) = migrate_provider_entries_batch::<Test>(50, None);
		assert_eq!(count1, 50);

		// Second batch
		let (_, count2, _) = migrate_provider_entries_batch::<Test>(50, last_key1);
		assert_eq!(count2, 1);

		// Verify all migrated
		for i in 1..=total_items {
			assert!(ProviderToRegistryEntryV2::<Test>::get(ProviderId(i)).is_some());
		}
	});
}

#[test]
fn on_initialize_migration_progresses_batches() {
	new_test_ext().execute_with(|| {
		StorageVersion::new(1).put::<Pallet<Test>>();

		let total_entries = 55u32;

		for i in 1..=total_entries {
			let old_entry = v1::ProviderRegistryEntry {
				provider_name: BoundedVec::try_from(format!("Provider{}", i).as_bytes().to_vec())
					.unwrap(),
			};
			ProviderToRegistryEntry::<Test>::insert(ProviderId(i.into()), old_entry);
		}

		let weight1 = on_initialize_migration::<Test>();
		assert!(!weight1.is_zero());

		let status = MigrationProgressV2::<Test>::get();
		assert_eq!(status.migrated_count, MAX_ITEMS_PER_BLOCK);
		assert!(!status.completed);
		assert!(status.last_raw_key.is_some());

		let weight2 = on_initialize_migration::<Test>();
		assert!(!weight2.is_zero());

		let status = MigrationProgressV2::<Test>::get();
		assert_eq!(status.migrated_count, total_entries);

		let weight3 = on_initialize_migration::<Test>();
		assert!(weight3.is_zero());

		assert!(!MigrationProgressV2::<Test>::exists());
		assert_eq!(Pallet::<Test>::on_chain_storage_version(), StorageVersion::new(2));

		for i in 1..=total_entries {
			assert!(ProviderToRegistryEntryV2::<Test>::get(ProviderId(i.into())).is_some());
			assert!(ProviderToRegistryEntry::<Test>::get(ProviderId(i.into())).is_some());
		}
	});
}
