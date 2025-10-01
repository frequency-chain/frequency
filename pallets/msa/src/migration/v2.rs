#[allow(deprecated)]
use crate::ProviderToRegistryEntry;
use crate::{migration::v1, Config, Pallet, ProviderToRegistryEntryV2};
pub use alloc::vec;
use common_primitives::msa::{ProviderId, ProviderRegistryEntry};
use frame_support::{pallet_prelude::*, storage_alias, weights::Weight};
pub use frame_system::pallet_prelude::BlockNumberFor;
#[cfg(feature = "try-runtime")]
pub use sp_runtime::TryRuntimeError;
const LOG_TARGET: &str = "runtime::provider";
pub const MAX_ITEMS_PER_BLOCK: u32 = 50; // Conservative batch size for Paseo

/// Storage item to track migration progress for Paseo
#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct MigrationStatus {
	/// Number of items migrated so far
	pub migrated_count: u32,
	/// Whether migration is complete
	pub completed: bool,
	/// Last key processed
	pub last_raw_key: Option<vec::Vec<u8>>,
}

/// Storage for tracking migration progress
#[storage_alias]
pub type MigrationProgressV2<T: Config> = StorageValue<Pallet<T>, MigrationStatus, ValueQuery>;

/// Core migration function that can be called from both contexts
#[allow(deprecated)]
pub fn migrate_provider_entries_batch<T: Config>(
	batch_size: usize,
	start_after_raw_key: Option<vec::Vec<u8>>,
) -> (Weight, u32, Option<vec::Vec<u8>>) {
	let mut reads = 0u64;
	let mut writes = 0u64;
	let mut bytes = 0u64;
	let mut migrated_count = 0u32;
	let mut last_raw_key = None;

	let iter = match start_after_raw_key {
		Some(raw_key) => ProviderToRegistryEntry::<T>::iter_from(raw_key),
		None => ProviderToRegistryEntry::<T>::iter(),
	};

	let entries: vec::Vec<(
		ProviderId,
		v1::ProviderRegistryEntry<<T as Config>::MaxProviderNameSize>,
	)> = iter.take(batch_size).collect();

	for (provider_id, old_entry) in entries {
		// Skip if already migrated
		if ProviderToRegistryEntryV2::<T>::contains_key(provider_id) {
			reads += 1;
			continue;
		}
		// Build new registry entry with old provider name
		let migrated_provider_entry = ProviderRegistryEntry {
			default_name: old_entry.provider_name.clone(),
			default_logo_250_100_png_cid: BoundedVec::default(),
			localized_logo_250_100_png_cids: BoundedBTreeMap::default(),
			localized_names: BoundedBTreeMap::default(),
		};
		// Insert into new storage
		ProviderToRegistryEntryV2::<T>::insert(provider_id, migrated_provider_entry);
		reads += 2;
		writes += 1;
		bytes += old_entry.encoded_size() as u64;
		migrated_count += 1;
		last_raw_key = Some(ProviderToRegistryEntry::<T>::hashed_key_for(provider_id));
	}

	let weight = T::DbWeight::get().reads_writes(reads, writes).add_proof_size(bytes);
	(weight, migrated_count, last_raw_key)
}

/// Hook-based multi-block migration for Frequency Provider Migration
/// This should be called from your pallet's on_initialize hook
pub fn on_initialize_migration<T: Config>() -> Weight {
	// Check if migration is needed
	let onchain_version = Pallet::<T>::on_chain_storage_version();
	if onchain_version >= 2 {
		// Clean up any leftover migration state
		if MigrationProgressV2::<T>::exists() {
			MigrationProgressV2::<T>::kill();
			return T::DbWeight::get().reads_writes(1, 1)
		}
		return T::DbWeight::get().reads(1)
	}

	let migration_status = MigrationProgressV2::<T>::get();
	let mut reads = 1u64;
	let mut writes = 0u64;

	// Initialize migration if not started
	if !migration_status.completed {
		let (batch_weight, migrated_in_this_block, last_key) = migrate_provider_entries_batch::<T>(
			MAX_ITEMS_PER_BLOCK as usize,
			migration_status.last_raw_key.clone(),
		);

		let mut updated_status = migration_status;
		updated_status.migrated_count += migrated_in_this_block;
		updated_status.completed = migrated_in_this_block == 0;
		updated_status.last_raw_key = last_key;
		reads += 1;
		// Check if migration is complete
		if updated_status.completed {
			StorageVersion::new(2).put::<Pallet<T>>();
			MigrationProgressV2::<T>::kill();
			writes += 2;
			log::info!(target: LOG_TARGET, "Hook-based migration completed! Total migrated: {}", updated_status.migrated_count);
		} else {
			MigrationProgressV2::<T>::put(&updated_status);
			writes += 1;
		}
		return T::DbWeight::get().reads_writes(reads, writes).saturating_add(batch_weight);
	}
	log::info!(target: LOG_TARGET, "Provider Registry Migration already completed.");
	T::DbWeight::get().reads(1)
}
