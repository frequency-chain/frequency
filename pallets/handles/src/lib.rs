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
//! - PRNG

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
/// Handle converter for canonical handles
pub mod utils;
use utils::{converter::HandleConverter, validator::HandleValidator};

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
use common_primitives::benchmarks::MsaBenchmarkHelper;
use sp_std::prelude::*;

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

pub mod suffix;
use suffix::SuffixGenerator;
pub mod weights;
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
		StorageMap<_, Twox64Concat, MessageSourceId, Handle, ValueQuery>;

	/// - Keys: Canonical base handle (no delimeter, no suffix)
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
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Deposited when a handle is created. [MSA id, full handle in UTF-8 bytes]
		HandleCreated {
			/// MSA id of handle owner
			msa_id: MessageSourceId,
			/// UTF-8 string in bytes
			handle: Handle,
		},
		/// Deposited when a handle is retired. [MSA id, full handle in UTF-8 bytes]
		HandleRetired {
			/// MSA id of handle owner
			msa_id: MessageSourceId,
			/// UTF-8 string in bytes
			handle: Handle,
		},
	}

	impl<T: Config> Pallet<T> {
		/// Get the next index into the shuffled suffix sequence for the specified canonical base handle
		///
		/// # Arguments
		///
		/// * `canonical_handle` - The canonical `Handle` to get suffixes for.
		///
		/// # Errors
		/// * [`Error::SuffixesExhausted`]
		///
		pub fn get_next_suffix_index_for_canonical_handle(
			canonical_handle: Handle,
		) -> Result<SequenceIndex, DispatchError> {
			let next;
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
		fn generate_suffix_for_canonical_handle(canonical_handle: &str, cursor: usize) -> u16 {
			let mut suffix_generator = SuffixGenerator::new(
				T::HandleSuffixMin::get() as usize,
				T::HandleSuffixMax::get() as usize,
				&canonical_handle,
			);
			let sequence: Vec<usize> = suffix_generator.suffix_iter().collect();
			sequence[cursor] as u16
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Claims a new handle
		///
		/// # Arguments
		///
		/// * `origin` - An `OriginFor<T>` that will provide the account that will be claiming the handle.
		/// * `delegator_key` - The account id of the MSA id owner.
		/// * `proof` - A `MultiSignature` that represents the signature of the payload by the `delegator_key`.
		/// * `payload` - A `ClaimHandlePayload` that contains the payload data required to claim the handle.
		///
		/// # Events
		/// * [`Event::HandleCreated`]
		///
		/// # Errors
		/// * [`Error::InvalidHandleByteLength`]
		/// * [`Error::InvalidMessageSourceAccount`]
		/// * [`Error::MSAHandleAlreadyExists`]
		/// * [`Error::InvalidHandleEncoding`]
		/// * [`Error::InvalidHandleCharacterLength`]
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::claim_handle(payload.base_handle.len() as u32))]
		pub fn claim_handle(
			origin: OriginFor<T>,
			delegator_key: T::AccountId,
			proof: MultiSignature,
			payload: ClaimHandlePayload,
		) -> DispatchResult {
			let provider_key = ensure_signed(origin)?;

			// Validation: Check for base_handle size to address potential panic condition
			ensure!(
				payload.base_handle.len() as u32 <= HANDLE_BASE_BYTES_MAX,
				Error::<T>::InvalidHandleByteLength
			);

			// Validation: The provider must already have a MSA id
			T::MsaInfoProvider::ensure_valid_msa_key(&provider_key)
				.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;

			// Validation: The delegator must already have a MSA id
			let delegator_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(&delegator_key)
				.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;

			// Validation: Verify the payload was signed
			Self::verify_signed_payload(&proof, &delegator_key, payload.encode())?;

			// Validation: The MSA must not already have a handle associated with it
			ensure!(
				MSAIdToDisplayName::<T>::try_get(delegator_msa_id).is_err(),
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

			// Validation: The handle must not contain reserved words or blocked characters
			let handle_validator = HandleValidator::new();
			ensure!(
				!handle_validator.is_reserved_handle(base_handle_str),
				Error::<T>::HandleIsNotAllowed
			);
			ensure!(
				!handle_validator.contains_blocked_characters(base_handle_str),
				Error::<T>::HandleContainsBlockedCharacters
			);

			// Convert base display handle into a canonical display handle
			let handle_converter = HandleConverter::new();
			let canonical_handle_str = handle_converter.convert_to_canonical(&base_handle_str);
			let canonical_handle_vec = canonical_handle_str.as_bytes().to_vec();
			let canonical_handle: Handle = canonical_handle_vec.try_into().unwrap();

			// Generate suffix from the next available index into the suffix sequence
			let suffix_sequence_index =
				Self::get_next_suffix_index_for_canonical_handle(canonical_handle.clone())
					.unwrap_or_default();
			let suffix = Self::generate_suffix_for_canonical_handle(
				&canonical_handle_str,
				suffix_sequence_index as usize,
			);

			// Store canonical handle and suffix to MSA id
			CanonicalBaseHandleAndSuffixToMSAId::<T>::insert(
				canonical_handle.clone(),
				suffix,
				delegator_msa_id,
			);
			// Store canonical handle to suffix sequence index
			CanonicalBaseHandleToSuffixIndex::<T>::set(
				canonical_handle.clone(),
				Some(suffix_sequence_index),
			);

			// Compose the full display handle from the base handle, "." delimeter and suffix
			let mut full_handle_vec: Vec<u8> = vec![];
			full_handle_vec.extend(base_handle_str.as_bytes());
			full_handle_vec.extend(b"."); // The delimeter
			let mut buff = [0u8; SUFFIX_MAX_DIGITS];
			full_handle_vec.extend(suffix.numtoa(10, &mut buff)); // Use base 10

			let full_handle: Handle = full_handle_vec.try_into().ok().unwrap();

			// Store the full display handle to MSA id
			MSAIdToDisplayName::<T>::insert(delegator_msa_id, full_handle.clone());

			Self::deposit_event(Event::HandleCreated {
				msa_id: delegator_msa_id,
				handle: full_handle.clone(),
			});
			Ok(())
		}

		/// Extrinsic to retire a handle
		///
		/// # Arguments
		///
		/// * `origin` - An `OriginFor<T>` that will provide the account that will be retiring the handle.
		/// * `delegator_key` - The account id of the MSA id owner.
		/// * `proof` - A `MultiSignature` that represents the signature of the payload by the `delegator_key`.
		/// * `payload` - A `RetireHandlePayload` that contains the payload data required to retire the handle.
		///
		/// # Events
		/// * [`Event::HandleRetired`]
		///
		/// # Errors
		/// * [`Error::InvalidHandleByteLength`]
		/// * [`Error::InvalidMessageSourceAccount`]
		/// * [`Error::MSAHandleAlreadyExists`]
		/// * [`Error::InvalidHandleEncoding`]
		/// * [`Error::InvalidHandleCharacterLength`]
		#[pallet::call_index(1)]
		#[pallet::weight((T::WeightInfo::retire_handle(payload.full_handle.len() as u32), DispatchClass::Normal, Pays::No))]
		pub fn retire_handle(
			origin: OriginFor<T>,
			delegator_key: T::AccountId,
			proof: MultiSignature,
			payload: RetireHandlePayload,
		) -> DispatchResult {
			let provider_key = ensure_signed(origin)?;

			// Validation: Check for base_handle size to address potential panic condition
			ensure!(
				payload.full_handle.len() as u32 <= HANDLE_BASE_BYTES_MAX,
				Error::<T>::InvalidHandleByteLength
			);

			// Validation: The provider must already have a MSA id
			T::MsaInfoProvider::ensure_valid_msa_key(&provider_key)
				.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;

			// Validation: The delegator must already have a MSA id
			let delegator_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(&delegator_key)
				.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;

			// Validation: Verify the payload was signed
			Self::verify_signed_payload(&proof, &delegator_key, payload.encode())?;

			// Validation: The MSA must already have a handle associated with it
			let display_name_handle = MSAIdToDisplayName::<T>::try_get(delegator_msa_id)
				.map_err(|_| Error::<T>::MSAHandleDoesNotExist)?;
			let display_name_str = core::str::from_utf8(&display_name_handle)
				.map_err(|_| Error::<T>::InvalidHandleEncoding)?;

			let handle_converter = HandleConverter::new();
			let (base_handle_str, suffix_num) =
				handle_converter.split_display_name(display_name_str);

			let canonical_handle_str = handle_converter.convert_to_canonical(&base_handle_str);
			let canonical_handle_vec = canonical_handle_str.as_bytes().to_vec();
			let canonical_handle: Handle = canonical_handle_vec.try_into().unwrap();

			// Remove handle from storage but not from CanonicalBaseHandleToSuffixIndex because retired handles can't be reused
			MSAIdToDisplayName::<T>::remove(delegator_msa_id);
			CanonicalBaseHandleAndSuffixToMSAId::<T>::remove(canonical_handle, suffix_num);

			let full_handle: Handle = payload.full_handle.try_into().ok().unwrap();

			Self::deposit_event(Event::HandleRetired {
				msa_id: delegator_msa_id,
				handle: full_handle.clone(),
			});
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Retrieve `HandleResponse` for the specified `MessageSourceAccount`
		/// # Arguments
		/// * `msa_id` - The `MessageSourceAccount` to retrieve the `HandleResponse` for
		/// # Errors
		/// * [`Error::InvalidHandleEncoding`]
		/// # Returns
		/// * `HandleResponse` - The `HandleResponse` for the specified `MessageSourceAccount`
		/// * `None` - If the `MessageSourceAccount` does not have a handle
		pub fn get_handle_for_msa(msa_id: MessageSourceId) -> Option<HandleResponse> {
			let full_handle = MSAIdToDisplayName::<T>::get(msa_id);
			if full_handle.is_empty() {
				return None
			}

			// convert to string
			let full_handle_str = core::str::from_utf8(&full_handle)
				.map_err(|_| Error::<T>::InvalidHandleEncoding)
				.ok()?;

			let handle_converter = HandleConverter::new();
			let (base_handle_str, suffix) = handle_converter.split_display_name(full_handle_str);
			let base_handle = base_handle_str.as_bytes().to_vec();
			let canonical_handle_str = handle_converter.convert_to_canonical(&base_handle_str);
			let canonical_handle = canonical_handle_str.as_bytes().to_vec();
			Some(HandleResponse { base_handle, suffix, canonical_handle })
		}

		/// Retrieve `count` of suffixes for specified base `handle`
		/// # Arguments
		/// * `handle` - The base handle to retrieve the suffixes for
		/// * `count` - The number of suffixes to retrieve
		/// # Returns
		/// * `Vec<u16>` - The suffixes for the specified `handle`
		pub fn get_next_suffixes(handle: Vec<u8>, count: u16) -> Vec<HandleSuffix> {
			let mut suffixes: Vec<u16> = vec![];
			let base_handle: Handle = handle.try_into().unwrap_or_default();
			let base_handle_str = core::str::from_utf8(&base_handle).unwrap_or("");

			let handle_converter = HandleConverter::new();
			let canonical_handle_str = handle_converter.convert_to_canonical(&base_handle_str);
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

			suffixes
		}

		/// Create a full handle from a base handle and an index into the suffix sequence
		/// # Arguments
		/// * `base_handle_str` - The base handle (as a string slice) to create the full handle from
		/// * `suffix_sequence_index` - The suffix to create the full handle from
		/// # Returns
		/// * `Handle` - The full handle
		pub fn create_full_handle(
			base_handle_str: &str,
			suffix_sequence_index: HandleSuffix,
		) -> Handle {
			// Convert base display handle into a canonical display handle
			let handle_converter = HandleConverter::new();
			let canonical_handle_str = handle_converter.convert_to_canonical(&base_handle_str);

			// Generate suffix from index into the suffix sequence
			let suffix = Self::generate_suffix_for_canonical_handle(
				&canonical_handle_str,
				suffix_sequence_index as usize,
			);

			// Compose the full display handle from the base handle, "." delimeter and suffix
			let mut full_handle_vec: Vec<u8> = vec![];
			full_handle_vec.extend(base_handle_str.as_bytes());
			full_handle_vec.extend(b"."); // The delimeter
			let mut buff = [0u8; SUFFIX_MAX_DIGITS];
			full_handle_vec.extend(suffix.numtoa(10, &mut buff)); // Use base 10

			let full_handle: Handle = full_handle_vec.try_into().ok().unwrap();
			full_handle
		}
	}
}
