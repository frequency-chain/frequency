//! Unique human-readable strings for MSAs
//!
//! ## Quick Links
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Runtime API: `HandlesRuntimeApi`](../pallet_handles_runtime_api/trait.HandlesRuntimeApi.html)
//! - [Custom RPC API: `HandlesApiServer`](../pallet_handles_rpc/trait.HandlesApiServer.html)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
#![doc = include_str!("../README.md")]
//!
//! ## Shuffled Sequences
//!
//! To limit the human value of low or particular suffixes, the pallet uses a shuffled sequence to choose a semi-randon suffix for a specific canonical handle string.
//! The shuffle only requires storage of the current index, same as an ordered suffix system.
//! The computational cost is greater the larger the index grows as it is lazy evaluation of the Rand32 given a deterministic seed generated from the canonical handle string.
//! See more details at [`handles_utils::suffix`]
//!
// Substrate macros are tripping the clippy::expect_used lint.
#![allow(clippy::expect_used)]
#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]
extern crate alloc;

use alloc::{string::String, vec};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

use handles_utils::{
	converter::{convert_to_canonical, split_display_name, HANDLE_DELIMITER},
	suffix::generate_unique_suffixes,
	validator::{
		consists_of_supported_unicode_character_sets, contains_blocked_characters,
		is_reserved_canonical_handle,
	},
};

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
use common_primitives::benchmarks::MsaBenchmarkHelper;
use common_primitives::{
	handles::*,
	msa::{MessageSourceId, MsaLookup, MsaValidator},
	utils::wrap_binary_data,
};
use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
use numtoa::*;
pub use pallet::*;
use sp_core::crypto::AccountId32;
use sp_runtime::{
	traits::{Convert, Verify},
	DispatchError, MultiSignature,
};
use sp_std::{prelude::*, vec::Vec};

pub mod handles_signed_extension;

pub mod weights;
pub use weights::*;

impl<T: Config> HandleProvider for Pallet<T> {
	fn get_handle_for_msa(msa_id: MessageSourceId) -> Option<HandleResponse> {
		Self::get_handle_for_msa(msa_id)
	}
}

#[frame_support::pallet]
pub mod pallet {

	use handles_utils::trim_and_collapse_whitespace;

	use super::*;
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// AccountId truncated to 32 bytes
		type ConvertIntoAccountId32: Convert<Self::AccountId, AccountId32>;

		/// A type that will supply MSA related information
		type MsaInfoProvider: MsaLookup + MsaValidator<AccountId = Self::AccountId>;

		/// The minimum suffix value
		#[pallet::constant]
		type HandleSuffixMin: Get<SuffixRangeType>;

		/// The maximum suffix value
		#[pallet::constant]
		type HandleSuffixMax: Get<SuffixRangeType>;

		/// The number of blocks before a signature can be ejected from the PayloadSignatureRegistryList
		#[pallet::constant]
		type MortalityWindowSize: Get<u32>;

		#[cfg(feature = "runtime-benchmarks")]
		/// A set of helper functions for benchmarking.
		type MsaBenchmarkHelper: MsaBenchmarkHelper<Self::AccountId>;
	}

	// Storage
	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// - Keys: k1: `CanonicalBase`, k2: `HandleSuffix`
	/// - Value: `MessageSourceId`
	#[pallet::storage]
	pub type CanonicalBaseHandleAndSuffixToMSAId<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		CanonicalBase,
		Twox64Concat,
		HandleSuffix,
		MessageSourceId,
		OptionQuery,
	>;

	/// - Key: `MessageSourceId`
	/// - Value: `DisplayHandle`
	#[pallet::storage]
	pub type MSAIdToDisplayName<T: Config> = StorageMap<
		_,
		Twox64Concat,
		MessageSourceId,
		(DisplayHandle, BlockNumberFor<T>),
		OptionQuery,
	>;

	/// - Key: `CanonicalBase`
	/// - Value: (Sequence Index, Suffix Min)
	/// - Sequence Index: The index of the next suffix to be used for this handle
	/// - Suffix Min: The minimum suffix value for this handle
	#[pallet::storage]
	pub type CanonicalBaseHandleToSuffixIndex<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		CanonicalBase,
		(SequenceIndex, SuffixRangeType),
		OptionQuery,
	>;

	#[derive(PartialEq, Eq)] // for testing
	#[pallet::error]
	pub enum Error<T> {
		/// Invalid handle encoding (should be UTF-8)
		InvalidHandleEncoding,
		/// Invalid handle byte length
		InvalidHandleByteLength,
		/// Invalid handle character length
		InvalidHandleCharacterLength,
		/// The handle name is reserved for chain use only
		HandleIsNotAllowed,
		/// The handle contains characters that are not allowed
		HandleContainsBlockedCharacters,
		/// The handle does not contain characters in the supported ranges of unicode characters
		HandleDoesNotConsistOfSupportedCharacterSets,
		/// Suffixes exhausted
		SuffixesExhausted,
		/// Invalid MSA
		InvalidMessageSourceAccount,
		/// Cryptographic signature failed verification
		InvalidSignature,
		/// The MSA already has a handle
		MSAHandleAlreadyExists,
		/// The MSA does not have a handle needed to retire it.
		MSAHandleDoesNotExist,
		/// The submitted proof has expired; the current block is less the expiration block
		ProofHasExpired,
		/// The submitted proof expiration block is too far in the future
		ProofNotYetValid,
		/// The handle is still in mortaility window
		HandleWithinMortalityPeriod,
		/// The handle is invalid
		InvalidHandle,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Deposited when a handle is claimed. [MSA id, display handle in UTF-8 bytes]
		HandleClaimed {
			/// MSA id of handle owner
			msa_id: MessageSourceId,
			/// UTF-8 string in bytes of the display handle
			handle: Vec<u8>,
		},
		/// Deposited when a handle is retired. [MSA id, display handle in UTF-8 bytes]
		HandleRetired {
			/// MSA id of handle owner
			msa_id: MessageSourceId,
			/// UTF-8 string in bytes
			handle: Vec<u8>,
		},
	}

	impl<T: Config> Pallet<T> {
		/// Gets the next suffix index for a canonical base.
		///
		/// This function takes a canonical base as input and returns the next available
		/// suffix index for that handle. If no current suffix index is found for the handle,
		/// it starts from 0. If a current suffix index is found, it increments it by 1 to
		/// get the next suffix index. If the increment operation fails due to overflow, it
		/// returns an error indicating that the suffixes are exhausted.
		///
		/// # Arguments
		///
		/// * `canonical_base` - The canonical base to get the next suffix index for.
		///
		/// # Returns
		///
		/// * `Ok(SequenceIndex)` - The next suffix index for the canonical base.
		/// * `Err(DispatchError)` - The suffixes are exhausted.
		pub fn get_next_suffix_index_for_canonical_handle(
			canonical_base: CanonicalBase,
		) -> Result<SequenceIndex, DispatchError> {
			let next: SequenceIndex;
			match CanonicalBaseHandleToSuffixIndex::<T>::get(canonical_base) {
				None => {
					next = 0;
				},
				Some((current, suffix_min)) => {
					let current_suffix_min = T::HandleSuffixMin::get();
					// if saved suffix min has changed, reset the suffix index
					if current_suffix_min != suffix_min {
						next = 0;
					} else {
						next = current.checked_add(1).ok_or(Error::<T>::SuffixesExhausted)?;
					}
				},
			}

			Ok(next)
		}

		/// Verifies that the signature is not expired.
		///
		/// This function takes a signature expiration block as input and verifies that
		/// the signature is not expired. It returns an error if the signature is expired.
		///
		/// # Arguments
		///
		/// * `signature_expires_at` - The block number at which the signature expires.
		///
		/// # Returns
		///
		/// * `Ok(())` - The signature is not expired.
		/// * `Err(DispatchError)` - The signature is expired.
		pub fn verify_signature_mortality(
			signature_expires_at: BlockNumberFor<T>,
		) -> DispatchResult {
			let current_block = frame_system::Pallet::<T>::block_number();

			let mortality_period = Self::mortality_block_limit(current_block);
			ensure!(mortality_period > signature_expires_at, Error::<T>::ProofNotYetValid);
			ensure!(current_block < signature_expires_at, Error::<T>::ProofHasExpired);
			Ok(())
		}

		/// Verifies the signature of a given payload, using the provided `signature`
		/// and `signer` information.
		///
		/// # Arguments
		///
		/// * `signature` - The `MultiSignature` to verify against the payload.
		/// * `signer` - The `T::AccountId` of the signer that signed the payload.
		/// * `payload` - The payload to verify the signature against.
		///
		/// # Errors
		/// * [`Error::InvalidSignature`]
		pub fn verify_signed_payload(
			signature: &MultiSignature,
			signer: &T::AccountId,
			payload: Vec<u8>,
		) -> DispatchResult {
			let key = T::ConvertIntoAccountId32::convert((*signer).clone());
			let wrapped_payload = wrap_binary_data(payload);

			ensure!(signature.verify(&wrapped_payload[..], &key), Error::<T>::InvalidSignature);

			Ok(())
		}

		/// Verifies max user handle size in bytes to address potential panic condition
		///
		/// # Arguments
		///
		/// * `handle` - The user handle.
		///
		/// # Errors
		/// * [`Error::InvalidHandleByteLength`]
		pub fn verify_max_handle_byte_length(handle: Vec<u8>) -> DispatchResult {
			ensure!(handle.len() as u32 <= HANDLE_BYTES_MAX, Error::<T>::InvalidHandleByteLength);
			Ok(())
		}

		/// The furthest in the future a mortality_block value is allowed
		/// to be for current_block
		/// This is calculated to be past the risk of a replay attack
		fn mortality_block_limit(current_block: BlockNumberFor<T>) -> BlockNumberFor<T> {
			let mortality_size = T::MortalityWindowSize::get();
			current_block + BlockNumberFor::<T>::from(mortality_size)
		}
		/// Generates a suffix for a canonical base, using the provided `canonical_base`
		/// and `cursor` information.
		///
		/// # Arguments
		///
		/// * `canonical_base` - The canonical base to generate a suffix for.
		/// * `cursor` - The cursor position in the sequence of suffixes to use for generation.
		///
		/// # Returns
		///
		/// The generated suffix as a `u16`.
		///
		pub fn generate_suffix_for_canonical_handle(
			canonical_base: &str,
			cursor: usize,
		) -> Result<u16, DispatchError> {
			let mut lazy_suffix_sequence = generate_unique_suffixes(
				T::HandleSuffixMin::get(),
				T::HandleSuffixMax::get(),
				&canonical_base,
			);
			match lazy_suffix_sequence.nth(cursor) {
				Some(suffix) => Ok(suffix),
				None => Err(Error::<T>::SuffixesExhausted.into()),
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Claims a handle for a caller's MSA (Message Source Account) based on the provided payload.
		/// This function performs several validations before claiming the handle, including checking
		/// the size of the base_handle, ensuring the caller have valid MSA keys,
		/// verifying the payload signature, and finally calling the internal `do_claim_handle` function
		/// to claim the handle.
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the caller.
		/// * `msa_owner_key` - The public key of the MSA owner.
		/// * `proof` - The multi-signature proof for the payload.
		/// * `payload` - The payload containing the information needed to claim a new handle.
		///
		/// # Errors
		///
		/// This function may return an error as part of `DispatchResult` if any of the following
		/// validations fail:
		///
		/// * [`Error::InvalidHandleByteLength`] - The base_handle size exceeds the maximum allowed size.
		/// * [`Error::InvalidMessageSourceAccount`] - The caller does not have a valid  `MessageSourceId`.
		/// * [`Error::InvalidSignature`] - The payload signature verification fails.
		///
		/// # Events
		/// * [`Event::HandleClaimed`]
		///
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::claim_handle(payload.base_handle.len() as u32))]
		pub fn claim_handle(
			origin: OriginFor<T>,
			msa_owner_key: T::AccountId,
			proof: MultiSignature,
			payload: ClaimHandlePayload<BlockNumberFor<T>>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			// Validation: Check for base_handle size to address potential panic condition
			Self::verify_max_handle_byte_length(payload.base_handle.clone())?;

			// Validation: caller must already have a MSA id
			let msa_id = T::MsaInfoProvider::ensure_valid_msa_key(&msa_owner_key)
				.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;

			// Validation: The signature is within the mortality window
			Self::verify_signature_mortality(payload.expiration)?;

			// Validation: Verify the payload was signed
			Self::verify_signed_payload(&proof, &msa_owner_key, payload.encode())?;

			let display_handle = Self::do_claim_handle(msa_id, payload)?;

			Self::deposit_event(Event::HandleClaimed { msa_id, handle: display_handle.clone() });
			Ok(())
		}

		/// Retire a handle for a given `DisplayHandle` owner.
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the call.
		///
		/// # Errors
		///
		/// This function can return the following errors:
		///
		/// * `InvalidHandleByteLength` - If the length of the `payload.display_handle` exceeds the maximum allowed size.
		/// * `InvalidMessageSourceAccount` - If caller of this extrinsic does not have a valid MSA (Message Source Account) ID.
		///
		/// # Events
		/// * [`Event::HandleRetired`]
		///
		#[pallet::call_index(1)]
		#[pallet::weight((T::WeightInfo::retire_handle(), DispatchClass::Normal, Pays::No))]
		pub fn retire_handle(origin: OriginFor<T>) -> DispatchResult {
			let msa_owner_key = ensure_signed(origin)?;

			// Validation: The caller must already have a MSA id
			let msa_id = T::MsaInfoProvider::ensure_valid_msa_key(&msa_owner_key)
				.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;

			let display_handle: Vec<u8> = Self::do_retire_handle(msa_id)?;

			Self::deposit_event(Event::HandleRetired { msa_id, handle: display_handle });
			Ok(())
		}

		/// Changes the handle for a caller's MSA (Message Source Account) based on the provided payload.
		/// This function performs several validations before claiming the handle, including checking
		/// the size of the base_handle, ensuring the caller has valid MSA keys,
		/// verifying the payload signature, and finally calling the internal `do_retire_handle` and `do_claim_handle` functions
		/// to retire the current handle and claim the new one.
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the caller.
		/// * `msa_owner_key` - The public key of the MSA owner.
		/// * `proof` - The multi-signature proof for the payload.
		/// * `payload` - The payload containing the information needed to change an existing handle.
		///
		/// # Errors
		///
		/// This function may return an error as part of `DispatchResult` if any of the following
		/// validations fail:
		///
		/// * [`Error::InvalidHandleByteLength`] - The base_handle size exceeds the maximum allowed size.
		/// * [`Error::InvalidMessageSourceAccount`] - The caller does not have a valid  `MessageSourceId`.
		/// * [`Error::InvalidSignature`] - The payload signature verification fails.
		///
		/// # Events
		/// * [`Event::HandleRetired`]
		/// * [`Event::HandleClaimed`]
		///
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::change_handle(payload.base_handle.len() as u32))]
		pub fn change_handle(
			origin: OriginFor<T>,
			msa_owner_key: T::AccountId,
			proof: MultiSignature,
			payload: ClaimHandlePayload<BlockNumberFor<T>>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			// Validation: Check for base_handle size to address potential panic condition
			Self::verify_max_handle_byte_length(payload.base_handle.clone())?;

			// Validation: caller must already have a MSA id
			let msa_id = T::MsaInfoProvider::ensure_valid_msa_key(&msa_owner_key)
				.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;

			// Validation: The signature is within the mortality window
			Self::verify_signature_mortality(payload.expiration)?;

			// Validation: Verify the payload was signed
			Self::verify_signed_payload(&proof, &msa_owner_key, payload.encode())?;

			// Get existing handle to retire
			MSAIdToDisplayName::<T>::get(msa_id).ok_or(Error::<T>::MSAHandleDoesNotExist)?;

			// Retire old handle
			let old_display_handle: Vec<u8> = Self::do_retire_handle(msa_id)?;
			Self::deposit_event(Event::HandleRetired { msa_id, handle: old_display_handle });

			let display_handle = Self::do_claim_handle(msa_id, payload.clone())?;

			Self::deposit_event(Event::HandleClaimed { msa_id, handle: display_handle.clone() });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Retrieves a handle for a given MSA (MessageSourceId).
		///
		/// # Arguments
		///
		/// * `msa_id` - The MSA ID to retrieve the handle for.
		///
		/// # Returns
		///
		/// * `Option<HandleResponse>` - The handle response if the MSA ID is valid.
		///
		pub fn get_handle_for_msa(msa_id: MessageSourceId) -> Option<HandleResponse> {
			let display_handle = match MSAIdToDisplayName::<T>::get(msa_id) {
				Some((handle, _)) => handle,
				_ => return None,
			};
			// convert to string
			let display_handle_str = core::str::from_utf8(&display_handle)
				.map_err(|_| Error::<T>::InvalidHandleEncoding)
				.ok()?;

			let (base_handle_str, suffix) = split_display_name(display_handle_str)?;
			let base_handle = base_handle_str.as_bytes().to_vec();
			// Convert base handle into a canonical base
			let (_, canonical_base) =
				Self::get_canonical_string_vec_from_base_handle(&base_handle_str);
			Some(HandleResponse { base_handle, suffix, canonical_base: canonical_base.into() })
		}

		/// Get the next available suffixes for a given handle.
		///
		/// This function takes a `Vec<u8>` handle and generates the next available
		/// suffixes for that handle. The number of suffixes to generate is determined
		/// by the `count` parameter, which is of type `u16`.
		///
		/// # Arguments
		///
		/// * `handle` - The handle to generate the next available suffixes for.
		/// * `count` - The number of suffixes to generate.
		///
		/// # Returns
		///
		/// * `PresumptiveSuffixesResponse` - The response containing the next available suffixes.
		///
		pub fn get_next_suffixes(
			base_handle: BaseHandle,
			count: u16,
		) -> PresumptiveSuffixesResponse {
			let base_handle_str = core::str::from_utf8(&base_handle).unwrap_or_default();

			// Convert base handle into a canonical base
			let (canonical_handle_str, canonical_base) =
				Self::get_canonical_string_vec_from_base_handle(&base_handle_str);

			let suffix_index =
				Self::get_next_suffix_index_for_canonical_handle(canonical_base.clone())
					.unwrap_or_default();

			let lazy_suffix_sequence = generate_unique_suffixes(
				T::HandleSuffixMin::get(),
				T::HandleSuffixMax::get(),
				&canonical_handle_str,
			)
			.enumerate();

			let mut suffixes: Vec<HandleSuffix> = vec![];

			for (i, suffix) in lazy_suffix_sequence {
				if i >= suffix_index as usize && i < (suffix_index as usize + count as usize) {
					suffixes.push(suffix);
				}
				if suffixes.len() == count as usize {
					break;
				}
			}
			PresumptiveSuffixesResponse { base_handle: base_handle.into(), suffixes }
		}

		/// Retrieve a `MessageSourceId` for a given handle.
		///
		/// # Arguments
		///
		/// * `display_handle` - The full display handle to retrieve the `MessageSourceId` for.
		///
		/// # Returns
		///
		/// * `Option<MessageSourceId>` - The `MessageSourceId` if the handle is valid.
		///
		pub fn get_msa_id_for_handle(display_handle: DisplayHandle) -> Option<MessageSourceId> {
			let display_handle_str = core::str::from_utf8(&display_handle).unwrap_or_default();
			let (base_handle_str, suffix) = split_display_name(display_handle_str)?;
			// Convert base handle into a canonical base
			let (_, canonical_base) =
				Self::get_canonical_string_vec_from_base_handle(&base_handle_str);
			CanonicalBaseHandleAndSuffixToMSAId::<T>::get(canonical_base, suffix)
		}

		/// Claims a handle for a given MSA (MessageSourceId) by validating and storing the base handle,
		/// generating a canonical base, generating a suffix, and composing the full display handle.
		///
		/// # Arguments
		///
		/// * `msa_id` - The MSA (MessageSourceId) to claim the handle for.
		/// * `payload` - The payload containing the base handle to claim.
		///
		/// # Returns
		///
		/// * `DisplayHandle` - The full display handle.
		///
		pub fn do_claim_handle(
			msa_id: MessageSourceId,
			payload: ClaimHandlePayload<BlockNumberFor<T>>,
		) -> Result<Vec<u8>, DispatchError> {
			// Validation: The MSA must not already have a handle associated with it
			ensure!(
				MSAIdToDisplayName::<T>::get(msa_id).is_none(),
				Error::<T>::MSAHandleAlreadyExists
			);

			let base_handle_str = Self::validate_base_handle(payload.base_handle)?;

			// Convert base handle into a canonical base
			let (canonical_handle_str, canonical_base) =
				Self::get_canonical_string_vec_from_base_handle(&base_handle_str);

			Self::validate_canonical_handle(&canonical_handle_str)?;

			// Generate suffix from the next available index into the suffix sequence
			let suffix_sequence_index =
				Self::get_next_suffix_index_for_canonical_handle(canonical_base.clone())?;

			let suffix = Self::generate_suffix_for_canonical_handle(
				&canonical_handle_str,
				suffix_sequence_index as usize,
			)?;

			let display_handle = Self::build_full_display_handle(&base_handle_str, suffix)?;

			// Store canonical base and suffix to MSA id
			CanonicalBaseHandleAndSuffixToMSAId::<T>::insert(
				canonical_base.clone(),
				suffix,
				msa_id,
			);
			// Store canonical base to suffix sequence index
			CanonicalBaseHandleToSuffixIndex::<T>::set(
				canonical_base.clone(),
				Some((suffix_sequence_index, T::HandleSuffixMin::get())),
			);

			// Store the full display handle to MSA id
			MSAIdToDisplayName::<T>::insert(msa_id, (display_handle.clone(), payload.expiration));

			Ok(display_handle.into_inner())
		}

		/// Checks that handle base string is valid before canonicalization
		fn validate_base_handle(base_handle: Vec<u8>) -> Result<String, DispatchError> {
			// Convert base handle to UTF-8 string slice while validating.
			let base_handle_str =
				String::from_utf8(base_handle).map_err(|_| Error::<T>::InvalidHandleEncoding)?;

			// Validation: The handle length must be valid.
			// Note: the count() can panic but won't because the base_handle byte length is already checked
			let len = base_handle_str.chars().count() as u32;

			// Validation: `BaseHandle` character length must be within range
			ensure!(
				len >= HANDLE_CHARS_MIN && len <= HANDLE_CHARS_MAX,
				Error::<T>::InvalidHandleCharacterLength
			);

			ensure!(
				!contains_blocked_characters(&base_handle_str),
				Error::<T>::HandleContainsBlockedCharacters
			);
			Ok(base_handle_str)
		}

		/// Validate that the canonical handle:
		/// - contains characters ONLY in supported ranges
		/// - Does NOT contain reserved words
		/// - Is between (inclusive) `HANDLE_CHARS_MIN` and `HANDLE_CHARS_MAX`
		fn validate_canonical_handle(canonical_base_handle_str: &str) -> DispatchResult {
			// Validation: The handle must consist of characters not containing reserved words or blocked characters
			ensure!(
				consists_of_supported_unicode_character_sets(&canonical_base_handle_str),
				Error::<T>::HandleDoesNotConsistOfSupportedCharacterSets
			);
			ensure!(
				!is_reserved_canonical_handle(&canonical_base_handle_str),
				Error::<T>::HandleIsNotAllowed
			);

			let len = canonical_base_handle_str.chars().count() as u32;

			// Validation: `Canonical` character length must be within range
			ensure!(
				len >= HANDLE_CHARS_MIN && len <= HANDLE_CHARS_MAX,
				Error::<T>::InvalidHandleCharacterLength
			);

			Ok(())
		}

		/// Creates a full display handle by combining a base handle string with supplied suffix
		///
		/// # Arguments
		///
		/// * `base_handle_str` - The base handle string.
		/// * `suffix` - The numeric suffix .
		///
		/// # Returns
		///
		/// * `DisplayHandle` - The full display handle.
		///
		pub fn build_full_display_handle(
			base_handle: &str,
			suffix: HandleSuffix,
		) -> Result<DisplayHandle, DispatchError> {
			let base_handle_trimmed = trim_and_collapse_whitespace(base_handle);
			let mut full_handle_vec: Vec<u8> = vec![];
			full_handle_vec.extend(base_handle_trimmed.as_bytes());
			full_handle_vec.push(HANDLE_DELIMITER as u8); // The delimiter
			let mut buff = [0u8; SUFFIX_MAX_DIGITS];
			full_handle_vec.extend(suffix.numtoa(10, &mut buff));
			let res = full_handle_vec.try_into().map_err(|_| Error::<T>::InvalidHandleEncoding)?;
			Ok(res)
		}

		/// Retires a handle associated with a given MessageSourceId (MSA) in the dispatch module.
		///
		/// # Arguments
		///
		/// * `msa_id` - The MSA (MessageSourceId) to retire the handle for.
		///
		/// # Returns
		///
		/// * `DispatchResult` - Returns `Ok` if the handle was successfully retired.
		pub fn do_retire_handle(msa_id: MessageSourceId) -> Result<Vec<u8>, DispatchError> {
			// Validation: The MSA must already have a handle associated with it
			let handle_from_state =
				MSAIdToDisplayName::<T>::get(msa_id).ok_or(Error::<T>::MSAHandleDoesNotExist)?;
			let display_name_handle = handle_from_state.0;
			let display_name_str = core::str::from_utf8(&display_name_handle)
				.map_err(|_| Error::<T>::InvalidHandleEncoding)?;

			let (base_handle_str, suffix_num) =
				split_display_name(display_name_str).ok_or(Error::<T>::InvalidHandle)?;

			// Convert base handle into a canonical base
			let (_, canonical_base) =
				Self::get_canonical_string_vec_from_base_handle(&base_handle_str);

			// Remove handle from storage but not from CanonicalBaseHandleToSuffixIndex because retired handles can't be reused
			MSAIdToDisplayName::<T>::remove(msa_id);
			CanonicalBaseHandleAndSuffixToMSAId::<T>::remove(canonical_base, suffix_num);

			Ok(display_name_str.as_bytes().to_vec())
		}

		/// Checks whether the supplied handle passes all the checks performed by a
		/// claim_handle call.
		/// # Returns
		/// * true if it is valid or false if invalid
		pub fn validate_handle(handle: Vec<u8>) -> bool {
			return match Self::validate_base_handle(handle) {
				Ok(base_handle_str) => {
					// Convert base handle into a canonical base
					let (canonical_handle_str, _) =
						Self::get_canonical_string_vec_from_base_handle(&base_handle_str);

					return Self::validate_canonical_handle(&canonical_handle_str).is_ok();
				},
				_ => false,
			};
		}

		/// Converts a base handle to a canonical base.
		fn get_canonical_string_vec_from_base_handle(
			base_handle_str: &str,
		) -> (String, CanonicalBase) {
			let canonical_handle_str = convert_to_canonical(&base_handle_str);
			let canonical_handle_vec = canonical_handle_str.as_bytes().to_vec();
			let canonical_base: CanonicalBase = canonical_handle_vec.try_into().unwrap_or_default();
			(canonical_handle_str, canonical_base)
		}
	}
}
