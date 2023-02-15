//! Migrations for the schemas pallet.
//! Following migrations are required:
//! - Schema struct from v1 to v2

use super::*;
use crate::types::{Schema, SchemaV2};
use codec::{Decode, Encode};
use common_primitives::schema::{SchemaId, SchemaSettings};
use frame_support::pallet_prelude::Weight;

fn migrate_schema_to_schema_v2<T: Config>() -> Weight {
	let mut weight: Weight = Weight::zero();
	let schema_count = <Schemas<T>>::iter().count();
	for i in 0..schema_count {
		weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
		let schema_id: SchemaId = i as SchemaId;

		let saved_schema = <Schemas<T>>::get(schema_id);
		if let Some(schema) = saved_schema {
			let schema_v2 = SchemaV2 {
				model_type: schema.model_type,
				model: schema.model,
				payload_location: schema.payload_location,
				settings: SchemaSettings::all_disabled(),
			};
			<SchemasV2<T>>::insert(schema_id, schema_v2);
		}
		<Schemas<T>>::remove(schema_id);
	}
	weight
}
