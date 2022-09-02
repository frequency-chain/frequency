//! # Messages pallet
//! A pallet for storing messages.
//!
//! This pallet contains functionality for storing, retrieving and eventually removing messages for
//! registered schemas on chain.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//! The Messages Pallet provides functions for:
//!
//! - Adding a message for given schema.
//! - Retrieving messages for a given schema.
//!
//! ### Terminology
//!
//! - **Message:** A message that matches a registered `Schema` (on-chain or off-chain).
//! - **Payload:** The user data in a `Message` that matches a `Schema`.
//! - **MSA Id:** The 64 bit unsigned integer associated with an `Message Source Account`.
//! - **MSA:** Message Source Account. A registered identifier with the MSA pallet.
//! - **Schema:** A registered data structure and the settings around it.
//! - **Schema Id:** A U16 bit identifier for a schema stored on-chain.
//!
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

use codec::{Decode, Encode};
use common_primitives::{
	messages::*,
	msa::{AccountProvider, Delegator, MessageSourceId, Provider},
	schema::*,
};
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

		/// A type that will supply account related information.
		type AccountProvider: AccountProvider<AccountId = Self::AccountId>;

		/// A type that will supply schema related information.
		type SchemaProvider: SchemaProvider<SchemaId>;

		/// The maximum number of messages in a block.
		#[pallet::constant]
		type MaxMessagesPerBlock: Get<u32>;

		/// The maximum size of a message payload bytes.
		#[pallet::constant]
		type MaxMessagePayloadSizeBytes: Get<u32> + Clone;
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
		/// Invalid Pagination Request
		InvalidPaginationRequest,
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
			// TODO: add retention policy execution
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Adds a message for a resource hosted on IPFS. The IPFS payload should
		/// contain both a
		/// [CID](https://docs.ipfs.io/concepts/content-addressing/#identifier-formats)
		/// as well as a 32-bit payload length.

		/// # Arguments
		/// * `origin` - A signed transaction origin from the provider
		/// * `on_behalf_of` - Optional. The msa id of delegate.
		/// * `schema_id` - Registered schema id for current message.
		/// * `cid` - The content address for an IPFS payload
		/// * `payload_length` - The size of the payload
		/// * Returns
		/// * [DispatchResultWithPostInfo](https://paritytech.github.io/substrate/master/frame_support/dispatch/type.DispatchResultWithPostInfo.html)
		#[pallet::weight(T::WeightInfo::add_ipfs_message(cid.len() as u32, 1_000))]
		pub fn add_ipfs_message(
			origin: OriginFor<T>,
			on_behalf_of: Option<MessageSourceId>,
			schema_id: SchemaId,
			cid: Vec<u8>,
			payload_length: u32,
		) -> DispatchResultWithPostInfo {
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

			let provider_msa_id = Self::find_msa_id(&provider_key, None)?;
			let msa_id = Self::find_msa_id(&provider_key, on_behalf_of)?;

			let message = Self::add_message(provider_msa_id, msa_id, bounded_payload, schema_id)?;

			Ok(Some(T::WeightInfo::add_ipfs_message(cid.len() as u32, message.index as u32)).into())
		}
		/// Add an on-chain message for a given schema-id.
		/// # Arguments
		/// * `origin` - A signed transaction origin from the provider
		/// * `on_behalf_of` - Optional. The msa id of delegate.
		/// * `schema_id` - Registered schema id for current message.
		/// * `payload` - Serialized payload data for a given schema.
		/// # Returns
		/// * [DispatchResultWithPostInfo](https://paritytech.github.io/substrate/master/frame_support/dispatch/type.DispatchResultWithPostInfo.html) The return type of a Dispatchable in frame.
		/// When returned explicitly from a dispatchable function it allows overriding the default PostDispatchInfo returned from a dispatch.
		#[pallet::weight(T::WeightInfo::add_onchain_message(payload.len() as u32, 1_000))]
		pub fn add_onchain_message(
			origin: OriginFor<T>,
			on_behalf_of: Option<MessageSourceId>,
			schema_id: SchemaId,
			payload: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			let provider_key = ensure_signed(origin)?;

			let bounded_payload: BoundedVec<u8, T::MaxMessagePayloadSizeBytes> =
				payload.try_into().map_err(|_| Error::<T>::ExceedsMaxMessagePayloadSizeBytes)?;

			let schema = T::SchemaProvider::get_schema_by_id(schema_id);
			ensure!(schema.is_some(), Error::<T>::InvalidSchemaId);
			ensure!(
				schema.unwrap().payload_location == PayloadLocation::OnChain,
				Error::<T>::InvalidPayloadLocation
			);

			let provider_msa_id = Self::find_msa_id(&provider_key, None)?;
			let msa_id = Self::find_msa_id(&provider_key, on_behalf_of)?;

			let message = Self::add_message(provider_msa_id, msa_id, bounded_payload, schema_id)?;

			Ok(Some(T::WeightInfo::add_onchain_message(
				message.payload.len() as u32,
				message.index as u32,
			))
			.into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Stores a message for a given schema-id.
	/// # Arguments
	/// * `provider_msa_id` - The MSA id of the provider that submitted the transaction.
	/// * `message_source_id` - The
	/// * `payload` - Serialized payload data for a given schema.
	/// * `schema_id` - Registered schema id for current message.
	/// # Returns
	/// * Result<Message<T::MaxMessagePayloadSizeBytes> - Returns the message stored.
	pub fn add_message(
		provider_msa_id: MessageSourceId,
		msa_id: MessageSourceId,
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

	/// Resolve an MSA from an account key(key) and an optional MSA Id (on_behalf_of).
	/// If a delegation relationship exists between an MSA held by the account key and the optional
	/// MSA, the MSA Id associated with the optional MSA is returned. Otherwise an MSA Id associated
	/// with the account key is returned, if one exists.
	///
	/// # Arguments
	/// * `key` - An MSA key for lookup.
	/// * `on_behalf_of` - Optional. The msa id of delegate.
	/// # Returns
	/// * Result<MessageSourceId, DispatchError> - Returns an MSA Id for storing a message.
	pub fn find_msa_id(
		key: &T::AccountId,
		on_behalf_of: Option<MessageSourceId>,
	) -> Result<MessageSourceId, DispatchError> {
		let sender_msa_id = T::AccountProvider::ensure_valid_msa_key(key)
			.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;

		let message_source_id = match on_behalf_of {
			Some(delegator) =>
				Self::ensure_valid_delegation(Provider(sender_msa_id), Delegator(delegator))?.1,
			None => sender_msa_id,
		};

		Ok(message_source_id)
	}

	/// Check for delegation between Delegator and Provider
	/// # Arguments
	/// * `provider` - An MSA of the provider.
	/// * `delegator` - An MSA of the delegator.
	/// # Returns
	/// * Result<MessageSourceId, MessageSourceId> - Returns MessageSourceId mapping of provider and delegator.
	pub fn ensure_valid_delegation(
		provider: Provider,
		delegator: Delegator,
	) -> Result<(MessageSourceId, MessageSourceId), DispatchError> {
		T::AccountProvider::ensure_valid_delegation(provider, delegator)
			.map_err(|_| Error::<T>::UnAuthorizedDelegate)?;

		Ok((provider.into(), delegator.into()))
	}

	/// Gets a messages for a given schema-id and block-number.
	/// # Arguments
	/// * `schema_id` - Registered schema id for current message.
	/// * `pagination` - [`BlockPaginationRequest`]. Request payload to retrieve paginated messages for a given block-range.
	/// # Returns
	/// * `Result<BlockPaginationResponse<T::BlockNumber, MessageResponse<T::BlockNumber>>, DispatchError>`
	///
	/// Result is a paginator response of type [`BlockPaginationResponse`].
	pub fn get_messages_by_schema(
		schema_id: SchemaId,
		pagination: BlockPaginationRequest<T::BlockNumber>,
	) -> Result<
		BlockPaginationResponse<T::BlockNumber, MessageResponse<T::BlockNumber>>,
		DispatchError,
	> {
		ensure!(pagination.validate(), Error::<T>::InvalidPaginationRequest);

		let schema = T::SchemaProvider::get_schema_by_id(schema_id);
		ensure!(schema.is_some(), Error::<T>::InvalidSchemaId);

		let payload_location = schema.unwrap().payload_location;
		let mut response = BlockPaginationResponse::new();
		let from: u32 = pagination
			.from_block
			.try_into()
			.map_err(|_| Error::<T>::TypeConversionOverflow)?;
		let to: u32 =
			pagination.to_block.try_into().map_err(|_| Error::<T>::TypeConversionOverflow)?;
		let mut from_index = pagination.from_index;

		'loops: for bid in from..to {
			let block_number: T::BlockNumber = bid.into();
			let list = <Messages<T>>::get(block_number, schema_id).into_inner();

			let list_size: u32 =
				list.len().try_into().map_err(|_| Error::<T>::TypeConversionOverflow)?;
			for i in from_index..list_size {
				let m = list[i as usize].clone();

				let (payload, payload_length): OffchainPayloadType = match payload_location {
					PayloadLocation::OnChain =>
						(m.payload.clone().into_inner(), m.payload.len().try_into().unwrap()),
					PayloadLocation::IPFS =>
						OffchainPayloadType::decode(&mut &m.payload[..]).unwrap(),
				};

				response.content.push(m.map_to_response(block_number, payload, payload_length));

				if Self::check_end_condition_and_set_next_pagination(
					block_number,
					i,
					list_size,
					&pagination,
					&mut response,
				) {
					break 'loops
				}
			}

			// next block starts from 0
			from_index = 0;
		}

		Ok(response)
	}

	/// Checks the end condition for paginated query and set the `PaginationResponse`
	///
	/// Returns `true` if page is filled
	fn check_end_condition_and_set_next_pagination(
		block_number: T::BlockNumber,
		current_index: u32,
		list_size: u32,
		request: &BlockPaginationRequest<T::BlockNumber>,
		result: &mut BlockPaginationResponse<T::BlockNumber, MessageResponse<T::BlockNumber>>,
	) -> bool {
		if result.content.len() as u32 == request.page_size {
			let mut next_block = block_number;
			let mut next_index = current_index + 1;

			// checking if it's end of current list
			if next_index == list_size {
				next_block = block_number + T::BlockNumber::one();
				next_index = 0;
			}

			if next_block < request.to_block {
				result.has_next = true;
				result.next_block = Some(next_block);
				result.next_index = Some(next_index);
			}
			return true
		}

		false
	}

	/// Moves messages from temporary storage `BlockMessages` into final storage `Messages`
	/// and calculates execution weight
	///
	/// * `block_number`: Target Block Number
	///
	/// Returns execution weights
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
			let bounded_vec: BoundedVec<_, _> = messages.try_into().unwrap();
			Messages::<T>::insert(&block_number, schema_id, &bounded_vec);
			Self::deposit_event(Event::MessagesStored { schema_id, block_number, count });
			schema_count += 1;
		}

		BlockMessages::<T>::set(BoundedVec::default());
		T::WeightInfo::on_initialize(message_count, schema_count)
	}
}
