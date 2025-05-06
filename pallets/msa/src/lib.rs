//! Message Source Account Management
//!
//! ## Quick Links
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Runtime API: `MsaRuntimeApi`](../pallet_msa_runtime_api/trait.MsaRuntimeApi.html)
//! - [Custom RPC API: `MsaApiServer`](../pallet_msa_rpc/trait.MsaApiServer.html)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
#![doc = include_str!("../README.md")]
//!
//! ## Implementations
//!
//! - [`MsaLookup`](../common_primitives/msa/trait.MsaLookup.html): Functions for accessing MSAs.
//! - [`MsaValidator`](../common_primitives/msa/trait.MsaValidator.html): Functions for validating MSAs.
//! - [`ProviderLookup`](../common_primitives/msa/trait.ProviderLookup.html): Functions for accessing Provider info.
//! - [`DelegationValidator`](../common_primitives/msa/trait.DelegationValidator.html): Functions for validating delegations.
//! - [`SchemaGrantValidator`](../common_primitives/msa/trait.SchemaGrantValidator.html): Functions for validating schema grants.
//!
//! ## Assumptions
//!
//! - Total MSA keys should be less than the constant `Config::MSA::MaxPublicKeysPerMsa`.
//! - Maximum schemas, for which any single provider has publishing rights on behalf of a single user, must be less than `Config::MSA::MaxSchemaGrantsPerDelegation`
#![allow(clippy::expect_used)]
#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

use frame_support::{
	dispatch::{DispatchInfo, DispatchResult, PostDispatchInfo},
	pallet_prelude::*,
	traits::IsSubType,
};
use parity_scale_codec::{Decode, Encode};

use common_runtime::signature::check_signature;

#[cfg(feature = "runtime-benchmarks")]
use common_primitives::benchmarks::{MsaBenchmarkHelper, RegisterProviderBenchmarkHelper};

use common_primitives::{
	capacity::TargetValidator,
	msa::{
		Delegation, DelegationValidator, DelegatorId, MsaLookup, MsaValidator, ProviderId,
		ProviderLookup, ProviderRegistryEntry, SchemaGrant, SchemaGrantValidator,
		SignatureRegistryPointer, H160,
	},
	node::ProposalProvider,
	schema::{SchemaId, SchemaValidator},
};
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_core::crypto::AccountId32;
use sp_io::hashing::keccak_256;
#[allow(deprecated)]
#[allow(unused)]
use sp_runtime::{
	traits::{
		BlockNumberProvider, Convert, DispatchInfoOf, DispatchOriginOf, Dispatchable,
		SignedExtension, TransactionExtension, ValidateResult, Zero,
	},
	ArithmeticError, DispatchError, MultiSignature,
};
extern crate alloc;
use alloc::{boxed::Box, vec, vec::Vec};

use common_primitives::msa::DelegationResponse;
pub use common_primitives::{
	handles::HandleProvider, msa::MessageSourceId, utils::wrap_binary_data,
};
pub use pallet::*;
pub use types::{AddKeyData, AddProvider, PermittedDelegationSchemas, EMPTY_FUNCTION};
pub use weights::*;

/// Offchain storage for MSA pallet
pub mod offchain_storage;
pub use offchain_storage::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod tests;

pub mod types;

pub mod weights;

#[frame_support::pallet]
pub mod pallet {
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
		type MaxSchemaGrantsPerDelegation: Get<u32>;

		/// Maximum provider name size allowed per MSA association
		#[pallet::constant]
		type MaxProviderNameSize: Get<u32>;

		/// A type that will supply schema related information.
		type SchemaValidator: SchemaValidator<SchemaId>;

		/// A type that will supply `Handle` related information.
		type HandleProvider: HandleProvider;

		/// The number of blocks before a signature can be ejected from the PayloadSignatureRegistryList
		#[pallet::constant]
		type MortalityWindowSize: Get<u32>;

		/// The maximum number of signatures that can be stored in PayloadSignatureRegistryList.
		#[pallet::constant]
		type MaxSignaturesStored: Get<Option<u32>>;

		/// The origin that is allowed to create providers via governance
		type CreateProviderViaGovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// The runtime call dispatch type.
		type Proposal: Parameter
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
			+ From<Call<Self>>;

		/// The Council proposal provider interface
		type ProposalProvider: ProposalProvider<Self::AccountId, Self::Proposal>;
	}

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// Storage type for the current MSA identifier maximum.
	/// We need to track this value because the identifier maximum
	/// is incremented each time a new identifier is created.
	/// - Value: The current maximum MSA Id
	#[pallet::storage]
	pub type CurrentMsaIdentifierMaximum<T> = StorageValue<_, MessageSourceId, ValueQuery>;

	/// Storage type for mapping the relationship between a Delegator and its Provider.
	/// - Keys: Delegator MSA, Provider MSA
	/// - Value: [`Delegation`](common_primitives::msa::Delegation)
	#[pallet::storage]
	pub type DelegatorAndProviderToDelegation<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		DelegatorId,
		Twox64Concat,
		ProviderId,
		Delegation<SchemaId, BlockNumberFor<T>, T::MaxSchemaGrantsPerDelegation>,
		OptionQuery,
	>;

	/// Provider registration information
	/// - Key: Provider MSA Id
	/// - Value: [`ProviderRegistryEntry`](common_primitives::msa::ProviderRegistryEntry)
	#[pallet::storage]
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
	pub type PublicKeyToMsaId<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, MessageSourceId, OptionQuery>;

	/// Storage type for a reference counter of the number of keys associated to an MSA
	/// - Key: MSA Id
	/// - Value: [`u8`] Counter of Keys associated with the MSA
	#[pallet::storage]
	pub(super) type PublicKeyCountForMsaId<T: Config> =
		StorageMap<_, Twox64Concat, MessageSourceId, u8, ValueQuery>;

	/// PayloadSignatureRegistryList is used to prevent replay attacks for extrinsics
	/// that take an externally-signed payload.
	/// For this to work, the payload must include a mortality block number, which
	/// is used in lieu of a monotonically increasing nonce.
	///
	/// The list is forwardly linked. (Example has a list size of 3)
	/// - signature, forward pointer -> n = new signature
	/// - 1,2 -> 2,3 (oldest)
	/// - 2,3 -> 3,4
	/// - 3,4 -> 4,5
	/// -   5 -> n (newest in pointer data)
	/// ### Storage
	/// - Key: Signature
	/// - Value: Tuple
	///     - `BlockNumber` when the keyed signature can be ejected from the registry
	///     - [`MultiSignature`] the forward linked list pointer. This pointer is the next "newest" value.
	#[pallet::storage]
	pub(super) type PayloadSignatureRegistryList<T: Config> = StorageMap<
		_,                                   // prefix
		Twox64Concat,                        // hasher for key1
		MultiSignature, // An externally-created Signature for an external payload, provided by an extrinsic
		(BlockNumberFor<T>, MultiSignature), // An actual flipping block number and the oldest signature at write time
		OptionQuery,                         // The type for the query
		GetDefault,                          // OnEmpty return type, defaults to None
		T::MaxSignaturesStored,              // Maximum total signatures to store
	>;

	/// This is the pointer for the Payload Signature Registry
	/// Contains the pointers to the data stored in the `PayloadSignatureRegistryList`
	/// - Value: [`SignatureRegistryPointer`]
	#[pallet::storage]
	pub(super) type PayloadSignatureRegistryPointer<T: Config> =
		StorageValue<_, SignatureRegistryPointer<BlockNumberFor<T>>>;

	/// A temporary storage for looking up relevant events from off-chain index
	/// At the start of the next block this storage is set to 0
	#[pallet::storage]
	#[pallet::whitelist_storage]
	pub(super) type OffchainIndexEventCount<T: Config> = StorageValue<_, u16, ValueQuery>;

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

		/// Attempted to add a new signature to a full signature registry
		SignatureRegistryLimitExceeded,

		/// Attempted to add a new signature to a corrupt signature registry
		SignatureRegistryCorrupted,
	}

	impl<T: Config> BlockNumberProvider for Pallet<T> {
		type BlockNumber = BlockNumberFor<T>;

		fn current_block_number() -> Self::BlockNumber {
			frame_system::Pallet::<T>::block_number()
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_block_number: BlockNumberFor<T>) -> Weight {
			<OffchainIndexEventCount<T>>::set(0u16);
			// allocates 1 read and 1 write for any access of `MSAEventCount` in every block
			T::DbWeight::get().reads_writes(1u64, 1u64)
		}

		fn offchain_worker(block_number: BlockNumberFor<T>) {
			log::info!("Running offchain workers! {:?}", block_number);
			do_offchain_worker::<T>(block_number)
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

			let event = Event::MsaCreated { msa_id: new_msa_id, key: new_public_key };
			offchain_index_event::<T>(Some(&event), new_msa_id);
			Self::deposit_event(event);
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

			ensure!(
				Self::verify_signature(&proof, &delegator_key, add_provider_payload.encode()),
				Error::<T>::InvalidSignature
			);

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

			let event =
				Event::MsaCreated { msa_id: new_delegator_msa_id, key: new_delegator_public_key };
			offchain_index_event::<T>(Some(&event), new_delegator_msa_id);
			Self::deposit_event(event);
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
		#[pallet::weight(T::WeightInfo::create_provider())]
		pub fn create_provider(origin: OriginFor<T>, provider_name: Vec<u8>) -> DispatchResult {
			let provider_key = ensure_signed(origin)?;
			let provider_msa_id = Self::ensure_valid_msa_key(&provider_key)?;
			Self::create_provider_for(provider_msa_id, provider_name)?;
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

			ensure!(
				Self::verify_signature(&proof, &delegator_key, add_provider_payload.encode()),
				Error::<T>::AddProviderSignatureVerificationFailed
			);

			Self::register_signature(&proof, add_provider_payload.expiration.into())?;
			let (provider_id, delegator_id) =
				Self::ensure_valid_registered_provider(&delegator_key, &provider_key)?;

			ensure!(
				add_provider_payload.authorized_msa_id == provider_id.0,
				Error::<T>::UnauthorizedDelegator
			);

			Self::upsert_schema_permissions(
				provider_id,
				delegator_id,
				add_provider_payload.schema_ids,
			)?;
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

			match PublicKeyToMsaId::<T>::get(&who) {
				Some(delegator_msa_id) => {
					let delegator_id = DelegatorId(delegator_msa_id);
					let provider_id = ProviderId(provider_msa_id);
					Self::revoke_provider(provider_id, delegator_id)?;
					Self::deposit_event(Event::DelegationRevoked { delegator_id, provider_id });
				},
				None => {
					log::error!(
						"SignedExtension did not catch invalid MSA for account {:?}, ",
						who
					);
				},
			}

			Ok(())
		}

		/// Adds a new public key to an existing Message Source Account (MSA). This functionality enables the MSA owner to manage multiple keys
		/// for their account or rotate keys for enhanced security.
		///
		/// The `origin` parameter represents the account from which the function is called and can be either the MSA owner's account or a delegated provider's account,
		/// depending on the intended use.
		///
		/// The function requires two signatures: `msa_owner_proof` and `new_key_owner_proof`, which serve as proofs of ownership for the existing MSA
		/// and the new public key account, respectively.  This ensures that a new key cannot be added without permission of both the MSA owner and the owner of the new key.
		///
		/// The necessary information for the key addition, the new public key and the MSA ID, is contained in the `add_key_payload` parameter of type [AddKeyData].
		/// It also contains an expiration block number for both proofs, ensuring they are valid and must be greater than the current block.
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

			ensure!(
				Self::verify_signature(
					&msa_owner_proof,
					&msa_owner_public_key,
					add_key_payload.encode()
				),
				Error::<T>::MsaOwnershipInvalidSignature
			);

			ensure!(
				Self::verify_signature(
					&new_key_owner_proof,
					&add_key_payload.new_public_key,
					add_key_payload.encode()
				),
				Error::<T>::NewKeyOwnershipInvalidSignature
			);

			Self::register_signature(&msa_owner_proof, add_key_payload.expiration)?;
			Self::register_signature(&new_key_owner_proof, add_key_payload.expiration)?;

			let msa_id = add_key_payload.msa_id;

			Self::ensure_msa_owner(&msa_owner_public_key, msa_id)?;

			Self::add_key(
				msa_id,
				&add_key_payload.new_public_key.clone(),
				|msa_id| -> DispatchResult {
					let event = Event::PublicKeyAdded {
						msa_id,
						key: add_key_payload.new_public_key.clone(),
					};
					offchain_index_event::<T>(Some(&event), msa_id);
					Self::deposit_event(event);
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

			match PublicKeyToMsaId::<T>::get(&who) {
				Some(who_msa_id) => {
					Self::delete_key_for_msa(who_msa_id, &public_key_to_delete)?;

					// Deposit the event
					let event = Event::PublicKeyDeleted { key: public_key_to_delete };
					offchain_index_event::<T>(Some(&event), who_msa_id);
					Self::deposit_event(event);
				},
				None => {
					log::error!(
						"SignedExtension did not catch invalid MSA for account {:?}, ",
						who
					);
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
			match PublicKeyToMsaId::<T>::get(&who) {
				Some(msa_id) => {
					let provider_id = ProviderId(msa_id);
					let delegator_id = DelegatorId(delegator);
					Self::revoke_provider(provider_id, delegator_id)?;
					Self::deposit_event(Event::DelegationRevoked { provider_id, delegator_id })
				},
				None => {
					log::error!(
						"SignedExtension did not catch invalid MSA for account {:?}, ",
						who
					);
				},
			}

			Ok(())
		}

		// REMOVED grant_schema_permissions() at call index 8
		// REMOVED revoke_schema_permissions() at call index 9

		/// Retires a MSA
		///
		/// When a user wants to disassociate themselves from Frequency, they can retire their MSA for free provided that:
		///  (1) They own the MSA
		///  (2) The MSA is not a registered provider.
		///  (3) They retire their user handle (if they have one)
		///  (4) There is only one account key
		///  (5) The user has already deleted all delegations to providers
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
		#[pallet::weight((T::WeightInfo::retire_msa(), DispatchClass::Normal, Pays::No))]
		pub fn retire_msa(origin: OriginFor<T>) -> DispatchResult {
			// Check and get the account id from the origin
			let who = ensure_signed(origin)?;

			// Delete the last and only account key and deposit the "PublicKeyDeleted" event
			// check for valid MSA is in SignedExtension.
			match PublicKeyToMsaId::<T>::get(&who) {
				Some(msa_id) => {
					Self::delete_key_for_msa(msa_id, &who)?;
					let event = Event::PublicKeyDeleted { key: who };
					offchain_index_event::<T>(Some(&event), msa_id);
					Self::deposit_event(event);
					Self::deposit_event(Event::MsaRetired { msa_id });
				},
				None => {
					log::error!(
						"SignedExtension did not catch invalid MSA for account {:?}, ",
						who
					);
				},
			}
			Ok(())
		}

		/// Propose to be a provider.  Creates a proposal for council approval to create a provider from a MSA
		///
		/// # Errors
		/// - [`NoKeyExists`](Error::NoKeyExists) - If there is not MSA for `origin`.
		#[pallet::call_index(11)]
		#[pallet::weight(T::WeightInfo::propose_to_be_provider())]
		pub fn propose_to_be_provider(
			origin: OriginFor<T>,
			provider_name: Vec<u8>,
		) -> DispatchResult {
			let bounded_name: BoundedVec<u8, T::MaxProviderNameSize> =
				provider_name.try_into().map_err(|_| Error::<T>::ExceedsMaxProviderNameSize)?;

			let proposer = ensure_signed(origin)?;
			Self::ensure_valid_msa_key(&proposer)?;

			let proposal: Box<T::Proposal> = Box::new(
				(Call::<T>::create_provider_via_governance {
					provider_key: proposer.clone(),
					provider_name: bounded_name.into(),
				})
				.into(),
			);
			let threshold = 1;
			T::ProposalProvider::propose(proposer, threshold, proposal)?;
			Ok(())
		}

		/// Create a provider by means of governance approval
		///
		/// # Events
		/// * [`Event::ProviderCreated`]
		///
		/// # Errors
		/// * [`Error::NoKeyExists`] - account does not have an MSA
		/// * [`Error::ExceedsMaxProviderNameSize`] - Too long of a provider name
		/// * [`Error::DuplicateProviderRegistryEntry`] - a ProviderRegistryEntry associated with the given MSA id already exists.
		#[pallet::call_index(12)]
		#[pallet::weight(T::WeightInfo::create_provider_via_governance())]
		pub fn create_provider_via_governance(
			origin: OriginFor<T>,
			provider_key: T::AccountId,
			provider_name: Vec<u8>,
		) -> DispatchResult {
			T::CreateProviderViaGovernanceOrigin::ensure_origin(origin)?;
			let provider_msa_id = Self::ensure_valid_msa_key(&provider_key)?;
			Self::create_provider_for(provider_msa_id, provider_name)?;
			Self::deposit_event(Event::ProviderCreated {
				provider_id: ProviderId(provider_msa_id),
			});
			Ok(())
		}

		/// A generic endpoint to replay any offchain related event to fix any potential issues
		#[pallet::call_index(13)]
		#[pallet::weight(T::WeightInfo::reindex_offchain())]
		pub fn reindex_offchain(
			origin: OriginFor<T>,
			event: OffchainReplayEvent<T>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			match event {
				OffchainReplayEvent::MsaPallet(MsaOffchainReplayEvent::KeyReIndex {
					msa_id,
					index_key,
				}) => {
					// don't need to check existence of msa_id since it would get checked on offchain side
					match index_key {
						Some(key) => {
							let event = Event::PublicKeyAdded { msa_id, key };
							offchain_index_event::<T>(Some(&event), msa_id);
						},
						None => {
							offchain_index_event::<T>(None, msa_id);
						},
					}
				},
			}

			Ok(())
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
		let next = CurrentMsaIdentifierMaximum::<T>::get()
			.checked_add(1)
			.ok_or(Error::<T>::MsaIdOverflow)?;

		Ok(next)
	}

	/// Set the current identifier in storage
	pub fn set_msa_identifier(identifier: MessageSourceId) -> DispatchResult {
		CurrentMsaIdentifierMaximum::<T>::set(identifier);

		Ok(())
	}

	/// Create Register Provider
	pub fn create_registered_provider(
		provider_id: ProviderId,
		name: BoundedVec<u8, T::MaxProviderNameSize>,
	) -> DispatchResult {
		ProviderToRegistryEntry::<T>::try_mutate(provider_id, |maybe_metadata| -> DispatchResult {
			ensure!(maybe_metadata.take().is_none(), Error::<T>::DuplicateProviderRegistryEntry);
			*maybe_metadata = Some(ProviderRegistryEntry { provider_name: name });
			Ok(())
		})
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
	pub fn ensure_all_schema_ids_are_valid(schema_ids: &[SchemaId]) -> DispatchResult {
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
	) -> bool {
		let key = T::ConvertIntoAccountId32::convert((*signer).clone());

		check_signature(signature, key, payload)
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

	/// Modify delegation's schema permissions
	///
	/// # Errors
	/// * [`Error::ExceedsMaxSchemaGrantsPerDelegation`]
	pub fn upsert_schema_permissions(
		provider_id: ProviderId,
		delegator_id: DelegatorId,
		schema_ids: Vec<SchemaId>,
	) -> DispatchResult {
		Self::try_mutate_delegation(delegator_id, provider_id, |delegation, _is_new_delegation| {
			Self::ensure_all_schema_ids_are_valid(&schema_ids)?;

			// Create revoke and insert lists
			let mut revoke_ids: Vec<SchemaId> = Vec::new();
			let mut update_ids: Vec<SchemaId> = Vec::new();
			let mut insert_ids: Vec<SchemaId> = Vec::new();

			let existing_keys = delegation.schema_permissions.keys();

			for existing_schema_id in existing_keys {
				if !schema_ids.contains(existing_schema_id) {
					if let Some(block) = delegation.schema_permissions.get(existing_schema_id) {
						if *block == BlockNumberFor::<T>::zero() {
							revoke_ids.push(*existing_schema_id);
						}
					}
				}
			}
			for schema_id in &schema_ids {
				if !delegation.schema_permissions.contains_key(schema_id) {
					insert_ids.push(*schema_id);
				} else {
					update_ids.push(*schema_id);
				}
			}

			let current_block = frame_system::Pallet::<T>::block_number();

			// Revoke any that are not in the new list that are not already revoked
			PermittedDelegationSchemas::<T>::try_get_mut_schemas(
				delegation,
				revoke_ids,
				current_block,
			)?;

			// Update any that are in the list but are not new
			PermittedDelegationSchemas::<T>::try_get_mut_schemas(
				delegation,
				update_ids,
				BlockNumberFor::<T>::zero(),
			)?;

			// Insert any new ones that are not in the existing list
			PermittedDelegationSchemas::<T>::try_insert_schemas(delegation, insert_ids)?;
			delegation.revoked_at = BlockNumberFor::<T>::zero();
			Ok(())
		})
	}

	/// Adds an association between MSA id and ProviderRegistryEntry. As of now, the
	/// only piece of metadata we are recording is provider name.
	///
	/// # Events
	/// * [`Event::ProviderCreated`]
	///
	/// # Errors
	/// * [`Error::NoKeyExists`] - account does not have an MSA
	/// * [`Error::ExceedsMaxProviderNameSize`] - Too long of a provider name
	/// * [`Error::DuplicateProviderRegistryEntry`] - a ProviderRegistryEntry associated with the given MSA id already exists.
	///
	pub fn create_provider_for(
		provider_msa_id: MessageSourceId,
		provider_name: Vec<u8>,
	) -> DispatchResult {
		let bounded_name: BoundedVec<u8, T::MaxProviderNameSize> =
			provider_name.try_into().map_err(|_| Error::<T>::ExceedsMaxProviderNameSize)?;

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
		Ok(())
	}

	/// Mutates the delegation relationship storage item only when the supplied function returns an 'Ok()' result.
	/// The callback function 'f' takes the value (a delegation) and a reference to a boolean variable. This callback
	/// sets the boolean variable to 'true' if the value is to be inserted and to 'false' if it is to be updated.
	pub fn try_mutate_delegation<R, E: From<DispatchError>>(
		delegator_id: DelegatorId,
		provider_id: ProviderId,
		f: impl FnOnce(
			&mut Delegation<SchemaId, BlockNumberFor<T>, T::MaxSchemaGrantsPerDelegation>,
			bool,
		) -> Result<R, E>,
	) -> Result<R, E> {
		DelegatorAndProviderToDelegation::<T>::try_mutate_exists(
			delegator_id,
			provider_id,
			|maybe_delegation_info| {
				let is_new = maybe_delegation_info.is_none();
				let mut delegation = maybe_delegation_info.take().unwrap_or_default();

				let result = f(&mut delegation, is_new)?;

				// only set the value if execution of 'f' is successful
				*maybe_delegation_info = Some(delegation);
				Ok(result)
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
					info.revoked_at == BlockNumberFor::<T>::default(),
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

	/// Retrieves the MSA Id for a given `AccountId`
	pub fn get_owner_of(key: &T::AccountId) -> Option<MessageSourceId> {
		PublicKeyToMsaId::<T>::get(key)
	}

	/// Retrieve MSA Id associated with `key` or return `NoKeyExists`
	pub fn ensure_valid_msa_key(key: &T::AccountId) -> Result<MessageSourceId, DispatchError> {
		let msa_id = PublicKeyToMsaId::<T>::get(key).ok_or(Error::<T>::NoKeyExists)?;
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
		provider: Option<ProviderId>,
	) -> Result<Vec<DelegationResponse<SchemaId, BlockNumberFor<T>>>, DispatchError> {
		let delegations = match provider {
			Some(provider_id) => vec![(
				provider_id,
				Self::get_delegation_of(delegator, provider_id)
					.ok_or(Error::<T>::DelegationNotFound)?,
			)],
			None => DelegatorAndProviderToDelegation::<T>::iter_prefix(delegator).collect(),
		};

		let mut result = vec![];
		for (provider_id, provider_info) in delegations {
			let schema_permissions = provider_info.schema_permissions;
			// checking only if this is called for a specific provider
			if provider.is_some() && schema_permissions.is_empty() {
				return Err(Error::<T>::SchemaNotGranted.into());
			}

			let mut schema_list = Vec::new();
			for (schema_id, revoked_at) in schema_permissions {
				if provider_info.revoked_at > BlockNumberFor::<T>::zero() &&
					(revoked_at > provider_info.revoked_at ||
						revoked_at == BlockNumberFor::<T>::zero())
				{
					schema_list
						.push(SchemaGrant { schema_id, revoked_at: provider_info.revoked_at });
				} else {
					schema_list.push(SchemaGrant { schema_id, revoked_at });
				}
			}

			result.push(DelegationResponse { provider_id, permissions: schema_list });
		}

		Ok(result)
	}

	/// Converts an MSA ID into a synthetic Ethereum address (raw 20-byte format) by
	/// taking the last 20 bytes of the keccak256 hash of the following:
	/// [0..1]: 0xD9 (first byte of the keccak256 hash of the domain prefix "Frequency")
	/// [1..9]:   u64 (big endian)
	/// [9..41]:  keccack256("MSA Generated")
	pub fn msa_id_to_eth_address(id: MessageSourceId) -> H160 {
		/// First byte of the keccak256 hash of the domain prefix "Frequency"
		/// This "domain separator" ensures that the generated address will not collide with Ethereum addresses
		/// generated by the standard 'CREATE2' opcode.
		const DOMAIN_PREFIX: u8 = 0xD9;
		/// keccak256 of "MSA Generated"
		const SALT: [u8; 32] = [
			0xb5, 0x4c, 0x54, 0x37, 0xca, 0x08, 0x73, 0xec, 0x72, 0x67, 0xa7, 0xe9, 0x37, 0x10,
			0xfe, 0x38, 0xe6, 0xab, 0xbe, 0x95, 0xea, 0x5e, 0x58, 0x1a, 0xc4, 0x86, 0x82, 0x29,
			0x1e, 0x61, 0x2e, 0x0d,
		];
		let input_value = id.to_be_bytes();

		let mut hash_input = [0u8; 41];
		hash_input[0] = DOMAIN_PREFIX;
		hash_input[1..9].copy_from_slice(&input_value);
		hash_input[9..].copy_from_slice(&SALT);

		let hash = keccak_256(&hash_input);
		H160::from_slice(&hash[12..])
	}

	/// Returns a boolean indicating whether the given Ethereum address was generated from the given MSA ID.
	pub fn validate_eth_address_for_msa(address: &H160, msa_id: MessageSourceId) -> bool {
		let generated_address = Self::msa_id_to_eth_address(msa_id);
		*address == generated_address
	}

	/// Converts a 20-byte synthetic Ethereum address into a checksummed string format,
	/// using ERC-55 checksum rules.
	/// Formats a 20-byte address into an EIP-55 checksummed `0x...` string.
	pub fn eth_address_to_checksummed_string(address: &H160) -> alloc::string::String {
		let addr_bytes = address.0;
		let addr_hex = hex::encode(addr_bytes);
		let hash = keccak_256(addr_hex.as_bytes());

		let mut result = alloc::string::String::with_capacity(42);
		result.push_str("0x");

		for (i, c) in addr_hex.chars().enumerate() {
			let hash_byte = hash[i / 2];
			let bit = if i % 2 == 0 { (hash_byte >> 4) & 0xf } else { hash_byte & 0xf };

			result.push(if c.is_ascii_hexdigit() && c.is_ascii_alphabetic() {
				if bit >= 8 {
					c.to_ascii_uppercase()
				} else {
					c
				}
			} else {
				c
			});
		}

		result
	}

	/// Adds a signature to the `PayloadSignatureRegistryList`
	/// Check that mortality_block is within bounds. If so, proceed and add the new entry.
	/// Raises `SignatureAlreadySubmitted` if the signature exists in the registry.
	/// Raises `SignatureRegistryLimitExceeded` if the oldest signature of the list has not yet expired.
	///
	/// Example list:
	/// - `1,2 (oldest)`
	/// - `2,3`
	/// - `3,4`
	/// - 4 (newest in pointer storage)`
	///
	/// # Errors
	/// * [`Error::ProofNotYetValid`]
	/// * [`Error::ProofHasExpired`]
	/// * [`Error::SignatureAlreadySubmitted`]
	/// * [`Error::SignatureRegistryLimitExceeded`]
	///
	pub fn register_signature(
		signature: &MultiSignature,
		signature_expires_at: BlockNumberFor<T>,
	) -> DispatchResult {
		let current_block = frame_system::Pallet::<T>::block_number();

		let max_lifetime = Self::mortality_block_limit(current_block);
		ensure!(max_lifetime > signature_expires_at, Error::<T>::ProofNotYetValid);
		ensure!(current_block < signature_expires_at, Error::<T>::ProofHasExpired);

		// Make sure it is not in the registry
		ensure!(
			!<PayloadSignatureRegistryList<T>>::contains_key(signature),
			Error::<T>::SignatureAlreadySubmitted
		);

		Self::enqueue_signature(signature, signature_expires_at, current_block)
	}

	/// Do the actual enqueuing into the list storage and update the pointer
	fn enqueue_signature(
		signature: &MultiSignature,
		signature_expires_at: BlockNumberFor<T>,
		current_block: BlockNumberFor<T>,
	) -> DispatchResult {
		// Get the current pointer, or if this is the initialization, generate an empty pointer
		let pointer =
			PayloadSignatureRegistryPointer::<T>::get().unwrap_or(SignatureRegistryPointer {
				newest: signature.clone(),
				newest_expires_at: signature_expires_at,
				oldest: signature.clone(),
				count: 0,
			});

		// Make sure it is not sitting as the `newest` in the pointer
		ensure!(
			!(pointer.count != 0 && pointer.newest.eq(signature)),
			Error::<T>::SignatureAlreadySubmitted
		);

		// Default to the current oldest signature in case we are still filling up
		let mut oldest: MultiSignature = pointer.oldest.clone();

		// We are now wanting to overwrite prior signatures
		let is_registry_full: bool = pointer.count == T::MaxSignaturesStored::get().unwrap_or(0);

		// Maybe remove the oldest signature and update the oldest
		if is_registry_full {
			let (expire_block_number, next_oldest) =
				PayloadSignatureRegistryList::<T>::get(pointer.oldest.clone())
					.ok_or(Error::<T>::SignatureRegistryCorrupted)?;

			ensure!(
				current_block.gt(&expire_block_number),
				Error::<T>::SignatureRegistryLimitExceeded
			);

			// Move the oldest in the list to the next oldest signature
			oldest = next_oldest.clone();

			<PayloadSignatureRegistryList<T>>::remove(pointer.oldest);
		}

		// Add the newest signature if we are not the first
		if pointer.count != 0 {
			<PayloadSignatureRegistryList<T>>::insert(
				pointer.newest,
				(pointer.newest_expires_at, signature.clone()),
			);
		}

		// Update the pointer.newest to have the signature that just came in
		PayloadSignatureRegistryPointer::<T>::put(SignatureRegistryPointer {
			// The count doesn't change if list is full
			count: if is_registry_full { pointer.count } else { pointer.count + 1 },
			newest: signature.clone(),
			newest_expires_at: signature_expires_at,
			oldest,
		});

		Ok(())
	}

	/// The furthest in the future a mortality_block value is allowed
	/// to be for current_block
	/// This is calculated to be past the risk of a replay attack
	fn mortality_block_limit(current_block: BlockNumberFor<T>) -> BlockNumberFor<T> {
		let mortality_size = T::MortalityWindowSize::get();
		current_block + BlockNumberFor::<T>::from(mortality_size)
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

#[cfg(feature = "runtime-benchmarks")]
impl<T: Config> RegisterProviderBenchmarkHelper for Pallet<T> {
	/// Create a registered provider for benchmarks
	fn create(provider_id: MessageSourceId, name: Vec<u8>) -> DispatchResult {
		let name = BoundedVec::<u8, T::MaxProviderNameSize>::try_from(name).expect("error");
		Self::create_registered_provider(provider_id.into(), name)?;

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
	type BlockNumber = BlockNumberFor<T>;
	type MaxSchemaGrantsPerDelegation = T::MaxSchemaGrantsPerDelegation;
	type SchemaId = SchemaId;

	fn get_delegation_of(
		delegator: DelegatorId,
		provider: ProviderId,
	) -> Option<Delegation<SchemaId, Self::BlockNumber, Self::MaxSchemaGrantsPerDelegation>> {
		DelegatorAndProviderToDelegation::<T>::get(delegator, provider)
	}
}

impl<T: Config> DelegationValidator for Pallet<T> {
	type BlockNumber = BlockNumberFor<T>;
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
		block_number: Option<BlockNumberFor<T>>,
	) -> Result<
		Delegation<SchemaId, BlockNumberFor<T>, T::MaxSchemaGrantsPerDelegation>,
		DispatchError,
	> {
		let info = DelegatorAndProviderToDelegation::<T>::get(delegator_id, provider_id)
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

		if info.revoked_at == BlockNumberFor::<T>::zero() {
			return Ok(info);
		}
		ensure!(info.revoked_at >= requested_block, Error::<T>::DelegationRevoked);

		Ok(info)
	}
}

impl<T: Config> TargetValidator for Pallet<T> {
	fn validate(target: MessageSourceId) -> bool {
		Self::is_registered_provider(target)
	}
}

impl<T: Config> SchemaGrantValidator<BlockNumberFor<T>> for Pallet<T> {
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
		block_number: BlockNumberFor<T>,
	) -> DispatchResult {
		let provider_info = Self::ensure_valid_delegation(provider, delegator, Some(block_number))?;

		let schema_permission_revoked_at_block_number = provider_info
			.schema_permissions
			.get(&schema_id)
			.ok_or(Error::<T>::SchemaNotGranted)?;

		if *schema_permission_revoked_at_block_number == BlockNumberFor::<T>::zero() {
			return Ok(());
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
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo)]
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
	///     owns the msa also associated with `key`
	/// * `public_key_to_delete`: the account id to revoke as an access key for account_id's msa
	///
	/// # Errors
	/// * [`ValidityError::InvalidSelfRemoval`] - if `signing_public_key` and `public_key_to_delete` are the same.
	/// * [`ValidityError::InvalidMsaKey`] - if  `account_id` does not have an MSA or if
	///     'public_key_to_delete' does not have an MSA.
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
			Pallet::<T>::ensure_valid_msa_key(signing_public_key)
				.map_err(|_| InvalidTransaction::Custom(ValidityError::InvalidMsaKey as u8))?;

		let msa_id_for_key_to_delete: MessageSourceId =
			Pallet::<T>::ensure_valid_msa_key(public_key_to_delete)
				.map_err(|_| InvalidTransaction::Custom(ValidityError::InvalidMsaKey as u8))?;

		ensure!(
			maybe_owner_msa_id == msa_id_for_key_to_delete,
			InvalidTransaction::Custom(ValidityError::NotKeyOwner as u8)
		);

		ValidTransaction::with_tag_prefix(TAG_PREFIX)
			.and_provides(signing_public_key)
			.build()
	}

	/// Validates that a MSA being retired exists, does not belong to a registered provider,
	/// that `account_id` is the only access key associated with the MSA,
	/// and that there are no delegations to providers.
	/// Returns a `ValidTransaction` or wrapped [`ValidityError]
	/// # Arguments:
	/// * account_id: the account id associated with the MSA to retire
	///
	/// # Errors
	/// * [`ValidityError::InvalidMsaKey`]
	/// * [`ValidityError::InvalidRegisteredProviderCannotBeRetired`]
	/// * [`ValidityError::InvalidMoreThanOneKeyExists`]
	/// * [`ValidityError::InvalidNonZeroProviderDelegations`]
	///
	pub fn ensure_msa_can_retire(account_id: &T::AccountId) -> TransactionValidity {
		const TAG_PREFIX: &str = "MSARetirement";
		let msa_id = Pallet::<T>::ensure_valid_msa_key(account_id)
			.map_err(|_| InvalidTransaction::Custom(ValidityError::InvalidMsaKey as u8))?;

		ensure!(
			!Pallet::<T>::is_registered_provider(msa_id),
			InvalidTransaction::Custom(
				ValidityError::InvalidRegisteredProviderCannotBeRetired as u8
			)
		);

		let msa_handle = T::HandleProvider::get_handle_for_msa(msa_id);
		ensure!(
			msa_handle.is_none(),
			InvalidTransaction::Custom(ValidityError::HandleNotRetired as u8)
		);

		let key_count = PublicKeyCountForMsaId::<T>::get(msa_id);
		ensure!(
			key_count == 1,
			InvalidTransaction::Custom(ValidityError::InvalidMoreThanOneKeyExists as u8)
		);

		let delegator_id = DelegatorId(msa_id);
		let has_active_delegations: bool = DelegatorAndProviderToDelegation::<T>::iter_key_prefix(
			delegator_id,
		)
		.any(|provider_id| {
			Pallet::<T>::ensure_valid_delegation(provider_id, delegator_id, None).is_ok()
		});

		ensure!(
			!has_active_delegations,
			InvalidTransaction::Custom(ValidityError::InvalidNonZeroProviderDelegations as u8)
		);

		ValidTransaction::with_tag_prefix(TAG_PREFIX).and_provides(account_id).build()
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
	/// InvalidNonZeroProviderDelegations
	InvalidNonZeroProviderDelegations,
	/// HandleNotRetired
	HandleNotRetired,
}

impl<T: Config + Send + Sync> CheckFreeExtrinsicUse<T> {
	/// Create new `SignedExtension` to check runtime version.
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<T: Config + Send + Sync> core::fmt::Debug for CheckFreeExtrinsicUse<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		write!(f, "CheckFreeExtrinsicUse<{:?}>", self.0)
	}
	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut core::fmt::Formatter) -> core::fmt::Result {
		Ok(())
	}
}

#[allow(deprecated)]
impl<T: Config + Send + Sync> SignedExtension for CheckFreeExtrinsicUse<T>
where
	T::RuntimeCall: Dispatchable<Info = DispatchInfo> + IsSubType<Call<T>>,
{
	type AccountId = T::AccountId;
	type Call = T::RuntimeCall;
	type AdditionalSigned = ();
	type Pre = ();
	const IDENTIFIER: &'static str = "CheckFreeExtrinsicUse";

	fn additional_signed(&self) -> core::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	#[allow(deprecated)]
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
	///
	/// Validate functions for the above MUST prevent errors in the extrinsic logic to prevent spam.
	///
	/// Arguments:
	/// who: AccountId calling the extrinsic
	/// call: The pallet extrinsic being called
	/// unused: _info, _len
	///
	#[allow(deprecated)]
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
			_ => Ok(Default::default()),
		}
	}
}
