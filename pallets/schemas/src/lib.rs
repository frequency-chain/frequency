//! # Schemas Pallet
//!
//! The Schemas pallet provides functionality for handling schemas.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//! This pallet provides an on chain repository for schemas, thereby allowing participants of the
//! network to flexibly interact and exchange messages with each other without facing the challenge
//! of sharing, managing and validating messages as well as schemas between them.
//!
//! <b>NOTE</b>: In this pallet we define the <b>payload</b> structure that is used in <a href="../pallet_messages/index.html">Messages Pallet</a>.
//!
//! The Schema pallet provides functions for:
//!
//! - Registering a new schema.
//! - Setting maximum schema model size by governance.
//! - Retrieving latest registered schema id.
//! - Retrieving schemas by their id.
//!
//!
//! ### Terminology
//!
//! - **Schema:** The structure that defines how a Message is stored and structured.
//!
//! - **Schema Model:** Serialization/Deserialization details of the schema
//!
//! - **Schema Model Type:** The type of the following Serialization/Deserialization. It can be
//!   Avro, Parquet or ...
//!
//! ### Dispatchable Functions
//!
//! - `register_schema` - Registers a new schema after some initial validation.
//! - `set_max_schema_model_bytes` - Sets the maximum schema model size (Bytes) by governance.
//!
//! The Schema pallet implements the following traits:
//!
//! - [`SchemaProvider`](common_primitives::schema::SchemaProvider<SchemaId>): Functions for accessing and validating Schemas.  This implementation is what is used in the runtime.
//!
//! ## Genesis config
//!
//! The Schemas pallet depends on the [`GenesisConfig`].
//!
#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

use common_primitives::{
	parquet::ParquetModel,
	schema::{ModelType, PayloadLocation, SchemaId, SchemaProvider, SchemaResponse},
};
use frame_support::{dispatch::DispatchResult, ensure, traits::Get};
#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;

pub use pallet::*;
pub mod weights;
pub use types::*;
pub use weights::*;
mod serde;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Minimum length of Schema model size in bytes
		#[pallet::constant]
		type MinSchemaModelSizeBytes: Get<u32>;

		/// Maximum length of a Schema model Bounded Vec
		#[pallet::constant]
		type SchemaModelMaxBytesBoundedVecLimit: Get<u32>;

		/// Maximum number of schemas that can be registered
		#[pallet::constant]
		type MaxSchemaRegistrations: Get<SchemaId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Emitted when a schema is registered. [who, schemas id]
		SchemaRegistered(T::AccountId, SchemaId),

		/// Emitted when maximum size for schema model is changed.
		SchemaMaxSizeChanged(u32),
	}

	#[derive(PartialEq, Eq)] // for testing
	#[pallet::error]
	pub enum Error<T> {
		/// Schema is malformed
		InvalidSchema,
		/// The maximum number of schemas is stored in the database.
		TooManySchemas,
		/// The schema model exceeds the maximum length allowed
		ExceedsMaxSchemaModelBytes,
		/// The governance schema model max value provided is too large (greater than the BoundedVec size)
		ExceedsGovernanceSchemaModelMaxValue,
		/// The schema is less than the minimum length allowed
		LessThanMinSchemaModelBytes,
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
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Storage for the Governance managed max bytes for schema model
	/// Allows for altering the max bytes without a full chain upgrade
	/// - Value: Max Bytes
	#[pallet::storage]
	#[pallet::getter(fn get_schema_model_max_bytes)]
	pub(super) type GovernanceSchemaModelMaxBytes<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Storage type for current number of schemas
	/// Useful for retrieving latest schema id
	/// - Value: Last Schema Id
	#[pallet::storage]
	#[pallet::getter(fn schema_count)]
	pub(super) type SchemaCount<T: Config> = StorageValue<_, SchemaId, ValueQuery>;

	/// Storage for message schema struct data
	/// - Key: Schema Id
	/// - Value: [`Schema`](Schema)
	#[pallet::storage]
	#[pallet::getter(fn get_schema)]
	pub(super) type Schemas<T: Config> = StorageMap<
		_,
		Twox64Concat,
		SchemaId,
		Schema<T::SchemaModelMaxBytesBoundedVecLimit>,
		OptionQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		/// Maximum schema size in bytes at genesis
		pub initial_max_schema_model_size: u32,
	}

	#[cfg(feature = "std")]
	impl sp_std::default::Default for GenesisConfig {
		fn default() -> Self {
			Self { initial_max_schema_model_size: 1024 }
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			GovernanceSchemaModelMaxBytes::<T>::put(self.initial_max_schema_model_size);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Adds a given schema to storage. The schema in question must be of length
		/// between the min and max model size allowed for schemas (see pallet
		/// constants above). If the pallet's maximum schema limit has been
		/// fulfilled by the time this extrinsic is called, a TooManySchemas error
		/// will be thrown.
		///
		/// # Arguments
		/// * `origin` - The originator of the transaction
		/// * `model` - The new schema model data
		/// * 'model_type' - The formatting type of the model data
		/// # Returns
		/// * `DispatchResult`
		///
		/// # Errors
		/// * [`Error::<T>::LessThanMinSchemaModelBytes`] - The schema's length is less than the minimum schema length
		/// * [`Error::<T>::ExceedsMaxSchemaModelBytes`] - The schema's length is greater than the maximum schema length
		/// * [`Error::<T>::TooManySchemas`] - The maximum number of schemas has been met
		/// * [`Error::<T>::SchemaCountOverflow`] - The schema count has exceeded its bounds
		///
		#[pallet::weight(< T as Config >::WeightInfo::register_schema(model.len() as u32, 1000))]
		pub fn register_schema(
			origin: OriginFor<T>,
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
			payload_location: PayloadLocation,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				model.len() > T::MinSchemaModelSizeBytes::get() as usize,
				Error::<T>::LessThanMinSchemaModelBytes
			);
			ensure!(
				model.len() < Self::get_schema_model_max_bytes() as usize,
				Error::<T>::ExceedsMaxSchemaModelBytes
			);

			Self::ensure_valid_model(&model_type, &model)?;
			let schema_id = Self::add_schema(model, model_type, payload_location)?;

			Self::deposit_event(Event::SchemaRegistered(sender, schema_id));
			Ok(())
		}

		/// Set a new value for the Schema maximum number of bytes.  Must be <= the limit of the
		/// Schema BoundedVec used for registration.
		#[pallet::weight(30_000)]
		pub fn set_max_schema_model_bytes(origin: OriginFor<T>, max_size: u32) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(
				max_size <= T::SchemaModelMaxBytesBoundedVecLimit::get(),
				Error::<T>::ExceedsGovernanceSchemaModelMaxValue
			);
			GovernanceSchemaModelMaxBytes::<T>::set(max_size);
			Self::deposit_event(Event::SchemaMaxSizeChanged(max_size));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn add_schema(
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
			payload_location: PayloadLocation,
		) -> Result<SchemaId, DispatchError> {
			let cur_count = Self::schema_count();
			ensure!(cur_count < T::MaxSchemaRegistrations::get(), Error::<T>::TooManySchemas);
			let schema_id = cur_count.checked_add(1).ok_or(Error::<T>::SchemaCountOverflow)?;

			let schema = Schema { model_type, model, payload_location };
			<SchemaCount<T>>::set(schema_id);
			<Schemas<T>>::insert(schema_id, schema);
			Ok(schema_id)
		}

		/// Retrieve latest schema id via total count of schemas on chain
		pub fn get_latest_schema_id() -> Option<SchemaId> {
			Some(Self::schema_count())
		}

		/// Retrieve a schema by id
		pub fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponse> {
			if let Some(schema) = Self::get_schema(schema_id) {
				let model_vec: Vec<u8> = schema.model.into_inner();

				let response = SchemaResponse {
					schema_id,
					model: model_vec,
					model_type: schema.model_type,
					payload_location: schema.payload_location,
				};
				return Some(response)
			}
			None
		}

		/// Ensures that a given u8 Vector conforms to a recognized Parquet shape
		pub fn ensure_valid_model(
			model_type: &ModelType,
			model: &BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
		) -> DispatchResult {
			match model_type {
				&ModelType::Parquet => {
					serde_json::from_slice::<ParquetModel>(model)
						.map_err(|_| Error::<T>::InvalidSchema)?;
				},
				&ModelType::AvroBinary => serde::validate_json_model(model.clone().into_inner())
					.map_err(|_| Error::<T>::InvalidSchema)?,
			};
			Ok(())
		}
	}
}

impl<T: Config> SchemaProvider<SchemaId> for Pallet<T> {
	#[cfg(not(feature = "runtime-benchmarks"))]
	fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponse> {
		Self::get_schema_by_id(schema_id)
	}
	/// Since benchmarks are using regular runtime, we can not use mocking for this loosely bounded
	/// pallet trait implementation. To be able to run benchmarks successfully for any other pallet
	/// that has dependencies on this one, we would need to define msa accounts on those pallets'
	/// benchmarks, but this will introduce direct dependencies between these pallets, which we
	/// would like to avoid.
	/// To successfully run benchmarks without adding dependencies between pallets we re-defined
	/// this method to return schema checks requested by the benchmark to also be some.
	#[cfg(feature = "runtime-benchmarks")]
	fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponse> {
		// To account for db read
		Self::get_schema_by_id(schema_id);

		// This method allows the benchmarks to cover both payload location types
		// used by the messages pallets. Maybe one day we can replace this with a
		// common "benchmarking" crate.
		const IPFS_SCHEMA_ID: u16 = 65535;
		let location: PayloadLocation = if schema_id == IPFS_SCHEMA_ID {
			PayloadLocation::IPFS
		} else {
			PayloadLocation::OnChain
		};
		Some(SchemaResponse {
			schema_id,
			model: "{}".as_bytes().to_vec(),
			model_type: ModelType::default(),
			payload_location: location,
		})
	}
}
