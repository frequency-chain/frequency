#[cfg(feature = "try-runtime")]
use crate::types::SCHEMA_STORAGE_VERSION;
use crate::{
	pallet::{SchemaInfos, SchemaNameToIds},
	Config, Pallet, SchemaId, SchemaName, LOG_TARGET,
};
use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight};
use log;
use sp_runtime::Saturating;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

/// get known schema names for mainnet
#[cfg(feature = "frequency")]
pub fn get_known_schemas() -> BTreeMap<SchemaId, Vec<u8>> {
	BTreeMap::from([
		(1, b"dsnp.tombstone".to_vec()),
		(2, b"dsnp.broadcast".to_vec()),
		(3, b"dsnp.reply".to_vec()),
		(4, b"dsnp.reaction".to_vec()),
		(5, b"dsnp.update".to_vec()),
		(6, b"dsnp.profile".to_vec()),
		(7, b"dsnp.public-key".to_vec()),
		(8, b"dsnp.public-follows".to_vec()),
		(9, b"dsnp.private-follows".to_vec()),
		(10, b"dsnp.private-connections".to_vec()),
	])
}
/// get known schema names for rococo
#[cfg(not(feature = "frequency"))]
pub fn get_known_schemas() -> BTreeMap<SchemaId, Vec<u8>> {
	BTreeMap::from([
		(1, b"dsnp.tombstone".to_vec()),
		(2, b"dsnp.broadcast".to_vec()),
		(3, b"dsnp.reply".to_vec()),
		(4, b"dsnp.reaction".to_vec()),
		(5, b"dsnp.profile".to_vec()),
		(6, b"dsnp.update".to_vec()),
		(18, b"dsnp.public-key".to_vec()),
		(13, b"dsnp.public-follows".to_vec()),
		(14, b"dsnp.private-follows".to_vec()),
		(15, b"dsnp.private-connections".to_vec()),
	])
}

/// migration to v2 implementation
pub struct MigrateToV3<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateToV3<T> {
	fn on_runtime_upgrade() -> Weight {
		migrate_to_v3::<T>()
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		log::info!(target: LOG_TARGET, "Running pre_upgrade...");
		let count = SchemaInfos::<T>::iter().count() as u32;
		log::info!(target: LOG_TARGET, "Finish pre_upgrade for {:?}", count);
		Ok(count.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_: Vec<u8>) -> Result<(), TryRuntimeError> {
		log::info!(target: LOG_TARGET, "Running post_upgrade...");
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		assert_eq!(onchain_version, SCHEMA_STORAGE_VERSION);
		log::info!(target: LOG_TARGET, "Finished post_upgrade");
		Ok(())
	}
}

/// migrating to v3
pub fn migrate_to_v3<T: Config>() -> Weight {
	log::info!(target: LOG_TARGET, "Running storage migration...");
	let onchain_version = Pallet::<T>::on_chain_storage_version();
	let current_version = Pallet::<T>::current_storage_version();
	log::info!(target: LOG_TARGET, "onchain_version= {:?}, current_version={:?}", onchain_version, current_version);
	let each_layer_access: u64 = 33 * 16;

	if onchain_version < 3 {
		let known_schemas = get_known_schemas();
		let mut reads = 1u64;
		let mut writes = 0u64;
		let mut bytes = 0u64;
		for (schema_id, schema_name) in known_schemas.iter() {
			reads.saturating_inc();

			if let Some(schema) = SchemaInfos::<T>::get(&schema_id) {
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

									let _ = schema_version_id.add::<T>(*schema_id);
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
			}
		}

		// Set storage version to `3`.
		StorageVersion::new(3).put::<Pallet<T>>();
		writes.saturating_inc();

		log::info!(target: LOG_TARGET, "Storage migrated to version 3  read={:?}, write={:?}, bytes={:?}", reads, writes, bytes);
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
