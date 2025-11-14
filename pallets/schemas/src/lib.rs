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

extern crate alloc;
use alloc::{boxed::Box, vec, vec::Vec};
use common_primitives::{
	node::ProposalProvider,
	parquet::ParquetModel,
	schema::{
		IntentGroupId, IntentGroupResponse, IntentId, IntentResponse, IntentSetting,
		IntentSettings, MappedEntityIdentifier, ModelType, NameLookupResponse, PayloadLocation,
		SchemaId, SchemaInfoResponse, SchemaProvider, SchemaStatus, SchemaValidator,
		SchemaVersionResponse,
	},
};
use frame_support::{
	dispatch::{DispatchResult, PostDispatchInfo},
	ensure,
	traits::{BuildGenesisConfig, Get},
};
use sp_runtime::{traits::Dispatchable, BoundedVec, DispatchError};

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(feature = "runtime-benchmarks")]
use common_primitives::benchmarks::SchemaBenchmarkHelper;
use common_primitives::schema::SchemaResponseV2;

mod types;

pub use pallet::*;
pub mod weights;
pub use types::*;
pub use weights::*;

/// Storage migrations
pub mod migration;
mod serde;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use common_primitives::schema::SchemaResponseV2;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		#[allow(deprecated)]
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

		/// Maximum number of Intents that can be registered
		#[pallet::constant]
		type MaxIntentRegistrations: Get<IntentId>;

		/// Maximum number of Intents that can belong to an IntentGroup
		#[pallet::constant]
		type MaxIntentsPerIntentGroup: Get<u32>;

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

		/// Emitted when an Intent is registered
		IntentCreated {
			/// Account ID
			key: T::AccountId,

			/// IntentId of newly created Intent
			intent_id: IntentId,

			/// Name of newly created Intent (ASCII bytes)
			intent_name: Vec<u8>,
		},

		/// Emitted when an IntentGroup is registered
		IntentGroupCreated {
			/// Account ID of creator
			key: T::AccountId,

			/// IntentGroupId of newly created IntentGroup
			intent_group_id: IntentGroupId,

			/// Name of newly created IntentGroup (ASCII bytes)
			intent_group_name: Vec<u8>,
		},

		/// Emitted when an IntentGroup is updated
		IntentGroupUpdated {
			/// Account ID that did the update
			key: T::AccountId,

			/// IntentGroupId of the IntentGroup that was updated
			intent_group_id: IntentGroupId,
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

		/// Name already exists in the registry
		NameAlreadyExists,

		/// Attempted to add a new Intent that would cause CurrentIntentIdentifierMaximum
		/// to exceed MaxIntentRegistrations
		IntentCountOverflow,

		/// The indicated [`IntentId`] does not exist
		InvalidIntentId,

		/// The indicated [`IntentGroupId`] does not exist
		InvalidIntentGroupId,

		/// Attempted to add a new IntentGroup that would cause CurrentIntentGroupIdentifierMaximum
		/// to exceed MaxIntentGroupRegistrations
		IntentGroupCountOverflow,

		/// Too many intents in the group
		TooManyIntentsInGroup,
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
	/// - Value: Last Schema ID
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
	pub(super) type SchemaPayloads<T: Config> =
		StorageMap<_, Twox64Concat, SchemaId, SchemaPayload<T>, OptionQuery>;

	/// Storage for mapping names to IDs
	/// - Key: Protocol Name
	/// - Key: Descriptor
	/// - Value: MappedEntityIdentifier
	#[pallet::storage]
	pub(super) type NameToMappedEntityIds<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		SchemaProtocolName,
		Blake2_128Concat,
		SchemaDescriptor,
		MappedEntityIdentifier,
		ValueQuery,
	>;

	/// Storage type for current number of Intents
	/// Useful for retrieving latest IntentId
	/// - Value: Last IntentId
	#[pallet::storage]
	pub(super) type CurrentIntentIdentifierMaximum<T: Config> =
		StorageValue<_, IntentId, ValueQuery>;

	/// Storage for Intents
	#[pallet::storage]
	pub(super) type IntentInfos<T: Config> =
		StorageMap<_, Twox64Concat, IntentId, IntentInfo, OptionQuery>;

	/// Storage type for current number of IntentGroups
	/// Useful for retrieving the latest IntentGroupId
	/// - Value: Last IntentGroupId
	#[pallet::storage]
	pub(super) type CurrentIntentGroupIdentifierMaximum<T: Config> =
		StorageValue<_, IntentGroupId, ValueQuery>;

	/// Storage for IntentGroups
	#[pallet::storage]
	pub(super) type IntentGroups<T: Config> =
		StorageMap<_, Twox64Concat, IntentGroupId, IntentGroup<T>, OptionQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Maximum schema identifier at genesis
		pub initial_schema_identifier_max: SchemaId,
		/// Maximum Intent identifier at genesis
		pub initial_intent_identifier_max: IntentId,
		/// Maximum IntentGroup identifier at genesis
		pub initial_intent_group_identifier_max: IntentGroupId,
		/// Maximum schema size in bytes at genesis
		pub initial_max_schema_model_size: u32,
		/// Genesis Intents to load for development
		pub initial_intents: Vec<GenesisIntent>,
		/// Genesis Schemas to load for development
		pub initial_schemas: Vec<GenesisSchema>,
		/// Genesis IntentGroups to load for development
		pub initial_intent_groups: Vec<GenesisIntentGroup>,
		/// Phantom type
		#[serde(skip)]
		pub _config: PhantomData<T>,
	}

	impl<T: Config> core::default::Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				initial_intent_identifier_max: 16_000,
				initial_schema_identifier_max: 16_000,
				initial_intent_group_identifier_max: 16_000,
				initial_max_schema_model_size: 1024,
				initial_intents: Default::default(),
				initial_schemas: Default::default(),
				initial_intent_groups: Default::default(),
				_config: Default::default(),
			}
		}
	}

	impl<T: Config> Into<GenesisConfig<T>> for GenesisSchemasPalletConfig {
		fn into(self) -> GenesisConfig<T> {
			GenesisConfig::<T> {
				initial_intent_identifier_max: self.intent_identifier_max.unwrap_or(16_000),
				initial_schema_identifier_max: self.schema_identifier_max.unwrap_or(16_000),
				initial_intent_group_identifier_max: self
					.intent_group_identifier_max
					.unwrap_or(16_000),
				initial_max_schema_model_size: self.max_schema_model_size.unwrap_or(1024),
				initial_intents: self.intents.unwrap_or_default(),
				initial_schemas: self.schemas.unwrap_or_default(),
				initial_intent_groups: self.intent_groups.unwrap_or_default(),
				_config: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			GovernanceSchemaModelMaxBytes::<T>::put(self.initial_max_schema_model_size);

			for intent in self.initial_intents.iter() {
				let name_payload: SchemaNamePayload =
					BoundedVec::try_from(intent.name.clone().into_bytes())
						.expect("Genesis Intent name larger than max bytes");
				let parsed_name = SchemaName::try_parse::<T>(name_payload, true)
					.expect("Bad Genesis Intent name");
				let settings_vec = BoundedVec::<IntentSetting, T::MaxSchemaSettingsPerSchema>::try_from(intent.settings.clone()).expect("Bad Genesis Intent settings. Perhaps larger than MaxSchemaSettingsPerSchema");
				let mut settings = IntentSettings::default();
				settings_vec.iter().for_each(|setting| settings.set(*setting));

				let intent_info =
					IntentInfo { payload_location: intent.payload_location, settings };
				Pallet::<T>::store_intent_info(intent.intent_id, intent_info, &parsed_name)
					.expect("Failed to set Intent in Genesis!");
			}

			for schema in self.initial_schemas.iter() {
				let intent = IntentInfos::<T>::try_get(schema.intent_id)
					.expect("Genesis Schema has no matching Intent.");
				let model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit> =
					BoundedVec::try_from(schema.model.clone().into_bytes()).expect(
						"Genesis Schema Model larger than SchemaModelMaxBytesBoundedVecLimit",
					);
				let schema_info = SchemaInfo {
					intent_id: schema.intent_id,
					payload_location: intent.payload_location,
					model_type: schema.model_type,
					settings: intent.settings,
					status: schema.status,
				};

				Pallet::<T>::store_schema_info_and_payload(schema.schema_id, schema_info, model)
					.expect("Failed to set Schema in Genesis!");
			}

			for intent_group in self.initial_intent_groups.iter() {
				let intent_ids = BoundedVec::try_from(intent_group.intent_ids.clone())
					.expect("Too many Intents in IntentGroup");
				intent_ids.iter().for_each(|intent_id| {
					assert!(
						IntentInfos::<T>::contains_key(intent_id),
						"Genesis IntentGroup has no matching Intent."
					);
				});
				let name_payload: SchemaNamePayload = BoundedVec::try_from(
					intent_group.name.clone().into_bytes(),
				)
				.expect("Genesis IntentGroup name larger than {SCHEMA_NAME_BYTES_MAX} bytes}");
				let parsed_name = SchemaName::try_parse::<T>(name_payload, true)
					.expect("Bad Genesis IntentGroup name");
				Pallet::<T>::store_intent_group(
					intent_group.intent_group_id,
					intent_ids,
					&parsed_name,
				)
				.expect("Failed to set Schema in Genesis!");
			}

			// Set the maximums manually
			CurrentIntentIdentifierMaximum::<T>::put(self.initial_intent_identifier_max);
			CurrentSchemaIdentifierMaximum::<T>::put(self.initial_schema_identifier_max);
			CurrentIntentGroupIdentifierMaximum::<T>::put(self.initial_intent_group_identifier_max);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// REMOVED create_schema() at call index 0

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

		// REMOVED propose_to_create_schema() at call index 2
		// REMOVED create_schema_via_governance() at call index 3
		// REMOVED create_schema_v2() at call index 4
		// REMOVED propose_to_create_schema_v2 at call index 5
		// REMOVED create_schema_via_governance_v2 at call index 6
		// REMOVED create_schema_v3 at call index 7
		// REMOVED propose_to_create_schema_name at call index 8
		// REMOVED create_schema_name_via_governance at call index 9

		/// Creates a new Intent with a name (testnet)
		///
		/// # Events
		/// * [`Event::IntentCreated`]
		///
		/// # Errors
		/// * [`Error::IntentCountOverflow`] - The Intent count has exceeded its bounds
		/// * [`Error::InvalidSetting`] - Invalid setting is provided
		/// * [`Error::NameAlreadyExists`] - The name already exists
		/// * [`Error::InvalidSchemaNameEncoding`] - The name has an invalid encoding
		/// * [`Error::InvalidSchemaNameCharacters`] - The name contains invalid characters
		/// * [`Error::InvalidSchemaNameStructure`] - The name has an invalid structure (i.e., not `protocol.descriptor`)
		/// * [`Error::InvalidSchemaNameLength`] - The name exceeds the allowed overall name length
		/// * [`Error::InvalidSchemaNamespaceLength`] - The protocol portion of the name exceeds the max allowed length
		/// * [`Error::InvalidSchemaDescriptorLength`] - The descriptor portion of the name exceeds the max allowed length
		///
		#[pallet::call_index(10)]
		#[pallet::weight(T::WeightInfo::create_intent(settings.len() as u32))]
		pub fn create_intent(
			origin: OriginFor<T>,
			intent_name: SchemaNamePayload,
			payload_location: PayloadLocation,
			settings: BoundedVec<IntentSetting, T::MaxSchemaSettingsPerSchema>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let (intent_id, parsed_name) =
				Self::create_intent_for(intent_name, payload_location, settings)?;

			Self::deposit_event(Event::IntentCreated {
				key: sender,
				intent_id,
				intent_name: parsed_name.get_combined_name(),
			});

			Ok(())
		}

		/// Create an Intent by means of council approval
		///
		/// # Events
		/// * [`Event::IntentCreated`]
		///
		/// # Errors
		/// * [`Error::IntentCountOverflow`] - The Intent count has exceeded its bounds
		/// * [`Error::InvalidSetting`] - Invalid setting is provided
		/// * [`Error::NameAlreadyExists`] - The name already exists
		/// * [`Error::InvalidSchemaNameEncoding`] - The name has an invalid encoding
		/// * [`Error::InvalidSchemaNameCharacters`] - The name contains invalid characters
		/// * [`Error::InvalidSchemaNameStructure`] - The name has an invalid structure (i.e., not `protocol.descriptor`)
		/// * [`Error::InvalidSchemaNameLength`] - The name exceeds the allowed overall name length
		/// * [`Error::InvalidSchemaNamespaceLength`] - The protocol portion of the name exceeds the max allowed length
		/// * [`Error::InvalidSchemaDescriptorLength`] - The descriptor portion of the name exceeds the max allowed length
		///
		#[pallet::call_index(11)]
		#[pallet::weight(T::WeightInfo::create_intent_via_governance(settings.len() as u32))]
		pub fn create_intent_via_governance(
			origin: OriginFor<T>,
			creator_key: T::AccountId,
			payload_location: PayloadLocation,
			settings: BoundedVec<IntentSetting, T::MaxSchemaSettingsPerSchema>,
			intent_name: SchemaNamePayload,
		) -> DispatchResult {
			T::CreateSchemaViaGovernanceOrigin::ensure_origin(origin)?;
			let (intent_id, parsed_name) =
				Self::create_intent_for(intent_name, payload_location, settings)?;

			Self::deposit_event(Event::IntentCreated {
				key: creator_key,
				intent_id,
				intent_name: parsed_name.get_combined_name(),
			});
			Ok(())
		}

		/// Propose to create an Intent.  Creates a proposal for council approval to create an Intent
		///
		#[pallet::call_index(12)]
		#[pallet::weight(T::WeightInfo::propose_to_create_intent())]
		pub fn propose_to_create_intent(
			origin: OriginFor<T>,
			payload_location: PayloadLocation,
			settings: BoundedVec<IntentSetting, T::MaxSchemaSettingsPerSchema>,
			intent_name: SchemaNamePayload,
		) -> DispatchResult {
			let proposer = ensure_signed(origin)?;

			let proposal: Box<T::Proposal> = Box::new(
				(Call::<T>::create_intent_via_governance {
					creator_key: proposer.clone(),
					payload_location,
					settings,
					intent_name,
				})
				.into(),
			);
			T::ProposalProvider::propose_with_simple_majority(proposer, proposal)?;
			Ok(())
		}

		/// Create an intent group (testnet)
		///
		/// # Events
		/// * [`Event::IntentGroupCreated`]
		///
		/// # Errors
		/// * [`Error::IntentGroupCountOverflow`] - The Intent Group count has exceeded its bounds.
		/// * [`Error::NameAlreadyExists`] - The name already exists.
		/// * [`Error::InvalidSchemaNameEncoding`] - The name has an invalid encoding.
		/// * [`Error::InvalidSchemaNameCharacters`] - The name contains invalid characters.
		/// * [`Error::InvalidSchemaNameStructure`] - The name has an invalid structure (i.e., not `protocol.descriptor`).
		/// * [`Error::InvalidSchemaNameLength`] - The name exceeds the allowed overall name length.
		/// * [`Error::InvalidSchemaNamespaceLength`] - The protocol portion of the name exceeds the max allowed length.
		/// * [`Error::InvalidSchemaDescriptorLength`] - The descriptor portion of the name exceeds the max allowed length.
		/// * [`Error::InvalidIntentId`] - At least one of the specified [`IntentId`]s does not exist.
		///
		#[pallet::call_index(13)]
		#[pallet::weight(T::WeightInfo::create_intent_group(intent_ids.len() as u32))]
		pub fn create_intent_group(
			origin: OriginFor<T>,
			intent_group_name: SchemaNamePayload,
			intent_ids: BoundedVec<IntentId, T::MaxIntentsPerIntentGroup>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let (intent_group_id, parsed_name) =
				Self::create_intent_group_for(intent_group_name, intent_ids)?;

			Self::deposit_event(Event::IntentGroupCreated {
				key: sender,
				intent_group_id,
				intent_group_name: parsed_name.get_combined_name(),
			});
			Ok(())
		}

		/// Create an IntentGroup by means of council approval
		///
		/// # Events
		/// * [`Event::IntentGroupCreated`]
		///
		/// # Errors
		/// * [`Error::IntentGroupCountOverflow`] - The Intent Group count has exceeded its bounds
		/// * [`Error::NameAlreadyExists`] - The name already exists.
		/// * [`Error::InvalidSchemaNameEncoding`] - The name has an invalid encoding.
		/// * [`Error::InvalidSchemaNameCharacters`] - The name contains invalid characters.
		/// * [`Error::InvalidSchemaNameStructure`] - The name has an invalid structure (i.e., not `protocol.descriptor`).
		/// * [`Error::InvalidSchemaNameLength`] - The name exceeds the allowed overall name length.
		/// * [`Error::InvalidSchemaNamespaceLength`] - The protocol portion of the name exceeds the max allowed length.
		/// * [`Error::InvalidSchemaDescriptorLength`] - The descriptor portion of the name exceeds the max allowed length.
		/// * [`Error::InvalidIntentId`] - At least one of the specified [`IntentId`]s does not exist
		///
		#[pallet::call_index(14)]
		#[pallet::weight(
            T::WeightInfo::create_intent_group_via_governance(intent_ids.len() as u32)
        )]
		pub fn create_intent_group_via_governance(
			origin: OriginFor<T>,
			creator_key: T::AccountId,
			intent_group_name: SchemaNamePayload,
			intent_ids: BoundedVec<IntentId, T::MaxIntentsPerIntentGroup>,
		) -> DispatchResult {
			T::CreateSchemaViaGovernanceOrigin::ensure_origin(origin)?;
			let (intent_group_id, parsed_name) =
				Self::create_intent_group_for(intent_group_name, intent_ids)?;

			Self::deposit_event(Event::IntentGroupCreated {
				key: creator_key,
				intent_group_id,
				intent_group_name: parsed_name.get_combined_name(),
			});
			Ok(())
		}

		/// Propose to create an Intent Group. Creates a proposal for council approval to create an Intent Group.
		///
		#[pallet::call_index(15)]
		#[pallet::weight(T::WeightInfo::propose_to_create_intent_group())]
		pub fn propose_to_create_intent_group(
			origin: OriginFor<T>,
			intent_group_name: crate::types::SchemaNamePayload,
			intent_ids: BoundedVec<IntentId, T::MaxIntentsPerIntentGroup>,
		) -> DispatchResult {
			let proposer = ensure_signed(origin)?;

			let proposal: Box<T::Proposal> = Box::new(
				(Call::<T>::create_intent_group_via_governance {
					creator_key: proposer.clone(),
					intent_group_name,
					intent_ids,
				})
				.into(),
			);
			T::ProposalProvider::propose_with_simple_majority(proposer, proposal)?;
			Ok(())
		}

		/// Update an [`IntentGroup`] (testnet)
		/// Replaces an IntentGroup's list of Intents with a new list.
		///
		/// # Events
		/// * [`Event::IntentGroupUpdated`]
		///
		/// # Errors
		/// * [`Error::InvalidIntentGroupId`] - The specified [`IntentGroupId`] does not exist.
		/// * [`Error::InvalidIntentId`] - At least one of the specified [`IntentId`]s does not exist.
		/// * [`Error::TooManyIntentsInGroup`] - The update would result in too many Intents in the group.
		#[pallet::call_index(16)]
		#[pallet::weight(T::WeightInfo::update_intent_group(intent_ids.len() as u32))]
		pub fn update_intent_group(
			origin: OriginFor<T>,
			intent_group_id: IntentGroupId,
			intent_ids: BoundedVec<IntentId, T::MaxIntentsPerIntentGroup>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::update_intent_group_for(intent_group_id, intent_ids)?;
			Self::deposit_event(Event::IntentGroupUpdated { key: sender, intent_group_id });
			Ok(())
		}

		/// Update an IntentGroup by means of council approval.
		/// Replaces an IntentGroup's list of Intents with a new list.
		///
		/// # Events
		/// * [`Event::IntentGroupUpdated`]
		///
		/// # Errors
		/// * [`Error::InvalidIntentGroupId`] - The specified [`IntentGroupId`] does not exist.
		/// * [`Error::InvalidIntentId`] - At least one of the specified [`IntentId`]s does not exist.
		/// * [`Error::TooManyIntentsInGroup`] - The update would result in too many Intents in the group.
		///
		#[pallet::call_index(17)]
		#[pallet::weight(
            T::WeightInfo::update_intent_group_via_governance(intent_ids.len() as u32)
        )]
		pub fn update_intent_group_via_governance(
			origin: OriginFor<T>,
			updater_key: T::AccountId,
			intent_group_id: IntentGroupId,
			intent_ids: BoundedVec<IntentId, T::MaxIntentsPerIntentGroup>,
		) -> DispatchResult {
			T::CreateSchemaViaGovernanceOrigin::ensure_origin(origin)?;
			ensure!(
				IntentGroups::<T>::contains_key(intent_group_id),
				Error::<T>::InvalidIntentGroupId
			);
			Self::update_intent_group_for(intent_group_id, intent_ids)?;

			Self::deposit_event(Event::IntentGroupUpdated { key: updater_key, intent_group_id });
			Ok(())
		}

		/// Propose to update an Intent Group. Creates a proposal for council approval to update an existing Intent Group.
		///
		#[pallet::call_index(18)]
		#[pallet::weight(T::WeightInfo::propose_to_update_intent_group())]
		pub fn propose_to_update_intent_group(
			origin: OriginFor<T>,
			intent_group_id: IntentGroupId,
			intent_ids: BoundedVec<IntentId, T::MaxIntentsPerIntentGroup>,
		) -> DispatchResult {
			let proposer = ensure_signed(origin)?;

			let proposal: Box<T::Proposal> = Box::new(
				(Call::<T>::update_intent_group_via_governance {
					updater_key: proposer.clone(),
					intent_group_id,
					intent_ids,
				})
				.into(),
			);
			T::ProposalProvider::propose_with_simple_majority(proposer, proposal)?;
			Ok(())
		}

		/// Propose to create a schema.  Creates a proposal for council approval to create a schema
		///
		#[pallet::call_index(19)]
		#[pallet::weight(T::WeightInfo::propose_to_create_schema_v3())]
		pub fn propose_to_create_schema_v3(
			origin: OriginFor<T>,
			intent_id: IntentId,
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
		) -> DispatchResult {
			let proposer = ensure_signed(origin)?;

			let proposal: Box<T::Proposal> = Box::new(
				(Call::<T>::create_schema_via_governance_v3 {
					creator_key: proposer.clone(),
					intent_id,
					model,
					model_type,
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
		///
		#[pallet::call_index(20)]
		#[pallet::weight(T::WeightInfo::create_schema_via_governance_v3(model.len() as u32))]
		pub fn create_schema_via_governance_v3(
			origin: OriginFor<T>,
			creator_key: T::AccountId,
			intent_id: IntentId,
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
		) -> DispatchResult {
			T::CreateSchemaViaGovernanceOrigin::ensure_origin(origin)?;
			let schema_id = Self::create_schema_for(intent_id, model, model_type)?;

			Self::deposit_event(Event::SchemaCreated { key: creator_key, schema_id });
			Ok(())
		}

		/// Adds a given schema to storage. (testnet)
		///
		/// The schema in question must be of length
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
		#[pallet::call_index(21)]
		#[pallet::weight(T::WeightInfo::create_schema_v4(model.len() as u32))]
		pub fn create_schema_v4(
			origin: OriginFor<T>,
			intent_id: IntentId,
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let schema_id = Self::create_schema_for(intent_id, model, model_type)?;

			Self::deposit_event(Event::SchemaCreated { key: sender, schema_id });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Set the maximum schema id to the supplied value.
		#[cfg(any(feature = "std", feature = "runtime-benchmarks", test))]
		pub fn set_schema_count(n: SchemaId) {
			<CurrentSchemaIdentifierMaximum<T>>::set(n);
		}

		/// Set the maximum intent id to the supplied value.
		#[cfg(any(feature = "std", feature = "runtime-benchmarks", test))]
		pub fn set_intent_count(n: IntentId) {
			<CurrentIntentIdentifierMaximum<T>>::set(n);
		}

		/// Lowest-level insertion function for a [`SchemaInfo`] and [`SchemaPayload`] into storage,
		/// using an already-allocated [`SchemaId`]
		pub fn store_schema_info_and_payload(
			schema_id: SchemaId,
			schema_info: SchemaInfo,
			schema_payload: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
		) -> Result<(), DispatchError> {
			<SchemaInfos<T>>::insert(schema_id, schema_info);
			<SchemaPayloads<T>>::insert(schema_id, schema_payload);
			Ok(())
		}

		/// Inserts both the [`SchemaInfo`] and Schema Payload into storage
		/// Updates the `CurrentSchemaIdentifierMaximum` storage
		pub fn add_schema(
			intent_id: IntentId,
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
			payload_location: PayloadLocation,
			settings: IntentSettings,
			status: SchemaStatus,
		) -> Result<SchemaId, DispatchError> {
			let schema_id = Self::get_next_schema_id()?;
			let schema_info =
				SchemaInfo { intent_id, model_type, payload_location, settings, status };
			<CurrentSchemaIdentifierMaximum<T>>::set(schema_id);
			Self::store_schema_info_and_payload(schema_id, schema_info, model)?;

			Ok(schema_id)
		}

		/// Lowest-level insertion function for a [`IntentInfo`] into storage,
		/// using an already-allocated [`IntentId`]
		pub fn store_intent_info(
			intent_id: IntentId,
			intent_info: IntentInfo,
			intent_name: &SchemaName,
		) -> Result<(), DispatchError> {
			NameToMappedEntityIds::<T>::insert(
				&intent_name.namespace,
				&intent_name.descriptor,
				MappedEntityIdentifier::Intent(intent_id),
			);
			<IntentInfos<T>>::insert(intent_id, intent_info);

			Ok(())
		}

		/// Inserts the [`IntentInfo`] into storage
		/// Updates the `CurrentIntentIdentifierMaximum` storage
		/// Does little validation, as this is an internal method intended to be called
		/// by higher-level extrinsics that perform various validations.
		///
		/// # Errors
		/// * [`Error::IntentCountOverflow`]
		pub fn add_intent(
			payload_location: PayloadLocation,
			settings: BoundedVec<IntentSetting, T::MaxSchemaSettingsPerSchema>,
			intent_name: &SchemaName,
		) -> Result<IntentId, DispatchError> {
			let intent_id = Self::get_next_intent_id()?;
			let mut set_settings = IntentSettings::all_disabled();
			if !settings.is_empty() {
				for i in settings.into_inner() {
					set_settings.set(i);
				}
			}

			let intent_info = IntentInfo { payload_location, settings: set_settings };
			<CurrentIntentIdentifierMaximum<T>>::set(intent_id);
			Self::store_intent_info(intent_id, intent_info, intent_name)?;

			Ok(intent_id)
		}

		/// Lowest-level insertion function for a [`IntentGroup`] into storage,
		/// using an already-allocated [`IntentGroupId`]
		pub fn store_intent_group(
			intent_group_id: IntentGroupId,
			intent_ids: BoundedVec<IntentId, T::MaxIntentsPerIntentGroup>,
			intent_group_name: &SchemaName,
		) -> Result<(), DispatchError> {
			NameToMappedEntityIds::<T>::insert(
				&intent_group_name.namespace,
				&intent_group_name.descriptor,
				MappedEntityIdentifier::IntentGroup(intent_group_id),
			);
			Self::update_intent_group_storage(intent_group_id, intent_ids)?;

			Ok(())
		}

		/// Inserts a list of [`IntentId`]s with a new [`IntentGroupId`] in storage, and adds.
		/// a new name mapping to the [`IntentGroupId`].
		/// Updates the `CurrentIntentGroupIdentifierMaximum` storage.
		/// Does little validation, as this is an internal method intended to be called by.
		/// higher-level extrinsics that perform the validations.
		///
		/// Errors
		/// * [`Error::IntentGroupCountOverflow`]
		pub fn add_intent_group(
			intent_group_name: &SchemaName,
			intent_ids: BoundedVec<IntentId, T::MaxIntentsPerIntentGroup>,
		) -> Result<IntentGroupId, DispatchError> {
			let intent_group_id = Self::get_next_intent_group_id()?;

			<CurrentIntentGroupIdentifierMaximum<T>>::set(intent_group_id);
			Self::store_intent_group(intent_group_id, intent_ids, intent_group_name)?;
			Ok(intent_group_id)
		}

		/// Updates the list of [`IntentId`]s associated with a given [`IntentGroupId`]
		/// Does little validation, as this is an internal method intended to be called by
		/// higher-level extrinsics that perform the validations.
		///
		pub fn update_intent_group_storage(
			intent_group_id: IntentGroupId,
			intent_ids: BoundedVec<IntentId, T::MaxIntentsPerIntentGroup>,
		) -> DispatchResult {
			IntentGroups::<T>::set(intent_group_id, Some(IntentGroup { intent_ids }));
			Ok(())
		}

		/// Retrieve a schema by id
		pub fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponseV2> {
			match (SchemaInfos::<T>::get(schema_id), SchemaPayloads::<T>::get(schema_id)) {
				(Some(schema_info), Some(payload)) => {
					let model_vec: Vec<u8> = payload.into_inner();
					let response = SchemaResponseV2 {
						schema_id,
						intent_id: schema_info.intent_id,
						model: model_vec,
						model_type: schema_info.model_type,
						payload_location: schema_info.payload_location,
						settings: schema_info.settings.0.iter().collect::<Vec<IntentSetting>>(),
						status: schema_info.status,
					};
					Some(response)
				},
				(None, Some(_)) | (Some(_), None) => {
					log::error!("Corrupted state for schema {schema_id:?}, Should never happen!");
					None
				},
				(None, None) => None,
			}
		}

		/// Retrieve a schema info by id
		pub fn get_schema_info_by_id(schema_id: SchemaId) -> Option<SchemaInfoResponse> {
			if let Some(schema_info) = SchemaInfos::<T>::get(schema_id) {
				let response = SchemaInfoResponse {
					schema_id,
					intent_id: schema_info.intent_id,
					model_type: schema_info.model_type,
					payload_location: schema_info.payload_location,
					settings: schema_info.settings.0.iter().collect::<Vec<IntentSetting>>(),
					status: schema_info.status,
				};
				return Some(response);
			}
			None
		}

		/// Retrieve an Intent by its ID
		pub fn get_intent_by_id(intent_id: IntentId) -> Option<IntentResponse> {
			IntentInfos::<T>::get(intent_id).map(|intent_info| IntentResponse {
				intent_id,
				payload_location: intent_info.payload_location,
				settings: intent_info.settings.0.iter().collect::<Vec<IntentSetting>>(),
				schema_ids: None,
			})
		}

		/// Retrieve an Intent by its ID
		/// NOTE: This must not be called on-chain due to inability to determine weight
		pub fn get_intent_by_id_with_schemas(intent_id: IntentId) -> Option<IntentResponse> {
			let response = Self::get_intent_by_id(intent_id);
			response.map(|mut intent| {
				intent.schema_ids = Some(
					SchemaInfos::<T>::iter()
						.filter(|(_, info)| info.intent_id == intent_id)
						.map(|(id, _)| id)
						.collect::<Vec<SchemaId>>(),
				);
				intent
			})
		}

		/// Retrieve an IntentGroup by its ID
		pub fn get_intent_group_by_id(
			intent_group_id: IntentGroupId,
		) -> Option<IntentGroupResponse> {
			if let Some(intent_group) = IntentGroups::<T>::get(intent_group_id) {
				return Some(IntentGroupResponse {
					intent_group_id,
					intent_ids: intent_group.intent_ids.into(),
				});
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
			match *model_type {
				ModelType::Parquet => {
					serde_json::from_slice::<ParquetModel>(model)
						.map_err(|_| Error::<T>::InvalidSchema)?;
				},
				ModelType::AvroBinary => serde::validate_json_model(model.clone().into_inner())
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

		/// Get the next available [`IntentId`]
		///
		/// # Errors
		/// * [`Error::IntentCountOverflow`]
		///
		fn get_next_intent_id() -> Result<IntentId, DispatchError> {
			let next = CurrentIntentIdentifierMaximum::<T>::get()
				.checked_add(1)
				.ok_or(Error::<T>::IntentCountOverflow)?;

			Ok(next)
		}

		/// Get the next available [`IntentGroupId`]
		///
		/// Errors
		/// * [`Error::IntentGroupCountOverflow`]
		///
		fn get_next_intent_group_id() -> Result<IntentGroupId, DispatchError> {
			let next = CurrentIntentGroupIdentifierMaximum::<T>::get()
				.checked_add(1)
				.ok_or(Error::<T>::IntentGroupCountOverflow)?;
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
			intent_id: IntentId,
			model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>,
			model_type: ModelType,
		) -> Result<SchemaId, DispatchError> {
			Self::ensure_valid_model(&model_type, &model)?;
			ensure!(
				model.len() >= T::MinSchemaModelSizeBytes::get() as usize,
				Error::<T>::LessThanMinSchemaModelBytes
			);
			ensure!(
				model.len() <= GovernanceSchemaModelMaxBytes::<T>::get() as usize,
				Error::<T>::ExceedsMaxSchemaModelBytes
			);
			let intent_info =
				IntentInfos::<T>::get(intent_id).ok_or(Error::<T>::InvalidIntentId)?;
			let schema_id = Self::add_schema(
				intent_id,
				model,
				model_type,
				intent_info.payload_location,
				intent_info.settings,
				SchemaStatus::Active,
			)?;
			Ok(schema_id)
		}

		/// Adds a given Intent to storage. If the pallet's maximum Intent limit has been
		/// fulfilled by the time this extrinsic is called, an IntentCountOverflow error
		/// will be thrown.
		///
		/// # Errors
		/// * [`Error::IntentCountOverflow`] - The Intent count has exceeded its bounds
		/// * [`Error::InvalidSetting`] - Invalid setting is provided
		/// * [`Error::NameAlreadyExists`] - The name already exists
		/// * [`Error::InvalidSchemaNameEncoding`] - The name has an invalid encoding
		/// * [`Error::InvalidSchemaNameCharacters`] - The name contains invalid characters
		/// * [`Error::InvalidSchemaNameStructure`] - The name has an invalid structure (i.e., not `protocol.descriptor`)
		/// * [`Error::InvalidSchemaNameLength`] - The name exceed the allowed overall name length
		/// * [`Error::InvalidSchemaNamespaceLength`] - The protocol portion of the name exceeds the max allowed length
		/// * [`Error::InvalidSchemaDescriptorLength`] - The descriptor portion of the name exceeds the max allowed length
		pub fn create_intent_for(
			intent_name_payload: SchemaNamePayload,
			payload_location: PayloadLocation,
			settings: BoundedVec<IntentSetting, T::MaxSchemaSettingsPerSchema>,
		) -> Result<(IntentId, SchemaName), DispatchError> {
			// AppendOnly is only valid for Itemized payload location
			ensure!(
				!settings.contains(&IntentSetting::AppendOnly) ||
					payload_location == PayloadLocation::Itemized,
				Error::<T>::InvalidSetting
			);
			// SignatureRequired is only valid for Itemized and Paginated payload locations
			ensure!(
				!settings.contains(&IntentSetting::SignatureRequired) ||
					payload_location == PayloadLocation::Itemized ||
					payload_location == PayloadLocation::Paginated,
				Error::<T>::InvalidSetting
			);
			let parsed_name = Self::parse_and_verify_new_name(&intent_name_payload)?;
			let intent_id = Self::add_intent(payload_location, settings, &parsed_name)?;
			Ok((intent_id, parsed_name))
		}

		/// Adds a given Intent to storage. If the pallet's maximum Intent limit has been
		/// fulfilled by the time this extrinsic is called, an IntentCountOverflow error
		/// will be thrown.
		///
		/// # Errors
		/// * [`Error::IntentGroupCountOverflow`] - The schema count has exceeded its bounds
		/// * [`Error::NameAlreadyExists`] - The name already exists
		/// * [`Error::InvalidSchemaNameEncoding`] - The name has an invalid encoding
		/// * [`Error::InvalidSchemaNameCharacters`] - The name contains invalid characters
		/// * [`Error::InvalidSchemaNameStructure`] - The name has an invalid structure (i.e., not `protocol.descriptor`)
		/// * [`Error::InvalidSchemaNameLength`] - The name exceed the allowed overall name length
		/// * [`Error::InvalidSchemaNamespaceLength`] - The protocol portion of the name exceeds the max allowed length
		/// * [`Error::InvalidSchemaDescriptorLength`] - The descriptor portion of the name exceeds the max allowed length
		/// * [`Error::InvalidIntentId`] - One of the [`IntentId`]s specified does not exist
		pub fn create_intent_group_for(
			intent_group_name: SchemaNamePayload,
			intent_ids: BoundedVec<IntentId, T::MaxIntentsPerIntentGroup>,
		) -> Result<(IntentGroupId, SchemaName), DispatchError> {
			let intent_group_name = Self::parse_and_verify_new_name(&intent_group_name)?;
			Self::validate_intent_ids(&intent_ids)?;
			let intent_id = Self::add_intent_group(&intent_group_name, intent_ids)?;
			Ok((intent_id, intent_group_name))
		}

		/// Update the list of [`IntentId`]s associated with a given [`IntentGroupId`]
		///
		/// # Errors
		/// * [`Error::InvalidIntentGroupId`] - The specified [`IntentGroupId`] doesn't exist
		/// * [`Error::InvalidIntentId`] - One of the [`IntentId`]s specified doesn't exist
		pub fn update_intent_group_for(
			intent_group_id: IntentGroupId,
			intent_ids: BoundedVec<IntentId, T::MaxIntentsPerIntentGroup>,
		) -> Result<(), DispatchError> {
			ensure!(
				IntentGroups::<T>::contains_key(intent_group_id),
				Error::<T>::InvalidIntentGroupId
			);
			Self::validate_intent_ids(&intent_ids)?;
			Self::update_intent_group_storage(intent_group_id, intent_ids)?;
			Ok(())
		}

		/// Validate that all items in a list of [`IntentId`]s exist
		///
		/// # Errors
		/// * [`Error::InvalidIntentId`] - At least one of the specified [`IntentId`]s does not exist
		///
		pub fn validate_intent_ids(
			intent_ids: &BoundedVec<IntentId, T::MaxIntentsPerIntentGroup>,
		) -> Result<(), DispatchError> {
			let max_intent_id = CurrentIntentIdentifierMaximum::<T>::get();
			intent_ids.iter().try_for_each::<_, _>(|intent_id| {
				ensure!(*intent_id <= max_intent_id, Error::<T>::InvalidIntentId);
				Ok::<(), DispatchError>(())
			})?;
			Ok(())
		}

		/// a method to return all versions of a schema name with their schemaIds
		/// Warning: Must only get called from RPC, since the number of DB accesses is not deterministic
		pub fn get_schema_versions(schema_name: Vec<u8>) -> Option<Vec<SchemaVersionResponse>> {
			if let Some(mut entities) = Self::get_intent_or_group_ids_by_name(schema_name.clone()) {
				entities.retain(|e| matches!(e.entity_id, MappedEntityIdentifier::Intent(_)));
				return Some(
					entities
						.iter()
						.flat_map(|e| {
							let schema_ids: Vec<SchemaId> = SchemaInfos::<T>::iter()
								.filter_map(|(id, info)| match e.entity_id {
									MappedEntityIdentifier::Intent(intent_id) => {
										if info.intent_id == intent_id {
											Some(id)
										} else {
											None
										}
									},
									_ => None,
								})
								.collect();
							schema_ids.convert_to_response(&e.name)
						})
						.collect::<Vec<SchemaVersionResponse>>(),
				);
			}
			None
		}

		/// method to return the entity ID (Intent or IntentGroup) associated with a name
		/// Warning: Must only get called from RPC, since the number of DB accesses is not deterministic
		pub fn get_intent_or_group_ids_by_name(
			entity_name: Vec<u8>,
		) -> Option<Vec<NameLookupResponse>> {
			let parsed_name =
				FullyQualifiedName::try_parse::<T>(BoundedVec::try_from(entity_name).ok()?, false)
					.ok()?;

			if parsed_name.descriptor_exists() {
				if let Ok(id) = NameToMappedEntityIds::<T>::try_get(
					&parsed_name.namespace,
					&parsed_name.descriptor,
				) {
					return Some(vec![id.convert_to_response(&parsed_name)]);
				}
				return None;
			}

			let responses: Vec<NameLookupResponse> =
				NameToMappedEntityIds::<T>::iter_prefix(&parsed_name.namespace)
					.map(|(descriptor, val)| {
						val.convert_to_response(&parsed_name.new_with_descriptor(descriptor))
					})
					.collect();

			(!responses.is_empty()).then_some(responses)
		}

		/// Parses and validates a new name and makes sure it does not already exist
		/// # Errors
		/// * [`Error::NameAlreadyExists`] - The name already exists
		/// * [`Error::InvalidSchemaNameEncoding`] - The name has an invalid encoding
		/// * [`Error::InvalidSchemaNameCharacters`] - The name contains invalid characters
		/// * [`Error::InvalidSchemaNameStructure`] - The name has an invalid structure (i.e., not `protocol.descriptor`)
		/// * [`Error::InvalidSchemaNameLength`] - The name exceed the allowed overall name length
		/// * [`Error::InvalidSchemaNamespaceLength`] - The protocol portion of the name exceeds the max allowed length
		/// * [`Error::InvalidSchemaDescriptorLength`] - The descriptor portion of the name exceeds the max allowed length
		fn parse_and_verify_new_name(
			name: &SchemaNamePayload,
		) -> Result<SchemaName, DispatchError> {
			let parsed_name = SchemaName::try_parse::<T>(name.clone(), true)?;

			ensure!(
				!NameToMappedEntityIds::<T>::contains_key(
					&parsed_name.namespace,
					&parsed_name.descriptor,
				),
				Error::<T>::NameAlreadyExists,
			);

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

	/// Sets Intent count.
	fn set_intent_count(intent_id: IntentId) {
		Self::set_intent_count(intent_id)
	}

	/// Creates a schema.
	fn create_schema(
		intent_id: IntentId,
		model: Vec<u8>,
		model_type: ModelType,
		payload_location: PayloadLocation,
	) -> Result<SchemaId, DispatchError> {
		let model: BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit> =
			model.try_into().unwrap();
		Self::ensure_valid_model(&model_type, &model)?;
		let schema_id = Self::add_schema(
			intent_id,
			model,
			model_type,
			payload_location,
			IntentSettings::default(),
			SchemaStatus::Active,
		)?;
		Ok(schema_id)
	}

	/// Creates an Intent
	fn create_intent(
		name_payload: Vec<u8>,
		payload_location: PayloadLocation,
		settings: Vec<IntentSetting>,
	) -> Result<IntentId, DispatchError> {
		let name = BoundedVec::try_from(name_payload).unwrap();
		let (intent_id, _) =
			Self::create_intent_for(name, payload_location, settings.try_into().unwrap())?;
		Ok(intent_id)
	}
}

impl<T: Config> SchemaValidator<SchemaId> for Pallet<T> {
	fn are_all_intent_ids_valid(intent_ids: &[IntentId]) -> bool {
		let latest_issue_intent_id = CurrentIntentIdentifierMaximum::<T>::get();
		intent_ids.iter().all(|id| id <= &latest_issue_intent_id)
	}

	#[cfg(any(feature = "std", feature = "runtime-benchmarks", test))]
	fn set_schema_count(n: SchemaId) {
		Self::set_schema_count(n);
	}

	#[cfg(any(feature = "std", feature = "runtime-benchmarks", test))]
	fn set_intent_count(n: IntentId) {
		Self::set_intent_count(n);
	}
}

impl<T: Config> SchemaProvider<SchemaId> for Pallet<T> {
	fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponseV2> {
		Self::get_schema_by_id(schema_id)
	}

	fn get_schema_info_by_id(schema_id: SchemaId) -> Option<SchemaInfoResponse> {
		Self::get_schema_info_by_id(schema_id)
	}

	fn get_intent_by_id(intent_id: IntentId) -> Option<IntentResponse> {
		Self::get_intent_by_id(intent_id)
	}
}
