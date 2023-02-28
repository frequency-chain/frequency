//! Migrations for the schemas pallet.
//! Following migrations are required:
//! - Adding settings to the Schema struct
//! Note: Post migration, this file should be deleted.
use crate::{types::Schema, *};
use codec::{Decode, Encode, MaxEncodedLen};
use common_primitives::schema::{ModelType, PayloadLocation, SchemaSettings};
use frame_support::{
	pallet_prelude::Weight,
	traits::{Get, StorageVersion},
	BoundedVec,
};
use scale_info::TypeInfo;

/// Migrations for the schemas pallet.
/// Following migrations are required:
/// - Adding settings to the Schema struct
/// Note: Post migration, this file should be deleted.
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

	/// translate schemas and return the weight to test the migration
	#[cfg(feature = "try-runtime")]
	pub fn pre_migrate_schemas_to_v1<T: Config>() -> Weight {
		migrate_to_v1::<T>()
	}

	/// Runs the actual migration when runtime upgrade is performed
	pub fn migrate_schemas_to_v1<T: Config>() -> Weight {
		migrate_to_v1::<T>()
	}

	/// post migration check
	#[cfg(feature = "try-runtime")]
	pub fn post_migrate_schemas_to_v1<T: Config>() -> Weight {
		migrate_to_v1::<T>()
	}

	fn migrate_to_v1<T: Config>() -> Weight {
		let mut weight: Weight = Weight::zero();

		if StorageVersion::get::<Pallet<T>>() < 1 {
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
			StorageVersion::new(1).put::<Pallet<T>>();
		}

		weight
	}
}
