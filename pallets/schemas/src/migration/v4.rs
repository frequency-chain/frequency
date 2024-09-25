use crate::{
	pallet::{SchemaInfos, SchemaNameToIds},
	Config, Pallet, SchemaId, SchemaName, LOG_TARGET,
};
#[cfg(feature = "try-runtime")]
use crate::{types::SCHEMA_STORAGE_VERSION, SchemaVersionId};
use common_primitives::utils::{get_chain_type_by_genesis_hash, DetectedChainType};
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight};
use frame_system::pallet_prelude::BlockNumberFor;
use log;
use sp_runtime::Saturating;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

/// get known schema names for mainnet
pub fn get_known_schemas<T: Config>() -> BTreeMap<SchemaId, (Vec<u8>, bool)> {
	let genesis_block: BlockNumberFor<T> = 0u32.into();
	let genesis = <frame_system::Pallet<T>>::block_hash(genesis_block);

	if let DetectedChainType::FrequencyMainNet =
		get_chain_type_by_genesis_hash(&genesis.encode()[..])
	{
		// only return what needs to change which for this case is only on mainnet
		// (schema_id, name, clear prior versions)
		BTreeMap::from([
			(5, (b"dsnp.update".to_vec(), true)),
			(19, (b"dsnp.update".to_vec(), false)),
			(6, (b"dsnp.profile".to_vec(), true)),
			(7, (b"dsnp.public-key-key-agreement".to_vec(), false)),
			(8, (b"dsnp.public-follows".to_vec(), false)),
			(9, (b"dsnp.private-follows".to_vec(), false)),
			(10, (b"dsnp.private-connections".to_vec(), false)),
		])
	} else {
		BTreeMap::new()
	}
}

/// migration to v4 implementation
pub struct MigrateToV4<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateToV4<T> {
	fn on_runtime_upgrade() -> Weight {
		migrate_to_v4::<T>()
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		log::info!(target: LOG_TARGET, "Running pre_upgrade...");
		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		let genesis_block: BlockNumberFor<T> = 0u32.into();
		let genesis = <frame_system::Pallet<T>>::block_hash(genesis_block);
		if on_chain_version >= 4 {
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
		if on_chain_version > 4 {
			return Ok(())
		}
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		assert_eq!(onchain_version, SCHEMA_STORAGE_VERSION);
		// check to ensure updates took place
		let known_schemas = get_known_schemas::<T>();
		for (schema_id, schema_name) in known_schemas.into_iter() {
			// safe to use unwrap since only used in try_runtime
			let bounded = BoundedVec::try_from(schema_name).unwrap();
			let parsed_name = SchemaName::try_parse::<T>(bounded, true).unwrap();
			let current_versions =
				SchemaNameToIds::<T>::get(parsed_name.namespace, parsed_name.descriptor);

			let mut expected = SchemaVersionId::default();
			let _ = expected.add::<T>(schema_id);

			assert_eq!(
				current_versions, expected,
				"Expected versions did not match for {}",
				schema_id
			);
		}
		log::info!(target: LOG_TARGET, "Finished post_upgrade");
		Ok(())
	}
}

/// migrating to v4
pub fn migrate_to_v4<T: Config>() -> Weight {
	log::info!(target: LOG_TARGET, "Running storage migration...");
	let onchain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::in_code_storage_version();
	log::info!(target: LOG_TARGET, "onchain_version= {:?}, current_version={:?}", onchain_version, current_version);
	let each_layer_access: u64 = 33 * 16;

	if onchain_version < 4 {
		let known_schemas = get_known_schemas::<T>();
		let mut reads = 1u64;
		let mut writes = 0u64;
		let mut bytes = 0u64;

		for (schema_id, (schema_name, should_clear)) in known_schemas.iter() {
			reads.saturating_inc();

			if let Some(mut schema) = SchemaInfos::<T>::get(&schema_id) {
				bytes = bytes.saturating_add(schema.encode().len() as u64);
				bytes = bytes.saturating_add(each_layer_access * 3); // three layers in merkle tree

				match BoundedVec::try_from(schema_name.clone()) {
					Ok(bounded_name) => match SchemaName::try_parse::<T>(bounded_name, true) {
						Ok(parsed_name) => {
							let _ = SchemaNameToIds::<T>::try_mutate(
								parsed_name.namespace,
								parsed_name.descriptor,
								|schema_version_id| -> Result<(), DispatchError> {
									bytes = bytes
										.saturating_add(schema_version_id.encode().len() as u64);

									// clear the old/wrong one in case they exist
									if *should_clear {
										schema_version_id.ids.clear();
									}
									let _ = schema_version_id.add::<T>(*schema_id);

									// set schema as having a name
									schema.has_name = true;
									SchemaInfos::<T>::set(&schema_id, Some(schema));
									writes.saturating_inc();

									Ok(())
								},
							);
							reads.saturating_inc();
							writes.saturating_inc();
						},
						Err(_) => {
							log::error!(target: LOG_TARGET, "Not able to parse the name {:?}", schema_name);
						},
					},
					Err(_) => {
						log::error!(target: LOG_TARGET, "Was not able to get bounded vec {:?}", schema_name);
					},
				}
			} else {
				log::error!(target: LOG_TARGET, "Schema id {:?} does not exist!", schema_id);
			}
		}

		// Set storage version to `4`.
		StorageVersion::new(4).put::<Pallet<T>>();
		writes.saturating_inc();

		log::info!(target: LOG_TARGET, "Storage migrated to version 4  read={:?}, write={:?}, bytes={:?}", reads, writes, bytes);
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
