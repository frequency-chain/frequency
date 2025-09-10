use crate::{migration::v1, Config, Pallet, ProviderToRegistryEntry, STORAGE_VERSION};
use common_primitives::msa::ProviderRegistryEntry;
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight};
use sp_runtime::Saturating;
#[cfg(feature = "try-runtime")]
mod try_runtime_imports {
	pub use alloc::vec;
	pub use common_primitives::{
		msa::ProviderId,
		utils::{get_chain_type_by_genesis_hash, DetectedChainType},
	};
	pub use frame_system::pallet_prelude::BlockNumberFor;
	pub use sp_runtime::TryRuntimeError;
}
#[cfg(feature = "try-runtime")]
use try_runtime_imports::*;

const LOG_TARGET: &str = "runtime::provider";

/// Get known provider ids from paseo and mainnet
#[cfg(feature = "try-runtime")]
pub fn get_known_provider_ids<T: Config>() -> vec::Vec<ProviderId> {
	let genesis_block: BlockNumberFor<T> = 0u32.into();
	let genesis = <frame_system::Pallet<T>>::block_hash(genesis_block);
	let detected_chain = get_chain_type_by_genesis_hash(&genesis.encode()[..]);
	match detected_chain {
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
		if onchain_version < current_version {
			log::info!(target: LOG_TARGET, "Migrating v1 ProviderToRegistryEntry to updated ProviderRegistryEntry...");

			// Count items in OLD storage using storage alias
			let total_count = v1::ProviderToRegistryEntry::<T>::iter().count();
			log::info!(target: LOG_TARGET, "Total items in OLD ProviderToRegistryEntry storage: {total_count}");
			// We expect to have providers in the old storage
			if total_count == 0 {
				log::error!(target: LOG_TARGET, "No items found in OLD ProviderToRegistryEntry storage, skipping migration.");
				return T::DbWeight::get().reads(1)
			}
			let mut reads = 1u64;
			let mut writes = 0u64;
			let mut bytes = 0u64;
			let mut migrated_count = 0u64;

			log::info!(target: LOG_TARGET, "Starting migration from v1 to v2...");
			// Drain from OLD storage and migrate to NEW storage
			for (provider_id, old_entry) in v1::ProviderToRegistryEntry::<T>::drain() {
				// Build new registry entry with old provider name
				let migrated_provider_entry = ProviderRegistryEntry {
					default_name: old_entry.provider_name.clone(),
					default_logo_250_100_png_cid: BoundedVec::default(),
					localized_logo_250_100_png_cids: BoundedBTreeMap::default(),
					localized_names: BoundedBTreeMap::default(),
				};

				ProviderToRegistryEntry::<T>::insert(provider_id, migrated_provider_entry);
				reads.saturating_inc();
				writes.saturating_inc();
				bytes += old_entry.encoded_size() as u64;
				migrated_count.saturating_inc();
			}

			log::info!(target: LOG_TARGET, "Migration completed: {migrated_count} items migrated");

			// Set storage version to `2`.
			StorageVersion::new(2).put::<Pallet<T>>();
			log::info!(target: LOG_TARGET, "MSA Storage migrated to version 2  read={reads:?}, write={writes:?}, bytes={bytes:?}");
			// compute the weight
			let weights = T::DbWeight::get().reads_writes(reads, writes).add_proof_size(bytes);
			log::info!(target: LOG_TARGET, "Migration Calculated weights={weights:?}");
			weights
		} else {
			log::info!(
				target: LOG_TARGET,
				"Migration did not execute. This can be removed as the storage version is already at 2. \
				onchain_version: {onchain_version:?}, current_version: {current_version:?}",
			);
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<vec::Vec<u8>, TryRuntimeError> {
		log::info!(target: LOG_TARGET, "Running pre_upgrade...");
		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		log::info!(target: LOG_TARGET, "Current on_chain_version: {on_chain_version:?}");
		if on_chain_version >= 2 {
			// Migration already completed, return current count for post_upgrade validation
			let current_count = ProviderToRegistryEntry::<T>::iter().count() as u64;
			log::info!(target: LOG_TARGET, "Migration already completed, current count: {current_count}");
			return Ok(current_count.encode().to_vec());
		}
		log::info!(target: LOG_TARGET, "Current on_chain_version to be upgraded: {on_chain_version:?}");
		// Check OLD storage using storage alias
		let old_count = v1::ProviderToRegistryEntry::<T>::iter().count() as u64;
		log::info!(target: LOG_TARGET, "Found {old_count} items in OLD storage format");

		let genesis_block: BlockNumberFor<T> = 0u32.into();
		let genesis = <frame_system::Pallet<T>>::block_hash(genesis_block);
		log::info!(target: LOG_TARGET, "Found genesis... {genesis:?}");
		let detected_chain = get_chain_type_by_genesis_hash(&genesis.encode()[..]);
		log::info!(target: LOG_TARGET,"Detected Chain is {detected_chain:?}");

		Ok(old_count.encode().to_vec())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(encoded_numbers: vec::Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: LOG_TARGET, "Running post_upgrade...");
		use parity_scale_codec::Decode;
		let expected: u64 =
			u64::decode(&mut encoded_numbers.as_slice()).map_err(|_| "Cannot decode expected")?;
		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		assert_eq!(on_chain_version, STORAGE_VERSION);
		let known_providers = get_known_provider_ids::<T>();
		if known_providers.is_empty() {
			log::error!(target: LOG_TARGET, "No known providers found for post-upgrade check");
			return Err(TryRuntimeError::Other("No known providers found in post-upgrade check"));
		}
		for id in known_providers {
			let entry = ProviderToRegistryEntry::<T>::get(id).ok_or_else(|| {
				log::error!(target: LOG_TARGET, "Missing provider entry for provider id: {id:?}");
				TryRuntimeError::Other("Missing provider entry in post-upgrade check")
			})?;

			if entry.default_name.is_empty() {
				log::error!(target: LOG_TARGET, "Missing provider name for provider id: {id:?}");
				return Err(TryRuntimeError::Other("Missing provider name in post-upgrade check"));
			}
		}
		let post_count = ProviderToRegistryEntry::<T>::iter().count() as u64;
		ensure!(post_count == expected, "Post-upgrade count mismatch: expected");
		log::info!(target: LOG_TARGET, "Migration completed successfully");
		Ok(())
	}
}
