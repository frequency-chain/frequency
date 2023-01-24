//! # MSA Pallet
//! The MSA pallet provides functionality for handling Message Source Accounts.
//!
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Runtime API: `MsaRuntimeApi`](../pallet_msa_runtime_api/trait.MsaRuntimeApi.html)
//! - [Custom RPC API: `MsaApiServer`](../pallet_msa_rpc/trait.MsaApiServer.html)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
//!
//! ## Overview
//!
//! The Message Source Account (MSA) is an account that can be sponsored such that public keys attached to the account
//! to control the MSA are not required to hold any balance, while still being able to control revocation of any delegation or control.
//!
//! The MSA is represented by an Id and has one or more public keys attached to it for control.
//! The same public key may only be attached to ONE MSA at any single point in time.
//!
//! The MSA pallet provides functions for:
//!
//! - Creating, reading, updating, and deleting operations for MSAs.
//! - Managing delegation relationships for MSAs.
//! - Managing keys associated with MSA.
//!
//! ## Terminology
//! * **MSA:** Message Source Account. A Source or Provider Account for Frequency Messages. It may or may not have `Capacity`.  It must have at least one public key (`AccountId`) associated with it.
//! An MSA is required for sending Capacity-based messages and for creating Delegations.
//! * **MSA ID:** the ID number created for a new Message Source Account and associated with one or more Public Keys.
//! * **MSA Public Key:** the keys that control the MSA, represented by Substrate `AccountId`.
//! * **Delegator:** a Message Source Account that has provably delegated certain actions to a Provider, typically sending a `Message`
//! * **Provider:** the Message Source Account that a Delegator has delegated specific actions to.
//! * **Delegation:** A stored Delegator-Provider association between MSAs which permits the Provider to perform specific actions on the Delegator's behalf.
//!
//! ## Implementations
//!
//! - [`MsaLookup`](../common_primitives/msa/trait.MsaLookup.html): Functions for accessing MSAs.
//! - [`MsaValidator`](../common_primitives/msa/trait.MsaValidator.html): Functions for validating MSAs.
//! - [`ProviderLookup`](../common_primitives/msa/trait.ProviderLookup.html): Functions for accessing Provider info.
//! - [`DelegationValidator`](../common_primitives/msa/trait.DelegationValidator.html): Functions for validating delegations.
//! - [`SchemaGrantValidator`](../common_primitives/msa/trait.SchemaGrantValidator.html): Functions for validating schema grants.
//!
//! ### Assumptions
//!
//! * Total MSA keys should be less than the constant `Config::MSA::MaxPublicKeysPerMsa`.
//! * Maximum schemas, for which provider has publishing rights, be less than `Config::MSA::MaxSchemaGrantsPerDelegation`
//!

#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchInfo, DispatchResult},
	ensure,
	pallet_prelude::*,
	traits::IsSubType,
};

#[cfg(feature = "runtime-benchmarks")]
use common_primitives::benchmarks::MsaBenchmarkHelper;

use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_core::crypto::AccountId32;
use sp_runtime::{
	traits::{Convert, DispatchInfoOf, Dispatchable, One, SignedExtension, Verify, Zero},
	ArithmeticError, DispatchError, MultiSignature,
};
use sp_std::prelude::*;

use common_primitives::{
	msa::{
		Delegation, DelegationValidator, DelegatorId, MsaLookup, MsaValidator, ProviderId,
		ProviderLookup, ProviderRegistryEntry, SchemaGrantValidator,
		EXPECTED_MAX_NUMBER_OF_PROVIDERS_PER_DELEGATOR,
	},
	schema::{SchemaId, SchemaValidator},
};

pub use common_primitives::{msa::MessageSourceId, utils::wrap_binary_data};
pub use pallet::*;
pub use types::{AddKeyData, AddProvider, PermittedDelegationSchemas, EMPTY_FUNCTION};
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod types;

#[cfg(test)]
mod audit_replay_tests;
#[cfg(test)]
mod replay_tests;
#[cfg(test)]
mod signature_registry_tests;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::log::error as log_err;

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// AccountId truncated to 32 bytes
		type ConvertIntoAccountId32: Convert<Self::AccountId, AccountId32>;

		/// Maximum count of keys allowed per MSA
		#[pallet::constant]
		type MaxPublicKeysPerMsa: Get<u8>;

		/// Maximum count of schemas granted for publishing data per Provider
		#[pallet::constant]
		type MaxSchemaGrantsPerDelegation: Get<u32> + Clone + sp_std::fmt::Debug + Eq;

		/// Maximum provider name size allowed per MSA association
		#[pallet::constant]
		type MaxProviderNameSize: Get<u32>;

		/// A type that will supply schema related information.
		type SchemaValidator: SchemaValidator<SchemaId>;

		/// The number of blocks per virtual "bucket" in the PayloadSignatureRegistry
		/// Virtual buckets are the first part of the double key in the PayloadSignatureRegistry
		/// StorageDoubleMap.  This permits a key grouping that enables mass removal
		/// of stale signatures which are no longer at risk of replay.
		#[pallet::constant]
		type MortalityWindowSize: Get<u32>;

		/// The maximum number of signatures that can be assigned to a virtual bucket. In other
		/// words, no more than this many signatures can be assigned a specific first-key value.
		#[pallet::constant]
		type MaxSignaturesPerBucket: Get<u32>;

		/// The total number of virtual buckets
		/// There are exactly NumberOfBuckets first-key values in PayloadSignatureRegistry.
		#[pallet::constant]
		type NumberOfBuckets: Get<u32>;

		/// The maximum number of signatures that can be stored in PayloadSignatureRegistry.
		/// This MUST be MaxSignaturesPerBucket * NumberOfBuckets.
		/// It's separate because this config is provided to signature storage and cannot be a
		/// calculated value.
		#[pallet::constant]
		type MaxSignaturesStored: Get<Option<u32>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	/// Storage type for the current MSA identifier maximum.
	/// We need to track this value because the identifier maximum
	/// is incremented each time a new identifier is created.
	/// - Value: The current maximum MSA Id
	#[pallet::storage]
	#[pallet::getter(fn get_current_msa_identifier_maximum)]
	pub type CurrentMsaIdentifierMaximum<T> = StorageValue<_, MessageSourceId, ValueQuery>;

	/// Storage type for mapping the relationship between a Delegator and its Provider.
	/// - Keys: Delegator MSA, Provider MSA
	/// - Value: [`Delegation`](common_primitives::msa::Delegation)
	#[pallet::storage]
	#[pallet::getter(fn get_delegation)]
	pub type DelegatorAndProviderToDelegation<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		DelegatorId,
		Twox64Concat,
		ProviderId,
		Delegation<SchemaId, T::BlockNumber, T::MaxSchemaGrantsPerDelegation>,
		OptionQuery,
	>;

	/// Provider registration information
	/// - Key: Provider MSA Id
	/// - Value: [`ProviderRegistryEntry`](common_primitives::msa::ProviderRegistryEntry)
	#[pallet::storage]
	#[pallet::getter(fn get_provider_registry_entry)]
	pub type ProviderToRegistryEntry<T: Config> = StorageMap<
		_,
		Twox64Concat,
		ProviderId,
		ProviderRegistryEntry<T::MaxProviderNameSize>,
		OptionQuery,
	>;

	/// Storage type for key to MSA information
	/// - Key: AccountId
	/// - Value: [`MessageSourceId`]
	#[pallet::storage]
	#[pallet::getter(fn get_msa_by_public_key)]
	pub type PublicKeyToMsaId<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, MessageSourceId, OptionQuery>;

	/// Storage type for a reference counter of the number of keys associated to an MSA
	/// - Key: MSA Id
	/// - Value: [`u8`] Counter of Keys associated with the MSA
	#[pallet::storage]
	#[pallet::getter(fn get_public_key_count_by_msa_id)]
	pub(super) type PublicKeyCountForMsaId<T: Config> =
		StorageMap<_, Twox64Concat, MessageSourceId, u8, ValueQuery>;

	/// PayloadSignatureRegistry is used to prevent replay attacks for extrinsics
	/// that take an externally-signed payload.
	/// For this to work, the payload must include a mortality block number, which
	/// is used in lieu of a monotonically increasing nonce.
	#[pallet::storage]
	#[pallet::getter(fn get_payload_signature_registry)]
	pub(super) type PayloadSignatureRegistry<T: Config> = StorageDoubleMap<
		_,                      // prefix
		Twox64Concat,           // hasher for key1
		T::BlockNumber, // Bucket number. Stored as BlockNumber because I'm done arguing with rust about it.
		Twox64Concat,   // hasher for key2
		MultiSignature, // An externally-created Signature for an external payload, provided by an extrinsic
		T::BlockNumber, // An actual flipping block number.
		OptionQuery,    // The type for the query
		GetDefault,     // OnEmpty return type, defaults to None
		T::MaxSignaturesStored, // Maximum total signatures to store
	>;

	/// Records how many signatures are currently stored in each virtual signature registration bucket
	#[pallet::storage]
	pub(super) type PayloadSignatureBucketCount<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::BlockNumber, // bucket number
		u32,            // number of signatures
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new Message Service Account was created with a new MessageSourceId
		MsaCreated {
			/// The MSA for the Event
			msa_id: MessageSourceId,

			/// The key added to the MSA
			key: T::AccountId,
		},
		/// An AccountId has been associated with a MessageSourceId
		PublicKeyAdded {
			/// The MSA for the Event
			msa_id: MessageSourceId,

			/// The key added to the MSA
			key: T::AccountId,
		},
		/// An AccountId had all permissions revoked from its MessageSourceId
		PublicKeyDeleted {
			/// The key no longer approved for the associated MSA
			key: T::AccountId,
		},
		/// A delegation relationship was added with the given provider and delegator
		DelegationGranted {
			/// The Provider MSA Id
			provider_id: ProviderId,

			/// The Delegator MSA Id
			delegator_id: DelegatorId,
		},
		/// A Provider-MSA relationship was registered
		ProviderCreated {
			/// The MSA id associated with the provider
			provider_id: ProviderId,
		},
		/// The Delegator revoked its delegation to the Provider
		DelegationRevoked {
			/// The Provider MSA Id
			provider_id: ProviderId,

			/// The Delegator MSA Id
			delegator_id: DelegatorId,
		},
		/// The MSA has been retired.
		MsaRetired {
			/// The MSA id for the Event
			msa_id: MessageSourceId,
		},
		/// A an update to the delegation occurred (ex. schema permissions where updated).
		DelegationUpdated {
			/// The Provider MSA Id
			provider_id: ProviderId,

			/// The Delegator MSA Id
			delegator_id: DelegatorId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Tried to add a key that was already registered to an MSA
		KeyAlreadyRegistered,

		/// MsaId values have reached the maximum
		MsaIdOverflow,

		/// Cryptographic signature verification failed for adding a key to MSA
		MsaOwnershipInvalidSignature,

		/// Ony the MSA Owner may perform the operation
		NotMsaOwner,

		/// Cryptographic signature failed verification
		InvalidSignature,

		/// Only the KeyOwner may perform the operation
		NotKeyOwner,

		/// An operation was attempted with an unknown Key
		NoKeyExists,

		/// The number of key values has reached its maximum
		KeyLimitExceeded,

		/// A transaction's Origin (AccountId) may not remove itself
		InvalidSelfRemoval,

		/// An MSA may not be its own delegate
		InvalidSelfProvider,

		/// An invalid schema Id was provided
		InvalidSchemaId,

		/// The delegation relationship already exists for the given MSA Ids
		DuplicateProvider,

		/// Cryptographic signature verification failed for adding the Provider as delegate
		AddProviderSignatureVerificationFailed,

		/// Origin attempted to add a delegate for someone else's MSA
		UnauthorizedDelegator,

		/// Origin attempted to add a different delegate than what was in the payload
		UnauthorizedProvider,

		/// The operation was attempted with a revoked delegation
		DelegationRevoked,

		/// The operation was attempted with an unknown delegation
		DelegationNotFound,

		/// The MSA id submitted for provider creation has already been associated with a provider
		DuplicateProviderRegistryEntry,

		/// The maximum length for a provider name has been exceeded
		ExceedsMaxProviderNameSize,

		/// The maximum number of schema grants has been exceeded
		ExceedsMaxSchemaGrantsPerDelegation,

		/// Provider is not permitted to publish for given schema_id
		SchemaNotGranted,

		/// The operation was attempted with a non-provider MSA
		ProviderNotRegistered,

		/// The submitted proof has expired; the current block is less the expiration block
		ProofHasExpired,

		/// The submitted proof expiration block is too far in the future
		ProofNotYetValid,

		/// Attempted to add a signature when the signature is already in the registry
		SignatureAlreadySubmitted,

		/// Cryptographic signature verification failed for proving ownership of new public-key.
		NewKeyOwnershipInvalidSignature,

		/// Attempted to request validity of schema permission or delegation in the future.
		CannotPredictValidityPastCurrentBlock,

		/// Attempted to add a new signature to a full virtual signature registration bucket
		SignatureRegistryLimitExceeded,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(current: T::BlockNumber) -> Weight {
			Self::reset_virtual_bucket_if_needed(current)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates an MSA for the Origin (sender of the transaction).  Origin is assigned an MSA ID.
		///
		/// # Events
		/// * [`Event::MsaCreated`]
		///
		/// # Errors
		///
		/// * [`Error::KeyAlreadyRegistered`] - MSA is already registered to the Origin.
		///
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create())]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let public_key = ensure_signed(origin)?;

			let (new_msa_id, new_public_key) =
				Self::create_account(public_key, |_| -> DispatchResult { Ok(()) })?;

			Self::deposit_event(Event::MsaCreated { msa_id: new_msa_id, key: new_public_key });

			Ok(())
		}

		/// `Origin` MSA creates an MSA on behalf of `delegator_key`, creates a Delegation with the `delegator_key`'s MSA as the Delegator and `origin` as `Provider`. Deposits events [`MsaCreated`](Event::MsaCreated) and [`DelegationGranted`](Event::DelegationGranted).
		///
		/// # Remarks
		/// * Origin MUST be the provider
		/// * Signatures should be over the [`AddProvider`] struct
		///
		/// # Events
		/// * [`Event::MsaCreated`]
		/// * [`Event::DelegationGranted`]
		///
		/// # Errors
		///
		/// * [`Error::UnauthorizedProvider`] - payload's MSA does not match given provider MSA.
		/// * [`Error::InvalidSignature`] - `proof` verification fails; `delegator_key` must have signed `add_provider_payload`
		/// * [`Error::NoKeyExists`] - there is no MSA for `origin`.
		/// * [`Error::KeyAlreadyRegistered`] - there is already an MSA for `delegator_key`.
		/// * [`Error::ProviderNotRegistered`] - the a non-provider MSA is used as the provider
		/// * [`Error::ProofNotYetValid`] - `add_provider_payload` expiration is too far in the future
		/// * [`Error::ProofHasExpired`] - `add_provider_payload` expiration is in the past
		/// * [`Error::SignatureAlreadySubmitted`] - signature has already been used
		///
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::create_sponsored_account_with_delegation(
		add_provider_payload.schema_ids.len() as u32
		))]
		pub fn create_sponsored_account_with_delegation(
			origin: OriginFor<T>,
			delegator_key: T::AccountId,
			proof: MultiSignature,
			add_provider_payload: AddProvider,
		) -> DispatchResult {
			let provider_key = ensure_signed(origin)?;

			Self::verify_signature(&proof, &delegator_key, add_provider_payload.encode())?;

			Self::register_signature(&proof, add_provider_payload.expiration.into())?;

			let provider_msa_id = Self::ensure_valid_msa_key(&provider_key)?;
			ensure!(
				add_provider_payload.authorized_msa_id == provider_msa_id,
				Error::<T>::UnauthorizedProvider
			);

			// Verify that the provider is a registered provider
			ensure!(
				Self::is_registered_provider(provider_msa_id),
				Error::<T>::ProviderNotRegistered
			);

			let (new_delegator_msa_id, new_delegator_public_key) =
				Self::create_account(delegator_key, |new_msa_id| -> DispatchResult {
					Self::add_provider(
						ProviderId(provider_msa_id),
						DelegatorId(new_msa_id),
						add_provider_payload.schema_ids,
					)?;
					Ok(())
				})?;

			Self::deposit_event(Event::MsaCreated {
				msa_id: new_delegator_msa_id,
				key: new_delegator_public_key,
			});

			Self::deposit_event(Event::DelegationGranted {
				delegator_id: DelegatorId(new_delegator_msa_id),
				provider_id: ProviderId(provider_msa_id),
			});

			Ok(())
		}

		/// Adds an association between MSA id and ProviderRegistryEntry. As of now, the
		/// only piece of metadata we are recording is provider name.
		///
		/// # Events
		/// * [`Event::ProviderCreated`]
		///
		/// # Errors
		/// * [`Error::NoKeyExists`] - origin does not have an MSA
		/// * [`Error::ExceedsMaxProviderNameSize`] - Too long of a provider name
		/// * [`Error::DuplicateProviderRegistryEntry`] - a ProviderRegistryEntry associated with the given MSA id already exists.
		///
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::create_provider(provider_name.len() as u32))]
		pub fn create_provider(origin: OriginFor<T>, provider_name: Vec<u8>) -> DispatchResult {
			let provider_key = ensure_signed(origin)?;
			let bounded_name: BoundedVec<u8, T::MaxProviderNameSize> =
				provider_name.try_into().map_err(|_| Error::<T>::ExceedsMaxProviderNameSize)?;

			let provider_msa_id = Self::ensure_valid_msa_key(&provider_key)?;
			ProviderToRegistryEntry::<T>::try_mutate(
				ProviderId(provider_msa_id),
				|maybe_metadata| -> DispatchResult {
					ensure!(
						maybe_metadata.take().is_none(),
						Error::<T>::DuplicateProviderRegistryEntry
					);
					*maybe_metadata = Some(ProviderRegistryEntry { provider_name: bounded_name });
					Ok(())
				},
			)?;
			Self::deposit_event(Event::ProviderCreated {
				provider_id: ProviderId(provider_msa_id),
			});
			Ok(())
		}

		/// Creates a new Delegation for an existing MSA, with `origin` as the Provider and `delegator_key` is the delegator.
		/// Since it is being sent on the Delegator's behalf, it requires the Delegator to authorize the new Delegation.
		///
		/// # Remarks
		/// * Origin MUST be the provider
		/// * Signatures should be over the [`AddProvider`] struct
		///
		/// # Events
		/// * [`Event::DelegationGranted`]
		///
		/// # Errors
		/// * [`Error::AddProviderSignatureVerificationFailed`] - `origin`'s MSA ID does not equal `add_provider_payload.authorized_msa_id`.
		/// * [`Error::DuplicateProvider`] - there is already a Delegation for `origin` MSA and `delegator_key` MSA.
		/// * [`Error::UnauthorizedProvider`] - `add_provider_payload.authorized_msa_id`  does not match MSA ID of `delegator_key`.
		/// * [`Error::InvalidSelfProvider`] - Cannot delegate to the same MSA
		/// * [`Error::InvalidSignature`] - `proof` verification fails; `delegator_key` must have signed `add_provider_payload`
		/// * [`Error::NoKeyExists`] - there is no MSA for `origin` or `delegator_key`.
		/// * [`Error::ProviderNotRegistered`] - the a non-provider MSA is used as the provider
		/// * [`Error::UnauthorizedDelegator`] - Origin attempted to add a delegate for someone else's MSA
		///
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::grant_delegation(add_provider_payload.schema_ids.len() as u32))]
		pub fn grant_delegation(
			origin: OriginFor<T>,
			delegator_key: T::AccountId,
			proof: MultiSignature,
			add_provider_payload: AddProvider,
		) -> DispatchResult {
			let provider_key = ensure_signed(origin)?;

			// delegator must have signed the payload.
			Self::verify_signature(&proof, &delegator_key, add_provider_payload.encode())
				.map_err(|_| Error::<T>::AddProviderSignatureVerificationFailed)?;

			Self::register_signature(&proof, add_provider_payload.expiration.into())?;
			let (provider_id, delegator_id) =
				Self::ensure_valid_registered_provider(&delegator_key, &provider_key)?;

			ensure!(
				add_provider_payload.authorized_msa_id == provider_id.0,
				Error::<T>::UnauthorizedDelegator
			);
			Self::add_provider(provider_id, delegator_id, add_provider_payload.schema_ids)?;
			Self::deposit_event(Event::DelegationGranted { delegator_id, provider_id });

			Ok(())
		}

		/// Delegator (Origin) MSA terminates a delegation relationship with the `Provider` MSA. Deposits event[`DelegationRevoked`](Event::DelegationRevoked).
		///
		/// # Events
		/// * [`Event::DelegationRevoked`]
		///
		/// # Errors
		///
		/// * [`Error::NoKeyExists`] - origin does not have an MSA
		/// * [`Error::DelegationRevoked`] - the delegation has already been revoked.
		/// * [`Error::DelegationNotFound`] - there is not delegation relationship between Origin and Delegator or Origin and Delegator are the same.
		///
		#[pallet::call_index(4)]
		#[pallet::weight((T::WeightInfo::revoke_delegation_by_delegator(), DispatchClass::Normal, Pays::No))]
		pub fn revoke_delegation_by_delegator(
			origin: OriginFor<T>,
			#[pallet::compact] provider_msa_id: MessageSourceId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match Self::get_msa_by_public_key(&who) {
				Some(delegator_msa_id) => {
					let delegator_id = DelegatorId(delegator_msa_id);
					let provider_id = ProviderId(provider_msa_id);
					Self::revoke_provider(provider_id, delegator_id)?;
					Self::deposit_event(Event::DelegationRevoked { delegator_id, provider_id });
				},
				None => {
					log_err!("SignedExtension did not catch invalid MSA for account {:?}, ", who);
				},
			}

			Ok(())
		}

		/// Adds a given `new_key` to `msa_id` of the account signing `msa_owner_proof`, which must match the MSA in `add_key_payload`.
		/// The `new_key` must sign the `add_key_payload` to authorize the addition.
		///
		/// # Remarks
		/// * Origin can be same as msa owner.
		/// * Signatures should be over the [`AddKeyData`] struct
		///
		/// # Events
		/// * [`Event::PublicKeyAdded`]
		///
		/// # Errors
		///
		/// * [`Error::MsaOwnershipInvalidSignature`] - `key` is not a valid signer of the provided `add_key_payload`.
		/// * [`Error::NewKeyOwnershipInvalidSignature`] - `key` is not a valid signer of the provided `add_key_payload`.
		/// * [`Error::NoKeyExists`] - the MSA id for the account in `add_key_payload` does not exist.
		/// * [`Error::NotMsaOwner`] - Origin's MSA is not the same as 'add_key_payload` MSA. Essentially you can only add a key to your own MSA.
		/// * [`Error::ProofHasExpired`] - the current block is less than the `expired` block number set in `AddKeyData`.
		/// * [`Error::ProofNotYetValid`] - the `expired` block number set in `AddKeyData` is greater than the current block number plus mortality_block_limit().
		/// * [`Error::SignatureAlreadySubmitted`] - signature has already been used.
		///
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::add_public_key_to_msa())]
		pub fn add_public_key_to_msa(
			origin: OriginFor<T>,
			msa_owner_public_key: T::AccountId,
			msa_owner_proof: MultiSignature,
			new_key_owner_proof: MultiSignature,
			add_key_payload: AddKeyData<T>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			Self::verify_signature(
				&msa_owner_proof,
				&msa_owner_public_key,
				add_key_payload.encode(),
			)
			.map_err(|_| Error::<T>::MsaOwnershipInvalidSignature)?;

			Self::verify_signature(
				&new_key_owner_proof,
				&add_key_payload.new_public_key.clone(),
				add_key_payload.encode(),
			)
			.map_err(|_| Error::<T>::NewKeyOwnershipInvalidSignature)?;

			Self::register_signature(&msa_owner_proof, add_key_payload.expiration.into())?;
			Self::register_signature(&new_key_owner_proof, add_key_payload.expiration.into())?;

			let msa_id = add_key_payload.msa_id;

			Self::ensure_msa_owner(&msa_owner_public_key, msa_id)?;

			Self::add_key(
				msa_id,
				&add_key_payload.new_public_key.clone(),
				|msa_id| -> DispatchResult {
					Self::deposit_event(Event::PublicKeyAdded {
						msa_id,
						key: add_key_payload.new_public_key.clone(),
					});
					Ok(())
				},
			)?;

			Ok(())
		}

		/// Remove a key associated with an MSA by expiring it at the current block.
		///
		/// # Remarks
		/// * Removal of key deletes the association of the key with the MSA.
		/// * The key can be re-added to same or another MSA if needed.
		///
		/// # Events
		/// * [`Event::PublicKeyDeleted`]
		///
		/// # Errors
		/// * [`Error::InvalidSelfRemoval`] - `origin` and `key` are the same.
		/// * [`Error::NotKeyOwner`] - `origin` does not own the MSA ID associated with `key`.
		/// * [`Error::NoKeyExists`] - `origin` or `key` are not associated with `origin`'s MSA ID.
		///
		#[pallet::call_index(6)]
		#[pallet::weight((T::WeightInfo::delete_msa_public_key(), DispatchClass::Normal, Pays::No))]
		pub fn delete_msa_public_key(
			origin: OriginFor<T>,
			public_key_to_delete: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match Self::get_msa_by_public_key(&who) {
				Some(who_msa_id) => {
					Self::delete_key_for_msa(who_msa_id, &public_key_to_delete)?;

					// Deposit the event
					Self::deposit_event(Event::PublicKeyDeleted { key: public_key_to_delete });
				},
				None => {
					log_err!("SignedExtension did not catch invalid MSA for account {:?}, ", who);
				},
			}
			Ok(())
		}

		/// Provider MSA terminates Delegation with a Delegator MSA by expiring the Delegation at the current block.
		///
		/// # Events
		/// * [`Event::DelegationRevoked`]
		///
		/// # Errors
		///
		/// * [`Error::NoKeyExists`] - `provider_key` does not have an MSA key.
		/// * [`Error::DelegationRevoked`] - delegation is already revoked
		/// * [`Error::DelegationNotFound`] - no Delegation found between origin MSA and delegator MSA.
		///
		#[pallet::call_index(7)]
		#[pallet::weight((T::WeightInfo::revoke_delegation_by_provider(), DispatchClass::Normal, Pays::No))]
		pub fn revoke_delegation_by_provider(
			origin: OriginFor<T>,
			#[pallet::compact] delegator: MessageSourceId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Revoke delegation relationship entry in the delegation registry by expiring it
			// at the current block
			// validity checks are in SignedExtension so in theory this should never error.
			match Self::get_msa_by_public_key(&who) {
				Some(msa_id) => {
					let provider_id = ProviderId(msa_id);
					let delegator_id = DelegatorId(delegator);
					Self::revoke_provider(provider_id, delegator_id)?;
					Self::deposit_event(Event::DelegationRevoked { provider_id, delegator_id })
				},
				None => {
					log_err!("SignedExtension did not catch invalid MSA for account {:?}, ", who);
				},
			}

			Ok(())
		}

		/// Grants a list of schema permissions to a provider. Schemas that have already
		/// been granted are ignored. Schemas that are revoked are re-granted.
		///
		/// # Events
		/// * [`Event::DelegationUpdated`]
		///
		/// # Errors
		/// * [`Error::NoKeyExists`] no MSA for `origin`.
		/// * [`Error::DelegationNotFound`] no delegation relationship between Origin and Delegator or Origin and Delegator are the same.
		/// * [`Error::ExceedsMaxSchemaGrantsPerDelegation`] the limit of maximum allowed grants per delegation relationship has been exceeded.
		///
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::grant_schema_permissions(schema_ids.len() as u32))]
		pub fn grant_schema_permissions(
			origin: OriginFor<T>,
			provider_msa_id: MessageSourceId,
			schema_ids: Vec<SchemaId>,
		) -> DispatchResult {
			let delegator_key = ensure_signed(origin)?;
			let delegator_msa_id = Self::ensure_valid_msa_key(&delegator_key)?;
			let provider_id = ProviderId(provider_msa_id);
			let delegator_id = DelegatorId(delegator_msa_id);

			Self::grant_permissions_for_schemas(delegator_id, provider_id, schema_ids)?;
			Self::deposit_event(Event::DelegationUpdated { provider_id, delegator_id });

			Ok(())
		}

		/// Revokes a list of schema permissions to a provider. Attempting to revoke a Schemas that have already
		/// been revoked are ignored.
		///
		/// # Events
		/// - [DelegationUpdated](Event::DelegationUpdated)
		///
		/// # Errors
		/// - [`NoKeyExists`](Error::NoKeyExists) - If there is not MSA for `origin`.
		/// - [`DelegationNotFound`](Error::DelegationNotFound) - If there is not delegation relationship between Origin and Delegator or Origin and Delegator are the same.
		/// - [`SchemaNotGranted`](Error::SchemaNotGranted) - If attempting to revoke a schema that has not previously been granted.
		///
		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::revoke_schema_permissions(
		schema_ids.len() as u32
		))]
		pub fn revoke_schema_permissions(
			origin: OriginFor<T>,
			provider_msa_id: MessageSourceId,
			schema_ids: Vec<SchemaId>,
		) -> DispatchResult {
			let delegator_key = ensure_signed(origin)?;
			let delegator_msa_id = Self::ensure_valid_msa_key(&delegator_key)?;
			let provider_id = ProviderId(provider_msa_id);
			let delegator_id = DelegatorId(delegator_msa_id);

			Self::revoke_permissions_for_schemas(delegator_id, provider_id, schema_ids)?;
			Self::deposit_event(Event::DelegationUpdated { provider_id, delegator_id });

			Ok(())
		}

		/// Retires an MSA
		///
		/// When a user wants to disassociate themselves from Frequency, they can retire their MSA for free provided that:
		///  (1) They own the MSA
		///  (2) There is only one account key
		///  (3) The MSA is not a registered provider.
		///
		/// This does not currently remove any messages related to the MSA.
		///
		/// # Events
		/// * [`Event::PublicKeyDeleted`]
		/// * [`Event::MsaRetired`]
		///
		/// # Errors
		/// * [`Error::NoKeyExists`] - `delegator` does not have an MSA key.
		///
		#[pallet::call_index(10)]
		#[pallet::weight((T::WeightInfo::retire_msa(EXPECTED_MAX_NUMBER_OF_PROVIDERS_PER_DELEGATOR), DispatchClass::Normal, Pays::No))]
		pub fn retire_msa(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			// Check and get the account id from the origin
			let who = ensure_signed(origin)?;

			// Delete the last and only account key and deposit the "PublicKeyDeleted" event
			// check for valid MSA is in SignedExtension.
			let mut num_deletions: u32 = 0_u32;
			match Self::get_msa_by_public_key(&who) {
				Some(msa_id) => {
					num_deletions = Self::delete_delegation_relationship(DelegatorId(msa_id));
					Self::delete_key_for_msa(msa_id, &who)?;
					Self::deposit_event(Event::PublicKeyDeleted { key: who });
					Self::deposit_event(Event::MsaRetired { msa_id });
				},
				None => {
					log_err!("SignedExtension did not catch invalid MSA for account {:?}, ", who);
				},
			}
			Ok(Some(T::WeightInfo::retire_msa(num_deletions)).into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Create the account for the `key`
	///
	/// # Errors
	/// * [`Error::MsaIdOverflow`]
	/// * [`Error::KeyLimitExceeded`]
	/// * [`Error::KeyAlreadyRegistered`]
	///
	pub fn create_account<F>(
		key: T::AccountId,
		on_success: F,
	) -> Result<(MessageSourceId, T::AccountId), DispatchError>
	where
		F: FnOnce(MessageSourceId) -> DispatchResult,
	{
		let next_msa_id = Self::get_next_msa_id()?;
		Self::add_key(next_msa_id, &key, on_success)?;
		let _ = Self::set_msa_identifier(next_msa_id);

		Ok((next_msa_id, key))
	}

	/// Generate the next MSA Id
	///
	/// # Errors
	/// * [`Error::MsaIdOverflow`]
	///
	pub fn get_next_msa_id() -> Result<MessageSourceId, DispatchError> {
		let next = Self::get_current_msa_identifier_maximum()
			.checked_add(1)
			.ok_or(Error::<T>::MsaIdOverflow)?;

		Ok(next)
	}

	/// Set the current identifier in storage
	pub fn set_msa_identifier(identifier: MessageSourceId) -> DispatchResult {
		CurrentMsaIdentifierMaximum::<T>::set(identifier);

		Ok(())
	}

	/// Adds a list of schema permissions to a delegation relationship.
	pub fn grant_permissions_for_schemas(
		delegator_id: DelegatorId,
		provider_id: ProviderId,
		schema_ids: Vec<SchemaId>,
	) -> DispatchResult {
		Self::try_mutate_delegation(delegator_id, provider_id, |delegation, is_new_delegation| {
			ensure!(!is_new_delegation, Error::<T>::DelegationNotFound);
			Self::ensure_all_schema_ids_are_valid(&schema_ids)?;

			PermittedDelegationSchemas::<T>::try_insert_schemas(delegation, schema_ids)?;

			Ok(())
		})
	}

	/// Revokes a list of schema permissions from a delegation relationship.
	pub fn revoke_permissions_for_schemas(
		delegator_id: DelegatorId,
		provider_id: ProviderId,
		schema_ids: Vec<SchemaId>,
	) -> DispatchResult {
		Self::try_mutate_delegation(delegator_id, provider_id, |delegation, is_new_delegation| {
			ensure!(!is_new_delegation, Error::<T>::DelegationNotFound);
			Self::ensure_all_schema_ids_are_valid(&schema_ids)?;

			let current_block = frame_system::Pallet::<T>::block_number();

			PermittedDelegationSchemas::<T>::try_get_mut_schemas(
				delegation,
				schema_ids,
				current_block,
			)?;

			Ok(())
		})
	}

	/// Add a new key to the MSA
	///
	/// # Errors
	/// * [`Error::KeyLimitExceeded`]
	/// * [`Error::KeyAlreadyRegistered`]
	///
	pub fn add_key<F>(msa_id: MessageSourceId, key: &T::AccountId, on_success: F) -> DispatchResult
	where
		F: FnOnce(MessageSourceId) -> DispatchResult,
	{
		PublicKeyToMsaId::<T>::try_mutate(key, |maybe_msa_id| {
			ensure!(maybe_msa_id.is_none(), Error::<T>::KeyAlreadyRegistered);
			*maybe_msa_id = Some(msa_id);

			// Increment the key counter
			<PublicKeyCountForMsaId<T>>::try_mutate(msa_id, |key_count| {
				// key_count:u8 should default to 0 if it does not exist
				let incremented_key_count =
					key_count.checked_add(1).ok_or(ArithmeticError::Overflow)?;

				ensure!(
					incremented_key_count <= T::MaxPublicKeysPerMsa::get(),
					Error::<T>::KeyLimitExceeded
				);

				*key_count = incremented_key_count;
				on_success(msa_id)
			})
		})
	}

	/// Check that schema ids are all valid
	///
	/// # Errors
	/// * [`Error::InvalidSchemaId`]
	/// * [`Error::ExceedsMaxSchemaGrantsPerDelegation`]
	///
	pub fn ensure_all_schema_ids_are_valid(schema_ids: &Vec<SchemaId>) -> DispatchResult {
		ensure!(
			schema_ids.len() <= T::MaxSchemaGrantsPerDelegation::get() as usize,
			Error::<T>::ExceedsMaxSchemaGrantsPerDelegation
		);

		let are_schemas_valid = T::SchemaValidator::are_all_schema_ids_valid(schema_ids);

		ensure!(are_schemas_valid, Error::<T>::InvalidSchemaId);

		Ok(())
	}

	/// Returns if provider is registered by checking if the [`ProviderToRegistryEntry`] contains the MSA id
	pub fn is_registered_provider(msa_id: MessageSourceId) -> bool {
		ProviderToRegistryEntry::<T>::contains_key(ProviderId(msa_id))
	}

	/// Checks that a provider and delegator keys are valid
	/// and that a provider and delegator are not the same
	/// and that a provider has authorized a delegator to create a delegation relationship.
	///
	/// # Errors
	/// * [`Error::ProviderNotRegistered`]
	/// * [`Error::InvalidSelfProvider`]
	/// * [`Error::NoKeyExists`]
	///
	pub fn ensure_valid_registered_provider(
		delegator_key: &T::AccountId,
		provider_key: &T::AccountId,
	) -> Result<(ProviderId, DelegatorId), DispatchError> {
		let provider_msa_id = Self::ensure_valid_msa_key(provider_key)?;
		let delegator_msa_id = Self::ensure_valid_msa_key(delegator_key)?;

		// Ensure that the delegator is not the provider.  You cannot delegate to yourself.
		ensure!(delegator_msa_id != provider_msa_id, Error::<T>::InvalidSelfProvider);

		// Verify that the provider is a registered provider
		ensure!(Self::is_registered_provider(provider_msa_id), Error::<T>::ProviderNotRegistered);

		Ok((provider_msa_id.into(), delegator_msa_id.into()))
	}

	/// Checks that the MSA for `who` is the same as `msa_id`
	///
	/// # Errors
	/// * [`Error::NotMsaOwner`]
	/// * [`Error::NoKeyExists`]
	///
	pub fn ensure_msa_owner(who: &T::AccountId, msa_id: MessageSourceId) -> DispatchResult {
		let provider_msa_id = Self::ensure_valid_msa_key(who)?;
		ensure!(provider_msa_id == msa_id, Error::<T>::NotMsaOwner);

		Ok(())
	}

	/// Verify the `signature` was signed by `signer` on `payload` by a wallet
	/// Note the `wrap_binary_data` follows the Polkadot wallet pattern of wrapping with `<Byte>` tags.
	///
	/// # Errors
	/// * [`Error::InvalidSignature`]
	///
	pub fn verify_signature(
		signature: &MultiSignature,
		signer: &T::AccountId,
		payload: Vec<u8>,
	) -> DispatchResult {
		let key = T::ConvertIntoAccountId32::convert((*signer).clone());
		let wrapped_payload = wrap_binary_data(payload);

		ensure!(signature.verify(&wrapped_payload[..], &key), Error::<T>::InvalidSignature);

		Ok(())
	}

	/// Add a provider to a delegator with the default permissions
	///
	/// # Errors
	/// * [`Error::ExceedsMaxSchemaGrantsPerDelegation`]
	///
	pub fn add_provider(
		provider_id: ProviderId,
		delegator_id: DelegatorId,
		schema_ids: Vec<SchemaId>,
	) -> DispatchResult {
		Self::try_mutate_delegation(delegator_id, provider_id, |delegation, is_new_delegation| {
			ensure!(is_new_delegation, Error::<T>::DuplicateProvider);
			Self::ensure_all_schema_ids_are_valid(&schema_ids)?;

			PermittedDelegationSchemas::<T>::try_insert_schemas(delegation, schema_ids)?;

			Ok(())
		})
	}

	/// Mutates the delegation relationship storage item only when the supplied function returns an 'Ok()' result.
	/// The callback function 'f' takes the value (a delegation) and a reference to a boolean variable. This callback
	/// sets the boolean variable to 'true' if the value is to be inserted and to 'false' if it is to be updated.
	pub fn try_mutate_delegation<R, E: From<DispatchError>>(
		delegator_id: DelegatorId,
		provider_id: ProviderId,
		f: impl FnOnce(
			&mut Delegation<SchemaId, T::BlockNumber, T::MaxSchemaGrantsPerDelegation>,
			bool,
		) -> Result<R, E>,
	) -> Result<R, E> {
		DelegatorAndProviderToDelegation::<T>::try_mutate_exists(
			delegator_id,
			provider_id,
			|maybe_delegation_info| {
				let is_new = maybe_delegation_info.is_none();
				let mut delegation = maybe_delegation_info.take().unwrap_or_default();

				f(&mut delegation, is_new).map(move |result| {
					*maybe_delegation_info = Some(delegation);
					result
				})
			},
		)
	}

	/// Deletes a key associated with a given MSA
	///
	/// # Errors
	/// * [`Error::NoKeyExists`]
	///
	pub fn delete_key_for_msa(msa_id: MessageSourceId, key: &T::AccountId) -> DispatchResult {
		PublicKeyToMsaId::<T>::try_mutate_exists(key, |maybe_msa_id| {
			ensure!(maybe_msa_id.is_some(), Error::<T>::NoKeyExists);

			// Delete the key if it exists
			*maybe_msa_id = None;

			<PublicKeyCountForMsaId<T>>::try_mutate_exists(msa_id, |key_count| {
				match key_count {
					Some(1) => *key_count = None,
					Some(count) => *count = *count - 1u8,
					None => (),
				}

				Ok(())
			})
		})
	}

	/// Revoke the grant for permissions from the delegator to the provider
	///
	/// # Errors
	/// * [`Error::DelegationRevoked`]
	/// * [`Error::DelegationNotFound`]
	///
	pub fn revoke_provider(provider_id: ProviderId, delegator_id: DelegatorId) -> DispatchResult {
		DelegatorAndProviderToDelegation::<T>::try_mutate_exists(
			delegator_id,
			provider_id,
			|maybe_info| -> DispatchResult {
				let mut info = maybe_info.take().ok_or(Error::<T>::DelegationNotFound)?;

				ensure!(
					info.revoked_at == T::BlockNumber::default(),
					Error::<T>::DelegationRevoked
				);

				let current_block = frame_system::Pallet::<T>::block_number();
				info.revoked_at = current_block;
				*maybe_info = Some(info);
				Ok(())
			},
		)?;

		Ok(())
	}

	/// Removes delegations from the specified delegator MSA id to providers
	/// up to the expected number of providers.
	pub fn delete_delegation_relationship(delegator: DelegatorId) -> u32 {
		// TODO: Handle case when the number of providers exceeds the expected number.  Issue #678
		let result = DelegatorAndProviderToDelegation::<T>::clear_prefix(
			delegator,
			EXPECTED_MAX_NUMBER_OF_PROVIDERS_PER_DELEGATOR,
			None,
		);
		result.unique
	}

	/// Retrieves the MSA Id for a given `AccountId`
	pub fn get_owner_of(key: &T::AccountId) -> Option<MessageSourceId> {
		Self::get_msa_by_public_key(&key)
	}

	// *Temporarily Removed* until https://github.com/LibertyDSNP/frequency/issues/418
	//
	// Fetches all the keys associated with a message Source Account
	// NOTE: This should only be called from RPC due to heavy database reads
	// pub fn fetch_msa_keys(msa_id: MessageSourceId) -> Vec<KeyInfoResponse<T::AccountId>> {
	// 	let mut response = Vec::new();
	// 	for key in Self::get_msa_keys(msa_id) {
	// 		if let Ok(_info) = Self::ensure_valid_msa_key(&key) {
	// 			response.push(KeyInfoResponse { key, msa_id });
	// 		}
	// 	}

	// 	response
	// }

	/// Retrieve MSA Id associated with `key` or return `NoKeyExists`
	pub fn ensure_valid_msa_key(key: &T::AccountId) -> Result<MessageSourceId, DispatchError> {
		let msa_id = Self::get_msa_by_public_key(key).ok_or(Error::<T>::NoKeyExists)?;
		Ok(msa_id)
	}

	/// Get a list of Schema Ids that a provider has been granted access to
	///
	/// # Errors
	/// * [`Error::DelegationNotFound`]
	/// * [`Error::SchemaNotGranted`]
	///
	pub fn get_granted_schemas_by_msa_id(
		delegator: DelegatorId,
		provider: ProviderId,
	) -> Result<Option<Vec<SchemaId>>, DispatchError> {
		let provider_info =
			Self::get_delegation_of(delegator, provider).ok_or(Error::<T>::DelegationNotFound)?;

		let schema_permissions = provider_info.schema_permissions;
		if schema_permissions.is_empty() {
			return Err(Error::<T>::SchemaNotGranted.into())
		}

		let mut schema_list = Vec::new();
		for (key, _) in schema_permissions {
			schema_list.push(key);
		}
		Ok(Some(schema_list))
	}

	/// Adds a signature to the PayloadSignatureRegistry based on a virtual "bucket" grouping.
	/// Check that mortality_block is within bounds. If so, proceed and add the new entry.
	/// Raises `SignatureAlreadySubmitted` if the bucket-signature double key exists in the
	/// registry.
	///
	/// # Errors
	/// * [`Error::ProofNotYetValid`]
	/// * [`Error::ProofHasExpired`]
	/// * [`Error::SignatureAlreadySubmitted`]
	///
	pub fn register_signature(
		signature: &MultiSignature,
		signature_expires_at: T::BlockNumber,
	) -> DispatchResult {
		let current_block = frame_system::Pallet::<T>::block_number();

		let max_lifetime = Self::mortality_block_limit(current_block);
		if max_lifetime <= signature_expires_at {
			Err(Error::<T>::ProofNotYetValid.into())
		} else if current_block >= signature_expires_at {
			Err(Error::<T>::ProofHasExpired.into())
		} else {
			let bucket_num = Self::bucket_for(signature_expires_at.into());
			<PayloadSignatureBucketCount<T>>::try_mutate(
				bucket_num,
				|bucket_signature_count: &mut u32| -> DispatchResult {
					let limit = T::MaxSignaturesPerBucket::get();
					ensure!(
						*bucket_signature_count < limit,
						Error::<T>::SignatureRegistryLimitExceeded
					);
					let new_count =
						bucket_signature_count.checked_add(1).ok_or(ArithmeticError::Overflow)?;
					*bucket_signature_count = new_count;

					// Tests for noop fail with "storage has been mutated"
					// if this try_mutate does not live inside this block
					<PayloadSignatureRegistry<T>>::try_mutate(
						bucket_num,
						signature,
						|maybe_mortality_block| -> DispatchResult {
							ensure!(
								maybe_mortality_block.is_none(),
								Error::<T>::SignatureAlreadySubmitted
							);
							*maybe_mortality_block = Some(signature_expires_at);
							Ok(())
						},
					)?;
					Ok(())
				},
			)
		}
	}

	/// Check if enough blocks have passed to reset bucket mortality storage.
	/// If so:
	///     1. delete all the stored bucket/signature values with key1 = bucket num
	///     2. add the WeightInfo proportional to the storage read/writes to the block weight
	/// If not, don't do anything.
	///
	fn reset_virtual_bucket_if_needed(current_block: T::BlockNumber) -> Weight {
		let current_bucket_num = Self::bucket_for(current_block);
		let prior_bucket_num = Self::bucket_for(current_block - T::BlockNumber::one());

		// If we did not cross a bucket boundary block, stop
		if prior_bucket_num == current_bucket_num {
			return Weight::zero()
		}
		// Clear the previous bucket block set
		let multi_removal_result = <PayloadSignatureRegistry<T>>::clear_prefix(
			prior_bucket_num,
			T::MaxSignaturesPerBucket::get(),
			None,
		);
		<PayloadSignatureBucketCount<T>>::mutate(prior_bucket_num, |bucket_signature_count| {
			*bucket_signature_count = 0;
		});
		T::WeightInfo::on_initialize(multi_removal_result.unique)
	}

	/// The furthest in the future a mortality_block value is allowed
	/// to be for current_block
	/// This is calculated to be past the risk of a replay attack
	fn mortality_block_limit(current_block: T::BlockNumber) -> T::BlockNumber {
		let mortality_size = (T::NumberOfBuckets::get() - 1) * T::MortalityWindowSize::get();
		current_block + T::BlockNumber::from(mortality_size)
	}

	/// calculate the virtual bucket number for the provided block number
	pub fn bucket_for(block_number: T::BlockNumber) -> T::BlockNumber {
		block_number / (T::BlockNumber::from(T::MortalityWindowSize::get())) %
			T::BlockNumber::from(T::NumberOfBuckets::get())
	}
}

#[cfg(feature = "runtime-benchmarks")]
impl<T: Config> MsaBenchmarkHelper<T::AccountId> for Pallet<T> {
	/// adds delegation relationship with permitted schema ids
	fn set_delegation_relationship(
		provider: ProviderId,
		delegator: DelegatorId,
		schemas: Vec<SchemaId>,
	) -> DispatchResult {
		Self::add_provider(provider, delegator, schemas)?;
		Ok(())
	}

	/// adds a new key to specified msa
	fn add_key(msa_id: MessageSourceId, key: T::AccountId) -> DispatchResult {
		Self::add_key(msa_id, &key, EMPTY_FUNCTION)?;
		Ok(())
	}
}

impl<T: Config> MsaLookup for Pallet<T> {
	type AccountId = T::AccountId;

	fn get_msa_id(key: &Self::AccountId) -> Option<MessageSourceId> {
		Self::get_owner_of(key)
	}
}

impl<T: Config> MsaValidator for Pallet<T> {
	type AccountId = T::AccountId;

	fn ensure_valid_msa_key(key: &T::AccountId) -> Result<MessageSourceId, DispatchError> {
		Self::ensure_valid_msa_key(key)
	}
}

impl<T: Config> ProviderLookup for Pallet<T> {
	type BlockNumber = T::BlockNumber;
	type MaxSchemaGrantsPerDelegation = T::MaxSchemaGrantsPerDelegation;
	type SchemaId = SchemaId;

	fn get_delegation_of(
		delegator: DelegatorId,
		provider: ProviderId,
	) -> Option<Delegation<SchemaId, Self::BlockNumber, Self::MaxSchemaGrantsPerDelegation>> {
		Self::get_delegation(delegator, provider)
	}
}

impl<T: Config> DelegationValidator for Pallet<T> {
	type BlockNumber = T::BlockNumber;
	type MaxSchemaGrantsPerDelegation = T::MaxSchemaGrantsPerDelegation;
	type SchemaId = SchemaId;

	/// Check that the delegator has an active delegation to the provider.
	/// `block_number`: Provide `None` to know if the delegation is active at the current block.
	///                 Provide Some(N) to know if the delegation was or will be active at block N.
	///
	/// # Errors
	/// * [`Error::DelegationNotFound`]
	/// * [`Error::DelegationRevoked`]
	/// * [`Error::CannotPredictValidityPastCurrentBlock`]
	///
	fn ensure_valid_delegation(
		provider_id: ProviderId,
		delegator_id: DelegatorId,
		block_number: Option<T::BlockNumber>,
	) -> Result<Delegation<SchemaId, T::BlockNumber, T::MaxSchemaGrantsPerDelegation>, DispatchError>
	{
		let info = Self::get_delegation(delegator_id, provider_id)
			.ok_or(Error::<T>::DelegationNotFound)?;
		let current_block = frame_system::Pallet::<T>::block_number();
		let requested_block = match block_number {
			Some(block_number) => {
				ensure!(
					current_block >= block_number,
					Error::<T>::CannotPredictValidityPastCurrentBlock
				);
				block_number
			},
			None => current_block,
		};

		if info.revoked_at == T::BlockNumber::zero() {
			return Ok(info)
		}
		ensure!(info.revoked_at >= requested_block, Error::<T>::DelegationRevoked);

		Ok(info)
	}
}

impl<T: Config> SchemaGrantValidator<T::BlockNumber> for Pallet<T> {
	/// Check if provider is allowed to publish for a given schema_id for a given delegator
	///
	/// # Errors
	/// * [`Error::DelegationNotFound`]
	/// * [`Error::DelegationRevoked`]
	/// * [`Error::SchemaNotGranted`]
	/// * [`Error::CannotPredictValidityPastCurrentBlock`]
	///
	fn ensure_valid_schema_grant(
		provider: ProviderId,
		delegator: DelegatorId,
		schema_id: SchemaId,
		block_number: T::BlockNumber,
	) -> DispatchResult {
		let provider_info = Self::ensure_valid_delegation(provider, delegator, Some(block_number))?;

		let schema_permission_revoked_at_block_number = provider_info
			.schema_permissions
			.get(&schema_id)
			.ok_or(Error::<T>::SchemaNotGranted)?;

		if *schema_permission_revoked_at_block_number == T::BlockNumber::zero() {
			return Ok(())
		}

		ensure!(
			block_number <= *schema_permission_revoked_at_block_number,
			Error::<T>::SchemaNotGranted
		);

		Ok(())
	}
}

/// The SignedExtension trait is implemented on CheckFreeExtrinsicUse to validate that a provider
/// has not already been revoked if the calling extrinsic is revoking a provider to an MSA. The
/// purpose of this is to ensure that the revoke_delegation_by_delegator extrinsic cannot be
/// repeatedly called and flood the network.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CheckFreeExtrinsicUse<T: Config + Send + Sync>(PhantomData<T>);

impl<T: Config + Send + Sync> CheckFreeExtrinsicUse<T> {
	/// Validates the delegation by making sure that the MSA ids used are valid and the delegation is
	/// is still active. Returns a `ValidTransaction` or wrapped [`ValidityError`]
	/// # Arguments:
	/// * `account_id`: the account id of the delegator that is revoking the delegation relationship
	/// *  `provider_msa_id` the MSA ID of the provider (the "other end" of the delegation).
	///
	/// # Errors
	/// * [`ValidityError::InvalidMsaKey`] - if  `account_id` does not have an MSA
	/// * [`ValidityError::InvalidDelegation`] - if the delegation with `delegator_msa_id` is invalid
	///
	pub fn validate_delegation_by_delegator(
		account_id: &T::AccountId,
		provider_msa_id: &MessageSourceId,
	) -> TransactionValidity {
		const TAG_PREFIX: &str = "DelegatorDelegationRevocation";
		let delegator_msa_id: DelegatorId = Pallet::<T>::ensure_valid_msa_key(account_id)
			.map_err(|_| InvalidTransaction::Custom(ValidityError::InvalidMsaKey as u8))?
			.into();
		let provider_msa_id = ProviderId(*provider_msa_id);

		Pallet::<T>::ensure_valid_delegation(provider_msa_id, delegator_msa_id, None)
			.map_err(|_| InvalidTransaction::Custom(ValidityError::InvalidDelegation as u8))?;
		ValidTransaction::with_tag_prefix(TAG_PREFIX).and_provides(account_id).build()
	}

	/// Validates the delegation by making sure that the MSA ids used are valid and that the delegation
	/// is still active. Returns a `ValidTransaction` or wrapped [`ValidityError`]
	/// # Arguments:
	/// * `account_id`: the account id of the provider that is revoking the delegation relationship
	/// *  `delegator_msa_id` the MSA ID of the delegator (the "other end" of the delegation).
	///
	/// # Errors
	/// * [`ValidityError::InvalidMsaKey`] - if  `account_id` does not have an MSA
	/// * [`ValidityError::InvalidDelegation`] - if the delegation with `delegator_msa_id` is invalid
	///
	pub fn validate_delegation_by_provider(
		account_id: &T::AccountId,
		delegator_msa_id: &MessageSourceId,
	) -> TransactionValidity {
		const TAG_PREFIX: &str = "ProviderDelegationRevocation";

		let provider_msa_id: ProviderId = Pallet::<T>::ensure_valid_msa_key(account_id)
			.map_err(|_| InvalidTransaction::Custom(ValidityError::InvalidMsaKey as u8))?
			.into();
		let delegator_msa_id = DelegatorId(*delegator_msa_id);

		// Verify the delegation exists and is active
		Pallet::<T>::ensure_valid_delegation(provider_msa_id, delegator_msa_id, None)
			.map_err(|_| InvalidTransaction::Custom(ValidityError::InvalidDelegation as u8))?;
		ValidTransaction::with_tag_prefix(TAG_PREFIX).and_provides(account_id).build()
	}

	/// Validates that a key being revoked is both valid and owned by a valid MSA account.
	/// Returns a `ValidTransaction` or wrapped [`ValidityError::InvalidMsaKey`]
	/// Arguments:
	/// * `signing_public_key`: the account id calling for revoking the key, and which
	/// 	owns the msa also associated with `key`
	/// * `public_key_to_delete`: the account id to revoke as an access key for account_id's msa
	///
	/// # Errors
	/// * [`ValidityError::InvalidSelfRemoval`] - if `signing_public_key` and `public_key_to_delete` are the same.
	/// * [`ValidityError::InvalidMsaKey`] - if  `account_id` does not have an MSA or if
	/// 'public_key_to_delete' does not have an MSA.
	/// * [`ValidityError::NotKeyOwner`] - if the `signing_public_key` and `public_key_to_delete` do not belong to the same MSA ID.
	pub fn validate_key_delete(
		signing_public_key: &T::AccountId,
		public_key_to_delete: &T::AccountId,
	) -> TransactionValidity {
		const TAG_PREFIX: &str = "KeyRevocation";

		ensure!(
			signing_public_key != public_key_to_delete,
			InvalidTransaction::Custom(ValidityError::InvalidSelfRemoval as u8)
		);

		let maybe_owner_msa_id: MessageSourceId =
			Pallet::<T>::ensure_valid_msa_key(&signing_public_key)
				.map_err(|_| InvalidTransaction::Custom(ValidityError::InvalidMsaKey as u8))?;

		let msa_id_for_key_to_delete: MessageSourceId =
			Pallet::<T>::ensure_valid_msa_key(&public_key_to_delete)
				.map_err(|_| InvalidTransaction::Custom(ValidityError::InvalidMsaKey as u8))?;

		ensure!(
			maybe_owner_msa_id == msa_id_for_key_to_delete,
			InvalidTransaction::Custom(ValidityError::NotKeyOwner as u8)
		);

		return ValidTransaction::with_tag_prefix(TAG_PREFIX)
			.and_provides(signing_public_key)
			.build()
	}

	/// Validates that a MSA being retired exists, does not belong to a registered provider, and
	/// that `account_id` is the only access key associated with the MSA.
	/// Returns a `ValidTransaction` or wrapped [`ValidityError]
	/// # Arguments:
	/// * account_id: the account id associated with the MSA to retire
	///
	/// # Errors
	/// * [`ValidityError::InvalidMsaKey`]
	/// * [`ValidityError::InvalidRegisteredProviderCannotBeRetired`]
	/// * [`ValidityError::InvalidMoreThanOneKeyExists`]
	///
	pub fn ensure_msa_can_retire(account_id: &T::AccountId) -> TransactionValidity {
		const TAG_PREFIX: &str = "MSARetirement";
		let msa_id = Pallet::<T>::ensure_valid_msa_key(account_id)
			.map_err(|_| InvalidTransaction::Custom(ValidityError::InvalidMsaKey as u8))?
			.into();

		ensure!(
			!Pallet::<T>::is_registered_provider(msa_id),
			InvalidTransaction::Custom(
				ValidityError::InvalidRegisteredProviderCannotBeRetired as u8
			)
		);

		let key_count = Pallet::<T>::get_public_key_count_by_msa_id(msa_id);
		ensure!(
			key_count == 1,
			InvalidTransaction::Custom(ValidityError::InvalidMoreThanOneKeyExists as u8)
		);
		return ValidTransaction::with_tag_prefix(TAG_PREFIX).and_provides(account_id).build()
	}
}

/// Errors related to the validity of the CheckFreeExtrinsicUse signed extension.
pub enum ValidityError {
	/// Delegation to provider is not found or expired.
	InvalidDelegation,
	/// MSA key as been revoked.
	InvalidMsaKey,
	/// Cannot retire a registered provider MSA
	InvalidRegisteredProviderCannotBeRetired,
	/// More than one account key exists for the MSA during retire attempt
	InvalidMoreThanOneKeyExists,
	/// A transaction's Origin (AccountId) may not remove itself
	InvalidSelfRemoval,
	/// NotKeyOwner
	NotKeyOwner,
}

impl<T: Config + Send + Sync> CheckFreeExtrinsicUse<T> {
	/// Create new `SignedExtension` to check runtime version.
	pub fn new() -> Self {
		Self(sp_std::marker::PhantomData)
	}
}

impl<T: Config + Send + Sync> sp_std::fmt::Debug for CheckFreeExtrinsicUse<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "CheckFreeExtrinsicUse<{:?}>", self.0)
	}
	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl<T: Config + Send + Sync> SignedExtension for CheckFreeExtrinsicUse<T>
where
	T::RuntimeCall: Dispatchable<Info = DispatchInfo> + IsSubType<Call<T>>,
{
	type AccountId = T::AccountId;
	type Call = T::RuntimeCall;
	type AdditionalSigned = ();
	type Pre = ();
	const IDENTIFIER: &'static str = "CheckFreeExtrinsicUse";

	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		self.validate(who, call, info, len).map(|_| ())
	}

	/// Frequently called by the transaction queue to validate all free MSA extrinsics:
	/// Returns a `ValidTransaction` or wrapped [`ValidityError`]
	/// * revoke_delegation_by_provider
	/// * revoke_delegation_by_delegator
	/// * delete_msa_public_key
	/// * retire_msa
	/// Validate functions for the above MUST prevent errors in the extrinsic logic to prevent spam.
	///
	/// Arguments:
	/// who: AccountId calling the extrinsic
	/// call: The pallet extrinsic being called
	/// unused: _info, _len
	///
	fn validate(
		&self,
		who: &Self::AccountId,
		call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		match call.is_sub_type() {
			Some(Call::revoke_delegation_by_provider { delegator, .. }) =>
				CheckFreeExtrinsicUse::<T>::validate_delegation_by_provider(who, delegator),
			Some(Call::revoke_delegation_by_delegator { provider_msa_id, .. }) =>
				CheckFreeExtrinsicUse::<T>::validate_delegation_by_delegator(who, provider_msa_id),
			Some(Call::delete_msa_public_key { public_key_to_delete, .. }) =>
				CheckFreeExtrinsicUse::<T>::validate_key_delete(who, public_key_to_delete),
			Some(Call::retire_msa { .. }) => CheckFreeExtrinsicUse::<T>::ensure_msa_can_retire(who),
			_ => return Ok(Default::default()),
		}
	}
}
