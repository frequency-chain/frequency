use crate::{migration::v1, Config, Pallet, ProviderToRegistryEntry, STORAGE_VERSION};
use common_primitives::{
	msa::ProviderRegistryEntry,
	utils::{get_chain_type_by_genesis_hash, DetectedChainType},
};
use frame_support::{pallet_prelude::*, storage_alias, traits::OnRuntimeUpgrade, weights::Weight};
pub use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::Saturating;

#[cfg(feature = "try-runtime")]
mod try_runtime_imports {
	pub use alloc::vec;
	pub use common_primitives::msa::ProviderId;
	pub use sp_runtime::TryRuntimeError;
}
#[cfg(feature = "try-runtime")]
use try_runtime_imports::*;

const LOG_TARGET: &str = "runtime::provider";
const MAX_ITEMS_PER_BLOCK: u32 = 50; // Conservative batch size for Paseo

/// Storage item to track migration progress for Paseo
#[derive(Default, Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
struct MigrationStatus {
	pub migrated_count: u32,
	pub total_count: u32,
	pub completed: bool,
}

// Storage for tracking migration progress
#[storage_alias]
type MigrationProgressV2<T: Config> = StorageValue<Pallet<T>, MigrationStatus, ValueQuery>;

/// Function to check current chain type - available in all contexts
fn get_chain_type<T: Config>() -> DetectedChainType {
	let genesis_block: BlockNumberFor<T> = 0u32.into();
	let genesis = <frame_system::Pallet<T>>::block_hash(genesis_block);
	get_chain_type_by_genesis_hash(&genesis.encode()[..])
}

/// Check if running on Paseo testnet
fn is_paseo_testnet<T: Config>() -> bool {
	matches!(get_chain_type::<T>(), DetectedChainType::FrequencyPaseoTestNet)
}

/// Get known provider ids from paseo and mainnet
#[cfg(feature = "try-runtime")]
pub fn get_known_provider_ids<T: Config>() -> vec::Vec<ProviderId> {
	match get_chain_type::<T>() {
		DetectedChainType::FrequencyPaseoTestNet => vec![
			// name: PrivateProvider
			ProviderId::from(5213u64),
			// name: CapacityProvider
			ProviderId::from(77_716u64),
			// name: Provider1
			ProviderId::from(77_674u64),
		],
		DetectedChainType::FrequencyMainNet => vec![
			// Soar.ai
			ProviderId::from(1_610_732u64),
			// AmplicaTest1
			ProviderId::from(1u64),
			// MeWe
			ProviderId::from(2u64),
		],
		_ => vec::Vec::new(),
	}
}

/// Core migration function that can be called from both contexts
fn migrate_provider_entries_batch<T: Config>(
	start_index: usize,
	batch_size: usize,
) -> (Weight, u32, bool) {
	let mut reads = 0u64;
	let mut writes = 0u64;
	let mut bytes = 0u64;
	let mut migrated_count = 0u32;

	let entries: vec::Vec<(
		ProviderId,
		v1::ProviderRegistryEntry<<T as Config>::MaxProviderNameSize>,
	)> = v1::ProviderToRegistryEntry::<T>::iter()
		.skip(start_index)
		.take(batch_size)
		.collect();

	let is_last_batch = entries.len() < batch_size;

	for (provider_id, old_entry) in entries {
		// Build new registry entry with old provider name
		let migrated_provider_entry = ProviderRegistryEntry {
			default_name: old_entry.provider_name.clone(),
			default_logo_250_100_png_cid: BoundedVec::default(),
			localized_logo_250_100_png_cids: BoundedBTreeMap::default(),
			localized_names: BoundedBTreeMap::default(),
		};

		// Remove from old storage and insert into new storage
		v1::ProviderToRegistryEntry::<T>::remove(provider_id);
		ProviderToRegistryEntry::<T>::insert(provider_id, migrated_provider_entry);

		reads += 1;
		writes += 2; // One remove, one insert
		bytes += old_entry.encoded_size() as u64;
		migrated_count += 1;
	}

	let weight = T::DbWeight::get().reads_writes(reads, writes).add_proof_size(bytes);
	(weight, migrated_count, is_last_batch)
}

/// Single-block migration for mainnet and smaller chains
fn migrate_all_provider_entries<T: Config>() -> Weight {
	log::info!(target: LOG_TARGET, "Starting single-block migration from v1 to v2...");

	let total_count = v1::ProviderToRegistryEntry::<T>::iter().count();
	log::info!(target: LOG_TARGET, "Total items in OLD ProviderToRegistryEntry storage: {total_count}");

	if total_count == 0 {
		log::error!(target: LOG_TARGET, "No items found in OLD ProviderToRegistryEntry storage, skipping migration.");
		return T::DbWeight::get().reads(1)
	}

	let mut total_reads = 1u64; // Initial count read
	let mut total_writes = 0u64;
	let mut total_bytes = 0u64;
	let mut migrated_count = 0u64;

	// Drain all entries in single block
	for (provider_id, old_entry) in v1::ProviderToRegistryEntry::<T>::drain() {
		let migrated_provider_entry = ProviderRegistryEntry {
			default_name: old_entry.provider_name.clone(),
			default_logo_250_100_png_cid: BoundedVec::default(),
			localized_logo_250_100_png_cids: BoundedBTreeMap::default(),
			localized_names: BoundedBTreeMap::default(),
		};

		ProviderToRegistryEntry::<T>::insert(provider_id, migrated_provider_entry);
		total_reads.saturating_inc();
		total_writes.saturating_inc();
		total_bytes += old_entry.encoded_size() as u64;
		migrated_count.saturating_inc();
	}

	// Set storage version
	StorageVersion::new(2).put::<Pallet<T>>();
	total_writes.saturating_inc();

	log::info!(target: LOG_TARGET, "Migration completed: {migrated_count} items migrated");
	log::info!(target: LOG_TARGET, "Storage migrated to version 2  read={total_reads:?}, write={total_writes:?}, bytes={total_bytes:?}");

	T::DbWeight::get()
		.reads_writes(total_reads, total_writes)
		.add_proof_size(total_bytes)
}

/// Hook-based multi-block migration for Paseo testnet
/// This should be called from your pallet's on_initialize hook
pub fn on_initialize_migration<T: Config>() -> Weight {
	// Only run on Paseo testnet
	if !is_paseo_testnet::<T>() {
		return Weight::zero()
	}

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
		let (batch_weight, migrated_in_this_block, _) = migrate_provider_entries_batch::<T>(
			migration_status.migrated_count as usize,
			MAX_ITEMS_PER_BLOCK as usize,
		);

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

/// migrate `ProviderToRegistryEntry` translating to new `ProviderRegistryEntry`
pub struct MigrateProviderToRegistryEntryV2<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateProviderToRegistryEntryV2<T> {
	fn on_runtime_upgrade() -> Weight {
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		let current_version = Pallet::<T>::in_code_storage_version();
		log::info!(target: LOG_TARGET, "onchain_version= {onchain_version:?}, current_version={current_version:?}");

		if STORAGE_VERSION != current_version {
			log::error!(target: LOG_TARGET, "Storage version mismatch. Expected: {STORAGE_VERSION:?}, Found: {current_version:?}");
			return T::DbWeight::get().reads(1)
		}

		if onchain_version >= current_version {
			log::info!(
				target: LOG_TARGET,
				"Migration already completed or not needed. onchain_version: {onchain_version:?}, current_version: {current_version:?}",
			);
			return T::DbWeight::get().reads(1)
		}

		// For Paseo testnet, migration runs via hooks
		if is_paseo_testnet::<T>() {
			log::info!(target: LOG_TARGET, "Paseo testnet detected - migration will run via on_initialize hooks");
			return T::DbWeight::get().reads(1)
		}

		// For mainnet and other chains, do single-block migration
		log::info!(target: LOG_TARGET, "Migrating v1 ProviderToRegistryEntry to updated ProviderRegistryEntry...");
		migrate_all_provider_entries::<T>()
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<vec::Vec<u8>, TryRuntimeError> {
		log::info!(target: LOG_TARGET, "Running pre_upgrade...");
		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		log::info!(target: LOG_TARGET, "Current on_chain_version: {on_chain_version:?}");

		if on_chain_version >= 2 {
			let current_count = ProviderToRegistryEntry::<T>::iter().count() as u64;
			log::info!(target: LOG_TARGET, "Migration already completed, current count: {current_count}");
			return Ok(current_count.encode().to_vec());
		}

		let old_count = v1::ProviderToRegistryEntry::<T>::iter().count() as u64;

		log::info!(target: LOG_TARGET, "Found {old_count} items in OLD storage format");

		let detected_chain = get_chain_type::<T>();
		log::info!(target: LOG_TARGET, "Detected Chain: {detected_chain:?}");

		Ok(old_count.encode().to_vec())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(encoded_numbers: vec::Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: LOG_TARGET, "Running post_upgrade...");
		if is_paseo_testnet::<T>() {
			log::info!(target: LOG_TARGET, "Paseo detected - migration may continue via hooks");
			return Ok(())
		}
		use parity_scale_codec::Decode;

		let expected: u64 = u64::decode(&mut encoded_numbers.as_slice())
			.map_err(|_| TryRuntimeError::Other("Cannot decode expected count"))?;

		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		assert_eq!(on_chain_version, STORAGE_VERSION);

		let known_providers = get_known_provider_ids::<T>();
		if known_providers.is_empty() {
			log::warn!(target: LOG_TARGET, "No known providers found for post-upgrade check");
		} else {
			for id in known_providers {
				let entry = ProviderToRegistryEntry::<T>::get(id).ok_or_else(|| {
					log::error!(target: LOG_TARGET, "Missing provider entry for provider id: {id:?}");
					TryRuntimeError::Other("Missing provider entry in post-upgrade check")
				})?;

				ensure!(
					!entry.default_name.is_empty(),
					"Missing provider name in post-upgrade check"
				);
			}
		}

		let post_count = ProviderToRegistryEntry::<T>::iter().count() as u64;
		ensure!(post_count == expected, "Post-upgrade count mismatch");

		// Ensure old storage is empty (for non-Paseo chains)
		let old_remaining = v1::ProviderToRegistryEntry::<T>::iter().count();
		ensure!(old_remaining == 0, "Old storage not completely drained");

		log::info!(target: LOG_TARGET, "Migration completed successfully. Migrated {} items", post_count);
		Ok(())
	}
}
