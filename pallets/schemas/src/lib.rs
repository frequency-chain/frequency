//! Schema Management for Message and Stateful Storage
//!
//! ## Quick Links
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Runtime API: `SchemasRuntimeApi`](../pallet_schemas_runtime_api/trait.SchemasRuntimeApi.html)
//! - [Custom RPC API: `SchemasApiServer`](../pallet_schemas_rpc/trait.SchemasApiServer.html)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
#![doc = include_str!("../README.md")]
//!
//! ## Implementations
//!
//! - [`SchemaValidator`](../common_primitives/schema/trait.SchemaValidator.html): Functions for accessing and validating Schemas. This implementation is what is used in the runtime.
//! - [`SchemaProvider`](../common_primitives/schema/trait.SchemaProvider.html): Allows another pallet to resolve schema information.
// Substrate macros are tripping the clippy::expect_used lint.
#![allow(clippy::expect_used)]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(rustdoc::private_intra_doc_links)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

use common_primitives::{
	node::ProposalProvider,
	parquet::ParquetModel,
	schema::{
		ModelType, PayloadLocation, SchemaId, SchemaProvider, SchemaResponse, SchemaSetting,
		SchemaSettings, SchemaValidator,
	},
};
use frame_support::{
	dispatch::{DispatchResult, PostDispatchInfo},
	ensure,
	traits::{BuildGenesisConfig, Get},
};
use sp_runtime::{traits::Dispatchable, BoundedVec, DispatchError};
use sp_std::{boxed::Box, vec::Vec};

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(feature = "runtime-benchmarks")]
use common_primitives::benchmarks::SchemaBenchmarkHelper;
use common_primitives::schema::{SchemaInfoResponse, SchemaVersionResponse};
/// migration module
pub mod migration;
mod types;

pub use pallet::*;
pub mod weights;
pub use types::*;
pub use weights::*;

mod serde;

const LOG_TARGET: &str = "runtime::schemas";

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
		type SchemaModelMaxBytesBoundedVecLimit: Get<u32> + MaxEncodedLen;

		/// Maximum number of schemas that can be registered
		#[pallet::constant]
		type MaxSchemaRegistrations: Get<SchemaId>;

		/// The origin that is allowed to create providers via governance
		type CreateSchemaViaGovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The runtime call dispatch type.
		type Proposal: Parameter
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
			+ From<Call<Self>>;

		/// The Council proposal provider interface
		type ProposalProvider: ProposalProvider<Self::AccountId, Self::Proposal>;

		/// Maximum number of schema settings that can be registered per schema (if any)
		#[pallet::constant]
		type MaxSchemaSettingsPerSchema: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Emitted when a schema is registered. [who, schemas id]
		SchemaCreated {
			/// Account ID
			key: T::AccountId,

			/// Schema ID of newly-created schema
			schema_id: SchemaId,
		},

		/// Emitted when maximum size for schema model is changed.
		SchemaMaxSizeChanged {
			/// Max size of schema document
			max_size: u32,
		},

		/// Emitted when a schema is assigned a name
		SchemaNameCreated {
			/// Schema ID which a name is assigned
			schema_id: SchemaId,
			/// ASCII string in bytes of the assigned name
			name: Vec<u8>,
		},
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

		/// Invalid setting for schema
		InvalidSetting,

		/// Invalid schema name encoding
		InvalidSchemaNameEncoding,

		/// Invalid schema name characters
		InvalidSchemaNameCharacters,

		/// Invalid schema name structure
		InvalidSchemaNameStructure,

		/// Invalid schema name length
		InvalidSchemaNameLength,

		/// Invalid schema namespace length
		InvalidSchemaNamespaceLength,

		/// Invalid schema descriptor length
		InvalidSchemaDescriptorLength,

		/// Schema version exceeds the maximum allowed number
		ExceedsMaxNumberOfVersions,

		/// Inserted schema id already exists
		SchemaIdAlreadyExists,

		///  SchemaId does not exist
		SchemaIdDoesNotExist,

		/// SchemaId has a name already
		SchemaIdAlreadyHasName,
	}

	#[pallet::pallet]
	#[pallet::storage_version(SCHEMA_STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// Storage for the Governance managed max bytes for schema model
	/// Allows for altering the max bytes without a full chain upgrade
	/// - Value: Max Bytes
	#[pallet::storage]
	pub(super) type GovernanceSchemaModelMaxBytes<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// Storage type for current number of schemas
	/// Useful for retrieving latest schema id
	/// - Value: Last Schema Id
	#[pallet::storage]
	pub(super) type CurrentSchemaIdentifierMaximum<T: Config> =
		StorageValue<_, SchemaId, ValueQuery>;

	/// Storage for message schema info struct data
	/// - Key: Schema Id
	/// - Value: [`SchemaInfo`](SchemaInfo)
	#[pallet::storage]
	pub(super) type SchemaInfos<T: Config> =
		StorageMap<_, Twox64Concat, SchemaId, SchemaInfo, OptionQuery>;

	/// Storage for message schema struct data
	/// - Key: Schema Id
	/// - Value: [`BoundedVec`](BoundedVec<T::SchemaModelMaxBytesBoundedVecLimit>)
	#[pallet::storage]
	pub(super) type SchemaPayloads<T: Config> = StorageMap<
		_,
		Twox64Concat,
		SchemaId,
		BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
		OptionQuery,
	>;

	/// Storage for message schema info struct data
	/// - Key: Schema Id
	/// - Value: [`SchemaInfo`](SchemaInfo)
	#[pallet::storage]
	pub(super) type SchemaNameToIds<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		SchemaNamespace,
		Blake2_128Concat,
		SchemaDescriptor,
		SchemaVersionId,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Maximum schema size in bytes at genesis
		pub initial_schema_identifier_max: u16,
		/// Maximum schema size in bytes at genesis
		pub initial_max_schema_model_size: u32,
		/// Genesis Schemas to load for development
		pub initial_schemas: Vec<GenesisSchema>,
		/// Phantom type
		#[serde(skip)]
		pub _config: PhantomData<T>,
	}

	impl<T: Config> sp_std::default::Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				initial_schema_identifier_max: 16_000,
				initial_max_schema_model_size: 1024,
				initial_schemas: Default::default(),
				_config: Default::default(),
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			GovernanceSchemaModelMaxBytes::<T>::put(self.initial_max_schema_model_size);

			// Load in the Genesis Schemas
			for schema in self.initial_schemas.iter() {
				let model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit> =
					BoundedVec::try_from(schema.model.clone().into_bytes()).expect(
						"Genesis Schema Model larger than SchemaModelMaxBytesBoundedVecLimit",
					);
				let name_payload: SchemaNamePayload =
					BoundedVec::try_from(schema.name.clone().into_bytes())
						.expect("Genesis Schema Name larger than SCHEMA_NAME_BYTES_MAX");
				let parsed_name: Option<SchemaName> = if name_payload.len() > 0 {
					Some(
						SchemaName::try_parse::<T>(name_payload, true)
							.expect("Bad Genesis Schema Name"),
					)
				} else {
					None
				};
				let settings: BoundedVec<SchemaSetting, T::MaxSchemaSettingsPerSchema> =
					BoundedVec::try_from(schema.settings.clone()).expect(
						"Bad Genesis Schema Settings. Perhaps larger than MaxSchemaSettingsPerSchema"
					);

				let _ = Pallet::<T>::add_schema(
					model,
					schema.model_type,
					schema.payload_location,
					settings,
					parsed_name,
				)
				.expect("Failed to set Schema in Genesis!");
			}

			// Set the maximum manually
			CurrentSchemaIdentifierMaximum::<T>::put(self.initial_schema_identifier_max);
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
		#[allow(deprecated)]
		#[deprecated(
			note = "please use `create_schema_v3` since `create_schema` has been deprecated."
		)]
		pub fn create_schema(
			origin: OriginFor<T>,
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
			payload_location: PayloadLocation,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let (schema_id, _) = Self::create_schema_for(
				model,
				model_type,
				payload_location,
				BoundedVec::default(),
				None,
			)?;

			Self::deposit_event(Event::SchemaCreated { key: sender, schema_id });
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
			Self::deposit_event(Event::SchemaMaxSizeChanged { max_size });
			Ok(())
		}

		/// Propose to create a schema.  Creates a proposal for council approval to create a schema
		///
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::propose_to_create_schema(model.len() as u32))]
		#[allow(deprecated)]
		#[deprecated(
			note = "please use `propose_to_create_schema_v2` since `propose_to_create_schema` has been deprecated."
		)]
		pub fn propose_to_create_schema(
			origin: OriginFor<T>,
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
			payload_location: PayloadLocation,
			settings: BoundedVec<SchemaSetting, T::MaxSchemaSettingsPerSchema>,
		) -> DispatchResult {
			let proposer = ensure_signed(origin)?;

			let proposal: Box<T::Proposal> = Box::new(
				(Call::<T>::create_schema_via_governance {
					creator_key: proposer.clone(),
					model,
					model_type,
					payload_location,
					settings,
				})
				.into(),
			);
			T::ProposalProvider::propose_with_simple_majority(proposer, proposal)?;
			Ok(())
		}

		/// Create a schema by means of council approval
		///
		/// # Events
		/// * [`Event::SchemaCreated`]
		///
		/// # Errors
		/// * [`Error::LessThanMinSchemaModelBytes`] - The schema's length is less than the minimum schema length
		/// * [`Error::ExceedsMaxSchemaModelBytes`] - The schema's length is greater than the maximum schema length
		/// * [`Error::InvalidSchema`] - Schema is malformed in some way
		/// * [`Error::SchemaCountOverflow`] - The schema count has exceeded its bounds
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::create_schema_via_governance(model.len() as u32+ settings.len() as u32))]
		#[allow(deprecated)]
		#[deprecated(
			note = "please use `create_schema_via_governance_v2` since `create_schema_via_governance` has been deprecated."
		)]
		pub fn create_schema_via_governance(
			origin: OriginFor<T>,
			creator_key: T::AccountId,
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
			payload_location: PayloadLocation,
			settings: BoundedVec<SchemaSetting, T::MaxSchemaSettingsPerSchema>,
		) -> DispatchResult {
			T::CreateSchemaViaGovernanceOrigin::ensure_origin(origin)?;
			let (schema_id, _) =
				Self::create_schema_for(model, model_type, payload_location, settings, None)?;

			Self::deposit_event(Event::SchemaCreated { key: creator_key, schema_id });
			Ok(())
		}

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
		/// * [`Error::InvalidSetting`] - Invalid setting is provided
		///
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::create_schema_v2(model.len() as u32 + settings.len() as u32))]
		#[allow(deprecated)]
		#[deprecated(
			note = "please use `create_schema_v3` since `create_schema_v2` has been deprecated."
		)]
		pub fn create_schema_v2(
			origin: OriginFor<T>,
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
			payload_location: PayloadLocation,
			settings: BoundedVec<SchemaSetting, T::MaxSchemaSettingsPerSchema>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let (schema_id, _) =
				Self::create_schema_for(model, model_type, payload_location, settings, None)?;

			Self::deposit_event(Event::SchemaCreated { key: sender, schema_id });
			Ok(())
		}

		/// Propose to create a schema.  Creates a proposal for council approval to create a schema
		///
		#[pallet::call_index(5)]
		#[pallet::weight(
			match schema_name {
				Some(_) => T::WeightInfo::propose_to_create_schema_v2(model.len() as u32),
				None => T::WeightInfo::propose_to_create_schema(model.len() as u32)
			}
		)]
		pub fn propose_to_create_schema_v2(
			origin: OriginFor<T>,
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
			payload_location: PayloadLocation,
			settings: BoundedVec<SchemaSetting, T::MaxSchemaSettingsPerSchema>,
			schema_name: Option<SchemaNamePayload>,
		) -> DispatchResult {
			let proposer = ensure_signed(origin)?;

			let proposal: Box<T::Proposal> = Box::new(
				(Call::<T>::create_schema_via_governance_v2 {
					creator_key: proposer.clone(),
					model,
					model_type,
					payload_location,
					settings,
					schema_name,
				})
				.into(),
			);
			T::ProposalProvider::propose_with_simple_majority(proposer, proposal)?;
			Ok(())
		}

		/// Create a schema by means of council approval
		///
		/// # Events
		/// * [`Event::SchemaCreated`]
		/// * [`Event::SchemaNameCreated`]
		///
		/// # Errors
		/// * [`Error::LessThanMinSchemaModelBytes`] - The schema's length is less than the minimum schema length
		/// * [`Error::ExceedsMaxSchemaModelBytes`] - The schema's length is greater than the maximum schema length
		/// * [`Error::InvalidSchema`] - Schema is malformed in some way
		/// * [`Error::SchemaCountOverflow`] - The schema count has exceeded its bounds
		/// * [`Error::InvalidSchemaNameEncoding`] - The schema name has invalid encoding
		/// * [`Error::InvalidSchemaNameCharacters`] - The schema name has invalid characters
		/// * [`Error::InvalidSchemaNameStructure`] - The schema name has invalid structure
		/// * [`Error::InvalidSchemaNameLength`] - The schema name has invalid length
		/// * [`Error::InvalidSchemaNamespaceLength`] - The schema namespace has invalid length
		/// * [`Error::InvalidSchemaDescriptorLength`] - The schema descriptor has invalid length
		/// * [`Error::ExceedsMaxNumberOfVersions`] - The schema name reached max number of versions
		///
		#[pallet::call_index(6)]
		#[pallet::weight(
			match schema_name {
				Some(_) => T::WeightInfo::create_schema_via_governance_v2(model.len() as u32+ settings.len() as u32),
				None => T::WeightInfo::create_schema_via_governance(model.len() as u32+ settings.len() as u32)
			}
		)]
		pub fn create_schema_via_governance_v2(
			origin: OriginFor<T>,
			creator_key: T::AccountId,
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
			payload_location: PayloadLocation,
			settings: BoundedVec<SchemaSetting, T::MaxSchemaSettingsPerSchema>,
			schema_name: Option<SchemaNamePayload>,
		) -> DispatchResult {
			T::CreateSchemaViaGovernanceOrigin::ensure_origin(origin)?;
			let (schema_id, schema_name) = Self::create_schema_for(
				model,
				model_type,
				payload_location,
				settings,
				schema_name,
			)?;

			Self::deposit_event(Event::SchemaCreated { key: creator_key, schema_id });
			if let Some(inner_name) = schema_name {
				Self::deposit_event(Event::SchemaNameCreated {
					schema_id,
					name: inner_name.get_combined_name(),
				});
			}
			Ok(())
		}

		/// Adds a given schema to storage. The schema in question must be of length
		/// between the min and max model size allowed for schemas (see pallet
		/// constants above). If the pallet's maximum schema limit has been
		/// fulfilled by the time this extrinsic is called, a SchemaCountOverflow error
		/// will be thrown.
		///
		/// # Events
		/// * [`Event::SchemaCreated`]
		/// * [`Event::SchemaNameCreated`]
		///
		/// # Errors
		/// * [`Error::LessThanMinSchemaModelBytes`] - The schema's length is less than the minimum schema length
		/// * [`Error::ExceedsMaxSchemaModelBytes`] - The schema's length is greater than the maximum schema length
		/// * [`Error::InvalidSchema`] - Schema is malformed in some way
		/// * [`Error::SchemaCountOverflow`] - The schema count has exceeded its bounds
		/// * [`Error::InvalidSetting`] - Invalid setting is provided
		/// * [`Error::InvalidSchemaNameEncoding`] - The schema name has invalid encoding
		/// * [`Error::InvalidSchemaNameCharacters`] - The schema name has invalid characters
		/// * [`Error::InvalidSchemaNameStructure`] - The schema name has invalid structure
		/// * [`Error::InvalidSchemaNameLength`] - The schema name has invalid length
		/// * [`Error::InvalidSchemaNamespaceLength`] - The schema namespace has invalid length
		/// * [`Error::InvalidSchemaDescriptorLength`] - The schema descriptor has invalid length
		/// * [`Error::ExceedsMaxNumberOfVersions`] - The schema name reached max number of versions
		///
		#[pallet::call_index(7)]
		#[pallet::weight(
			match schema_name {
				Some(_) => T::WeightInfo::create_schema_v3(model.len() as u32 + settings.len() as u32),
				None => T::WeightInfo::create_schema_v2(model.len() as u32 + settings.len() as u32)
			}
		)]
		pub fn create_schema_v3(
			origin: OriginFor<T>,
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
			payload_location: PayloadLocation,
			settings: BoundedVec<SchemaSetting, T::MaxSchemaSettingsPerSchema>,
			schema_name: Option<SchemaNamePayload>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let (schema_id, schema_name) = Self::create_schema_for(
				model,
				model_type,
				payload_location,
				settings,
				schema_name,
			)?;

			Self::deposit_event(Event::SchemaCreated { key: sender, schema_id });
			if let Some(inner_name) = schema_name {
				Self::deposit_event(Event::SchemaNameCreated {
					schema_id,
					name: inner_name.get_combined_name(),
				});
			}
			Ok(())
		}

		/// Propose to create a schema name.  Creates a proposal for council approval to create a schema name
		/// * [`Error::LessThanMinSchemaModelBytes`] - The schema's length is less than the minimum schema length
		/// * [`Error::ExceedsMaxSchemaModelBytes`] - The schema's length is greater than the maximum schema length
		/// * [`Error::InvalidSchema`] - Schema is malformed in some way
		/// * [`Error::InvalidSchemaNameEncoding`] - The schema name has invalid encoding
		/// * [`Error::InvalidSchemaNameCharacters`] - The schema name has invalid characters
		/// * [`Error::InvalidSchemaNameStructure`] - The schema name has invalid structure
		/// * [`Error::InvalidSchemaNameLength`] - The schema name has invalid length
		/// * [`Error::InvalidSchemaNamespaceLength`] - The schema namespace has invalid length
		/// * [`Error::InvalidSchemaDescriptorLength`] - The schema descriptor has invalid length
		/// * [`Error::ExceedsMaxNumberOfVersions`] - The schema name reached max number of versions
		/// * [`Error::SchemaIdDoesNotExist`] - The schema id does not exist
		/// * [`Error::SchemaIdAlreadyHasName`] - The schema id already has a name
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::propose_to_create_schema_name())]
		pub fn propose_to_create_schema_name(
			origin: OriginFor<T>,
			schema_id: SchemaId,
			schema_name: SchemaNamePayload,
		) -> DispatchResult {
			let proposer = ensure_signed(origin)?;

			let _ = Self::parse_and_verify_schema_name(schema_id, &schema_name)?;

			let proposal: Box<T::Proposal> = Box::new(
				(Call::<T>::create_schema_name_via_governance { schema_id, schema_name }).into(),
			);
			T::ProposalProvider::propose_with_simple_majority(proposer, proposal)?;
			Ok(())
		}

		/// Assigns a name to a schema without any name
		///
		/// # Events
		/// * [`Event::SchemaNameCreated`]
		///
		/// # Errors
		/// * [`Error::LessThanMinSchemaModelBytes`] - The schema's length is less than the minimum schema length
		/// * [`Error::ExceedsMaxSchemaModelBytes`] - The schema's length is greater than the maximum schema length
		/// * [`Error::InvalidSchema`] - Schema is malformed in some way
		/// * [`Error::SchemaCountOverflow`] - The schema count has exceeded its bounds
		/// * [`Error::InvalidSchemaNameEncoding`] - The schema name has invalid encoding
		/// * [`Error::InvalidSchemaNameCharacters`] - The schema name has invalid characters
		/// * [`Error::InvalidSchemaNameStructure`] - The schema name has invalid structure
		/// * [`Error::InvalidSchemaNameLength`] - The schema name has invalid length
		/// * [`Error::InvalidSchemaNamespaceLength`] - The schema namespace has invalid length
		/// * [`Error::InvalidSchemaDescriptorLength`] - The schema descriptor has invalid length
		/// * [`Error::ExceedsMaxNumberOfVersions`] - The schema name reached max number of versions
		/// * [`Error::SchemaIdDoesNotExist`] - The schema id does not exist
		/// * [`Error::SchemaIdAlreadyHasName`] - The schema id already has a name
		///
		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::create_schema_name_via_governance())]
		pub fn create_schema_name_via_governance(
			origin: OriginFor<T>,
			schema_id: SchemaId,
			schema_name: SchemaNamePayload,
		) -> DispatchResult {
			T::CreateSchemaViaGovernanceOrigin::ensure_origin(origin)?;

			let parsed_name = Self::parse_and_verify_schema_name(schema_id, &schema_name)?;
			SchemaNameToIds::<T>::try_mutate(
				&parsed_name.namespace,
				&parsed_name.descriptor,
				|schema_version_id| -> DispatchResult {
					schema_version_id.add::<T>(schema_id)?;

					Self::deposit_event(Event::SchemaNameCreated {
						schema_id,
						name: parsed_name.get_combined_name(),
					});
					Ok(())
				},
			)
		}
	}

	impl<T: Config> Pallet<T> {
		/// Set the schema count to something in particular.
		#[cfg(any(feature = "std", feature = "runtime-benchmarks", test))]
		pub fn set_schema_count(n: SchemaId) {
			<CurrentSchemaIdentifierMaximum<T>>::set(n);
		}

		/// Inserts both the [`SchemaInfo`] and Schema Payload into storage
		/// Updates the `CurrentSchemaIdentifierMaximum` storage
		pub fn add_schema(
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
			payload_location: PayloadLocation,
			settings: BoundedVec<SchemaSetting, T::MaxSchemaSettingsPerSchema>,
			schema_name_option: Option<SchemaName>,
		) -> Result<SchemaId, DispatchError> {
			let schema_id = Self::get_next_schema_id()?;
			let has_name = schema_name_option.is_some();
			let mut set_settings = SchemaSettings::all_disabled();
			if !settings.is_empty() {
				for i in settings.into_inner() {
					set_settings.set(i);
				}
			}

			if let Some(schema_name) = schema_name_option {
				SchemaNameToIds::<T>::try_mutate(
					schema_name.namespace,
					schema_name.descriptor,
					|schema_version_id| -> Result<(), DispatchError> {
						schema_version_id.add::<T>(schema_id)?;
						Ok(())
					},
				)?;
			};

			let schema_info =
				SchemaInfo { model_type, payload_location, settings: set_settings, has_name };
			<CurrentSchemaIdentifierMaximum<T>>::set(schema_id);
			<SchemaInfos<T>>::insert(schema_id, schema_info);
			<SchemaPayloads<T>>::insert(schema_id, model);

			Ok(schema_id)
		}

		/// Retrieve a schema by id
		pub fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponse> {
			match (SchemaInfos::<T>::get(schema_id), SchemaPayloads::<T>::get(schema_id)) {
				(Some(schema_info), Some(payload)) => {
					let model_vec: Vec<u8> = payload.into_inner();
					let saved_settings = schema_info.settings;
					let settings = saved_settings.0.iter().collect::<Vec<SchemaSetting>>();
					let response = SchemaResponse {
						schema_id,
						model: model_vec,
						model_type: schema_info.model_type,
						payload_location: schema_info.payload_location,
						settings,
					};
					Some(response)
				},
				(None, Some(_)) | (Some(_), None) => {
					log::error!("Corrupted state for schema {:?}, Should never happen!", schema_id);
					None
				},
				(None, None) => None,
			}
		}

		/// Retrieve a schema info by id
		pub fn get_schema_info_by_id(schema_id: SchemaId) -> Option<SchemaInfoResponse> {
			if let Some(schema_info) = SchemaInfos::<T>::get(schema_id) {
				let saved_settings = schema_info.settings;
				let settings = saved_settings.0.iter().collect::<Vec<SchemaSetting>>();
				let response = SchemaInfoResponse {
					schema_id,
					model_type: schema_info.model_type,
					payload_location: schema_info.payload_location,
					settings,
				};
				return Some(response);
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
			let next = CurrentSchemaIdentifierMaximum::<T>::get()
				.checked_add(1)
				.ok_or(Error::<T>::SchemaCountOverflow)?;

			Ok(next)
		}

		/// Adds a given schema to storage. The schema in question must be of length
		/// between the min and max model size allowed for schemas (see pallet
		/// constants above). If the pallet's maximum schema limit has been
		/// fulfilled by the time this extrinsic is called, a SchemaCountOverflow error
		/// will be thrown.
		///
		/// # Errors
		/// * [`Error::LessThanMinSchemaModelBytes`] - The schema's length is less than the minimum schema length
		/// * [`Error::ExceedsMaxSchemaModelBytes`] - The schema's length is greater than the maximum schema length
		/// * [`Error::InvalidSchema`] - Schema is malformed in some way
		/// * [`Error::SchemaCountOverflow`] - The schema count has exceeded its bounds
		/// * [`Error::InvalidSetting`] - Invalid setting is provided
		pub fn create_schema_for(
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
			payload_location: PayloadLocation,
			settings: BoundedVec<SchemaSetting, T::MaxSchemaSettingsPerSchema>,
			optional_schema_name: Option<SchemaNamePayload>,
		) -> Result<(SchemaId, Option<SchemaName>), DispatchError> {
			Self::ensure_valid_model(&model_type, &model)?;
			ensure!(
				model.len() >= T::MinSchemaModelSizeBytes::get() as usize,
				Error::<T>::LessThanMinSchemaModelBytes
			);
			ensure!(
				model.len() <= GovernanceSchemaModelMaxBytes::<T>::get() as usize,
				Error::<T>::ExceedsMaxSchemaModelBytes
			);
			// AppendOnly is only valid for Itemized payload location
			ensure!(
				!settings.contains(&SchemaSetting::AppendOnly) ||
					payload_location == PayloadLocation::Itemized,
				Error::<T>::InvalidSetting
			);
			// SignatureRequired is only valid for Itemized and Paginated payload locations
			ensure!(
				!settings.contains(&SchemaSetting::SignatureRequired) ||
					payload_location == PayloadLocation::Itemized ||
					payload_location == PayloadLocation::Paginated,
				Error::<T>::InvalidSetting
			);
			let schema_name = match optional_schema_name {
				None => None,
				Some(name_payload) => {
					let parsed_name = SchemaName::try_parse::<T>(name_payload, true)?;
					Some(parsed_name)
				},
			};
			let schema_id = Self::add_schema(
				model,
				model_type,
				payload_location,
				settings,
				schema_name.clone(),
			)?;
			Ok((schema_id, schema_name))
		}

		/// a method to return all versions of a schema name with their schemaIds
		/// Warning: Must only get called from RPC, since the number of DB accesses is not deterministic
		pub fn get_schema_versions(schema_name: Vec<u8>) -> Option<Vec<SchemaVersionResponse>> {
			let bounded_name = BoundedVec::try_from(schema_name).ok()?;
			let parsed_name = SchemaName::try_parse::<T>(bounded_name, false).ok()?;
			let versions: Vec<_> = match parsed_name.descriptor_exists() {
				true => SchemaNameToIds::<T>::get(&parsed_name.namespace, &parsed_name.descriptor)
					.convert_to_response(&parsed_name),
				false => SchemaNameToIds::<T>::iter_prefix(&parsed_name.namespace)
					.flat_map(|(descriptor, val)| {
						val.convert_to_response(&parsed_name.new_with_descriptor(descriptor))
					})
					.collect(),
			};
			Some(versions)
		}

		/// Parses the schema name and makes sure the schema does not have a name
		fn parse_and_verify_schema_name(
			schema_id: SchemaId,
			schema_name: &SchemaNamePayload,
		) -> Result<SchemaName, DispatchError> {
			let schema_option = SchemaInfos::<T>::get(schema_id);
			ensure!(schema_option.is_some(), Error::<T>::SchemaIdDoesNotExist);
			if let Some(info) = schema_option {
				ensure!(!info.has_name, Error::<T>::SchemaIdAlreadyHasName);
			}
			let parsed_name = SchemaName::try_parse::<T>(schema_name.clone(), true)?;
			Ok(parsed_name)
		}
	}
}

#[allow(clippy::unwrap_used)]
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
		Self::add_schema(model, model_type, payload_location, BoundedVec::default(), None)?;
		Ok(())
	}
}

impl<T: Config> SchemaValidator<SchemaId> for Pallet<T> {
	fn are_all_schema_ids_valid(schema_ids: &Vec<SchemaId>) -> bool {
		let latest_issue_schema_id = CurrentSchemaIdentifierMaximum::<T>::get();
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

	fn get_schema_info_by_id(schema_id: SchemaId) -> Option<SchemaInfoResponse> {
		Self::get_schema_info_by_id(schema_id)
	}
}
