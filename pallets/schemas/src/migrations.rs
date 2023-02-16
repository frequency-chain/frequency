//! Migrations for the schemas pallet.
//! Following migrations are required:
//! - Adding settings to the Schema struct
//! Note: Post migration, this file should be deleted.
use crate::{types::Schema, *};
use codec::{Decode, Encode, MaxEncodedLen};
use common_primitives::schema::{ModelType, PayloadLocation, SchemaSettings};
use frame_support::{pallet_prelude::Weight, traits::Get, BoundedVec};
use scale_info::TypeInfo;

pub mod v1 {
	use super::*;
	#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
	#[scale_info(skip_type_params(MaxModelSize))]
	/// A structure defining a Schema
	pub struct OldSchema<MaxModelSize>
	where
		MaxModelSize: Get<u32>,
	{
		/// The type of model (AvroBinary, Parquet, etc.)
		pub model_type: ModelType,
		/// Defines the structure of the message payload using model_type
		pub model: BoundedVec<u8, MaxModelSize>,
		/// The payload location
		pub payload_location: PayloadLocation,
	}

	pub fn migrate_schemas_with_additional_settings<T: Config>() -> Weight {
		let mut weight: Weight = Weight::zero();
		<Schemas<T>>::translate_values(
			|old_schema: OldSchema<T::SchemaModelMaxBytesBoundedVecLimit>| {
				weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
				Some(Schema {
					model_type: old_schema.model_type,
					model: old_schema.model,
					payload_location: old_schema.payload_location,
					settings: SchemaSettings::all_disabled(),
				})
			},
		);
		weight
	}
}
