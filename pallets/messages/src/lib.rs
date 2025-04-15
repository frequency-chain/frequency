//! Stores messages for `IPFS` and `OnChain` Schema payload locations
//!
//! ## Quick Links
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Runtime API: `MessagesRuntimeApi`](../pallet_messages_runtime_api/trait.MessagesRuntimeApi.html)
//! - [Custom RPC API: `MessagesApiServer`](../pallet_messages_rpc/trait.MessagesApiServer.html)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
#![doc = include_str!("../README.md")]
// Substrate macros are tripping the clippy::expect_used lint.
#![allow(clippy::expect_used)]
// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod tests;

pub mod weights;

mod types;

use core::{convert::TryInto, fmt::Debug};
use frame_support::{ensure, pallet_prelude::Weight, traits::Get, BoundedVec};
use sp_runtime::DispatchError;

extern crate alloc;
use alloc::vec::Vec;
use common_primitives::{
	messages::*,
	msa::{
		DelegatorId, MessageSourceId, MsaLookup, MsaValidator, ProviderId, SchemaGrantValidator,
	},
	schema::*,
};
use frame_support::dispatch::DispatchResult;
use parity_scale_codec::Encode;

#[cfg(feature = "runtime-benchmarks")]
use common_primitives::benchmarks::{MsaBenchmarkHelper, SchemaBenchmarkHelper};

pub use pallet::*;
pub use types::*;
pub use weights::*;

use cid::Cid;
use frame_system::pallet_prelude::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;

	/// The current storage version.
	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// A type that will supply MSA related information
		type MsaInfoProvider: MsaLookup + MsaValidator<AccountId = Self::AccountId>;

		/// A type that will validate schema grants
		type SchemaGrantValidator: SchemaGrantValidator<BlockNumberFor<Self>>;

		/// A type that will supply schema related information.
		type SchemaProvider: SchemaProvider<SchemaId>;

		/// The maximum size of a message payload bytes.
		#[pallet::constant]
		type MessagesMaxPayloadSizeBytes: Get<u32> + Clone + Debug + MaxEncodedLen;

		#[cfg(feature = "runtime-benchmarks")]
		/// A set of helper functions for benchmarking.
		type MsaBenchmarkHelper: MsaBenchmarkHelper<Self::AccountId>;

		#[cfg(feature = "runtime-benchmarks")]
		/// A set of helper functions for benchmarking.
		type SchemaBenchmarkHelper: SchemaBenchmarkHelper;
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// A temporary storage for getting the index for messages
	/// At the start of the next block this storage is set to 0
	#[pallet::storage]
	#[pallet::whitelist_storage]
	pub(super) type BlockMessageIndex<T: Config> = StorageValue<_, MessageIndex, ValueQuery>;

	#[pallet::storage]
	pub(super) type MessagesV2<T: Config> = StorageNMap<
		_,
		(
			storage::Key<Twox64Concat, BlockNumberFor<T>>,
			storage::Key<Twox64Concat, SchemaId>,
			storage::Key<Twox64Concat, MessageIndex>,
		),
		Message<T::MessagesMaxPayloadSizeBytes>,
		OptionQuery,
	>;

	#[pallet::error]
	pub enum Error<T> {
		/// Deprecated: Too many messages are added to existing block
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

		/// Unsupported CID version
		UnsupportedCidVersion,

		/// Invalid CID
		InvalidCid,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Deprecated: please use [`Event::MessagesInBlock`]
		/// Messages are stored for a specified schema id and block number
		MessagesStored {
			/// The schema for these messages
			schema_id: SchemaId,
			/// The block number for these messages
			block_number: BlockNumberFor<T>,
		},
		/// Messages stored in the current block
		MessagesInBlock,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_current: BlockNumberFor<T>) -> Weight {
			<BlockMessageIndex<T>>::set(0u16);
			// allocates 1 read and 1 write for any access of `MessageIndex` in every block
			T::DbWeight::get().reads(1u64).saturating_add(T::DbWeight::get().writes(1u64))
			// TODO: add retention policy execution GitHub Issue: #126 and #25
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Adds a message for a resource hosted on IPFS. The input consists of
		/// both a Base32-encoded [CID](https://docs.ipfs.tech/concepts/content-addressing/#version-1-v1)
		/// as well as a 32-bit content length. The stored payload will contain the
		/// CID encoded as binary, as well as the 32-bit message content length.
		/// The actual message content will be on IPFS.
		///
		/// # Events
		/// * [`Event::MessagesInBlock`] - Messages Stored in the block
		///
		/// # Errors
		/// * [`Error::ExceedsMaxMessagePayloadSizeBytes`] - Payload is too large
		/// * [`Error::InvalidSchemaId`] - Schema not found
		/// * [`Error::InvalidPayloadLocation`] - The schema is not an IPFS payload location
		/// * [`Error::InvalidMessageSourceAccount`] - Origin must be from an MSA
		/// * [`Error::TypeConversionOverflow`] - Failed to add the message to storage as it is very full
		/// * [`Error::UnsupportedCidVersion`] - CID version is not supported (V0)
		/// * [`Error::InvalidCid`] - Unable to parse provided CID
		///
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::add_ipfs_message())]
		pub fn add_ipfs_message(
			origin: OriginFor<T>,
			#[pallet::compact] schema_id: SchemaId,
			cid: Vec<u8>,
			#[pallet::compact] payload_length: u32,
		) -> DispatchResult {
			let provider_key = ensure_signed(origin)?;
			let cid_binary = Self::validate_cid(&cid)?;
			let payload_tuple: OffchainPayloadType = (cid_binary, payload_length);
			let bounded_payload: BoundedVec<u8, T::MessagesMaxPayloadSizeBytes> = payload_tuple
				.encode()
				.try_into()
				.map_err(|_| Error::<T>::ExceedsMaxMessagePayloadSizeBytes)?;

			if let Some(schema) = T::SchemaProvider::get_schema_info_by_id(schema_id) {
				ensure!(
					schema.payload_location == PayloadLocation::IPFS,
					Error::<T>::InvalidPayloadLocation
				);

				let provider_msa_id = Self::find_msa_id(&provider_key)?;
				let current_block = frame_system::Pallet::<T>::block_number();
				if Self::add_message(
					provider_msa_id,
					None,
					bounded_payload,
					schema_id,
					current_block,
				)? {
					Self::deposit_event(Event::MessagesInBlock);
				}
				Ok(())
			} else {
				Err(Error::<T>::InvalidSchemaId.into())
			}
		}

		/// Add an on-chain message for a given schema id.
		///
		/// # Events
		/// * [`Event::MessagesInBlock`] - In the next block
		///
		/// # Errors
		/// * [`Error::ExceedsMaxMessagePayloadSizeBytes`] - Payload is too large
		/// * [`Error::InvalidSchemaId`] - Schema not found
		/// * [`Error::InvalidPayloadLocation`] - The schema is not an IPFS payload location
		/// * [`Error::InvalidMessageSourceAccount`] - Origin must be from an MSA
		/// * [`Error::UnAuthorizedDelegate`] - Trying to add a message without a proper delegation between the origin and the on_behalf_of MSA
		/// * [`Error::TypeConversionOverflow`] - Failed to add the message to storage as it is very full
		///
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::add_onchain_message(payload.len() as u32))]
		pub fn add_onchain_message(
			origin: OriginFor<T>,
			on_behalf_of: Option<MessageSourceId>,
			#[pallet::compact] schema_id: SchemaId,
			payload: Vec<u8>,
		) -> DispatchResult {
			let provider_key = ensure_signed(origin)?;

			let bounded_payload: BoundedVec<u8, T::MessagesMaxPayloadSizeBytes> =
				payload.try_into().map_err(|_| Error::<T>::ExceedsMaxMessagePayloadSizeBytes)?;

			if let Some(schema) = T::SchemaProvider::get_schema_info_by_id(schema_id) {
				ensure!(
					schema.payload_location == PayloadLocation::OnChain,
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

				if Self::add_message(
					provider_msa_id,
					Some(maybe_delegator.into()),
					bounded_payload,
					schema_id,
					current_block,
				)? {
					Self::deposit_event(Event::MessagesInBlock);
				}

				Ok(())
			} else {
				Err(Error::<T>::InvalidSchemaId.into())
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Stores a message for a given schema id.
	/// returns true if it needs to emit an event
	/// # Errors
	/// * [`Error::TypeConversionOverflow`]
	///
	pub fn add_message(
		provider_msa_id: MessageSourceId,
		msa_id: Option<MessageSourceId>,
		payload: BoundedVec<u8, T::MessagesMaxPayloadSizeBytes>,
		schema_id: SchemaId,
		current_block: BlockNumberFor<T>,
	) -> Result<bool, DispatchError> {
		let index = BlockMessageIndex::<T>::get();
		let first = index == 0;
		let msg = Message {
			payload, // size is checked on top of extrinsic
			provider_msa_id,
			msa_id,
		};

		<MessagesV2<T>>::insert((current_block, schema_id, index), msg);
		BlockMessageIndex::<T>::set(index.saturating_add(1));
		Ok(first)
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
		block_number: BlockNumberFor<T>,
	) -> Vec<MessageResponse> {
		let block_number_value: u32 = block_number.try_into().unwrap_or_default();

		match schema_payload_location {
			PayloadLocation::Itemized | PayloadLocation::Paginated => return Vec::new(),
			_ => {
				let mut messages: Vec<_> = <MessagesV2<T>>::iter_prefix((block_number, schema_id))
					.map(|(index, msg)| {
						msg.map_to_response(block_number_value, schema_payload_location, index)
					})
					.collect();
				messages.sort_by(|a, b| a.index.cmp(&b.index));
				return messages
			},
		}
	}

	/// Validates a CID to conform to IPFS CIDv1 (or higher) formatting (does not validate decoded CID fields)
	///
	/// # Errors
	/// * [`Error::UnsupportedCidVersion`] - CID version is not supported (V0)
	/// * [`Error::InvalidCid`] - Unable to parse provided CID
	///
	pub fn validate_cid(in_cid: &Vec<u8>) -> Result<Vec<u8>, DispatchError> {
		// Decode SCALE encoded CID into string slice
		let cid_str: &str =
			core::str::from_utf8(&in_cid[..]).map_err(|_| Error::<T>::InvalidCid)?;
		ensure!(cid_str.len() > 2, Error::<T>::InvalidCid);
		// starts_with handles Unicode multibyte characters safely
		ensure!(!cid_str.starts_with("Qm"), Error::<T>::UnsupportedCidVersion);

		// Assume it's a multibase-encoded string. Decode it to a byte array so we can parse the CID.
		let cid_b = multibase::decode(cid_str).map_err(|_| Error::<T>::InvalidCid)?.1;
		ensure!(Cid::read_bytes(&cid_b[..]).is_ok(), Error::<T>::InvalidCid);

		Ok(cid_b)
	}
}
