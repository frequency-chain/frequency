//! # Handles Pallet
//! The Handles pallet provides functionality for claiming and retiring user handles.  Each MSA can have one user handle
//! associated with it.  The handle consists of a canonical base, a "." delimiter and a unique numeric suffix.  It also has
//! a display name.  e.g. user.734 with a display of "User"
//!
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
//!
//! ## Overview
//!
//! ## Features
//!
//! * Handle creation and retirement
//! * Shuffled sequence generation for handle suffixes
//! * Homoglyph detection
//!
//! ## Terminology
//! - Handle
//! - Suffix
//!
//! ## Glossary
//! - **Handle:** A handle is a unique identifier for a user. Handle on frequency is composed of a `base_handle`, its canonical version as, `canonical_handle` and a unique numeric `suffix`.
//! - **Base Handle:** A base handle is a user's chosen handle name.  It is not guaranteed to be unique.
//! - **Canonical Handle:** A canonical handle is a base handle that has been converted to a canonical form. Canonicals are unique representations of a base handle.
//! - **Delimiter:** Period character (".") is reserved on Frequency to form full handle as `canonical_handle`.`suffix`.
//! - **Suffix:** A suffix is a unique numeric value appended to a handle's canonical base to make it unique.

// Ensure we're `no_std` when compiling for WASM.
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

use handles_utils::{converter::HandleConverter, validator::HandleValidator};

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
	MultiSignature,
};
use sp_std::{prelude::*, vec::Vec};

pub mod handles_signed_extension;

pub mod suffix;
use suffix::SuffixGenerator;
pub mod weights;
pub use weights::*;

impl<T: Config> HandleProvider for Pallet<T> {
	fn get_handle_for_msa(msa_id: MessageSourceId) -> Option<HandleResponse> {
		Self::get_handle_for_msa(msa_id)
	}
}

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

		/// A type that will supply MSA related information
		type MsaInfoProvider: MsaLookup + MsaValidator<AccountId = Self::AccountId>;

		/// The minimum suffix value
		#[pallet::constant]
		type HandleSuffixMin: Get<u32>;

		/// The maximum suffix value
		#[pallet::constant]
		type HandleSuffixMax: Get<u32>;

		/// The number of blocks before a signature can be ejected from the PayloadSignatureRegistryList
		#[pallet::constant]
		type MortalityWindowSize: Get<u32>;

		#[cfg(feature = "runtime-benchmarks")]
		/// A set of helper functions for benchmarking.
		type MsaBenchmarkHelper: MsaBenchmarkHelper<Self::AccountId>;
	}

	// Storage
	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	/// - Keys: k1: Canonical base handle, k2: Suffix
	/// - Value: MSA id
	#[pallet::storage]
	#[pallet::getter(fn get_msa_id_for_canonical_and_suffix)]
	pub type CanonicalBaseHandleAndSuffixToMSAId<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Handle,
		Twox64Concat,
		HandleSuffix,
		MessageSourceId,
		OptionQuery,
	>;

	/// - Key: MSA id
	/// - Value: Display name
	#[pallet::storage]
	#[pallet::getter(fn get_display_name_for_msa_id)]
	pub type MSAIdToDisplayName<T: Config> =
		StorageMap<_, Twox64Concat, MessageSourceId, (Handle, T::BlockNumber), OptionQuery>;

	/// - Value: Cursor u16
	#[pallet::storage]
	#[pallet::getter(fn get_current_suffix_index_for_canonical_handle)]
	pub type CanonicalBaseHandleToSuffixIndex<T: Config> =
		StorageMap<_, Blake2_128Concat, Handle, SequenceIndex, OptionQuery>;

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
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Deposited when a handle is claimed. [MSA id, full handle in UTF-8 bytes]
		HandleClaimed {
			/// MSA id of handle owner
			msa_id: MessageSourceId,
			/// UTF-8 string in bytes
			handle: Vec<u8>,
		},
		/// Deposited when a handle is retired. [MSA id, full handle in UTF-8 bytes]
		HandleRetired {
			/// MSA id of handle owner
			msa_id: MessageSourceId,
			/// UTF-8 string in bytes
			handle: Vec<u8>,
		},
	}

	impl<T: Config> Pallet<T> {
		/// Gets the next suffix index for a canonical handle.
		///
		/// This function takes a canonical handle as input and returns the next available
		/// suffix index for that handle. If no current suffix index is found for the handle,
		/// it starts from 0. If a current suffix index is found, it increments it by 1 to
		/// get the next suffix index. If the increment operation fails due to overflow, it
		/// returns an error indicating that the suffixes are exhausted.
		///
		/// # Arguments
		///
		/// * `canonical_handle` - The canonical handle to get the next suffix index for.
		///
		/// # Returns
		///
		/// * `Ok(SequenceIndex)` - The next suffix index for the canonical handle.
		/// * `Err(DispatchError)` - The suffixes are exhausted.
		pub fn get_next_suffix_index_for_canonical_handle(
			canonical_handle: Handle,
		) -> Result<SequenceIndex, DispatchError> {
			let next: SequenceIndex;
			match Self::get_current_suffix_index_for_canonical_handle(canonical_handle) {
				None => {
					next = 0;
				},
				Some(current) => {
					next = current.checked_add(1).ok_or(Error::<T>::SuffixesExhausted)?;
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
		pub fn verify_signature_mortality(signature_expires_at: T::BlockNumber) -> DispatchResult {
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

		/// The furthest in the future a mortality_block value is allowed
		/// to be for current_block
		/// This is calculated to be past the risk of a replay attack
		fn mortality_block_limit(current_block: T::BlockNumber) -> T::BlockNumber {
			let mortality_size = T::MortalityWindowSize::get();
			current_block + T::BlockNumber::from(mortality_size)
		}
		/// Generates a suffix for a canonical handle, using the provided `canonical_handle`
		/// and `cursor` information.
		///
		/// # Arguments
		///
		/// * `canonical_handle` - The canonical handle to generate a suffix for.
		/// * `cursor` - The cursor position in the sequence of suffixes to use for generation.
		///
		/// # Returns
		///
		/// The generated suffix as a `u16`.
		///
		fn generate_suffix_for_canonical_handle(
			canonical_handle: &str,
			cursor: usize,
		) -> HandleSuffix {
			let mut suffix_generator = SuffixGenerator::new(
				T::HandleSuffixMin::get() as usize,
				T::HandleSuffixMax::get() as usize,
				&canonical_handle,
			);
			let sequence: Vec<usize> = suffix_generator.suffix_iter().collect();
			sequence[cursor] as HandleSuffix
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
		/// * `payload` - The payload containing the information needed to claim the handle.
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
			payload: ClaimHandlePayload<T::BlockNumber>,
		) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			// Validation: Check for base_handle size to address potential panic condition
			ensure!(
				payload.base_handle.len() as u32 <= HANDLE_BASE_BYTES_MAX,
				Error::<T>::InvalidHandleByteLength
			);

			// Validation: caller must already have a MSA id
			let msa_id = T::MsaInfoProvider::ensure_valid_msa_key(&msa_owner_key)
				.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;

			// Validation: The signature is within the mortality window
			Self::verify_signature_mortality(payload.expiration.into())?;

			// Validation: Verify the payload was signed
			Self::verify_signed_payload(&proof, &msa_owner_key, payload.encode())?;

			let full_handle = Self::do_claim_handle(msa_id, payload)?;

			Self::deposit_event(Event::HandleClaimed { msa_id, handle: full_handle.clone() });
			Ok(())
		}

		/// Retire a handle for a given `Handle` owner.
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the call.
		///
		/// # Errors
		///
		/// This function can return the following errors:
		///
		/// * `InvalidHandleByteLength` - If the length of the `payload.full_handle` exceeds the maximum allowed size.
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

			let full_handle = Self::do_retire_handle(msa_id)?;

			Self::deposit_event(Event::HandleRetired { msa_id, handle: full_handle });
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
			let full_handle = match Self::get_display_name_for_msa_id(msa_id) {
				Some((handle, _)) => handle,
				_ => return None,
			};
			// convert to string
			let full_handle_str = core::str::from_utf8(&full_handle)
				.map_err(|_| Error::<T>::InvalidHandleEncoding)
				.ok()?;

			let (base_handle_str, suffix) = HandleConverter::split_display_name(full_handle_str);
			let base_handle = base_handle_str.as_bytes().to_vec();
			let canonical_handle_str = HandleConverter::convert_to_canonical(&base_handle_str);
			let canonical_handle = canonical_handle_str.as_bytes().to_vec();
			Some(HandleResponse { base_handle, suffix, canonical_handle })
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
		/// ```
		pub fn get_next_suffixes(handle: Vec<u8>, count: u16) -> PresumptiveSuffixesResponse {
			let mut suffixes: Vec<HandleSuffix> = vec![];
			let base_handle: Handle = handle.try_into().unwrap_or_default();
			let base_handle_str = core::str::from_utf8(&base_handle).unwrap_or("");

			let canonical_handle_str = HandleConverter::convert_to_canonical(&base_handle_str);
			let canonical_handle_vec = canonical_handle_str.as_bytes().to_vec();
			let canonical_handle: Handle = canonical_handle_vec.try_into().unwrap();
			let suffix_index =
				Self::get_next_suffix_index_for_canonical_handle(canonical_handle.clone())
					.unwrap_or_default();

			// Generate suffixes from the next available suffix index
			let mut suffix_generator = SuffixGenerator::new(
				T::HandleSuffixMin::get() as usize,
				T::HandleSuffixMax::get() as usize,
				&canonical_handle_str,
			);
			let sequence = suffix_generator.suffix_iter().enumerate();

			for (i, suffix) in sequence {
				if i >= suffix_index as usize && i < (suffix_index as usize + count as usize) {
					suffixes.push(suffix as u16);
				}
				if suffixes.len() == count as usize {
					break
				}
			}
			let response =
				PresumptiveSuffixesResponse { base_handle: base_handle.into(), suffixes };
			response
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
		pub fn get_msa_id_for_handle(display_handle: Handle) -> Option<MessageSourceId> {
			let full_handle_str = core::str::from_utf8(&display_handle).unwrap_or("");
			let (base_handle_str, suffix) = HandleConverter::split_display_name(full_handle_str);
			let canonical_handle_str = HandleConverter::convert_to_canonical(&base_handle_str);
			let canonical_handle = canonical_handle_str.as_bytes().to_vec();
			let canonical_handle: Handle = canonical_handle.try_into().unwrap();
			let msa = Self::get_msa_id_for_canonical_and_suffix(canonical_handle, suffix);
			msa
		}

		/// Creates a full display handle by combining a base handle string with a suffix generated
		/// from an index into the suffix sequence.
		///
		/// # Arguments
		///
		/// * `base_handle_str` - The base handle string.
		/// * `suffix_sequence_index` - The index into the suffix sequence.
		///
		/// # Returns
		///
		/// * `Handle` - The full display handle.
		///
		#[cfg(test)]
		pub fn create_full_handle_for_index(
			base_handle_str: &str,
			suffix_sequence_index: SequenceIndex,
		) -> Vec<u8> {
			// Convert base display handle into a canonical display handle
			let canonical_handle_str = HandleConverter::convert_to_canonical(&base_handle_str);

			// Generate suffix from index into the suffix sequence
			let suffix = Self::generate_suffix_for_canonical_handle(
				&canonical_handle_str,
				suffix_sequence_index as usize,
			);

			let full_handle = Self::create_full_handle(base_handle_str, suffix);
			full_handle
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
		/// * `Handle` - The full display handle.
		///
		#[cfg(test)]
		pub fn create_full_handle(base_handle_str: &str, suffix: HandleSuffix) -> Vec<u8> {
			// Compose the full display handle from the base handle, "." delimiter and suffix
			let mut full_handle_vec: Vec<u8> = vec![];
			full_handle_vec.extend(base_handle_str.as_bytes());
			full_handle_vec.extend(b"."); // The delimiter
			let mut buff = [0u8; SUFFIX_MAX_DIGITS];
			full_handle_vec.extend(suffix.numtoa(10, &mut buff)); // Use base 10
			full_handle_vec
		}

		/// Claims a handle for a given MSA (MessageSourceId) by validating and storing the base handle,
		/// generating a canonical handle, generating a suffix, and composing the full display handle.
		///
		/// # Arguments
		///
		/// * `msa_id` - The MSA (MessageSourceId) to claim the handle for.
		/// * `payload` - The payload containing the base handle to claim.
		///
		/// # Returns
		///
		/// * `Handle` - The full display handle.
		///
		pub fn do_claim_handle(
			msa_id: MessageSourceId,
			payload: ClaimHandlePayload<T::BlockNumber>,
		) -> Result<Vec<u8>, DispatchError> {
			// Validation: The MSA must not already have a handle associated with it
			ensure!(
				Self::get_display_name_for_msa_id(msa_id).is_none(),
				Error::<T>::MSAHandleAlreadyExists
			);

			// Convert base handle to UTF-8 string slice while validating.
			let base_handle_str = core::str::from_utf8(&payload.base_handle)
				.map_err(|_| Error::<T>::InvalidHandleEncoding)?;

			// Validation: The handle length must be valid.
			// Note: the count() can panic but won't because the base_handle byte length is already checked
			let len = base_handle_str.chars().count() as u32;

			// Validation: Handle character length must be within range
			ensure!(
				len >= HANDLE_BASE_CHARS_MIN && len <= HANDLE_BASE_CHARS_MAX,
				Error::<T>::InvalidHandleCharacterLength
			);

			// Validation: The handle must consist of characters not containing reserved words or blocked characters
			let handle_validator = HandleValidator::new();
			ensure!(
				handle_validator.consists_of_supported_unicode_character_sets(base_handle_str),
				Error::<T>::HandleDoesNotConsistOfSupportedCharacterSets
			);
			ensure!(
				!handle_validator.is_reserved_handle(base_handle_str),
				Error::<T>::HandleIsNotAllowed
			);
			ensure!(
				!handle_validator.contains_blocked_characters(base_handle_str),
				Error::<T>::HandleContainsBlockedCharacters
			);

			// Convert base display handle into a canonical display handle
			let canonical_handle_str = HandleConverter::convert_to_canonical(&base_handle_str);
			let canonical_handle_vec = canonical_handle_str.as_bytes().to_vec();
			let canonical_handle: Handle = canonical_handle_vec.try_into().unwrap();

			// Generate suffix from the next available index into the suffix sequence
			let suffix_sequence_index =
				Self::get_next_suffix_index_for_canonical_handle(canonical_handle.clone())?;
			let suffix = Self::generate_suffix_for_canonical_handle(
				&canonical_handle_str,
				suffix_sequence_index as usize,
			);

			// Store canonical handle and suffix to MSA id
			CanonicalBaseHandleAndSuffixToMSAId::<T>::insert(
				canonical_handle.clone(),
				suffix,
				msa_id,
			);
			// Store canonical handle to suffix sequence index
			CanonicalBaseHandleToSuffixIndex::<T>::set(
				canonical_handle.clone(),
				Some(suffix_sequence_index),
			);

			// Compose the full display handle from the base handle, "." delimiter and suffix
			let mut full_handle_vec: Vec<u8> = vec![];
			full_handle_vec.extend(base_handle_str.as_bytes());
			full_handle_vec.extend(b"."); // The delimiter
			let mut buff = [0u8; SUFFIX_MAX_DIGITS];
			full_handle_vec.extend(suffix.numtoa(10, &mut buff)); // Use base 10

			let full_handle: Handle = full_handle_vec.clone().try_into().ok().unwrap();

			// Store the full display handle to MSA id
			MSAIdToDisplayName::<T>::insert(msa_id, (full_handle.clone(), payload.expiration));

			Ok(full_handle_vec)
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
			let handle_from_state = Self::get_display_name_for_msa_id(msa_id)
				.ok_or(Error::<T>::MSAHandleDoesNotExist)?;
			let display_name_handle = handle_from_state.0;
			let display_name_str = core::str::from_utf8(&display_name_handle)
				.map_err(|_| Error::<T>::InvalidHandleEncoding)?;

			let (base_handle_str, suffix_num) =
				HandleConverter::split_display_name(display_name_str);

			let canonical_handle_str = HandleConverter::convert_to_canonical(&base_handle_str);
			let canonical_handle_vec = canonical_handle_str.as_bytes().to_vec();
			let canonical_handle: Handle = canonical_handle_vec.try_into().unwrap();

			// Remove handle from storage but not from CanonicalBaseHandleToSuffixIndex because retired handles can't be reused
			MSAIdToDisplayName::<T>::remove(msa_id);
			CanonicalBaseHandleAndSuffixToMSAId::<T>::remove(canonical_handle, suffix_num);

			Ok(display_name_str.as_bytes().to_vec())
		}
	}
}
