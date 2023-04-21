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

/// v0: Initial version
/// Notes:
/// - `OldSchema` is a copy of the original `Schema` struct with the `settings` field removed
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
}

/// v1: Adding settings to the Schema struct
/// Notes:
/// - `Schema` is a copy of the original `Schema` struct with the `settings` field added
pub mod v1 {
	use super::*;

	/// Runs the actual migration when runtime upgrade is performed
	pub fn migrate<T: Config>() -> Weight {
		let mut weight: Weight = Weight::zero();

		if StorageVersion::get::<Pallet<T>>() < 1 {
			<Schemas<T>>::translate_values(
				|old_schema: v0::OldSchema<T::SchemaModelMaxBytesBoundedVecLimit>| {
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
pub struct Migration<T: Config>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for Migration<T> {
	fn on_runtime_upgrade() -> Weight {
		let version = StorageVersion::get::<Pallet<T>>();
		let mut weight: Weight = Weight::zero();

		if version < 1 {
			weight = weight.saturating_add(v1::migrate::<T>());
		}

		weight
	}
}
// ==============================================
//       END RUNTIME STORAGE MIGRATION: Schemas
// ==============================================

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::mock::{Test as T, *};

	#[test]
	fn no_migrations_means_zero_weight() {
		new_test_ext().execute_with(|| {
			StorageVersion::new(100).put::<Pallet<T>>();
			let weight = Migration::<T>::on_runtime_upgrade();
			assert_eq!(weight, Weight::zero());
		});
	}

	#[test]
	fn v1_can_migrate() {
		new_test_ext().execute_with(|| {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 0);
			v1::migrate::<T>();
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 1);
		});
	}
}
