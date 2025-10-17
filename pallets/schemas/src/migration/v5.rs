use crate::{
	migration::v4,
	pallet::{CurrentIntentIdentifierMaximum, IntentInfos, NameToMappedEntityIds, SchemaInfos},
	Config, IntentInfo, Pallet, SchemaInfo, SchemaVersionId, SCHEMA_NAME_BYTES_MAX,
	SCHEMA_STORAGE_VERSION,
};
use alloc::{format, vec::Vec};
use common_primitives::schema::{IntentId, MappedEntityIdentifier, SchemaId, SchemaStatus};
use core::marker::PhantomData;
#[cfg(feature = "try-runtime")]
use frame_support::ensure;
use frame_support::{
	pallet_prelude::{Get, GetStorageVersion, Weight},
	traits::{Len, UncheckedOnRuntimeUpgrade},
	BoundedVec,
};
#[cfg(feature = "try-runtime")]
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

const LOG_TARGET: &str = "pallet::schemas::migration::v5";

fn convert_from_old(id: SchemaId, old_info: &v4::SchemaInfo) -> crate::SchemaInfo {
	SchemaInfo {
		intent_id: id as IntentId,
		model_type: old_info.model_type,
		payload_location: old_info.payload_location,
		settings: old_info.settings,
		status: SchemaStatus::Active,
	}
}

fn convert_to_intent(old_info: &v4::SchemaInfo) -> IntentInfo {
	IntentInfo { payload_location: old_info.payload_location, settings: old_info.settings }
}

fn append_or_overlay<S1: Get<u32>, S2: Get<u32>>(
	target: &mut BoundedVec<u8, S1>,
	source: &[u8],
	protocol: &BoundedVec<u8, S2>,
) -> Result<(), ()> {
	let max_len = (SCHEMA_NAME_BYTES_MAX as usize - protocol.len() - 1).min(S1::get() as usize);
	let additional = source.len();
	let len = target.len();
	if len + additional > max_len {
		target.truncate(max_len - additional);
	}
	target.try_append(&mut source.to_vec())
}

/// Migrate from v4 to v5.
pub struct InnerMigrateV4ToV5<T: Config>(PhantomData<T>);

impl<T: Config> UncheckedOnRuntimeUpgrade for InnerMigrateV4ToV5<T> {
	fn on_runtime_upgrade() -> Weight {
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		let current_version = Pallet::<T>::in_code_storage_version();
		log::info!(target: LOG_TARGET, "onchain_version={:?}, current_version={:?}", onchain_version, current_version);
		if SCHEMA_STORAGE_VERSION != current_version {
			log::error!(target: LOG_TARGET, "storage version mismatch: expected {:?}, found {:?}", SCHEMA_STORAGE_VERSION, current_version);
			return T::DbWeight::get().reads(1)
		}
		if onchain_version < current_version {
			log::info!(target: LOG_TARGET, "Migrating from v4 to v5");
			let mut reads = 0;
			let mut writes = 0;
			let mut schemas_migrated = 0;
			let mut intents_created = 0;
			let mut new_names = 0;
			let mut old_names = 0;
			let mut max_intent_id = 0u16;

			// Translate schema infos and create corresponding intent infos.
			SchemaInfos::<T>::translate(|schema_id, old_schema_info: v4::SchemaInfo| {
				reads += 1;
				writes += 2;
				max_intent_id = max_intent_id.max(schema_id);
				IntentInfos::<T>::insert(
					schema_id as IntentId,
					convert_to_intent(&old_schema_info),
				);
				intents_created += 1;
				schemas_migrated += 1;
				Some(convert_from_old(schema_id, &old_schema_info))
			});

			// Remove old schema name map and create new entity name map, appending version id to names
			v4::SchemaNameToIds::<T>::translate(
				|schema_id, old_schema_name, version_id: SchemaVersionId| {
					reads += 1;
					old_names += 1;
					let max_version = version_id.ids.len();
					for (index, id) in version_id.ids.iter().enumerate() {
						let version = index + 1;
						let mut name = old_schema_name.clone();
						// Let the latest schema version be the original name, and postfix previous versions
						// with "-<version>"
						if version < max_version {
							let version_str = Vec::<u8>::from(format!("-{}", version).as_bytes());
							match append_or_overlay(&mut name, &version_str, &schema_id) {
								Ok(_) => (),
								Err(e) => {
									log::error!(target: LOG_TARGET, "{:?} unable to append id {:?} to name: {:?}", e, version_str, name);
									return Some(version_id)
								},
							};
						}
						NameToMappedEntityIds::<T>::insert(
							&schema_id,
							name,
							MappedEntityIdentifier::Intent(*id as IntentId),
						);
						writes += 1;
						new_names += 1;
					}
					writes += 1;
					None
				},
			);

			CurrentIntentIdentifierMaximum::<T>::set(max_intent_id);

			log::info!(target: LOG_TARGET,
				"Migration complete. \
				schemas_migrated={:?}, intents_created={:?}, \
				new_names={:?}, old_names={:?}",
				schemas_migrated, intents_created, new_names, old_names,
			);

			// Set storage version to current version
			SCHEMA_STORAGE_VERSION.put::<Pallet<T>>();
			let weight = T::DbWeight::get().reads_writes(reads, writes);
			log::info!(target: LOG_TARGET, "Schemas storage migrated to version {:?}. reads={:?}, writes={:?}, proof_size={:?}, ref_time={:?}", current_version, reads, writes, weight.proof_size(), weight.ref_time());

			weight
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
		let schemas_count = v4::SchemaInfos::<T>::iter_values().count() as u32;
		let mut named_schemas_count = 0u32;
		v4::SchemaNameToIds::<T>::iter_values().for_each(|schema_versions| {
			named_schemas_count += schema_versions.ids.len() as u32;
		});
		Ok((schemas_count, named_schemas_count).encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
		let (old_count, old_named_schemas_count) = <(u32, u32)>::decode(&mut &state[..])
			.map_err(|_| TryRuntimeError::Other("unable to decode old schema count"))?;
		let new_schema_count = SchemaInfos::<T>::iter_values().count() as u32;
		let new_intent_count = IntentInfos::<T>::iter_values().count() as u32;
		let new_names_count = NameToMappedEntityIds::<T>::iter_values().count() as u32;
		let remaining_schema_names = v4::SchemaNameToIds::<T>::iter_values().count() as u32;

		ensure!(remaining_schema_names == 0, "named schemas still exist");
		ensure!(old_count == new_schema_count, "schema count mismatch");
		ensure!(old_count == new_intent_count, "intent count mismatch");
		ensure!(old_named_schemas_count == new_names_count, "name count mismatch");
		Ok(())
	}
}

/// [`UncheckedOnRuntimeUpgrade`] implementation [`InnerMigrateV4ToV5`] wrapped in a
/// [`VersionedMigration`](frame_support::migrations::VersionedMigration), which ensures that:
/// - The migration only runs once when the on-chain storage version is 4
/// - The on-chain storage version is updated to `5` after the migration executes
/// - Reads/Writes from checking/settings the on-chain storage version are accounted for
pub type MigrateV4ToV5<T> = frame_support::migrations::VersionedMigration<
	4, // The migration will only execute when the on-chain storage version is 4
	5, // The on-chain storage version will be set to 5 after the migration is complete
	InnerMigrateV4ToV5<T>,
	crate::pallet::Pallet<T>,
	<T as frame_system::Config>::DbWeight,
>;
