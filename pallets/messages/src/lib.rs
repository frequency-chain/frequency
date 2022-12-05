//! # Messages pallet
//! A pallet for storing messages.
//!
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Runtime API: `MessagesRuntimeApi`](../pallet_messages_runtime_api/trait.MessagesRuntimeApi.html)
//! - [Custom RPC API: `MessagesApiServer`](../pallet_messages_rpc/trait.MessagesApiServer.html)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
//!
//! ## Overview
//!
//! Messages allow for discovery of new content matching a given Schema.
//! Each message MUST have either a provider or a validated source and provider.
//! Message is the metadata of the source, order, schema for a payload or referenced payload.
//!
//! The Messages Pallet provides functions for:
//!
//! - Adding messages for a given schema.
//! - Enabling retrieving messages for a given schema.
//! - (FUTURE) Removing messages after expiration
//!
//! ## Terminology
//!
//! - **Message:** A message that matches a registered Schema (on-chain or off-chain).
//! - **Payload:** The data in a `Message` that matches a `Schema`.
//! - **MSA Id:** The 64 bit unsigned integer associated with an `Message Source Account`.
//! - **MSA:** Message Source Account. A registered identifier with the [MSA pallet](../pallet_msa/index.html).
//! - **Schema:** A registered data structure and the settings around it. (See [Schemas pallet](../pallet_schemas/index.html))
//! - **Schema Id:** A U16 bit identifier for a schema stored on-chain.
//!
//! ## Implementations
//! - None
//!
// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

mod types;

use frame_support::{ensure, pallet_prelude::Weight, traits::Get, BoundedVec};
use sp_runtime::{traits::One, DispatchError};
use sp_std::{collections::btree_map::BTreeMap, convert::TryInto, prelude::*};

use codec::Encode;
use common_primitives::{
	messages::*,
	msa::{
		DelegatorId, MessageSourceId, MsaLookup, MsaValidator, ProviderId, SchemaGrantValidator,
	},
	schema::*,
};

#[cfg(feature = "runtime-benchmarks")]
use common_primitives::benchmarks::{MsaBenchmarkHelper, SchemaBenchmarkHelper};

pub use pallet::*;
pub use types::*;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// A type that will supply MSA related information
		type MsaInfoProvider: MsaLookup + MsaValidator<AccountId = Self::AccountId>;

		/// A type that will validate schema grants
		type SchemaGrantValidator: SchemaGrantValidator<Self::BlockNumber>;

		/// A type that will supply schema related information.
		type SchemaProvider: SchemaProvider<SchemaId>;

		/// The maximum number of messages in a block.
		#[pallet::constant]
		type MaxMessagesPerBlock: Get<u32>;

		/// The maximum size of a message payload bytes.
		#[pallet::constant]
		type MaxMessagePayloadSizeBytes: Get<u32> + Clone;

		#[cfg(feature = "runtime-benchmarks")]
		/// A set of helper functions for benchmarking.
		type MsaBenchmarkHelper: MsaBenchmarkHelper<Self::AccountId>;

		#[cfg(feature = "runtime-benchmarks")]
		/// A set of helper functions for benchmarking.
		type SchemaBenchmarkHelper: SchemaBenchmarkHelper;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// A temporary storage of messages, given a schema id, for a duration of block period.
	/// At the start of the next block this storage is cleared and moved into Messages storage.
	/// - Value: List of Messages
	#[pallet::storage]
	#[pallet::getter(fn get_block_messages)]
	pub(super) type BlockMessages<T: Config> = StorageValue<
		_,
		BoundedVec<(Message<T::MaxMessagePayloadSizeBytes>, SchemaId), T::MaxMessagesPerBlock>,
		ValueQuery,
	>;

	/// A permanent storage for messages mapped by block number and schema id.
	/// - Keys: BlockNumber, Schema Id
	/// - Value: List of Messages
	#[pallet::storage]
	#[pallet::getter(fn get_messages)]
	pub(super) type Messages<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::BlockNumber,
		Twox64Concat,
		SchemaId,
		BoundedVec<Message<T::MaxMessagePayloadSizeBytes>, T::MaxMessagesPerBlock>,
		ValueQuery,
	>;

	#[pallet::error]
	pub enum Error<T> {
		/// Too many messages are added to existing block
		TooManyMessagesInBlock,

		/// Message payload size is too large
		ExceedsMaxMessagePayloadSizeBytes,

		/// Type Conversion Overflow
		TypeConversionOverflow,

		/// Invalid Message Source Account
		InvalidMessageSourceAccount,

		/// Invalid SchemaId or Schema not found
		InvalidSchemaId,

		/// UnAuthorizedDelegate
		UnAuthorizedDelegate,

		/// Invalid payload location
		InvalidPayloadLocation,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Messages are stored for a specified schema id and block number
		MessagesStored {
			/// The schema for these messages
			schema_id: SchemaId,
			/// The block number for these messages
			block_number: T::BlockNumber,
			/// Number of messages in this block for this schema
			count: u16,
		},
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(current: T::BlockNumber) -> Weight {
			let prev_block = current - T::BlockNumber::one();
			Self::move_messages_into_final_storage(prev_block)
			// TODO: add retention policy execution GitHub Issue: #126 and #25
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Adds a message for a resource hosted on IPFS. The payload storage will
		/// contain both a
		/// [CID](https://docs.ipfs.io/concepts/content-addressing/#identifier-formats)
		/// as well as a 32-bit payload length.
		/// The actual payload will be on IPFS
		///
		/// # Events
		/// * [`Event::MessagesStored`] - In the next block
		///
		/// # Errors
		/// * [`Error::ExceedsMaxMessagePayloadSizeBytes`] - Payload is too large
		/// * [`Error::InvalidSchemaId`] - Schema not found
		/// * [`Error::InvalidPayloadLocation`] - The schema is not an IPFS payload location
		/// * [`Error::InvalidMessageSourceAccount`] - Origin must be from an MSA
		/// * [`Error::TooManyMessagesInBlock`] - Block is full of messages already
		/// * [`Error::TypeConversionOverflow`] - Failed to add the message to storage as it is very full
		///
		#[pallet::weight(T::WeightInfo::add_ipfs_message(cid.len() as u32))]
		pub fn add_ipfs_message(
			origin: OriginFor<T>,
			#[pallet::compact] schema_id: SchemaId,
			cid: Vec<u8>,
			#[pallet::compact] payload_length: u32,
		) -> DispatchResult {
			let provider_key = ensure_signed(origin)?;
			let payload_tuple: OffchainPayloadType = (cid.clone(), payload_length);
			let bounded_payload: BoundedVec<u8, T::MaxMessagePayloadSizeBytes> = payload_tuple
				.encode()
				.try_into()
				.map_err(|_| Error::<T>::ExceedsMaxMessagePayloadSizeBytes)?;

			let schema = T::SchemaProvider::get_schema_by_id(schema_id);
			ensure!(schema.is_some(), Error::<T>::InvalidSchemaId);
			ensure!(
				schema.unwrap().payload_location == PayloadLocation::IPFS,
				Error::<T>::InvalidPayloadLocation
			);

			let provider_msa_id = Self::find_msa_id(&provider_key)?;
			Self::add_message(provider_msa_id, None, bounded_payload, schema_id)?;

			Ok(())
		}
		/// Add an on-chain message for a given schema id.
		///
		/// # Events
		/// * [`Event::MessagesStored`] - In the next block
		///
		/// # Errors
		/// * [`Error::ExceedsMaxMessagePayloadSizeBytes`] - Payload is too large
		/// * [`Error::InvalidSchemaId`] - Schema not found
		/// * [`Error::InvalidPayloadLocation`] - The schema is not an IPFS payload location
		/// * [`Error::InvalidMessageSourceAccount`] - Origin must be from an MSA
		/// * [`Error::UnAuthorizedDelegate`] - Trying to add a message without a proper delegation between the origin and the on_behalf_of MSA
		/// * [`Error::TooManyMessagesInBlock`] - Block is full of messages already
		/// * [`Error::TypeConversionOverflow`] - Failed to add the message to storage as it is very full
		///
		#[pallet::weight(T::WeightInfo::add_onchain_message(payload.len() as u32))]
		pub fn add_onchain_message(
			origin: OriginFor<T>,
			on_behalf_of: Option<MessageSourceId>,
			#[pallet::compact] schema_id: SchemaId,
			payload: Vec<u8>,
		) -> DispatchResult {
			let provider_key = ensure_signed(origin)?;

			let bounded_payload: BoundedVec<u8, T::MaxMessagePayloadSizeBytes> =
				payload.try_into().map_err(|_| Error::<T>::ExceedsMaxMessagePayloadSizeBytes)?;

			let schema = T::SchemaProvider::get_schema_by_id(schema_id);
			ensure!(schema.is_some(), Error::<T>::InvalidSchemaId);
			ensure!(
				schema.unwrap().payload_location == PayloadLocation::OnChain,
				Error::<T>::InvalidPayloadLocation
			);

			let provider_msa_id = Self::find_msa_id(&provider_key)?;
			let provider_id = ProviderId(provider_msa_id);

			let current_block = frame_system::Pallet::<T>::block_number();
			// On-chain messages either are sent from the user themselves, or on behalf of another MSA Id
			let maybe_delegator = match on_behalf_of {
				Some(delegator_msa_id) => {
					let delegator_id = DelegatorId(delegator_msa_id);
					T::SchemaGrantValidator::ensure_valid_schema_grant(
						provider_id,
						delegator_id,
						schema_id,
						current_block,
					)
					.map_err(|_| Error::<T>::UnAuthorizedDelegate)?;
					delegator_id
				},
				None => DelegatorId(provider_msa_id), // Delegate is also the Provider
			};

			Self::add_message(
				provider_msa_id,
				Some(maybe_delegator.into()),
				bounded_payload,
				schema_id,
			)?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Stores a message for a given schema id.
	///
	/// # Errors
	/// * [`Error::TooManyMessagesInBlock`]
	/// * [`Error::TypeConversionOverflow`]
	///
	pub fn add_message(
		provider_msa_id: MessageSourceId,
		msa_id: Option<MessageSourceId>,
		payload: BoundedVec<u8, T::MaxMessagePayloadSizeBytes>,
		schema_id: SchemaId,
	) -> Result<Message<T::MaxMessagePayloadSizeBytes>, DispatchError> {
		<BlockMessages<T>>::try_mutate(
			|existing_messages| -> Result<Message<T::MaxMessagePayloadSizeBytes>, DispatchError> {
				let current_size: u16 = existing_messages
					.len()
					.try_into()
					.map_err(|_| Error::<T>::TypeConversionOverflow)?;

				let msg = Message {
					payload, // size is checked on top of extrinsic
					provider_msa_id,
					msa_id,
					index: current_size,
				};

				existing_messages
					.try_push((msg.clone(), schema_id))
					.map_err(|_| Error::<T>::TooManyMessagesInBlock)?;

				Ok(msg)
			},
		)
	}

	/// Resolve an MSA from an account key(key)
	/// An MSA Id associated with the account key is returned, if one exists.
	///
	/// # Errors
	/// * [`Error::InvalidMessageSourceAccount`]
	///
	pub fn find_msa_id(key: &T::AccountId) -> Result<MessageSourceId, DispatchError> {
		Ok(T::MsaInfoProvider::ensure_valid_msa_key(key)
			.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?)
	}

	/// Gets a messages for a given schema-id and block-number.
	///
	/// Payload location is included to map to correct response (To avoid fetching the schema in this method)
	///
	/// Result is a vector of [`MessageResponse`].
	///
	pub fn get_messages_by_schema_and_block(
		schema_id: SchemaId,
		schema_payload_location: PayloadLocation,
		block_number: T::BlockNumber,
	) -> Vec<MessageResponse> {
		let block_number_value: u32 = block_number.try_into().unwrap_or_default();

		<Messages<T>>::get(block_number, schema_id)
			.into_inner()
			.iter()
			.map(|msg| msg.map_to_response(block_number_value, schema_payload_location))
			.collect()
	}

	/// Moves messages from temporary storage `BlockMessages` into final storage `Messages`
	/// and calculates execution weight
	///
	/// Called inside of `on_initialize`
	///
	/// # Events
	/// * [`Event::MessagesStored`]
	///
	/// Returns execution weights
	///
	fn move_messages_into_final_storage(block_number: T::BlockNumber) -> Weight {
		let mut map = BTreeMap::new();
		let block_messages = BlockMessages::<T>::get();
		let message_count = block_messages.len() as u32;
		let mut schema_count = 0u32;

		if message_count == 0 {
			return T::DbWeight::get().reads(1)
		}

		// grouping messages by schema_id
		for (m, schema_id) in block_messages {
			let list = map.entry(schema_id).or_insert(vec![]);
			list.push(m);
		}

		// insert into storage and create events
		for (schema_id, messages) in map {
			let count = messages.len() as u16;
			let bounded_vec: BoundedVec<_, _> = messages.try_into().unwrap_or_default();

			if bounded_vec.is_empty() {
				log::warn!("empty bounded_vec for schema id {}", schema_id);
				continue
			}
			Messages::<T>::insert(&block_number, schema_id, &bounded_vec);
			Self::deposit_event(Event::MessagesStored { schema_id, block_number, count });
			schema_count += 1;
		}

		BlockMessages::<T>::set(BoundedVec::default());
		T::WeightInfo::on_initialize(message_count, schema_count)
	}
}
