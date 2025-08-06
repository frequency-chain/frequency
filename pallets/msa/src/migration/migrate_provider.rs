use crate::{Config, Pallet, STORAGE_VERSION};
use alloc::vec;
use common_primitives::{
	msa::ProviderId,
	utils::{get_chain_type_by_genesis_hash, DetectedChainType},
};
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;
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
		Weight::zero()
		//migrate_provider_to_registry_entry::<T>()
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
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		assert_eq!(onchain_version, STORAGE_VERSION);
		Ok(())
	}
}
