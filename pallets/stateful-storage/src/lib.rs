//! MSA-centric Storage for [`PayloadLocation::Paginated`] and [`PayloadLocation::Itemized`]
//!
//! ## Quick Links
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Runtime API: `StatefulStorageRuntimeApi`](../pallet_stateful_storage_runtime_api/trait.StatefulStorageRuntimeApi.html)
//! - [Custom RPC API: `StatefulStorageApiServer`](../pallet_stateful_storage_rpc/trait.StatefulStorageApiServer.html)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
#![doc = include_str!("../README.md")]
//!
//! ## Terminology
//! - [`Page`](../pallet_stateful_storage/types/struct.Page.html): Block of on-chain data of a fixed size, which is the underlying type for Itemized and Paginated storage.
//! - [`ItemizedPage`](../pallet_stateful_storage/types/type.ItemizedPage.html): A page containing itemized data
//! - [`PaginatedPage`](../pallet_stateful_storage/types/type.PaginatedPage.html): A page containing paginated data
//!

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
#[cfg(any(feature = "runtime-benchmarks", test))]
mod test_common;
#[cfg(test)]
mod tests;
#[cfg(feature = "runtime-benchmarks")]
use common_primitives::benchmarks::{MsaBenchmarkHelper, SchemaBenchmarkHelper};

mod stateful_child_tree;
pub mod types;
pub mod weights;

extern crate alloc;
use alloc::vec::Vec;

use crate::{stateful_child_tree::StatefulChildTree, types::*};
use common_primitives::{
	msa::{
		DelegatorId, MessageSourceId, MsaLookup, MsaValidator, ProviderId, SchemaGrantValidator,
	},
	node::EIP712Encode,
	schema::{PayloadLocation, SchemaId, SchemaInfoResponse, SchemaProvider, SchemaSetting},
	stateful_storage::{
		ItemizedStoragePageResponse, ItemizedStorageResponse, PageHash, PageId,
		PaginatedStorageResponse,
	},
};

use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_core::{bounded::BoundedVec, crypto::AccountId32};
use sp_runtime::{traits::Convert, DispatchError, MultiSignature};
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

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

		/// The maximum size of a page (in bytes) for an Itemized storage model
		#[pallet::constant]
		type MaxItemizedPageSizeBytes: Get<u32> + Default;

		/// The maximum size of a page (in bytes) for a Paginated storage model
		#[pallet::constant]
		type MaxPaginatedPageSizeBytes: Get<u32> + Default;

		/// The maximum size of a single item in an itemized storage model (in bytes)
		#[pallet::constant]
		type MaxItemizedBlobSizeBytes: Get<u32> + Clone + core::fmt::Debug + PartialEq;

		/// The maximum number of pages in a Paginated storage model
		#[pallet::constant]
		type MaxPaginatedPageId: Get<u16>;

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
	#[pallet::storage_version(STATEFUL_STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// A temporary storage for migration
	#[pallet::storage]
	pub(super) type MigrationPageIndex<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		/// Page would exceed the highest allowable PageId
		PageIdExceedsMaxAllowed,

		/// Page size exceeds max allowable page size
		PageExceedsMaxPageSizeBytes,

		/// Invalid SchemaId or Schema not found
		InvalidSchemaId,

		/// Unsupported operation for schema
		UnsupportedOperationForSchema,

		/// Schema is not valid for storage model
		SchemaPayloadLocationMismatch,

		/// Invalid Message Source Account
		InvalidMessageSourceAccount,

		/// UnauthorizedDelegate
		UnauthorizedDelegate,

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

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_current: BlockNumberFor<T>) -> Weight {
			Weight::zero()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Applies the Add or Delete Actions on the requested Itemized page.
		/// This is treated as a transaction so either all actions succeed or none will be executed.
		///
		/// Note: if called by the state owner, call may succeed even on `SignatureRequired` schemas.
		/// The fact that the entire (signed) transaction is submitted by the owner's keypair is
		/// considered equivalent to supplying a separate signature. Note in that case that a delegate
		/// submitting this extrinsic on behalf of a user would fail.
		///
		/// # Events
		/// * [`Event::ItemizedPageUpdated`]
		/// * [`Event::ItemizedPageDeleted`]
		///
		#[pallet::call_index(0)]
		#[pallet::weight(
			T::WeightInfo::apply_item_actions_delete(actions.len() as u32)
			.max(T::WeightInfo::apply_item_actions_add(Pallet::<T>::sum_add_actions_bytes(actions)))
		)]
		pub fn apply_item_actions(
			origin: OriginFor<T>,
			#[pallet::compact] state_owner_msa_id: MessageSourceId,
			#[pallet::compact] schema_id: SchemaId,
			#[pallet::compact] target_hash: PageHash,
			actions: BoundedVec<
				ItemAction<T::MaxItemizedBlobSizeBytes>,
				T::MaxItemizedActionsCount,
			>,
		) -> DispatchResult {
			let key = ensure_signed(origin)?;
			let is_pruning = actions.iter().any(|a| matches!(a, ItemAction::Delete { .. }));
			let caller_msa_id = Self::check_msa_and_grants(key, state_owner_msa_id, schema_id)?;
			let caller_is_state_owner = caller_msa_id == state_owner_msa_id;
			Self::check_schema_for_write(
				schema_id,
				PayloadLocation::Itemized,
				caller_is_state_owner,
				is_pruning,
			)?;
			Self::update_itemized(state_owner_msa_id, schema_id, target_hash, actions)?;
			Ok(())
		}

		/// Creates or updates an Paginated storage with new payload
		///
		/// Note: if called by the state owner, call may succeed even on `SignatureRequired` schemas.
		/// The fact that the entire (signed) transaction is submitted by the owner's keypair is
		/// considered equivalent to supplying a separate signature. Note in that case that a delegate
		/// submitting this extrinsic on behalf of a user would fail.
		///
		/// # Events
		/// * [`Event::PaginatedPageUpdated`]
		///
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
			ensure!(page_id <= T::MaxPaginatedPageId::get(), Error::<T>::PageIdExceedsMaxAllowed);
			let caller_msa_id =
				Self::check_msa_and_grants(provider_key, state_owner_msa_id, schema_id)?;
			let caller_is_state_owner = caller_msa_id == state_owner_msa_id;
			Self::check_schema_for_write(
				schema_id,
				PayloadLocation::Paginated,
				caller_is_state_owner,
				false,
			)?;
			Self::update_paginated(
				state_owner_msa_id,
				schema_id,
				page_id,
				target_hash,
				PaginatedPage::<T>::from(payload),
			)?;
			Ok(())
		}

		/// Deletes a Paginated storage
		///
		/// Note: if called by the state owner, call may succeed even on `SignatureRequired` schemas.
		/// The fact that the entire (signed) transaction is submitted by the owner's keypair is
		/// considered equivalent to supplying a separate signature. Note in that case that a delegate
		/// submitting this extrinsic on behalf of a user would fail.
		///
		/// # Events
		/// * [`Event::PaginatedPageDeleted`]
		///
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
			ensure!(page_id <= T::MaxPaginatedPageId::get(), Error::<T>::PageIdExceedsMaxAllowed);
			let caller_msa_id =
				Self::check_msa_and_grants(provider_key, state_owner_msa_id, schema_id)?;
			let caller_is_state_owner = caller_msa_id == state_owner_msa_id;
			Self::check_schema_for_write(
				schema_id,
				PayloadLocation::Paginated,
				caller_is_state_owner,
				true,
			)?;
			Self::delete_paginated(state_owner_msa_id, schema_id, page_id, target_hash)?;
			Ok(())
		}

		// REMOVED apply_item_actions_with_signature() at call index 3
		// REMOVED upsert_page_with_signature() at call index 4
		// REMOVED delete_page_with_signature() at call index 5

		/// Applies the Add or Delete Actions on the requested Itemized page that requires signature
		/// since the signature of delegator is checked there is no need for delegation validation
		/// This is treated as a transaction so either all actions succeed or none will be executed.
		///
		/// # Events
		/// * [`Event::ItemizedPageUpdated`]
		/// * [`Event::ItemizedPageDeleted`]
		///
		#[pallet::call_index(6)]
		#[pallet::weight(
		T::WeightInfo::apply_item_actions_with_signature_v2_delete(payload.actions.len() as u32)
		.max(T::WeightInfo::apply_item_actions_with_signature_v2_add(Pallet::<T>::sum_add_actions_bytes(&payload.actions)))
		)]
		pub fn apply_item_actions_with_signature_v2(
			origin: OriginFor<T>,
			delegator_key: T::AccountId,
			proof: MultiSignature,
			payload: ItemizedSignaturePayloadV2<T>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			let is_pruning = payload.actions.iter().any(|a| matches!(a, ItemAction::Delete { .. }));
			Self::check_payload_expiration(
				frame_system::Pallet::<T>::block_number(),
				payload.expiration,
			)?;
			Self::check_signature(&proof, &delegator_key.clone(), &payload)?;
			let state_owner_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(&delegator_key)
				.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;
			Self::check_schema_for_write(
				payload.schema_id,
				PayloadLocation::Itemized,
				true,
				is_pruning,
			)?;
			Self::update_itemized(
				state_owner_msa_id,
				payload.schema_id,
				payload.target_hash,
				payload.actions,
			)?;
			Ok(())
		}

		/// Creates or updates an Paginated storage with new payload that requires signature
		/// since the signature of delegator is checked there is no need for delegation validation
		///
		/// # Events
		/// * [`Event::PaginatedPageUpdated`]
		///
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::upsert_page_with_signature_v2(payload.payload.len() as u32))]
		pub fn upsert_page_with_signature_v2(
			origin: OriginFor<T>,
			delegator_key: T::AccountId,
			proof: MultiSignature,
			payload: PaginatedUpsertSignaturePayloadV2<T>,
		) -> DispatchResult {
			ensure_signed(origin)?;
			ensure!(
				payload.page_id <= T::MaxPaginatedPageId::get(),
				Error::<T>::PageIdExceedsMaxAllowed
			);
			Self::check_payload_expiration(
				frame_system::Pallet::<T>::block_number(),
				payload.expiration,
			)?;
			Self::check_signature(&proof, &delegator_key.clone(), &payload)?;
			let state_owner_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(&delegator_key)
				.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;
			Self::check_schema_for_write(
				payload.schema_id,
				PayloadLocation::Paginated,
				true,
				false,
			)?;
			Self::update_paginated(
				state_owner_msa_id,
				payload.schema_id,
				payload.page_id,
				payload.target_hash,
				PaginatedPage::<T>::from(payload.payload),
			)?;
			Ok(())
		}

		/// Deletes a Paginated storage that requires signature
		/// since the signature of delegator is checked there is no need for delegation validation
		///
		/// # Events
		/// * [`Event::PaginatedPageDeleted`]
		///
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::delete_page_with_signature_v2())]
		pub fn delete_page_with_signature_v2(
			origin: OriginFor<T>,
			delegator_key: T::AccountId,
			proof: MultiSignature,
			payload: PaginatedDeleteSignaturePayloadV2<T>,
		) -> DispatchResult {
			ensure_signed(origin)?;
			ensure!(
				payload.page_id <= T::MaxPaginatedPageId::get(),
				Error::<T>::PageIdExceedsMaxAllowed
			);
			Self::check_payload_expiration(
				frame_system::Pallet::<T>::block_number(),
				payload.expiration,
			)?;
			Self::check_signature(&proof, &delegator_key.clone(), &payload)?;
			let state_owner_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(&delegator_key)
				.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;
			Self::check_schema_for_write(
				payload.schema_id,
				PayloadLocation::Paginated,
				true,
				true,
			)?;
			Self::delete_paginated(
				state_owner_msa_id,
				payload.schema_id,
				payload.page_id,
				payload.target_hash,
			)?;
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Sums the total bytes of each item actions
	pub fn sum_add_actions_bytes(
		actions: &BoundedVec<
			ItemAction<<T as Config>::MaxItemizedBlobSizeBytes>,
			<T as Config>::MaxItemizedActionsCount,
		>,
	) -> u32 {
		actions.iter().fold(0, |acc, a| {
			acc.saturating_add(match a {
				ItemAction::Add { data } => data.len() as u32,
				_ => 0,
			})
		})
	}

	/// This function returns all the paginated storage associated with `msa_id` and `schema_id`
	///
	/// Warning: since this function iterates over all the potential keys it should never called
	/// from runtime.
	pub fn get_paginated_storage(
		msa_id: MessageSourceId,
		schema_id: SchemaId,
	) -> Result<Vec<PaginatedStorageResponse>, DispatchError> {
		Self::check_schema_for_read(schema_id, PayloadLocation::Paginated)?;
		let prefix: PaginatedPrefixKey = (schema_id,);
		Ok(StatefulChildTree::<T::KeyHasher>::prefix_iterator::<
			PaginatedPage<T>,
			PaginatedKey,
			PaginatedPrefixKey,
		>(&msa_id, PALLET_STORAGE_PREFIX, PAGINATED_STORAGE_PREFIX, &prefix)
		.map(|(k, v)| {
			let content_hash = v.get_hash();
			let nonce = v.nonce;
			PaginatedStorageResponse::new(
				k.1,
				msa_id,
				schema_id,
				content_hash,
				nonce,
				v.data.into_inner(),
			)
		})
		.collect())
	}

	/// This function returns all the itemized storage associated with `msa_id` and `schema_id`
	pub fn get_itemized_storage(
		msa_id: MessageSourceId,
		schema_id: SchemaId,
	) -> Result<ItemizedStoragePageResponse, DispatchError> {
		Self::check_schema_for_read(schema_id, PayloadLocation::Itemized)?;
		let page = Self::get_itemized_page_for(msa_id, schema_id)?.unwrap_or_default();
		let items: Vec<ItemizedStorageResponse> = ItemizedOperations::<T>::try_parse(&page, false)
			.map_err(|_| Error::<T>::CorruptedState)?
			.items
			.iter()
			.map(|(key, v)| ItemizedStorageResponse::new(*key, v.to_vec()))
			.collect();
		Ok(ItemizedStoragePageResponse::new(msa_id, schema_id, page.get_hash(), page.nonce, items))
	}

	/// This function checks to ensure `payload_expire_block` is in a valid range
	///
	/// # Errors
	/// * [`Error::ProofHasExpired`]
	/// * [`Error::ProofNotYetValid`]
	///
	pub fn check_payload_expiration(
		current_block: BlockNumberFor<T>,
		payload_expire_block: BlockNumberFor<T>,
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
	pub fn check_signature<P>(
		signature: &MultiSignature,
		signer: &T::AccountId,
		payload: &P,
	) -> DispatchResult
	where
		P: Encode + EIP712Encode,
	{
		let key = T::ConvertIntoAccountId32::convert(signer.clone());

		ensure!(
			common_runtime::signature::check_signature(signature, key, payload),
			Error::<T>::InvalidSignature
		);

		Ok(())
	}

	/// The furthest in the future a mortality_block value is allowed
	/// to be for current_block
	/// This is calculated to be past the risk of a replay attack
	fn mortality_block_limit(current_block: BlockNumberFor<T>) -> BlockNumberFor<T> {
		current_block + BlockNumberFor::<T>::from(T::MortalityWindowSize::get())
	}

	/// Checks that the schema is valid for is action
	///
	/// # Errors
	/// * [`Error::InvalidSchemaId`]
	/// * [`Error::SchemaPayloadLocationMismatch`]
	///
	fn check_schema_for_read(
		schema_id: SchemaId,
		expected_payload_location: PayloadLocation,
	) -> Result<SchemaInfoResponse, DispatchError> {
		let schema = T::SchemaProvider::get_schema_info_by_id(schema_id)
			.ok_or(Error::<T>::InvalidSchemaId)?;

		// Ensure that the schema's payload location matches the expected location.
		ensure!(
			schema.payload_location == expected_payload_location,
			Error::<T>::SchemaPayloadLocationMismatch
		);

		Ok(schema)
	}

	/// Checks that the schema is valid for is action
	///
	/// # Errors
	/// * [`Error::InvalidSchemaId`]
	/// * [`Error::SchemaPayloadLocationMismatch`]
	/// * [`Error::UnsupportedOperationForSchema`]
	///
	fn check_schema_for_write(
		schema_id: SchemaId,
		expected_payload_location: PayloadLocation,
		is_payload_signed: bool,
		is_deleting: bool,
	) -> DispatchResult {
		let schema = Self::check_schema_for_read(schema_id, expected_payload_location)?;

		// Ensure that the schema allows signed payloads.
		// If so, calling extrinsic must be of signature type.
		if schema.settings.contains(&SchemaSetting::SignatureRequired) {
			ensure!(is_payload_signed, Error::<T>::UnsupportedOperationForSchema);
		}

		// Ensure that the schema does not allow deletion for AppendOnly SchemaSetting.
		if schema.settings.contains(&SchemaSetting::AppendOnly) {
			ensure!(!is_deleting, Error::<T>::UnsupportedOperationForSchema);
		}

		Ok(())
	}

	/// Checks that existence of Msa for certain key and if the grant is valid when the caller Msa
	/// is different from the state owner Msa
	///
	/// # Errors
	/// * [`Error::InvalidMessageSourceAccount`]
	/// * [`Error::UnauthorizedDelegate`]
	///
	fn check_msa_and_grants(
		key: T::AccountId,
		state_owner_msa_id: MessageSourceId,
		schema_id: SchemaId,
	) -> Result<MessageSourceId, DispatchError> {
		let caller_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(&key)
			.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;

		// if caller and owner are the same no delegation is needed
		if caller_msa_id != state_owner_msa_id {
			let current_block = frame_system::Pallet::<T>::block_number();
			T::SchemaGrantValidator::ensure_valid_schema_grant(
				ProviderId(caller_msa_id),
				DelegatorId(state_owner_msa_id),
				schema_id,
				current_block,
			)
			.map_err(|_| Error::<T>::UnauthorizedDelegate)?;
		}

		Ok(caller_msa_id)
	}

	/// Updates an itemized storage by applying provided actions and deposit events
	///
	/// # Events
	/// * [`Event::ItemizedPageUpdated`]
	/// * [`Event::ItemizedPageDeleted`]
	///
	fn update_itemized(
		state_owner_msa_id: MessageSourceId,
		schema_id: SchemaId,
		target_hash: PageHash,
		actions: BoundedVec<ItemAction<T::MaxItemizedBlobSizeBytes>, T::MaxItemizedActionsCount>,
	) -> DispatchResult {
		let key: ItemizedKey = (schema_id,);
		let existing_page =
			Self::get_itemized_page_for(state_owner_msa_id, schema_id)?.unwrap_or_default();

		let prev_content_hash = existing_page.get_hash();
		ensure!(target_hash == prev_content_hash, Error::<T>::StalePageState);

		let mut updated_page =
			ItemizedOperations::<T>::apply_item_actions(&existing_page, &actions[..]).map_err(
				|e| match e {
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
				},
			)?;
		updated_page.nonce = existing_page.nonce.wrapping_add(1);

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

	/// Updates a page from paginated storage by provided new page
	///
	/// # Events
	/// * [`Event::PaginatedPageUpdated`]
	///
	fn update_paginated(
		state_owner_msa_id: MessageSourceId,
		schema_id: SchemaId,
		page_id: PageId,
		target_hash: PageHash,
		mut new_page: PaginatedPage<T>,
	) -> DispatchResult {
		let keys: PaginatedKey = (schema_id, page_id);
		let existing_page: PaginatedPage<T> =
			Self::get_paginated_page_for(state_owner_msa_id, schema_id, page_id)?
				.unwrap_or_default();

		let prev_content_hash: PageHash = existing_page.get_hash();
		ensure!(target_hash == prev_content_hash, Error::<T>::StalePageState);

		new_page.nonce = existing_page.nonce.wrapping_add(1);

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

	/// Deletes a page from paginated storage
	///
	/// # Events
	/// * [`Event::PaginatedPageDeleted`]
	///
	fn delete_paginated(
		state_owner_msa_id: MessageSourceId,
		schema_id: SchemaId,
		page_id: PageId,
		target_hash: PageHash,
	) -> DispatchResult {
		let keys: PaginatedKey = (schema_id, page_id);
		if let Some(existing_page) =
			Self::get_paginated_page_for(state_owner_msa_id, schema_id, page_id)?
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

	/// Gets a paginated storage for desired parameters
	pub fn get_paginated_page_for(
		msa_id: MessageSourceId,
		schema_id: SchemaId,
		page_id: PageId,
	) -> Result<Option<PaginatedPage<T>>, DispatchError> {
		let keys: PaginatedKey = (schema_id, page_id);
		Ok(StatefulChildTree::<T::KeyHasher>::try_read::<_, PaginatedPage<T>>(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
		)
		.map_err(|_| Error::<T>::CorruptedState)?)
	}

	/// Gets an itemized storage for desired parameters
	pub fn get_itemized_page_for(
		msa_id: MessageSourceId,
		schema_id: SchemaId,
	) -> Result<Option<ItemizedPage<T>>, DispatchError> {
		let keys: ItemizedKey = (schema_id,);
		Ok(StatefulChildTree::<T::KeyHasher>::try_read::<_, ItemizedPage<T>>(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&keys,
		)
		.map_err(|_| Error::<T>::CorruptedState)?)
	}
}
