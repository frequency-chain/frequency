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

use common_primitives::utils::wrap_binary_data;
use frame_support::{
	dispatch::{DispatchInfo, GetDispatchInfo, PostDispatchInfo},
	pallet_prelude::*,
	traits::{Contains, IsSubType},
};
use frame_system::pallet_prelude::*;
use sp_runtime::{
	traits::{Convert, Dispatchable, One, SignedExtension, Verify},
	AccountId32, MultiSignature,
};
use sp_std::{vec, vec::Vec};

use common_runtime::extensions::check_nonce::CheckNonce;

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
			// + IsSubType<Call<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// AccountId truncated to 32 bytes
		type ConvertIntoAccountId32: Convert<Self::AccountId, AccountId32>;

		/// Filters the inner calls for passkey which is set in runtime
		type PasskeyCallFilter: Contains<<Self as Config>::RuntimeCall>;
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
			let overhead = T::WeightInfo::pre_dispatch();
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
	impl<T: Config> ValidateUnsigned for Pallet<T>
	where
		<T as frame_system::Config>::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
	{
		type Call = Call<T>;
		fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
			let payload = Self::filter_valid_calls(&call)?;
			Self::validate_signatures(&payload)?;
			Self::validate_nonce(&payload)
		}

		fn pre_dispatch(call: &Self::Call) -> Result<(), TransactionValidityError> {
			Self::validate_unsigned(TransactionSource::InBlock, call)?;

			let payload = Self::filter_valid_calls(&call)?;
			Self::pre_dispatch_nonce(&payload)
		}
	}
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
struct PasskeyNonce<T: Config>(pub T::Nonce);

impl<T: Config> PasskeyNonce<T>
where
	<T as frame_system::Config>::RuntimeCall:
		Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
{
	pub fn new(nonce: T::Nonce) -> Self {
		Self(nonce)
	}

	pub fn validate(
		&self,
		who: &T::AccountId,
		call: <T as frame_system::Config>::RuntimeCall,
		info: &DispatchInfo,
		len: usize,
	) -> TransactionValidity {
		let some = CheckNonce::<T>::from(self.0);
		some.validate(who, &call, info, len)
	}
}

impl<T: Config> Pallet<T>
where
	<T as frame_system::Config>::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
{
	fn filter_valid_calls(call: &Call<T>) -> Result<PasskeyPayload<T>, TransactionValidityError> {
		match call {
			Call::proxy { payload }
				if T::PasskeyCallFilter::contains(&payload.clone().passkey_call.call) =>
			{
				return Ok(payload.clone())
			},
			_ => return Err(InvalidTransaction::Call.into()),
		}
	}

	fn validate_nonce(payload: &PasskeyPayload<T>) -> TransactionValidity {
		let who = &payload.passkey_call.account_id.clone();
		let nonce = payload.passkey_call.account_nonce;

		let some_call: &<T as Config>::RuntimeCall = &payload.passkey_call.call;
		let info = &some_call.get_dispatch_info();

		let passkey_nonce = PasskeyNonce::<T>::new(nonce);
		passkey_nonce.validate(who, some_call.clone().into(), info, 0usize)
	}

	fn pre_dispatch_nonce(payload: &PasskeyPayload<T>) -> Result<(), TransactionValidityError> {
		let who = &payload.passkey_call.account_id.clone();
		let nonce = payload.passkey_call.account_nonce;

		// Get TOKEN account from "who" key
		let mut account = frame_system::Account::<T>::get(who);

		// The default account (no account) has a nonce of 0.
		// If account nonce is not equal to the tx nonce, the tx is invalid.  Therefore, check if it is a stale or future tx.
		if nonce != account.nonce {
			return Err(if nonce < account.nonce {
				InvalidTransaction::Stale
			} else {
				InvalidTransaction::Future
			}
			.into());
		}

		// Increment account nonce by 1
		account.nonce += T::Nonce::one();

		frame_system::Account::<T>::insert(who, account);

		Ok(())
	}

	fn validate_signatures(payload: &PasskeyPayload<T>) -> TransactionValidity {
		let signed_data = payload.passkey_public_key;
		let signature = payload.passkey_call.account_ownership_proof.clone();
		let signer = &payload.passkey_call.account_id;
		match Self::check_account_signature(signer, &signed_data.into(), &signature) {
			Ok(_) => Ok(ValidTransaction::default()),
			Err(_e) => InvalidTransaction::BadSigner.into(),
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
		let key = T::ConvertIntoAccountId32::convert((*signer).clone());
		let signed_payload: Vec<u8> = wrap_binary_data(signed_data.clone().into());

		let verified = signature.verify(&signed_payload[..], &key);
		if verified {
			Ok(())
		} else {
			Err(Error::<T>::InvalidAccountSignature.into())
		}
	}
}
