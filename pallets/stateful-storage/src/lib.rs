//! # Stateful Storage Pallet
//! The Stateful Storage pallet provides functionality for reading and writing stateful data
//! representing stateful data for which we are only ever interested in the latest state.
//!
//! ## Overview
//! For state transitions for which we only care about the latest state, Stateful Storage provides a way to store and retrieve such data
//! outside of the existing Announcement mechanism, which would require the latest state to be tracked using some kind of 3rd-party indexer.
//!
//! This pallet supports two models for storing stateful data:
//! 1. **Itemized:** Data is stored in a single **page** (max size: `MaxItemizedPageSizeBytes`) containing multiple items (max item size `MaxItemizedBlobSizeBytes`) of the associated schema.
//! Useful for schemas with a relative small item size and higher potential item count.
//! 2. **Paginated:** Data is stored in multiple **pages** of size `MaxPaginatedPageSizeBytes`, each containing a single item of the associated schema.
//! Page IDs range from 0 .. `MaxPaginatedPageId` (implying there may be at most `MaxPaginatedPageId` + 1 pages per MSA+Schema at any given time, though
//! there may be holes in that range). Useful for schemas with a larger item size and smaller potential item count.
//!
//! ## Features
//! * Provide for storage of stateful data with flexible schemas on-chain.
//! * Data stored for one MSA does not have impact read/write access times or storage costs for any data stored for another MSA.
//! * High write throughput.
//! * High read throughput.
//! * Data race condition protection
//!
//! The Stateful Storage pallet provides functions for:
//!
//! - Appending items in an **itemized** model
//! - Removing items in an **itemized** model
//! - Upserting items in a **paginated** model
//! - Removing pages in a **paginated**  model
//!
//! ## Terminology
//! * **Item:** Data payload mapping to a defined schema
//! * **Page:** Block of on-chain data of a fixed size, containing one (Paginated model) or more (Itemized model) **items**.
//!

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
// #![deny(
// 	rustdoc::broken_intra_doc_links,
// 	rustdoc::missing_crate_level_docs,
// 	rustdoc::invalid_codeblock_attributes,
// 	missing_docs
// )]

#[cfg(any(feature = "runtime-benchmarks", test))]
mod test_common;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(feature = "runtime-benchmarks")]
use common_primitives::benchmarks::{MsaBenchmarkHelper, SchemaBenchmarkHelper};
use sp_std::prelude::*;

mod stateful_child_tree;
pub mod types;

pub mod weights;

use crate::{stateful_child_tree::StatefulChildTree, types::*};
// Weird compiler issue--warns about unused PageId import when building, but if missing here, `cargo test` fails due to missing import.
#[allow(unused_imports)]
use common_primitives::stateful_storage::PageId;
use common_primitives::{
	msa::{DelegatorId, MessageSourceId, MsaValidator, ProviderId, SchemaGrantValidator},
	node::Verify,
	schema::{PayloadLocation, SchemaId, SchemaProvider, SchemaSetting},
	stateful_storage::{
		ItemizedStoragePageResponse, ItemizedStorageResponse, PageHash, PaginatedStorageResponse,
	},
	utils::wrap_binary_data,
};
use frame_support::{dispatch::DispatchResult, ensure, traits::Get};
pub use pallet::*;
use sp_core::bounded::BoundedVec;
use sp_runtime::{traits::Convert, DispatchError, MultiSignature};
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use common_primitives::{
		msa::{MessageSourceId, MsaLookup, MsaValidator, SchemaGrantValidator},
		schema::{SchemaId, SchemaProvider},
		stateful_storage::{PageHash, PageId},
	};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_core::crypto::AccountId32;
	use sp_runtime::{traits::Convert, MultiSignature};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// A type that will supply MSA related information
		type MsaInfoProvider: MsaLookup + MsaValidator<AccountId = Self::AccountId>;

		/// A type that will validate schema grants
		type SchemaGrantValidator: SchemaGrantValidator<Self::BlockNumber>;

		/// A type that will supply schema related information.
		type SchemaProvider: SchemaProvider<SchemaId>;

		/// The maximum size of a page (in bytes) for an Itemized storage model
		#[pallet::constant]
		type MaxItemizedPageSizeBytes: Get<u32> + Default;

		/// The maximum size of a page (in bytes) for a Paginated storage model
		#[pallet::constant]
		type MaxPaginatedPageSizeBytes: Get<u32> + Default;

		/// The maximum size of a single item in an itemized storage model (in bytes)
		#[pallet::constant]
		type MaxItemizedBlobSizeBytes: Get<u32>;

		/// The maximum number of pages in a Paginated storage model
		#[pallet::constant]
		type MaxPaginatedPageId: Get<u32>;

		/// The maximum number of actions in itemized actions
		#[pallet::constant]
		type MaxItemizedActionsCount: Get<u32>;

		#[cfg(feature = "runtime-benchmarks")]
		/// A set of helper functions for benchmarking.
		type MsaBenchmarkHelper: MsaBenchmarkHelper<Self::AccountId>;

		#[cfg(feature = "runtime-benchmarks")]
		/// A set of helper functions for benchmarking.
		type SchemaBenchmarkHelper: SchemaBenchmarkHelper;

		/// Hasher to use for MultipartKey
		type KeyHasher: stateful_child_tree::MultipartKeyStorageHasher;

		/// AccountId truncated to 32 bytes
		type ConvertIntoAccountId32: Convert<Self::AccountId, AccountId32>;

		/// The number of blocks that we allow for a signed payload to be valid. This is mainly used
		/// to make sure a signed payload would not be replayable.
		#[pallet::constant]
		type MortalityWindowSize: Get<u32>;
	}

	// Simple declaration of the `Pallet` type. It is placeholder we use to implement traits and
	// method.
	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::error]
	pub enum Error<T> {
		/// Item payload exceeds max item blob size
		ItemExceedsMaxBlobSizeBytes,

		/// Page would exceed the highest allowable PageId
		PageIdExceedsMaxAllowed,

		/// Page size exceeds max allowable page size
		PageExceedsMaxPageSizeBytes,

		/// Invalid SchemaId or Schema not found
		InvalidSchemaId,

		/// Unsupported schema
		SchemaNotSupported,

		/// Schema is not valid for storage model
		SchemaPayloadLocationMismatch,

		/// Invalid Message Source Account
		InvalidMessageSourceAccount,

		/// UnAuthorizedDelegate
		UnAuthorizedDelegate,

		/// Corrupted State
		CorruptedState,

		/// Invalid item action
		InvalidItemAction,

		/// Target page hash does not match current page hash
		StalePageState,

		/// Invalid Signature for payload
		InvalidSignature,

		/// The submitted proof has expired; the current block is less the expiration block
		ProofHasExpired,

		/// The submitted proof expiration block is too far in the future
		ProofNotYetValid,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An event for when an itemized storage is updated
		ItemizedPageUpdated {
			/// message source id of storage owner
			msa_id: MessageSourceId,
			/// schema related to the storage
			schema_id: SchemaId,
			/// previous content hash before update
			prev_content_hash: PageHash,
			/// current content hash after update
			curr_content_hash: PageHash,
		},

		/// An event for when an itemized storage is deleted
		ItemizedPageDeleted {
			/// message source id of storage owner
			msa_id: MessageSourceId,
			/// schema related to the storage
			schema_id: SchemaId,
			/// previous content hash before removal
			prev_content_hash: PageHash,
		},

		/// An event for when an paginated storage is updated
		PaginatedPageUpdated {
			/// message source id of storage owner
			msa_id: MessageSourceId,
			/// schema related to the storage
			schema_id: SchemaId,
			/// id of updated page
			page_id: PageId,
			/// previous content hash before update
			prev_content_hash: PageHash,
			/// current content hash after update
			curr_content_hash: PageHash,
		},

		/// An event for when an paginated storage is deleted
		PaginatedPageDeleted {
			/// message source id of storage owner
			msa_id: MessageSourceId,
			/// schema related to the storage
			schema_id: SchemaId,
			/// id of updated page
			page_id: PageId,
			/// previous content hash before removal
			prev_content_hash: PageHash,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Applies the Add or Delete Actions on the requested Itemized page
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::apply_item_actions( actions.len() as u32 ,
			actions.iter().fold(0, |acc, a| acc + match a {
				ItemAction::Add { data } => data.len() as u32,
				_ => 0,
			}),
		    0
		))]
		pub fn apply_item_actions(
			origin: OriginFor<T>,
			#[pallet::compact] state_owner_msa_id: MessageSourceId,
			#[pallet::compact] schema_id: SchemaId,
			#[pallet::compact] target_hash: PageHash,
			actions: BoundedVec<ItemAction, T::MaxItemizedActionsCount>,
		) -> DispatchResult {
			let provider_key = ensure_signed(origin)?;
			let is_pruning = actions.iter().any(|a| matches!(a, ItemAction::Delete { .. }));
			Self::check_actions(&actions)?;
			Self::check_schema(schema_id, PayloadLocation::Itemized, false, is_pruning)?;
			Self::check_grants(provider_key, state_owner_msa_id, schema_id)?;
			Self::modify_itemized(state_owner_msa_id, schema_id, target_hash, actions)?;
			Ok(())
		}

		/// Creates or updates an Paginated storage with new payload
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::upsert_page(payload.len() as u32))]
		pub fn upsert_page(
			origin: OriginFor<T>,
			#[pallet::compact] state_owner_msa_id: MessageSourceId,
			#[pallet::compact] schema_id: SchemaId,
			#[pallet::compact] page_id: PageId,
			#[pallet::compact] target_hash: PageHash,
			payload: BoundedVec<u8, <T>::MaxPaginatedPageSizeBytes>,
		) -> DispatchResult {
			let provider_key = ensure_signed(origin)?;
			ensure!(
				page_id as u32 <= T::MaxPaginatedPageId::get(),
				Error::<T>::PageIdExceedsMaxAllowed
			);
			Self::check_schema(schema_id, PayloadLocation::Paginated, false, false)?;
			Self::check_grants(provider_key, state_owner_msa_id, schema_id)?;
			Self::modify_paginated(
				state_owner_msa_id,
				schema_id,
				page_id,
				target_hash,
				PaginatedPage::<T>::from(payload),
			)?;
			Ok(())
		}

		/// Deletes a Paginated storage
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::delete_page())]
		pub fn delete_page(
			origin: OriginFor<T>,
			#[pallet::compact] state_owner_msa_id: MessageSourceId,
			#[pallet::compact] schema_id: SchemaId,
			#[pallet::compact] page_id: PageId,
			#[pallet::compact] target_hash: PageHash,
		) -> DispatchResult {
			let provider_key = ensure_signed(origin)?;
			ensure!(
				page_id as u32 <= T::MaxPaginatedPageId::get(),
				Error::<T>::PageIdExceedsMaxAllowed
			);
			Self::check_schema(schema_id, PayloadLocation::Paginated, false, true)?;
			Self::check_grants(provider_key, state_owner_msa_id, schema_id)?;
			Self::delete_paginated(state_owner_msa_id, schema_id, page_id, target_hash)?;
			Ok(())
		}

		/// Applies the Add or Delete Actions on the requested Itemized page that requires signature
		/// since the signature of delegator is checked there is no need for delegation validation
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::apply_item_actions_with_signature( payload.actions.len() as u32 ,
			payload.actions.iter().fold(0, |acc, a| acc + match a {
			ItemAction::Add { data } => data.len() as u32,
			_ => 0,
			})
		))]
		pub fn apply_item_actions_with_signature(
			origin: OriginFor<T>,
			delegator_key: T::AccountId,
			proof: MultiSignature,
			payload: ItemizedSignaturePayload<T>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			let is_pruning = payload.actions.iter().any(|a| matches!(a, ItemAction::Delete { .. }));
			Self::check_actions(&payload.actions)?;
			Self::check_payload_expiration(
				frame_system::Pallet::<T>::block_number(),
				payload.expiration,
			)?;
			Self::check_signature(&proof, &delegator_key.clone(), payload.encode())?;
			Self::check_msa(delegator_key, payload.msa_id)?;
			Self::check_schema(payload.schema_id, PayloadLocation::Itemized, true, is_pruning)?;
			Self::modify_itemized(
				payload.msa_id,
				payload.schema_id,
				payload.target_hash,
				payload.actions,
			)?;
			Ok(())
		}

		/// Creates or updates an Paginated storage with new payload that requires signature
		/// since the signature of delegator is checked there is no need for delegation validation
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::upsert_page_with_signature(payload.payload.len() as u32))]
		pub fn upsert_page_with_signature(
			origin: OriginFor<T>,
			delegator_key: T::AccountId,
			proof: MultiSignature,
			payload: PaginatedUpsertSignaturePayload<T>,
		) -> DispatchResult {
			ensure_signed(origin)?;
			ensure!(
				payload.page_id as u32 <= T::MaxPaginatedPageId::get(),
				Error::<T>::PageIdExceedsMaxAllowed
			);
			Self::check_payload_expiration(
				frame_system::Pallet::<T>::block_number(),
				payload.expiration,
			)?;
			Self::check_signature(&proof, &delegator_key.clone(), payload.encode())?;
			Self::check_msa(delegator_key, payload.msa_id)?;
			Self::check_schema(payload.schema_id, PayloadLocation::Paginated, true, false)?;
			Self::modify_paginated(
				payload.msa_id,
				payload.schema_id,
				payload.page_id,
				payload.target_hash,
				PaginatedPage::<T>::from(payload.payload),
			)?;
			Ok(())
		}

		/// Deletes a Paginated storage that requires signature
		/// since the signature of delegator is checked there is no need for delegation validation
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::delete_page_with_signature())]
		pub fn delete_page_with_signature(
			origin: OriginFor<T>,
			delegator_key: T::AccountId,
			proof: MultiSignature,
			payload: PaginatedDeleteSignaturePayload<T>,
		) -> DispatchResult {
			ensure_signed(origin)?;
			ensure!(
				payload.page_id as u32 <= T::MaxPaginatedPageId::get(),
				Error::<T>::PageIdExceedsMaxAllowed
			);
			Self::check_payload_expiration(
				frame_system::Pallet::<T>::block_number(),
				payload.expiration,
			)?;
			Self::check_signature(&proof, &delegator_key.clone(), payload.encode())?;
			Self::check_msa(delegator_key, payload.msa_id)?;
			Self::check_schema(payload.schema_id, PayloadLocation::Paginated, true, true)?;
			Self::delete_paginated(
				payload.msa_id,
				payload.schema_id,
				payload.page_id,
				payload.target_hash,
			)?;
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// This function returns all the paginated storages associated with `msa_id` and `schema_id`
	///
	/// [Warning] since this function iterates over all the potential keys it should never called
	/// from runtime.
	pub fn get_paginated_storages(
		msa_id: MessageSourceId,
		schema_id: SchemaId,
	) -> Result<Vec<PaginatedStorageResponse>, DispatchError> {
		Self::check_schema(schema_id, PayloadLocation::Paginated, false, false)?;
		let prefix: PaginatedPrefixKey = (schema_id,);
		Ok(StatefulChildTree::<T::KeyHasher>::prefix_iterator::<
			PaginatedPage<T>,
			PaginatedKey,
			PaginatedPrefixKey,
		>(&msa_id, PALLET_STORAGE_PREFIX, PAGINATED_STORAGE_PREFIX, &prefix)
		.map(|(k, v)| {
			let content_hash = v.get_hash();
			PaginatedStorageResponse::new(k.1, msa_id, schema_id, v.data.into_inner(), content_hash)
		})
		.collect())
	}

	/// This function returns all the itemized storages associated with `msa_id` and `schema_id`
	pub fn get_itemized_storages(
		msa_id: MessageSourceId,
		schema_id: SchemaId,
	) -> Result<ItemizedStoragePageResponse, DispatchError> {
		Self::check_schema(schema_id, PayloadLocation::Itemized, false, false)?;
		let key: ItemizedKey = (schema_id,);
		let page = StatefulChildTree::<T::KeyHasher>::try_read::<ItemizedKey, ItemizedPage<T>>(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&key,
		)
		.map_err(|_| Error::<T>::CorruptedState)?
		.unwrap_or_default();
		let items: Vec<ItemizedStorageResponse> = page
			.parse_as_itemized(false)
			.map_err(|_| Error::<T>::CorruptedState)?
			.items
			.iter()
			.map(|(key, v)| ItemizedStorageResponse::new(*key, v.to_vec()))
			.collect();
		Ok(ItemizedStoragePageResponse::new(msa_id, schema_id, page.get_hash(), items))
	}

	/// This function checks to ensure `payload_expire_block` is in a valid range
	///
	/// # Errors
	/// * [`Error::ProofHasExpired`]
	/// * [`Error::ProofNotYetValid`]
	///
	pub fn check_payload_expiration(
		current_block: T::BlockNumber,
		payload_expire_block: T::BlockNumber,
	) -> Result<(), DispatchError> {
		ensure!(payload_expire_block > current_block, Error::<T>::ProofHasExpired);
		let max_supported_signature_block = Self::mortality_block_limit(current_block);
		ensure!(payload_expire_block < max_supported_signature_block, Error::<T>::ProofNotYetValid);
		Ok(())
	}

	/// Verify the `signature` was signed by `signer` on `payload` by a wallet
	/// Note the `wrap_binary_data` follows the Polkadot wallet pattern of wrapping with `<Byte>` tags.
	///
	/// # Errors
	/// * [`Error::InvalidSignature`]
	///
	pub fn check_signature(
		signature: &MultiSignature,
		signer: &T::AccountId,
		payload: Vec<u8>,
	) -> DispatchResult {
		let key = T::ConvertIntoAccountId32::convert((*signer).clone());
		let wrapped_payload = wrap_binary_data(payload);

		ensure!(signature.verify(&wrapped_payload[..], &key), Error::<T>::InvalidSignature);

		Ok(())
	}

	/// The furthest in the future a mortality_block value is allowed
	/// to be for current_block
	/// This is calculated to be past the risk of a replay attack
	fn mortality_block_limit(current_block: T::BlockNumber) -> T::BlockNumber {
		current_block + T::BlockNumber::from(T::MortalityWindowSize::get())
	}

	fn check_schema(
		schema_id: SchemaId,
		expected_payload_location: PayloadLocation,
		is_payload_signed: bool,
		is_deleting: bool,
	) -> DispatchResult {
		let schema =
			T::SchemaProvider::get_schema_by_id(schema_id).ok_or(Error::<T>::InvalidSchemaId)?;

		// Ensure that the schema's payload location matches the expected location.
		ensure!(
			schema.payload_location == expected_payload_location,
			Error::<T>::SchemaPayloadLocationMismatch
		);

		// Ensure that the schema allows signed payloads.
		// If so, calling extrinsic must be of signature type.
		if schema.settings.contains(&SchemaSetting::SignatureRequired) {
			ensure!(is_payload_signed, Error::<T>::SchemaNotSupported);
		}

		// Ensure that the schema does not allow deletion for AppendOnly SchemaSetting.
		if schema.settings.contains(&SchemaSetting::AppendOnly) {
			ensure!(!is_deleting, Error::<T>::SchemaNotSupported);
		}

		Ok(())
	}

	fn check_grants(
		provider_key: T::AccountId,
		state_owner_msa_id: MessageSourceId,
		schema_id: SchemaId,
	) -> Result<MessageSourceId, DispatchError> {
		let provider_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(&provider_key)
			.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;

		// if provider and owner are the same no delegation is needed
		if provider_msa_id != state_owner_msa_id {
			let current_block = frame_system::Pallet::<T>::block_number();
			T::SchemaGrantValidator::ensure_valid_schema_grant(
				ProviderId(provider_msa_id),
				DelegatorId(state_owner_msa_id),
				schema_id,
				current_block,
			)
			.map_err(|_| Error::<T>::UnAuthorizedDelegate)?;
		}

		Ok(provider_msa_id)
	}

	/// Verifies if the key has an Msa and if it matches with expected one
	///
	/// # Errors
	/// * [`Error::InvalidMessageSourceAccount`]
	///
	fn check_msa(key: T::AccountId, expected_msa_id: MessageSourceId) -> DispatchResult {
		let state_owner_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(&key)
			.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;
		ensure!(state_owner_msa_id == expected_msa_id, Error::<T>::InvalidMessageSourceAccount);
		Ok(())
	}

	/// Verifies the actions size
	///
	/// # Errors
	/// * [`Error::ItemExceedsMaxBlobSizeBytes`]
	///
	fn check_actions(
		actions: &BoundedVec<ItemAction, T::MaxItemizedActionsCount>,
	) -> DispatchResult {
		ensure!(
			actions.iter().all(|a| match a {
				ItemAction::Add { data } =>
					data.len() <= T::MaxItemizedBlobSizeBytes::get() as usize,
				_ => true,
			}),
			Error::<T>::ItemExceedsMaxBlobSizeBytes
		);
		Ok(())
	}

	/// Modifies an itemized storage by applying provided actions and deposit events
	fn modify_itemized(
		state_owner_msa_id: MessageSourceId,
		schema_id: SchemaId,
		target_hash: PageHash,
		actions: BoundedVec<ItemAction, T::MaxItemizedActionsCount>,
	) -> DispatchResult {
		let key: ItemizedKey = (schema_id,);
		let existing_page = StatefulChildTree::<T::KeyHasher>::try_read::<_, ItemizedPage<T>>(
			&state_owner_msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&key,
		)
		.map_err(|_| Error::<T>::CorruptedState)?
		.unwrap_or_default();

		let prev_content_hash = existing_page.get_hash();
		ensure!(target_hash == prev_content_hash, Error::<T>::StalePageState);

		let updated_page = existing_page.apply_item_actions(&actions[..]).map_err(|e| match e {
			PageError::ErrorParsing(err) => {
				log::warn!(
					"failed parsing Itemized msa={:?} schema_id={:?} {:?}",
					state_owner_msa_id,
					schema_id,
					err
				);
				Error::<T>::CorruptedState
			},
			_ => Error::<T>::InvalidItemAction,
		})?;

		match updated_page.is_empty() {
			true => {
				StatefulChildTree::<T::KeyHasher>::kill(
					&state_owner_msa_id,
					PALLET_STORAGE_PREFIX,
					ITEMIZED_STORAGE_PREFIX,
					&key,
				);
				Self::deposit_event(Event::ItemizedPageDeleted {
					msa_id: state_owner_msa_id,
					schema_id,
					prev_content_hash,
				});
			},
			false => {
				StatefulChildTree::<T::KeyHasher>::write(
					&state_owner_msa_id,
					PALLET_STORAGE_PREFIX,
					ITEMIZED_STORAGE_PREFIX,
					&key,
					&updated_page,
				);
				Self::deposit_event(Event::ItemizedPageUpdated {
					msa_id: state_owner_msa_id,
					schema_id,
					curr_content_hash: updated_page.get_hash(),
					prev_content_hash,
				});
			},
		};
		Ok(())
	}

	/// Modifies an paginated storage by provided new page and deposit events
	fn modify_paginated(
		state_owner_msa_id: MessageSourceId,
		schema_id: SchemaId,
		page_id: PageId,
		target_hash: PageHash,
		new_page: PaginatedPage<T>,
	) -> DispatchResult {
		let keys: PaginatedKey = (schema_id, page_id);
		let existing_page: PaginatedPage<T> = StatefulChildTree::<T::KeyHasher>::try_read(
			&state_owner_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
		)
		.map_err(|_| Error::<T>::CorruptedState)?
		.unwrap_or_default();

		let prev_content_hash: PageHash = existing_page.get_hash();
		ensure!(target_hash == prev_content_hash, Error::<T>::StalePageState);

		StatefulChildTree::<T::KeyHasher>::write(
			&state_owner_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
			&new_page,
		);
		Self::deposit_event(Event::PaginatedPageUpdated {
			msa_id: state_owner_msa_id,
			schema_id,
			page_id,
			curr_content_hash: new_page.get_hash(),
			prev_content_hash,
		});
		Ok(())
	}

	/// Deletes an paginated storage and deposit events
	fn delete_paginated(
		state_owner_msa_id: MessageSourceId,
		schema_id: SchemaId,
		page_id: PageId,
		target_hash: PageHash,
	) -> DispatchResult {
		let keys: PaginatedKey = (schema_id, page_id);
		if let Some(existing_page) =
			StatefulChildTree::<T::KeyHasher>::try_read::<_, PaginatedPage<T>>(
				&state_owner_msa_id,
				PALLET_STORAGE_PREFIX,
				PAGINATED_STORAGE_PREFIX,
				&keys,
			)
			.map_err(|_| Error::<T>::CorruptedState)?
		{
			let prev_content_hash: PageHash = existing_page.get_hash();
			ensure!(target_hash == prev_content_hash, Error::<T>::StalePageState);
			StatefulChildTree::<T::KeyHasher>::kill(
				&state_owner_msa_id,
				PALLET_STORAGE_PREFIX,
				PAGINATED_STORAGE_PREFIX,
				&keys,
			);
			Self::deposit_event(Event::PaginatedPageDeleted {
				msa_id: state_owner_msa_id,
				schema_id,
				page_id,
				prev_content_hash,
			});
		}

		Ok(())
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub fn get_itemized_page(
		msa_id: MessageSourceId,
		schema_id: SchemaId,
	) -> Option<ItemizedPage<T>> {
		let key: ItemizedKey = (schema_id,);
		StatefulChildTree::<T::KeyHasher>::try_read::<_, ItemizedPage<T>>(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&key,
		)
		.unwrap_or(None)
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub fn get_paginated_page(
		msa_id: MessageSourceId,
		schema_id: SchemaId,
		page_id: PageId,
	) -> Option<PaginatedPage<T>> {
		let key: PaginatedKey = (schema_id, page_id);
		StatefulChildTree::<T::KeyHasher>::try_read::<_, PaginatedPage<T>>(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key,
		)
		.unwrap_or(None)
	}
}
