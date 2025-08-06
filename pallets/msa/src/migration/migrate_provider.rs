use crate::{Config, Pallet, ProviderToRegistryEntry, STORAGE_VERSION};
use alloc::vec;
use common_primitives::{
	msa::{ProviderId, ProviderRegistryEntry},
	utils::{get_chain_type_by_genesis_hash, DetectedChainType},
};
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::Saturating;
#[cfg(feature = "try-runtime")]
use sp_runtime::{TryRuntimeError, Vec};

const LOG_TARGET: &str = "runtime::provider";

/// Get known provider ids from paseo and mainnet
pub fn get_known_provider_ids<T: Config>() -> Vec<ProviderId> {
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
		_ => Vec::new(),
	}
}

/// Old provider registry entry structure.
#[derive(Encode, Decode)]
pub struct OldProviderRegistryEntry<NameSize: Get<u32>> {
	/// name of the provider.
	pub provider_name: BoundedVec<u8, NameSize>,
}

/// migrate `ProviderToRegistryEntry` translating to new `ProviderRegistryEntry`
pub struct MigrateProviderToRegistryEntry<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateProviderToRegistryEntry<T> {
	fn on_runtime_upgrade() -> Weight {
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		let current_version = Pallet::<T>::in_code_storage_version();
		log::info!(target: LOG_TARGET, "onchain_version= {:?}, current_version={:?}", onchain_version, current_version);
		if STORAGE_VERSION != current_version {
			log::error!(target: LOG_TARGET, "Storage version mismatch. Expected: {:?}, Found: {:?}", STORAGE_VERSION, current_version);
			return T::DbWeight::get().reads(1)
		}
		if onchain_version < current_version {
			log::info!(target: LOG_TARGET, "Migrating ProviderToRegistryEntry to updated ProviderRegistryEntry...");
			let mut reads = 1u64;
			let mut writes = 0u64;
			let mut bytes = 0u64;
			for (provider_id, old_value_encoded) in
				ProviderToRegistryEntry::<T>::drain().map(|(k, v)| (k, v.encode()))
			{
				let old: OldProviderRegistryEntry<T::MaxProviderNameSize> =
					match Decode::decode(&mut &old_value_encoded[..]) {
						Ok(val) => val,
						Err(_) => {
							log::warn!(target: LOG_TARGET, "Failed to decode old value for provider {:?}", provider_id);
							continue;
						},
					};

				// Build new default value with old provider name.
				let new = ProviderRegistryEntry {
					default_name: old.provider_name,
					default_logo_250_100_png_cid: BoundedVec::default(),
					localized_logo_250_100_png_cids: BoundedBTreeMap::default(),
					localized_names: BoundedBTreeMap::default(),
				};

				ProviderToRegistryEntry::<T>::insert(provider_id, new);
				reads.saturating_inc();
				writes.saturating_inc();
				bytes += old_value_encoded.len() as u64;
			}
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
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		log::info!(target: LOG_TARGET, "Running pre_upgrade...");
		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		let genesis_block: BlockNumberFor<T> = 0u32.into();
		let genesis = <frame_system::Pallet<T>>::block_hash(genesis_block);
		if on_chain_version >= 2 {
			return Ok(Vec::new())
		}
		log::info!(target: LOG_TARGET, "Found genesis... {:?}", genesis);
		let detected_chain = get_chain_type_by_genesis_hash(&genesis.encode()[..]);
		log::info!(target: LOG_TARGET,"Detected Chain is {:?}", detected_chain);
		Ok(Vec::new())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_: Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: LOG_TARGET, "Running post_upgrade...");
		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		if on_chain_version > 2 {
			return Ok(())
		}
		assert_eq!(on_chain_version, STORAGE_VERSION);
		let known_providers = get_known_provider_ids::<T>();
		for id in known_providers {
			let entry = ProviderToRegistryEntry::<T>::get(id).ok_or_else(|| {
				TryRuntimeError::Other("Missing provider entry in post upgrade check")
			})?;
			// Optionally check the name isn't empty
			if entry.default_name.is_empty() {
				log::error!(target: LOG_TARGET, "Missing provider name for provider id: {:?}", id);
				return Err(TryRuntimeError::Other("Missing provider name in post upgrade check"));
			}
		}
		log::info!(target: LOG_TARGET, "Migration completed successfully. Storage version is now {:?}", STORAGE_VERSION);
		Ok(())
	}
}
