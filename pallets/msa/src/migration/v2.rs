use crate::{migration::v1, Config, Pallet, ProviderToRegistryEntryV2};
pub use alloc::vec;
use common_primitives::msa::{ProviderId, ProviderRegistryEntry};
use frame_support::{pallet_prelude::*, storage_alias, traits::OnRuntimeUpgrade, weights::Weight};
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
	/// Total number of items to migrate
	pub total_count: u32,
	/// Whether migration is complete
	pub completed: bool,
}

/// Storage for tracking migration progress
#[storage_alias]
pub type MigrationProgressV2<T: Config> = StorageValue<Pallet<T>, MigrationStatus, ValueQuery>;

/// Core migration function that can be called from both contexts
pub fn migrate_provider_entries_batch<T: Config>(batch_size: usize) -> (Weight, u32) {
	let mut reads = 0u64;
	let mut writes = 0u64;
	let mut bytes = 0u64;
	let mut migrated_count = 0u32;

	let entries: vec::Vec<(
		ProviderId,
		v1::ProviderRegistryEntry<<T as Config>::MaxProviderNameSize>,
	)> = v1::ProviderToRegistryEntry::<T>::drain().take(batch_size).collect();

	for (provider_id, old_entry) in entries {
		// Build new registry entry with old provider name
		let migrated_provider_entry = ProviderRegistryEntry {
			default_name: old_entry.provider_name.clone(),
			default_logo_250_100_png_cid: BoundedVec::default(),
			localized_logo_250_100_png_cids: BoundedBTreeMap::default(),
			localized_names: BoundedBTreeMap::default(),
		};
		// Insert into new storage
		ProviderToRegistryEntryV2::<T>::insert(provider_id, migrated_provider_entry);
		reads += 1;
		writes += 1;
		bytes += old_entry.encoded_size() as u64;
		migrated_count += 1;
	}

	let weight = T::DbWeight::get().reads_writes(reads, writes).add_proof_size(bytes);
	(weight, migrated_count)
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
			return T::DbWeight::get().writes(1)
		}
		return Weight::zero()
	}

	let migration_status = MigrationProgressV2::<T>::get();
	let mut reads = 1u64;
	let mut writes = 0u64;

	// Initialize migration if not started
	if migration_status.total_count == 0 {
		let total_count = v1::ProviderToRegistryEntry::<T>::iter().count() as u32;
		reads += 1;

		if total_count == 0 {
			// No items to migrate, complete immediately
			StorageVersion::new(2).put::<Pallet<T>>();
			return T::DbWeight::get().reads_writes(reads, 1)
		}

		log::info!(target: LOG_TARGET, "Initializing hook-based migration for Paseo. Total items: {total_count}");

		MigrationProgressV2::<T>::put(MigrationStatus {
			migrated_count: 0,
			total_count,
			completed: false,
		});
		writes += 1;
		return T::DbWeight::get().reads_writes(reads, writes)
	}

	// Continue migration if not complete
	if migration_status.migrated_count < migration_status.total_count {
		let (batch_weight, migrated_in_this_block) =
			migrate_provider_entries_batch::<T>(MAX_ITEMS_PER_BLOCK as usize);

		let mut updated_status = migration_status;
		updated_status.migrated_count += migrated_in_this_block;

		// Check if migration is complete
		if updated_status.migrated_count >= updated_status.total_count {
			updated_status.completed = true;
			StorageVersion::new(2).put::<Pallet<T>>();
			MigrationProgressV2::<T>::kill();
			writes += 2;
			log::info!(target: LOG_TARGET, "Hook-based migration completed! Total migrated: {}", updated_status.migrated_count);
		} else {
			MigrationProgressV2::<T>::put(&updated_status);
			writes += 1;
			log::info!(
				target: LOG_TARGET,
				"Provider registry migration progress: {}/{} (this block: {})",
				updated_status.migrated_count,
				updated_status.total_count,
				migrated_in_this_block
			);
		}

		return T::DbWeight::get().reads_writes(reads, writes).saturating_add(batch_weight)
	}

	Weight::zero()
}

/// migrate `ProviderToRegistryEntryV2` translating to new `ProviderRegistryEntry`
pub struct MigrateProviderToRegistryEntryV2<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateProviderToRegistryEntryV2<T> {
	fn on_runtime_upgrade() -> Weight {
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		let current_version = Pallet::<T>::in_code_storage_version();

		if onchain_version >= current_version {
			log::info!(
				target: LOG_TARGET,
				"Migration already completed or not needed. onchain_version: {onchain_version:?}, current_version: {current_version:?}",
			);
		}
		return T::DbWeight::get().reads(1)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<vec::Vec<u8>, TryRuntimeError> {
		log::info!(target: LOG_TARGET, "Running pre_upgrade...");
		Ok(vec::Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_encoded_numbers: vec::Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: LOG_TARGET, "Running post_upgrade...");
		Ok(())
	}
}
