//! # Transaction Fee Pallet
//!
//! The TxFee pallet provides RPCs for estimating transaction fees
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//! ### Dispatchable Functions
//!
//! None
//!
#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

use frame_support::weights::{DispatchInfo, GetDispatchInfo};
use pallet_transaction_payment::{FeeDetails, OnChargeTransaction};
use sp_runtime::{traits::Dispatchable, FixedPointOperand};

#[cfg(test)]
mod tests;

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

	#[pallet::error]
	pub enum Error<T> {}

	impl<T: Config> Pallet<T> {
		/// Computer fee details of a given extrinsic
		pub fn compute_extrinsic_cost<Extrinsic: sp_runtime::traits::Extrinsic + GetDispatchInfo>(
			unchecked_extrinsic: Extrinsic,
			len: u32,
		) -> FeeDetails<BalanceOf<T>>
		where
			T::Call: Dispatchable<Info = DispatchInfo>,
			BalanceOf<T>: Send + Sync + From<u64> + FixedPointOperand,
		{
			let tip = 0_u32.into();
			let dispatch_info =
				<Extrinsic as GetDispatchInfo>::get_dispatch_info(&unchecked_extrinsic);
			if unchecked_extrinsic.is_signed().unwrap_or(false) {
				<pallet_transaction_payment::Pallet<T>>::compute_fee_details(
					len,
					&dispatch_info,
					tip,
				)
			} else {
				// Unsigned extrinsics have no inclusion fee.
				FeeDetails { inclusion_fee: None, tip }
			}
		}
	}
}
