#![cfg_attr(not(feature = "std"), no_std)]

extern crate core;

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{Currency, Get},
	weights::WeightToFeePolynomial,
	BoundedVec,
};
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use pallet::*;
pub mod weights;
pub use weights::*;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use common_primitives::schema::{Schema, SchemaId};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_transaction_payment::{FeeDetails, InclusionFee};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;

		// Maximum length of Schema field name
		#[pallet::constant]
		type MinSchemaSizeBytes: Get<u32>;

		// Maximum length of Schema field name
		#[pallet::constant]
		type MaxSchemaSizeBytes: Get<u32>;

		// Maximum number of schemas that can be registered
		#[pallet::constant]
		type MaxSchemaRegistrations: Get<SchemaId>;

		type Currency: Currency<Self::AccountId>;

		/// Convert a weight value into a deductible fee based on the currency type.
		type WeightToFee: WeightToFeePolynomial<Balance = BalanceOf<Self>>;
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
		StorageMap<_, Twox64Concat, SchemaId, Schema<T::MaxSchemaSizeBytes>, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::register_schema(schema.len() as u32))]
		pub fn register_schema(origin: OriginFor<T>, schema: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			let cur_count = Self::schema_count();
			ensure!(cur_count < T::MaxSchemaRegistrations::get(), <Error<T>>::TooManySchemas);
			let schema_id = cur_count.checked_add(1).ok_or(<Error<T>>::SchemaCountOverflow)?;

			Self::add_schema(schema.clone(), schema_id)?;

			Self::deposit_event(Event::SchemaRegistered(sender, schema_id));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn add_schema(schema: Vec<u8>, schema_id: SchemaId) -> DispatchResult {
			let bounded_fields = Self::require_valid_schema_size(schema)?;

			<SchemaCount<T>>::set(schema_id);
			<Schemas<T>>::insert(schema_id, bounded_fields);

			Ok(())
		}

		pub fn get_latest_schema_id() -> Result<SchemaId, DispatchError> {
			let cur_count = Self::schema_count();
			Ok(cur_count)
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

		pub fn calculate_schema_cost(schema: Vec<u8>) -> FeeDetails<BalanceOf<T>> {
			let schema_len = schema.len() as u32;
			let schema_weight = T::WeightInfo::register_schema(schema_len);
			let unadjusted_weight_fee = T::WeightToFee::calc(&schema_weight);
			// TODO: for issue: #77
			//let multiplier = Self::next_fee_multiplier();
			//let adjusted_weight_fee = multiplier.saturating_mul_int(unadjusted_weight_fee);
			//let extrinsic_weight = T::BlockWeights::get().get(Self).base_extrinsic;
			//let base_fee = T::WeightToFee::calc(&extrinsic_weight);
			// TODO fixed len fee updates if this is relevant for mrc
			// TODO add multiplier to fee if schema registration will have increasing cost
			let fixed_len_fee = 0u32.into();
			let tip = 0u32.into();
			FeeDetails {
				inclusion_fee: Some(InclusionFee {
					base_fee: unadjusted_weight_fee,
					len_fee: fixed_len_fee,
					adjusted_weight_fee: unadjusted_weight_fee,
				}),
				tip,
			}
		}
	}
}
