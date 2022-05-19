#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::weights::{DispatchInfo, GetDispatchInfo};
use pallet_transaction_payment::{FeeDetails, OnChargeTransaction};
use sp_runtime::traits::Dispatchable;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

pub use pallet::*;

// Type aliases used for interaction with `OnChargeTransaction`.
pub(crate) type OnChargeTransactionOf<T> =
	<T as pallet_transaction_payment::Config>::OnChargeTransaction;

// Balance type alias.
pub(crate) type BalanceOf<T> = <OnChargeTransactionOf<T> as OnChargeTransaction<T>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_transaction_payment::Config {}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	impl<T: Config> Pallet<T> {
		pub fn compute_extrinsic_cost<
			Extrinsic: sp_runtime::traits::Extrinsic + GetDispatchInfo,
		>(
			unchecked_extrinsic: Extrinsic,
			len: u32,
		) -> FeeDetails<BalanceOf<T>>
		where
			T::Call: Dispatchable<Info = DispatchInfo>,
		{
		}
	}
}
