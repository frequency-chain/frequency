//! Using Passkeys to execute transactions
//!
//! ## Quick Links
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
#![doc = include_str!("../README.md")]
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

use frame_support::{
	dispatch::{DispatchResult, GetDispatchInfo, PostDispatchInfo},
	pallet_prelude::*,
	traits::IsSubType,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{Convert, Dispatchable, Verify};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use common_primitives::utils::wrap_binary_data;
	use sp_runtime::{AccountId32, MultiSignature};

	use super::*;

	/// the storage version for this pallet
	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The overarching call type.
		type RuntimeCall: Parameter
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
			+ GetDispatchInfo
			+ From<frame_system::Call<Self>>
			+ IsSubType<Call<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Convert AccountId to AccountId32
		type ConvertIntoAccountId32: Convert<Self::AccountId, AccountId32>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// InvalidAccountSignature
		InvalidAccountSignature,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// PlaceHolder event
		PlaceHolderEvent,
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// proxy call
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::proxy())]
		pub fn proxy(_origin: OriginFor<T>) -> DispatchResult {
			Self::deposit_event(Event::PlaceHolderEvent);
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Check the signature on passkey public key by the account id
		pub fn check_account_signature(
			account_id: &T::AccountId,
			passkey_public_key: Vec<u8>,
			account_ownership_proof: MultiSignature,
		) -> DispatchResult {
			let key: AccountId32 = T::ConvertIntoAccountId32::convert(account_id.clone());
			// check signature for the account
			let passkey_publickey_payload = wrap_binary_data(passkey_public_key.clone().into());

			let verified = account_ownership_proof.verify(&passkey_publickey_payload[..], &key);
			if verified {
				Ok(())
			} else {
				Err(Error::<T>::InvalidAccountSignature.into())
			}
		}
	}
}

impl<T: Config> Pallet<T> {}
