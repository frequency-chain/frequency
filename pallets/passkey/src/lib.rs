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
use sp_runtime::traits::Dispatchable;

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
			+ IsSubType<Call<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// PlaceHolder error
		PlaceHolderError,
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
}

impl<T: Config> Pallet<T> {}
