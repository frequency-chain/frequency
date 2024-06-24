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
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	pallet_prelude::*,
	traits::IsSubType,
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{Convert, Dispatchable, Verify};
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// defines all new types for this pallet
pub mod types;
pub use types::*;

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

		/// AccountId truncated to 32 bytes
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
		/// When a passkey transaction is successfully executed
		TransactionExecutionSuccess {
			/// transaction account id
			account_id: T::AccountId,
		},
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// proxy call
		#[pallet::call_index(0)]
		#[pallet::weight({
			let dispatch_info = payload.passkey_call.call.get_dispatch_info();
			// TODO: calculate overhead after all validations
			let overhead = T::WeightInfo::proxy();
			let total = overhead.saturating_add(dispatch_info.weight);
			(total, dispatch_info.class)
		})]
		pub fn proxy(
			origin: OriginFor<T>,
			payload: PasskeyPayload<T>,
		) -> DispatchResultWithPostInfo {
			ensure_none(origin)?;
			let transaction_account_id = payload.passkey_call.account_id.clone();
			let main_origin = T::RuntimeOrigin::from(frame_system::RawOrigin::Signed(
				transaction_account_id.clone(),
			));
			let result = payload.passkey_call.call.dispatch(main_origin);
			if result.is_ok() {
				Self::deposit_event(Event::TransactionExecutionSuccess {
					account_id: transaction_account_id,
				});
			}
			result
		}
	}

	#[pallet::validate_unsigned]
	impl<T: Config> ValidateUnsigned for Pallet<T> {
		type Call = Call<T>;
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			Self::validate_signatures(call)?;
			Ok(ValidTransaction::default())
		}
	}

	impl<T: Config> Pallet<T> {
		fn validate_signatures(call: &Call<T>) -> TransactionValidity {
			match call {
				Call::proxy { payload } => {
					let signed_data = payload.passkey_public_key;
					let signature = payload.passkey_call.account_ownership_proof.clone();
					let signer = &payload.passkey_call.account_id;
					match Self::check_account_signature(signer, &signed_data.into(), &signature) {
						Ok(_) => Ok(ValidTransaction::default()),
						Err(_e) => InvalidTransaction::BadSigner.into(),
					}
				},
				_ => InvalidTransaction::Call.into(),
			}
		}

		/// Check the signature on passkey public key by the account id
		/// Returns Ok(()) if the signature is valid
		/// Returns Err(InvalidAccountSignature) if the signature is invalid
		/// # Arguments
		/// * `signer` - The account id of the signer
		/// * `signed_data` - The signed data
		/// * `signature` - The signature
		/// # Return
		/// * `Ok(())` if the signature is valid
		/// * `Err(InvalidAccountSignature)` if the signature is invalid
		fn check_account_signature(
			signer: &T::AccountId,
			signed_data: &Vec<u8>,
			signature: &MultiSignature,
		) -> DispatchResult {
			let key: AccountId32 = T::ConvertIntoAccountId32::convert((*signer).clone());
			let signed_payload: Vec<u8> = wrap_binary_data(signed_data.clone().into());

			let verified = signature.verify(&signed_payload[..], &key);
			if verified {
				Ok(())
			} else {
				Err(Error::<T>::InvalidAccountSignature.into())
			}
		}
	}
}

impl<T: Config> Pallet<T> {}
