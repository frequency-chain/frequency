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

use common_primitives::payment::*;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	pallet_prelude::*,
	traits::IsSubType,
};
use frame_system::pallet_prelude::*;
use pallet_transaction_payment::OnChargeTransaction;
use sp_runtime::{
	traits::{Convert, DispatchInfoOf, Dispatchable, Verify, Zero},
	transaction_validity::{TransactionValidity, TransactionValidityError},
};
use sp_std::prelude::*;

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
	use frame_support::dispatch::DispatchInfo;
	use sp_runtime::{AccountId32, MultiSignature};

	use super::*;

	/// the storage version for this pallet
	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
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
		fn validate_unsigned(
			_source: TransactionSource,
			_call: &Self::Call,
		) -> TransactionValidity {
			Ok(ValidTransaction::default())
		}
	}

	impl<T: Config> Pallet<T>
	where
		<T as frame_system::Config>::RuntimeCall:
			IsSubType<Call<T>> + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
	{
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
		#[allow(unused)]
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

		/// Withdraws transaction fee paid with tokens.
		#[allow(unused)]
		fn withdraw_token_fee(
			&self,
			who: &T::AccountId,
			call: &<T as frame_system::Config>::RuntimeCall,
			info: &DispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
			len: usize,
		) -> Result<(BalanceOf<T>, InitialPayment<T>), TransactionValidityError> {
			let tip = Zero::zero();
			let fee = pallet_transaction_payment::Pallet::<T>::compute_fee(len as u32, info, tip);
			if fee.is_zero() {
				return Ok((fee, InitialPayment::Free));
			}

			<OnChargeTransactionOf<T> as OnChargeTransaction<T>>::withdraw_fee(
				who, call, info, fee, tip,
			)
			.map(|i| (fee, InitialPayment::Token(i)))
			.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })
		}
	}
}

impl<T: Config> Pallet<T> {}
