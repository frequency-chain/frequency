//! Migrations for the schemas pallet.
//! Following migrations are required:
//! - Adding settings to the Schema struct
//! Note: Post migration, this file should be deleted.
use crate::{types::Schema, *};
use common_primitives::schema::SchemaSettings;
use frame_support::pallet_prelude::Weight;

pub fn migrate_schemas_with_additional_settings<T: Config>() -> Weight {
	let mut weight: Weight = Weight::zero();
	//translate_values of Schemas storage add settings: SchemaSettings::all_disabled()
	//to all schemas
	<Schemas<T>>::translate_values(|old_schema: Schema<T::SchemaModelMaxBytesBoundedVecLimit>| {
		weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
		Some(Schema {
			model_type: old_schema.model_type,
			model: old_schema.model,
			payload_location: old_schema.payload_location,
			settings: SchemaSettings::all_disabled(),
		})
	});
	weight
}
