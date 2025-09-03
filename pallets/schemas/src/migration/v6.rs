use crate::{
	migration::v5,
	pallet::{IntentInfos, NameToMappedEntityIds, SchemaInfos},
	Config, IntentInfo, SchemaInfo, SchemaVersionId,
};
use common_primitives::schema::{IntentId, MappedEntityIdentifier, SchemaId};
use core::marker::PhantomData;
use frame_support::{
	pallet_prelude::{Get, Weight},
	traits::UncheckedOnRuntimeUpgrade,
	BoundedVec,
};
use numtoa::NumToA;

fn convert_from_old(id: SchemaId, old_info: &v5::SchemaInfo) -> crate::SchemaInfo {
	SchemaInfo {
		intent_id: id as IntentId,
		model_type: old_info.model_type,
		payload_location: old_info.payload_location,
		settings: old_info.settings,
	}
}

fn convert_to_intent(id: SchemaId, old_info: &v5::SchemaInfo) -> IntentInfo {
	IntentInfo { payload_location: old_info.payload_location, settings: old_info.settings }
}

/// Migrate from v5 to v6.
pub struct MigrateV5ToV6<T: Config>(PhantomData<T>);

impl<T: Config> UncheckedOnRuntimeUpgrade for MigrateV5ToV6<T> {
	fn on_runtime_upgrade() -> Weight {
		let mut reads = 0;
		let mut writes = 0;

		// Translate schema infos and create corresponding intent infos.
		SchemaInfos::<T>::translate(|schema_id, old_schema_info: v5::SchemaInfo| {
			reads += 1;
			writes += 2;
			IntentInfos::<T>::insert(
				schema_id as IntentId,
				convert_to_intent(schema_id, &old_schema_info),
			);
			Some(convert_from_old(schema_id, &old_schema_info))
		});

		// Remove old schema name map and create new entity name map, appending version id to names
		v5::SchemaNameToIds::<T>::translate(
			|schema_id, old_schema_name, version_id: SchemaVersionId| {
				reads += 1;
				let mut new_descriptor = BoundedVec::default(); // = old_schema_name.clone();
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
				}
				writes += 1;
				None
			},
		);

		T::DbWeight::get().reads_writes(reads, writes)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		let count = v5::SchemaInfos::<T>::count();
		count.encode()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
		let old_count = Option::<u32>::decode(&mut &state[..])
			.map_err(|_| TryRuntimeError::Other("unable to decode old schema count"))?;
		let new_count = SchemaInfos::<T>::count();

		match old_count {
			Some(old_count) => {
				ensure!(old_count == new_count, "schema/intent count mismatch");
			},
			None => {
				ensure!(new_count == 0, "schema/intent count mismatch");
			},
		}
		Ok(())
	}
}
