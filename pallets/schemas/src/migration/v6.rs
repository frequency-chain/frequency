use crate::{
	migration::v5,
	pallet::{IntentInfos, NameToMappedEntityIds, SchemaInfos},
	Config, IntentInfo, Pallet, SchemaInfo, SchemaVersionId, SCHEMA_STORAGE_VERSION,
};
#[cfg(feature = "try-runtime")]
use alloc::vec::Vec;
use common_primitives::schema::{IntentId, MappedEntityIdentifier, SchemaId};
use core::marker::PhantomData;
use frame_support::{
	pallet_prelude::{Get, GetStorageVersion, Weight},
	traits::{OnRuntimeUpgrade},
	LOG_TARGET,
};
use numtoa::NumToA;
#[cfg(feature = "try-runtime")]
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "try-runtime")]
use frame_support::{ensure};
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

fn convert_from_old(id: SchemaId, old_info: &v5::SchemaInfo) -> crate::SchemaInfo {
	SchemaInfo {
		intent_id: id as IntentId,
		model_type: old_info.model_type,
		payload_location: old_info.payload_location,
		settings: old_info.settings,
	}
}

fn convert_to_intent(old_info: &v5::SchemaInfo) -> IntentInfo {
	IntentInfo { payload_location: old_info.payload_location, settings: old_info.settings }
}

/// Migrate from v5 to v6.
pub struct MigrateV5ToV6<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateV5ToV6<T> {
	fn on_runtime_upgrade() -> Weight {
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		let current_version = Pallet::<T>::in_code_storage_version();
		log::info!(target: LOG_TARGET, "onchain_version={:?}, current_version={:?}", onchain_version, current_version);
		if SCHEMA_STORAGE_VERSION != current_version {
			log::error!(target: LOG_TARGET, "storage version mismatch: expected {:?}, found {:?}", SCHEMA_STORAGE_VERSION, current_version);
			return T::DbWeight::get().reads(1)
		}
		if onchain_version < current_version {
			log::info!(target: LOG_TARGET, "Migrating from v5 to v6");
			let mut reads = 0;
			let mut writes = 0;
			let mut schemas_migrated = 0;
			let mut intents_created = 0;
			let mut new_names = 0;
			let mut old_names = 0;

			// Translate schema infos and create corresponding intent infos.
			SchemaInfos::<T>::translate(|schema_id, old_schema_info: v5::SchemaInfo| {
				reads += 1;
				writes += 2;
				IntentInfos::<T>::insert(
					schema_id as IntentId,
					convert_to_intent(&old_schema_info),
				);
				intents_created += 1;
				schemas_migrated += 1;
				Some(convert_from_old(schema_id, &old_schema_info))
			});

			// Remove old schema name map and create new entity name map, appending version id to names
			v5::SchemaNameToIds::<T>::translate(
				|schema_id, old_schema_name, version_id: SchemaVersionId| {
					reads += 1;
					let mut new_descriptor = old_schema_name.clone();
					match new_descriptor.try_push('-' as u8) {
						Ok(_) => (),
						Err(_) => return None,
					};
					let mut buf = [0u8; 30];
					for (index, id) in version_id.ids.iter().enumerate() {
						let mut index_str = (index + 1).numtoa(10, &mut buf).to_vec();
						let mut name = new_descriptor.clone();
						match name.try_append(&mut index_str) {
							Ok(_) => (),
							Err(_) => return None,
						};
						NameToMappedEntityIds::<T>::insert(
							&schema_id,
							name,
							MappedEntityIdentifier::Intent(*id as IntentId),
						);
						writes += 1;
						new_names += 1;
					}
					writes += 1;
					old_names += 1;
					None
				},
			);

			log::info!(target: LOG_TARGET,
				"Migration complete. \
				schemas_migrated={:?}, intents_created={:?}, \
				new_names={:?}, old_names={:?}",
				schemas_migrated, intents_created, new_names, old_names,
			);

			// Set storage version to current version
			SCHEMA_STORAGE_VERSION.put::<Pallet<T>>();
			log::info!(target: LOG_TARGET, "Schemas storage migrated to version {:?}. reads={:?}, writes={:?}", current_version, reads, writes);

			T::DbWeight::get().reads_writes(reads, writes)
		} else {
			log::info!(target: LOG_TARGET,
			"Migration did not execute; storage version is already up to date. \
			onchain_version={:?}, current_version={:?}",
				onchain_version, current_version,
			);
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		let count = v5::SchemaInfos::<T>::iter_values().count();
		log::info!(target: LOG_TARGET, "old schemas count={:?}", count);
		Ok((count as u32).encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
		let old_count = <u32>::decode(&mut &state[..])
			.map_err(|_| TryRuntimeError::Other("unable to decode old schema count"))?;
		let new_count = SchemaInfos::<T>::iter_values().count() as u32;
		log::info!(target: LOG_TARGET, "new schemas count={:?}", new_count);

		ensure!(old_count == new_count, "schema/intent count mismatch");
		Ok(())
	}
}
