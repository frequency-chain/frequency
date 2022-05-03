#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{convert::TryInto, vec::Vec};
use frame_support::{BoundedVec, dispatch::DispatchResult, ensure, traits::Get};
use sp_runtime::DispatchError;


#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use common_primitives::schema::SchemaId;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// Maximum length of Schema field name
		#[pallet::constant]
		type MinSchemaSize: Get<u32>;

		// Maximum length of Schema field name
		#[pallet::constant]
		type MaxSchemaSize: Get<u32>;

		// Maximum number of schemas that can be registered
		#[pallet::constant]
		type MaxSchemaRegistrations: Get<SchemaId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Emitted when a schemas is registered. [who, schemas id, block]
		SchemaRegistered(T::AccountId, SchemaId, T::BlockNumber),
	}

	#[derive(PartialEq)] // for testing
	#[pallet::error]
	pub enum Error<T> {
		/// Schema is malformed
		InvalidSchema,
		/// The maximum number of schemas is stored in the database.
		TooManySchemas,
		/// The schemas exceeds the maximum length allowed
		TooLongSchema,
		/// The schemas is less than the minimum length allowed
		TooShortSchema,
		/// Schema does not exist
		NoSuchSchema,
		/// Error is converting to string
		StringConversionError,
		/// Error in Deserialization
		DeserializationError,
		/// Error in Serialization
		SerializationError,
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
	pub(super) type Schemas<T: Config> = StorageMap<_, Twox64Concat, SchemaId, BoundedVec<u8, T::MaxSchemaSize>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight((10_000, Pays::Yes))]
		pub fn register_schema(origin: OriginFor<T>, _schema: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			let cur_count = Self::schema_count();
			ensure!(cur_count < T::MaxSchemaRegistrations::get(), <Error<T>>::TooManySchemas);
			// let schema_id = cur_count.checked_add(1).ok_or(<Error<T>>::TooManySchemas)?;
			//
			// Self::require_valid_schema_size(&schema);
			// Self::add_schema(schema.clone(), schema_id)?;
			//
			// let current_block = <frame_system::Pallet<T>>::block_number();
			// Self::deposit_event(Event::SchemaRegistered(sender, schema_id, current_block));
			Ok(())
		}
	}
}


impl<T: Config> Pallet<T> {
	pub fn require_valid_schema_size(schema: Vec<u8>) -> Result<BoundedVec<u8, T::MaxSchemaSize>, Error::<T>> {
		let bounded_fields: BoundedVec<u8, T::MaxSchemaSize> =
			schema.try_into().map_err(|()| Error::<T>::TooLongSchema)?;
		ensure!(bounded_fields.len() >= T::MinSchemaSize::get() as usize, <Error::<T>>::TooShortSchema);
		Ok(bounded_fields)
	}
}


