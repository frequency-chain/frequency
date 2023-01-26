//! # Schemas Pallet
//! The Schemas pallet provides functionality for handling schemas.
//!
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Runtime API: `SchemasRuntimeApi`](../pallet_schemas_runtime_api/trait.SchemasRuntimeApi.html)
//! - [Custom RPC API: `SchemasApiServer`](../pallet_schemas_rpc/trait.SchemasApiServer.html)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
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
//! ## Terminology
//!
//! - **Schema:** The structure that defines how a Message is stored and structured.
//! - **Schema Model:** Serialization/Deserialization details of the schema
//! - **Schema Model Type:** The type of the following Serialization/Deserialization. It can be
//!   Avro, Parquet or ...
//!
//! ## Implementations
//!
//! - [`SchemaValidator`](../common_primitives/schema/trait.SchemaValidator.html): Functions for accessing and validating Schemas. This implementation is what is used in the runtime.
//! - [`SchemaProvider`](../common_primitives/schema/trait.SchemaProvider.html): Allows another pallet to resolve schema information.
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
	schema::{
		ModelType, PayloadLocation, SchemaId, SchemaProvider, SchemaResponse, SchemaValidator,
	},
};
use frame_support::{dispatch::DispatchResult, ensure, traits::Get};
use sp_runtime::BoundedVec;
use sp_std::vec::Vec;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(feature = "runtime-benchmarks")]
use common_primitives::benchmarks::SchemaBenchmarkHelper;

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

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

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
		SchemaCreated(T::AccountId, SchemaId),

		/// Emitted when maximum size for schema model is changed.
		SchemaMaxSizeChanged(u32),
	}

	#[derive(PartialEq, Eq)] // for testing
	#[pallet::error]
	pub enum Error<T> {
		/// Schema is malformed
		InvalidSchema,

		/// The schema model exceeds the maximum length allowed
		ExceedsMaxSchemaModelBytes,

		/// The schema is less than the minimum length allowed
		LessThanMinSchemaModelBytes,

		/// CurrentSchemaIdentifierMaximum was attempted to overflow max, means MaxSchemaRegistrations is too big
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
	#[pallet::getter(fn get_current_schema_identifier_maximum)]
	pub(super) type CurrentSchemaIdentifierMaximum<T: Config> =
		StorageValue<_, SchemaId, ValueQuery>;

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
		/// fulfilled by the time this extrinsic is called, a SchemaCountOverflow error
		/// will be thrown.
		///
		/// # Events
		/// * [`Event::SchemaCreated`]
		///
		/// # Errors
		/// * [`Error::LessThanMinSchemaModelBytes`] - The schema's length is less than the minimum schema length
		/// * [`Error::ExceedsMaxSchemaModelBytes`] - The schema's length is greater than the maximum schema length
		/// * [`Error::InvalidSchema`] - Schema is malformed in some way
		/// * [`Error::SchemaCountOverflow`] - The schema count has exceeded its bounds
		///
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_schema(model.len() as u32))]
		pub fn create_schema(
			origin: OriginFor<T>,
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
			payload_location: PayloadLocation,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				model.len() >= T::MinSchemaModelSizeBytes::get() as usize,
				Error::<T>::LessThanMinSchemaModelBytes
			);
			ensure!(
				model.len() <= Self::get_schema_model_max_bytes() as usize,
				Error::<T>::ExceedsMaxSchemaModelBytes
			);

			Self::ensure_valid_model(&model_type, &model)?;
			let schema_id = Self::add_schema(model, model_type, payload_location)?;

			Self::deposit_event(Event::SchemaCreated(sender, schema_id));
			Ok(())
		}

		/// Root and Governance can set a new max value for Schema bytes.
		/// Must be <= the limit of the Schema BoundedVec used for registration.
		///
		/// # Requires
		/// * Root Origin
		///
		/// # Events
		/// * [`Event::SchemaMaxSizeChanged`]
		///
		/// # Errors
		/// * [`Error::ExceedsMaxSchemaModelBytes`] - Cannot set to above the hard coded maximum [`Config::SchemaModelMaxBytesBoundedVecLimit`]
		///
		#[pallet::call_index(1)]
		#[pallet::weight((T::WeightInfo::set_max_schema_model_bytes(), DispatchClass::Operational))]
		pub fn set_max_schema_model_bytes(
			origin: OriginFor<T>,
			#[pallet::compact] max_size: u32,
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(
				max_size <= T::SchemaModelMaxBytesBoundedVecLimit::get(),
				Error::<T>::ExceedsMaxSchemaModelBytes
			);
			GovernanceSchemaModelMaxBytes::<T>::set(max_size);
			Self::deposit_event(Event::SchemaMaxSizeChanged(max_size));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Set the schema count to something in particular.
		#[cfg(any(feature = "std", feature = "runtime-benchmarks", test))]
		pub fn set_schema_count(n: SchemaId) {
			<CurrentSchemaIdentifierMaximum<T>>::set(n);
		}

		/// Build the [`Schema`] and insert it into storage
		/// Updates the [`CurrentSchemaIdentifierMaximum`] storage
		pub fn add_schema(
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
			payload_location: PayloadLocation,
		) -> Result<SchemaId, DispatchError> {
			let schema_id = Self::get_next_schema_id()?;
			let schema = Schema { model_type, model, payload_location };
			<CurrentSchemaIdentifierMaximum<T>>::set(schema_id);
			<Schemas<T>>::insert(schema_id, schema);
			Ok(schema_id)
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
		///
		/// # Errors
		/// * [`Error::InvalidSchema`]
		/// * [`Error::SchemaCountOverflow`]
		///
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

		/// Get the next available schema id
		///
		/// # Errors
		/// * [`Error::SchemaCountOverflow`]
		///
		fn get_next_schema_id() -> Result<SchemaId, DispatchError> {
			let next = Self::get_current_schema_identifier_maximum()
				.checked_add(1)
				.ok_or(Error::<T>::SchemaCountOverflow)?;

			Ok(next)
		}
	}
}

#[cfg(feature = "runtime-benchmarks")]
impl<T: Config> SchemaBenchmarkHelper for Pallet<T> {
	/// Sets schema count.
	fn set_schema_count(schema_id: SchemaId) {
		Self::set_schema_count(schema_id);
	}

	/// Creates a schema.
	fn create_schema(
		model: Vec<u8>,
		model_type: ModelType,
		payload_location: PayloadLocation,
	) -> DispatchResult {
		let model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit> =
			model.try_into().unwrap();
		Self::ensure_valid_model(&model_type, &model)?;
		Self::add_schema(model, model_type, payload_location)?;
		Ok(())
	}
}

impl<T: Config> SchemaValidator<SchemaId> for Pallet<T> {
	fn are_all_schema_ids_valid(schema_ids: &Vec<SchemaId>) -> bool {
		let latest_issue_schema_id = Self::get_current_schema_identifier_maximum();
		schema_ids.iter().all(|id| id <= &latest_issue_schema_id)
	}

	#[cfg(any(feature = "std", feature = "runtime-benchmarks", test))]
	fn set_schema_count(n: SchemaId) {
		Self::set_schema_count(n);
	}
}

impl<T: Config> SchemaProvider<SchemaId> for Pallet<T> {
	fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponse> {
		Self::get_schema_by_id(schema_id)
	}
}
