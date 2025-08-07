use crate::{migration::v1, Config, Pallet, ProviderToRegistryEntry, STORAGE_VERSION};
use alloc::vec;
use common_primitives::{
	msa::{ProviderId, ProviderRegistryEntry},
	utils::{get_chain_type_by_genesis_hash, DetectedChainType},
};
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::Saturating;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

const LOG_TARGET: &str = "runtime::provider";

/// Get known provider ids from paseo and mainnet
pub fn get_known_provider_ids<T: Config>() -> vec::Vec<ProviderId> {
	let genesis_block: BlockNumberFor<T> = 0u32.into();
	let genesis = <frame_system::Pallet<T>>::block_hash(genesis_block);
	let detected_chain = get_chain_type_by_genesis_hash(&genesis.encode()[..]);
	log::info!(target: LOG_TARGET, "Detected Chain is {:?}", detected_chain);
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
		log::info!(target: LOG_TARGET, "onchain_version= {:?}, current_version={:?}", onchain_version, current_version);
		if STORAGE_VERSION != current_version {
			log::error!(target: LOG_TARGET, "Storage version mismatch. Expected: {:?}, Found: {:?}", STORAGE_VERSION, current_version);
			return T::DbWeight::get().reads(1)
		}
		if onchain_version < 2 {
			log::info!(target: LOG_TARGET, "Migrating v1 ProviderToRegistryEntry to updated ProviderRegistryEntry...");

			// Count items in OLD storage using storage alias
			let total_count = v1::ProviderToRegistryEntry::<T>::iter().count();
			log::info!(target: LOG_TARGET, "Total items in OLD ProviderToRegistryEntry storage: {}", total_count);

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

			log::info!(target: LOG_TARGET, "Migration completed: {} items migrated", migrated_count);

			// Set storage version to `2`.
			StorageVersion::new(2).put::<Pallet<T>>();
			log::info!(target: LOG_TARGET, "MSA Storage migrated to version 2  read={:?}, write={:?}, bytes={:?}", reads, writes, bytes);
			// compute the weight
			let weights = T::DbWeight::get().reads_writes(reads, writes).add_proof_size(bytes);
			log::info!(target: LOG_TARGET, "Migration Calculated weights={:?}",weights);
			weights
		} else {
			log::info!(
				target: LOG_TARGET,
				"Migration did not execute. This probably should be removed onchain:{:?}, current:{:?}",
				onchain_version,
				current_version
			);
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<vec::Vec<u8>, TryRuntimeError> {
		log::info!(target: LOG_TARGET, "Running pre_upgrade...");
		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		if on_chain_version >= 2 {
			return Ok(vec::Vec::new())
		}

		// Check OLD storage using storage alias
		let old_count = v1::ProviderToRegistryEntry::<T>::iter().count();
		log::info!(target: LOG_TARGET, "Found {} items in OLD storage format", old_count);

		let genesis_block: BlockNumberFor<T> = 0u32.into();
		let genesis = <frame_system::Pallet<T>>::block_hash(genesis_block);
		log::info!(target: LOG_TARGET, "Found genesis... {:?}", genesis);
		let detected_chain = get_chain_type_by_genesis_hash(&genesis.encode()[..]);
		log::info!(target: LOG_TARGET,"Detected Chain is {:?}", detected_chain);

		// Store the old count for verification in post_upgrade
		Ok((old_count as u64).encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(pre: vec::Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: LOG_TARGET, "Running post_upgrade...");
		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		if on_chain_version > 2 {
			return Ok(())
		}
		assert_eq!(on_chain_version, STORAGE_VERSION);
		let old_count: u64 = Decode::decode(&mut &pre[..])
			.map_err(|_| TryRuntimeError::Other("Failed to decode pre_upgrade data"))?;
		let new_count = ProviderToRegistryEntry::<T>::iter().count() as u64;
		log::info!(target: LOG_TARGET, "Old count: {}, New count: {}", old_count, new_count);
		if old_count != new_count {
			log::error!(target: LOG_TARGET, "Migration count mismatch: old={}, new={}", old_count, new_count);
			return Err(TryRuntimeError::Other("Migration count mismatch"))
		}
		let known_providers = get_known_provider_ids::<T>();
		for id in known_providers {
			let entry = ProviderToRegistryEntry::<T>::get(id).ok_or_else(|| {
				log::error!(target: LOG_TARGET, "Missing provider entry for provider id: {:?}", id);
				TryRuntimeError::Other("Missing provider entry in post-upgrade check")
			})?;

			if entry.default_name.is_empty() {
				log::error!(target: LOG_TARGET, "Missing provider name for provider id: {:?}", id);
				return Err(TryRuntimeError::Other("Missing provider name in post-upgrade check"));
			}
		}

		log::info!(target: LOG_TARGET, "Migration completed successfully");
		Ok(())
	}
}
