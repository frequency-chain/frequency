#![cfg_attr(not(feature = "std"), no_std)]

use core;

use frame_support::{dispatch::DispatchResult, ensure, traits::Get, BoundedVec};
use sp_std::{convert::TryInto, vec::Vec};
extern crate core;
use sp_runtime::{
	DispatchError,
};
use frame_support::{
	dispatch::DispatchResult, ensure, traits::Get, BoundedVec,
};

pub use pallet::*;
pub use weights::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use pallet::*;
pub mod weights;

pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use common_primitives::schema::{SchemaId, SchemaResponse};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;

		// Maximum length of Schema field name
		#[pallet::constant]
		type MinSchemaSizeBytes: Get<u32>;

		// Maximum length of Schema field name
		#[pallet::constant]
		type SchemaMaxBytesBoundedVecLimit: Get<u32>;

		// Maximum number of schemas that can be registered
		#[pallet::constant]
		type MaxSchemaRegistrations: Get<SchemaId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Emitted when a schemas is registered. [who, schemas id]
		SchemaRegistered(T::AccountId, SchemaId),
	}

	#[derive(PartialEq)] // for testing
	#[pallet::error]
	pub enum Error<T> {
		/// Schema is malformed
		InvalidSchema,
		/// The maximum number of schemas is stored in the database.
		TooManySchemas,
		/// The schema exceeds the maximum length allowed
		ExceedsMaxSchemaBytes,
		/// The schema is less than the minimum length allowed
		LessThanMinSchemaBytes,
		/// Schema does not exist
		NoSuchSchema,
		/// Error is converting to string
		StringConversionError,
		/// Error in Deserialization
		DeserializationError,
		/// Error in Serialization
		SerializationError,
		/// SchemaCount was attempted to overflow max, means MaxSchemaRegistrations is too big
		SchemaCountOverflow,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn schema_count)]
	pub(super) type SchemaCount<T: Config> = StorageValue<_, SchemaId, ValueQuery>;

	// storage for message schemas hashes
	#[pallet::storage]
	#[pallet::getter(fn get_schema)]
	pub(super) type Schemas<T: Config> =
	StorageMap<_, Twox64Concat, SchemaId, BoundedVec<u8, T::SchemaMaxBytesBoundedVecLimit>, ValueQuery>;

	////////////////////  TESTING
	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub initial_max_schema_size: u32,
	}

	#[cfg(feature = "std")]
	impl sp_std::default::Default for GenesisConfig {
		fn default() -> Self {
			Self { initial_max_schema_size: 1024 }
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			SchemaMaxBytes::<T>::put(self.initial_max_schema_size.clone());
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(< T as Config >::WeightInfo::register_schema(schema.len() as u32))]
		pub fn register_schema(origin: OriginFor<T>, schema: BoundedVec<u8, T::SchemaMaxBytesBoundedVecLimit>) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			ensure!(schema.len() > T::MinSchemaSizeBytes::get() as usize, Error::<T>::LessThanMinSchemaBytes);
			ensure!(schema.len() < Self::get_schema_max_bytes() as usize, Error::<T>::ExceedsMaxSchemaBytes);

			let cur_count = Self::schema_count();
			ensure!(cur_count < T::MaxSchemaRegistrations::get(), <Error<T>>::TooManySchemas);
			let schema_id = cur_count.checked_add(1).ok_or(<Error<T>>::SchemaCountOverflow)?;

			Self::add_schema(schema.clone(), schema_id)?;

			Self::deposit_event(Event::SchemaRegistered(sender, schema_id));
			Ok(())
		}

		#[pallet::weight(30_000)]
		pub fn set_max_schema_bytes(origin: OriginFor<T>, max_size: u32) -> DispatchResult {
			ensure_signed(origin.clone())?;
			SchemaMaxBytes::<T>::set(max_size);
			Ok(())
		}

	}

	impl<T: Config> Pallet<T> {
		fn add_schema(schema: BoundedVec<u8, T::SchemaMaxBytesBoundedVecLimit>, schema_id: SchemaId) -> DispatchResult {
			<SchemaCount<T>>::set(schema_id);
			<Schemas<T>>::insert(schema_id, bounded_fields);

			Ok(())
		}

		pub fn get_latest_schema_id() -> Option<SchemaId> {
			Some(Self::schema_count())
		}

		pub fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponse> {
			if let Some(schema) = Self::get_schema(schema_id) {
				return Some(SchemaResponse { schema_id, data: schema.into_inner() })
			}
			None
		}

		pub fn require_valid_schema_size(
			schema: Vec<u8>,
		) -> Result<BoundedVec<u8, T::MaxSchemaSizeBytes>, Error<T>> {
			let bounded_fields: BoundedVec<u8, T::MaxSchemaSizeBytes> =
				schema.try_into().map_err(|()| Error::<T>::ExceedsMaxSchemaBytes)?;
			ensure!(
				bounded_fields.len() >= T::MinSchemaSizeBytes::get() as usize,
				Error::<T>::LessThanMinSchemaBytes
			);
			Ok(bounded_fields)
		}

		pub fn calculate_schema_cost(schema: Vec<u8>) -> Weight {
			let schema_len = schema.len() as u32;
			let schema_weight = T::WeightInfo::register_schema(schema_len);
			schema_weight
		}
	}
}
