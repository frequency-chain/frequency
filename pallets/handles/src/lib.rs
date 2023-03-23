//! # Handles Pallet
//! The Handles pallet provides functionality for generating user handles.
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
#[cfg(test)]
mod tests;

// pub mod weights;

use sp_std::prelude::*;

use common_primitives::{
	handles::*,
	msa::{MessageSourceId, MsaLookup, MsaValidator},
};
use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::*, traits::Get};
use frame_system::pallet_prelude::*;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		// type WeightInfo: WeightInfo;

		/// A type that will supply MSA related information
		type MsaInfoProvider: MsaLookup + MsaValidator<AccountId = Self::AccountId>;

		/// The minimum suffix value
		#[pallet::constant]
		type HandleSuffixMin: Get<u32>;

		/// The maximum suffix value
		#[pallet::constant]
		type HandleSuffixMax: Get<u32>;
	}

	// Simple declaration of the `Pallet` type. It is placeholder we use to implement traits and
	// method.
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
	/// - Value: Cursor u32
	#[pallet::storage]
	#[pallet::getter(fn get_cursor_for_canonical)]
	pub type CanonicalBaseHandleToCursor<T: Config> =
		StorageMap<_, Blake2_128Concat, Handle, SequenceCursor, OptionQuery>;

	#[derive(PartialEq, Eq)] // for testing
	#[pallet::error]
	pub enum Error<T> {
		/// Invalid handle encoding
		InvalidHandleEncoding,
		/// Invalid handle length
		InvalidHandleLength,
		/// Suffixes exhausted
		SuffixesExhausted,
		/// Invalid MSA
		InvalidMessageSourceAccount,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Emitted when a schema is registered. [who, schemas id]
		HandleCreated {
			/// message source id of handle owner
			msa_id: MessageSourceId,
		},
	}

	fn convert_to_canonical(handle_str: &str) -> codec::alloc::string::String {
		let mut normalized = unicode_security::skeleton(handle_str).collect::<codec::alloc::string::String>();
		normalized.make_ascii_lowercase();
		normalized
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Claim handle
		///
		#[pallet::call_index(0)]
		#[pallet::weight(1000)]
		pub fn claim_handle(origin: OriginFor<T>, base_handle: Handle) -> DispatchResult {
			let delegator_key = ensure_signed(origin)?;

			// Validation:  The caller must already have a MSA id
			let caller_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(&delegator_key)
				.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;

			// Validation:  The MSA must not already have a handle associated with it

			// Validation:  The base handle MUST be UTF-8 encoded.
			let base_handle_str =
				core::str::from_utf8(&base_handle).map_err(|_| Error::<T>::InvalidHandleEncoding)?;

			// Validation:  The handle length must be valid.
			// WARNING: This can panic.  Need to handle it!
			let len = base_handle_str.chars().count() as u32;
			ensure!(
				len < HANDLE_BASE_BYTES_MIN || len > HANDLE_BASE_BYTES_MAX,
				Error::<T>::InvalidHandleLength
			);

			let canonical_handle_vec = convert_to_canonical(base_handle_str).as_bytes().to_vec();
			let canonical_handle: Handle = canonical_handle_vec.try_into().unwrap();
			CanonicalBaseHandleToCursor::<T>::insert(canonical_handle, 0);
			Self::deposit_event(Event::HandleCreated { msa_id: caller_msa_id });
			Ok(())
		}
	}
}
