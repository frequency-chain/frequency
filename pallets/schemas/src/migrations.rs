//! Migrations for the schemas pallet.
//! Following migrations are required:
//! - Adding settings to the Schema struct
//! Note: Post migration, this file should be deleted.
use super::*;
use codec::{Decode, Encode, MaxEncodedLen};
use common_primitives::schema::{ModelType, PayloadLocation, SchemaSettings};
use frame_support::{
	pallet_prelude::Weight,
	traits::{Get, OnRuntimeUpgrade, StorageVersion},
	BoundedVec,
};
use scale_info::TypeInfo;
use sp_std::marker::PhantomData;

/// Migrations for the schemas pallet.
/// Following migrations are required:
/// - Adding settings to the Schema struct
/// Note: Post migration, this file should be deleted.
pub mod v0 {
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

// ==============================================
//        RUNTIME STORAGE MIGRATION: Schemas
// ==============================================
/// Schema migration to v1 for pallet-stateful-storage
/// This struct derives OnRuntimeUpgrade trait which is used to run the migration (check runtime)
pub struct SchemaMigrationToV1<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for SchemaMigrationToV1<T> {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		let weight = v0::pre_migrate_schemas_to_v1::<T>();
		log::info!("pre_upgrade weight: {:?}", weight);
		Ok(Vec::new())
	}

	// try-runtime migration code
	#[cfg(not(feature = "try-runtime"))]
	fn on_runtime_upgrade() -> Weight {
		v0::migrate_schemas_to_v1::<T>()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
		let weight = v0::post_migrate_schemas_to_v1::<T>();
		log::info!("post_upgrade weight: {:?}", weight);
		Ok(())
	}
}
// ==============================================
//       END RUNTIME STORAGE MIGRATION: Schemas
// ==============================================
