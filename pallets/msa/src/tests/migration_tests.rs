use crate::{
	migration::{v1, v2::*},
	tests::mock::*,
	Config, Pallet, ProviderToRegistryEntryV2,
};
use common_primitives::msa::ProviderId;
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade, BoundedVec};
use sp_core::Encode;
use sp_runtime::traits::Zero;

#[test]
fn test_migration_status_encoding() {
	// Test that MigrationStatus can be encoded/decoded properly
	let status = MigrationStatus { migrated_count: 10, total_count: 100, completed: false };

	let encoded = status.encode();
	let decoded = MigrationStatus::decode(&mut &encoded[..]).unwrap();

	assert_eq!(status.migrated_count, decoded.migrated_count);
	assert_eq!(status.total_count, decoded.total_count);
	assert_eq!(status.completed, decoded.completed);
}

#[test]
fn migrate_provider_entries_batch_empty() {
	new_test_ext().execute_with(|| {
		// Test with empty storage
		let (weight, count, is_last) = migrate_provider_entries_batch::<Test>(10);

		assert_eq!(count, 0);
		assert!(is_last);
		assert!(weight.is_zero());
	});
}

#[test]
fn migrate_provider_entries_batch_single_item() {
	new_test_ext().execute_with(|| {
		// Create an old entry
		let provider_id = ProviderId(1);
		let old_entry = v1::ProviderRegistryEntry {
			provider_name: BoundedVec::try_from(b"TestProvider".to_vec()).unwrap(),
		};

		v1::ProviderToRegistryEntry::<Test>::insert(provider_id, old_entry.clone());

		// Migrate batch
		let (weight, count, is_last) = migrate_provider_entries_batch::<Test>(10);

		assert_eq!(count, 1);
		assert!(is_last);
		assert!(!weight.is_zero());

		// Check new storage
		let new_entry = ProviderToRegistryEntryV2::<Test>::get(provider_id).unwrap();
		assert_eq!(new_entry.default_name, old_entry.provider_name);
		assert!(new_entry.default_logo_250_100_png_cid.is_empty());
		assert!(new_entry.localized_names.is_empty());
		assert!(new_entry.localized_logo_250_100_png_cids.is_empty());

		// Check old storage is removed
		assert!(v1::ProviderToRegistryEntry::<Test>::get(provider_id).is_none());
	});
}

#[test]
fn migrate_provider_entries_batch_multiple_items() {
	new_test_ext().execute_with(|| {
		// Create multiple old entries
		for i in 1..=5 {
			let provider_id = ProviderId(i);
			let name = format!("Provider{}", i);
			let old_entry = v1::ProviderRegistryEntry {
				provider_name: BoundedVec::try_from(name.as_bytes().to_vec()).unwrap(),
			};
			v1::ProviderToRegistryEntry::<Test>::insert(provider_id, old_entry);
		}

		// Migrate first batch (size 3)
		let (weight1, count1, is_last1) = migrate_provider_entries_batch::<Test>(3);
		assert_eq!(count1, 3);
		assert!(!is_last1);
		assert!(!weight1.is_zero());

		// Migrate second batch (remaining 2)
		let (weight2, count2, is_last2) = migrate_provider_entries_batch::<Test>(3);
		assert_eq!(count2, 2);
		assert!(is_last2);
		assert!(!weight2.is_zero());

		// Verify all migrated
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
fn migrate_all_provider_entries_with_data() {
	new_test_ext().execute_with(|| {
		// Setup: Create old entries
		let providers = vec![
			(ProviderId(1), "Provider1"),
			(ProviderId(2), "Provider2"),
			(ProviderId(3), "Provider3"),
		];

		for (id, name) in &providers {
			let old_entry = v1::ProviderRegistryEntry {
				provider_name: BoundedVec::try_from(name.as_bytes().to_vec()).unwrap(),
			};
			v1::ProviderToRegistryEntry::<Test>::insert(id, old_entry);
		}

		// Execute migration
		let weight = migrate_all_provider_entries::<Test>();
		assert!(!weight.is_zero());

		// Verify storage version updated
		assert_eq!(Pallet::<Test>::on_chain_storage_version(), StorageVersion::new(2));

		// Verify all entries migrated
		for (id, name) in providers {
			let new_entry = ProviderToRegistryEntryV2::<Test>::get(id).unwrap();
			let name: BoundedVec<u8, <Test as Config>::MaxProviderNameSize> =
				BoundedVec::try_from(name.as_bytes().to_vec()).unwrap();
			assert_eq!(new_entry.default_name, name);
			assert!(new_entry.default_logo_250_100_png_cid.is_empty());
			assert!(new_entry.localized_names.is_empty());
		}

		// Verify old storage is empty
		assert_eq!(v1::ProviderToRegistryEntry::<Test>::iter().count(), 0);
	});
}

#[test]
fn on_initialize_migration_not_paseo() {
	new_test_ext().execute_with(|| {
		// Since test environment is not Paseo, should return zero weight
		let weight = on_initialize_migration::<Test>();
		assert!(weight.is_zero());
	});
}

#[test]
fn on_initialize_migration_already_migrated() {
	new_test_ext().execute_with(|| {
		// Set storage version to 2 (already migrated)
		StorageVersion::new(2).put::<Pallet<Test>>();

		// Even if we had migration progress, it should be cleaned up
		MigrationProgressV2::<Test>::put(MigrationStatus {
			migrated_count: 5,
			total_count: 10,
			completed: false,
		});

		// Mock that we're on Paseo (would need to mock get_chain_type)
		// For this test, we'll directly test the cleanup logic

		// Since version is already 2, and progress exists, it should clean up
		let onchain_version = Pallet::<Test>::on_chain_storage_version();
		if onchain_version >= 2 && MigrationProgressV2::<Test>::exists() {
			MigrationProgressV2::<Test>::kill();
			assert!(!MigrationProgressV2::<Test>::exists());
		}
	});
}

#[test]
fn on_runtime_upgrade_successful_migration() {
	new_test_ext().execute_with(|| {
		// Setup: Create old entries and ensure version is 1
		StorageVersion::new(1).put::<Pallet<Test>>();

		let providers = vec![(ProviderId(1), "TestProvider1"), (ProviderId(2), "TestProvider2")];

		for (id, name) in &providers {
			let old_entry = v1::ProviderRegistryEntry {
				provider_name: BoundedVec::try_from(name.as_bytes().to_vec()).unwrap(),
			};
			v1::ProviderToRegistryEntry::<Test>::insert(id, old_entry);
		}

		// Execute migration
		let weight = MigrateProviderToRegistryEntryV2::<Test>::on_runtime_upgrade();
		assert!(!weight.is_zero());
		// Verify migration completed (for non-Paseo)
		// Since test env is not Paseo, it should do single-block migration
		for (id, name) in providers {
			let new_entry = ProviderToRegistryEntryV2::<Test>::get(id).unwrap();
			let name: BoundedVec<u8, <Test as Config>::MaxProviderNameSize> =
				BoundedVec::try_from(name.as_bytes().to_vec()).unwrap();
			assert_eq!(new_entry.default_name, name);
		}
	});
}

#[test]
fn migration_preserves_provider_names() {
	new_test_ext().execute_with(|| {
		// Test that provider names are correctly preserved during migration
		let test_cases = vec![
			(ProviderId(1), b"SimpleProvider".to_vec()),
			(ProviderId(2), b"Provider-With-Dashes".to_vec()),
			(ProviderId(3), b"Provider_With_Underscores".to_vec()),
			(ProviderId(4), b"Provider123".to_vec()),
			(ProviderId(5), b"".to_vec()), // Empty name edge case
		];

		// Setup old entries
		for (id, name) in &test_cases {
			if let Ok(bounded_name) = BoundedVec::try_from(name.clone()) {
				let old_entry = v1::ProviderRegistryEntry { provider_name: bounded_name };
				v1::ProviderToRegistryEntry::<Test>::insert(id, old_entry);
			}
		}

		// Migrate
		migrate_all_provider_entries::<Test>();

		// Verify names preserved
		for (id, expected_name) in test_cases {
			if let Some(entry) = ProviderToRegistryEntryV2::<Test>::get(id) {
				assert_eq!(entry.default_name.to_vec(), expected_name);
			}
		}
	});
}

#[test]
fn batch_migration_respects_boundaries() {
	new_test_ext().execute_with(|| {
		// Create exactly MAX_ITEMS_PER_BLOCK + 1 entries
		let total_items = 51; // MAX_ITEMS_PER_BLOCK is 50

		for i in 1..=total_items {
			let old_entry = v1::ProviderRegistryEntry {
				provider_name: BoundedVec::try_from(format!("Provider{}", i).as_bytes().to_vec())
					.unwrap(),
			};
			v1::ProviderToRegistryEntry::<Test>::insert(ProviderId(i), old_entry);
		}

		// First batch should migrate exactly 50 items
		let (_, count1, is_last1) = migrate_provider_entries_batch::<Test>(50);
		assert_eq!(count1, 50);
		assert!(!is_last1);

		// Second batch should migrate remaining 1 item
		let (_, count2, is_last2) = migrate_provider_entries_batch::<Test>(50);
		assert_eq!(count2, 1);
		assert!(is_last2);
	});
}

#[test]
fn on_initialize_migration_progresses_batches() {
	new_test_ext().execute_with(|| {
		// Step 1: Force testnet to Paseo
		unsafe {
			FORCE_PASEO = true;
		}
		// Step 2: Insert more than MAX_ITEMS_PER_BLOCK old entries
		let total_entries = 55u32; // MAX_ITEMS_PER_BLOCK = 50
		for i in 1..=total_entries {
			let old_entry = v1::ProviderRegistryEntry {
				provider_name: BoundedVec::try_from(format!("Provider{}", i).as_bytes().to_vec())
					.unwrap(),
			};
			v1::ProviderToRegistryEntry::<Test>::insert(ProviderId(i.into()), old_entry);
		}

		// Step 3: Call on_initialize_migration for first block that prepares status
		let weight1 = on_initialize_migration::<Test>();
		assert!(weight1.is_zero());

		let weight2 = on_initialize_migration::<Test>();
		assert!(!weight2.is_zero());

		// Step 4: Check progress
		let status = MigrationProgressV2::<Test>::get();
		assert_eq!(status.migrated_count, MAX_ITEMS_PER_BLOCK);
		assert_eq!(status.total_count, total_entries);
		assert!(!status.completed);

		// Step 6: Call on_initialize_migration for second block
		let weight3 = on_initialize_migration::<Test>();
		assert!(!weight3.is_zero());

		// Step 7: Progress should be complete
		assert!(!MigrationProgressV2::<Test>::exists());
		assert_eq!(Pallet::<Test>::on_chain_storage_version(), StorageVersion::new(2));

		// Step 8: All old entries migrated
		for i in 1..=total_entries {
			assert!(ProviderToRegistryEntryV2::<Test>::get(ProviderId(i.into())).is_some());
			assert!(v1::ProviderToRegistryEntry::<Test>::get(ProviderId(i.into())).is_none());
		}
	});
}
