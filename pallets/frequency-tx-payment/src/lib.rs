//! # Frequency-Transactions Pallet
//! The Frequency-transaction pallet allows users to transact using either capacity or token.
//!
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
//!
//! ## Overview
//! Capacity is a refillable resource that can be used to make transactions on the network.
//! This pallets allows users to toggle between submitting transactions with capacity or tokens.
//! Note that tipping for capacity transactions is not allowed and is set to zero. Users who
//! use tokens to transact can continue to use a tip.
//!
//! The Frequency-transaction pallet provides functions for:
//!
//! - pay_with_capacity
//!

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::{DispatchInfo, Dispatchable, GetDispatchInfo, PostDispatchInfo},
	pallet_prelude::*,
	traits::{IsSubType, IsType},
	DefaultNoBound,
};
use frame_system::pallet_prelude::*;

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::prelude::*;

use sp_runtime::{
	traits::{DispatchInfoOf, PostDispatchInfoOf, SignedExtension, Zero},
	transaction_validity::{TransactionValidity, TransactionValidityError},
	FixedPointOperand,
};

use common_primitives::capacity::Nontransferable;

use pallet_transaction_payment::OnChargeTransaction;

/// Type aliases used for interaction with `OnChargeTransaction`.
pub(crate) type OnChargeTransactionOf<T> =
	<T as pallet_transaction_payment::Config>::OnChargeTransaction;

/// Balance type alias.
pub(crate) type BalanceOf<T> = <OnChargeTransactionOf<T> as OnChargeTransaction<T>>::Balance;

/// Liquidity info type alias (imbalances).
pub(crate) type LiquidityInfoOf<T> =
	<OnChargeTransactionOf<T> as OnChargeTransaction<T>>::LiquidityInfo;

/// Capacity Balance type
pub(crate) type CapacityOf<T> = <T as Config>::Capacity;

/// Capacity Balance alias
pub(crate) type CapacityBalanceOf<T> = <CapacityOf<T> as Nontransferable>::Balance;

/// Used to pass the initial payment info from pre- to post-dispatch.
#[derive(Encode, Decode, DefaultNoBound, TypeInfo)]
pub enum InitialPayment<T: Config> {
	/// No initial fee was paid.
	#[default]
	Free,
	/// The initial fee was payed in the native currency.
	Token(LiquidityInfoOf<T>),
	/// The initial fee was paid in an asset.
	Capacity,
}

#[cfg(feature = "std")]
impl<T: Config> InitialPayment<T> {
	pub fn is_free(&self) -> bool {
		match *self {
			InitialPayment::Free => true,
			_ => false,
		}
	}

	pub fn is_capacity(&self) -> bool {
		match *self {
			InitialPayment::Capacity => true,
			_ => false,
		}
	}

	pub fn is_token(&self) -> bool {
		match *self {
			InitialPayment::Token(_) => true,
			_ => false,
		}
	}
}

impl<T: Config> sp_std::fmt::Debug for InitialPayment<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		match *self {
			InitialPayment::Free => write!(f, "Nothing"),
			InitialPayment::Capacity => write!(f, "Token"),
			InitialPayment::Token(_) => write!(f, "Imbalance"),
		}
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

pub use pallet::*;

#[allow(dead_code)]
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	// Simple declaration of the `Pallet` type. It is placeholder we use to implement traits and
	// method.
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_transaction_payment::Config + pallet_msa::Config
	{
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The overarching call type.
		type RuntimeCall: Parameter
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin, PostInfo = PostDispatchInfo>
			+ GetDispatchInfo
			+ IsSubType<Call<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>;

		type Capacity: Nontransferable;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Dispatch the given call as a sub_type of pay_with_capacity. Calls dispatched in this
		/// fashion, if allowed, will pay with Capacity,
		#[pallet::call_index(0)]
		#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(dispatch_info.weight, dispatch_info.class)
		})]
		pub fn pay_with_capacity(
			origin: OriginFor<T>,
			call: Box<<T as Config>::RuntimeCall>,
		) -> DispatchResult {
			ensure_signed(origin.clone())?;

			let _result = call.dispatch(origin);
			Ok(())
		}
	}
}

/// Require the transactor pay for themselves and maybe include a tip to gain additional priority
/// in the queue.
///
/// # Transaction Validity
///
/// This extension sets the `priority` field of `TransactionValidity` depending on the amount
/// of tip being paid per weight unit.
///
/// Operational transactions will receive an additional priority bump, so that they are normally
/// considered before regular transactions.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ChargeFrqTransactionPayment<T: Config>(#[codec(compact)] BalanceOf<T>);

impl<T: Config> ChargeFrqTransactionPayment<T>
where
	BalanceOf<T>: Send + Sync + FixedPointOperand + IsType<CapacityBalanceOf<T>> + From<u64>,
	<T as frame_system::Config>::RuntimeCall:
		Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo> + IsSubType<Call<T>>,
{
	/// Utility construct from tip.
	pub fn from(tip: BalanceOf<T>) -> Self {
		Self(tip)
	}

	/// Return the tip as being chosen by the transaction sender.
	pub fn tip(&self, call: &<T as frame_system::Config>::RuntimeCall) -> BalanceOf<T> {
		match call.is_sub_type() {
			Some(Call::pay_with_capacity { .. }) => Zero::zero(),
			_ => self.0,
		}
	}

	/// Withdraws fee from eigher Capacity ledger or Token account.
	fn withdraw_fee(
		&self,
		who: &T::AccountId,
		call: &<T as frame_system::Config>::RuntimeCall,
		info: &DispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
		len: usize,
	) -> Result<(BalanceOf<T>, InitialPayment<T>), TransactionValidityError> {
		let fee =
			pallet_transaction_payment::Pallet::<T>::compute_fee(len as u32, info, self.tip(call));

		match call.is_sub_type() {
			Some(Call::pay_with_capacity { .. }) => {
				let msa_id = pallet_msa::Pallet::<T>::ensure_valid_msa_key(who).map_err(
					|_| -> TransactionValidityError { InvalidTransaction::Payment.into() },
				)?;

				T::Capacity::withdraw(msa_id, fee.into()).map_err(
					|_| -> TransactionValidityError { InvalidTransaction::Payment.into() },
				)?;

				Ok((fee, InitialPayment::Capacity))
			},
			_ => {
				if fee.is_zero() {
					return Ok((fee, InitialPayment::Free))
				}

				<OnChargeTransactionOf<T> as OnChargeTransaction<T>>::withdraw_fee(
					who,
					call,
					info,
					fee,
					self.tip(&call),
				)
				.map(|i| (fee, InitialPayment::Token(i)))
				.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })
			},
		}
	}
}

impl<T: Config> sp_std::fmt::Debug for ChargeFrqTransactionPayment<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "ChargeFrqTransactionPayment<{:?}>", self.0)
	}
	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl<T: Config> SignedExtension for ChargeFrqTransactionPayment<T>
where
	<T as frame_system::Config>::RuntimeCall:
		IsSubType<Call<T>> + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,

	BalanceOf<T>: Send + Sync + FixedPointOperand + From<u64> + IsType<CapacityBalanceOf<T>>,
{
	const IDENTIFIER: &'static str = "ChargeTransactionPayment";
	type AccountId = T::AccountId;
	type Call = <T as frame_system::Config>::RuntimeCall;
	type AdditionalSigned = ();
	type Pre = (
		// tip
		BalanceOf<T>,
		Self::AccountId,
		InitialPayment<T>,
	);

	/// Construct any additional data that should be in the signed payload of the transaction. Can
	/// also perform any pre-signature-verification checks and return an error if needed.
	fn additional_signed(&self) -> Result<(), TransactionValidityError> {
		Ok(())
	}

	/// Frequently called by the transaction queue to validate all extrinsics:
	fn validate(
		&self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> TransactionValidity {
		let (fee, _) = self.withdraw_fee(who, call, info, len)?;

		let priority = pallet_transaction_payment::ChargeTransactionPayment::<T>::get_priority(
			info,
			len,
			self.tip(call),
			fee,
		);

		Ok(ValidTransaction { priority, ..Default::default() })
	}

	/// Do any pre-flight stuff for a signed transaction.
	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		let (_fee, initial_payment) = self.withdraw_fee(who, call, info, len)?;

		Ok((self.tip(call), who.clone(), initial_payment))
	}

	/// Do any post-flight stuff for an extrinsic.
	fn post_dispatch(
		maybe_pre: Option<Self::Pre>,
		info: &DispatchInfoOf<Self::Call>,
		post_info: &PostDispatchInfoOf<Self::Call>,
		len: usize,
		result: &DispatchResult,
	) -> Result<(), TransactionValidityError> {
		if let Some((tip, who, initial_payment)) = maybe_pre {
			match initial_payment {
				InitialPayment::Token(already_withdrawn) => {
					pallet_transaction_payment::ChargeTransactionPayment::<T>::post_dispatch(
						Some((tip, who, already_withdrawn)),
						info,
						post_info,
						len,
						result,
					)?;
				},
				InitialPayment::Capacity => {
					debug_assert!(tip.is_zero(), "tip should be zero for Capacity tx.");
				},
				InitialPayment::Free => {
					// `actual_fee` should be zero here for any signed extrinsic. It would be
					// non-zero here in case of unsigned extrinsics as they don't pay fees but
					// `compute_actual_fee` is not aware of them. In both cases it's fine to just
					// move ahead without adjusting the fee, though, so we do nothing.
					debug_assert!(tip.is_zero(), "tip should be zero if initial fee was zero.");
				},
			}
		}
		Ok(())
	}
}
