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
//! The key that is used to pay with Capacity must have a minimum balance of the existential deposit.
//!
//! The Frequency-transaction pallet provides functions for:
//!
//! - pay_with_capacity
//!

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchInfo, Dispatchable, GetDispatchInfo, PostDispatchInfo},
	pallet_prelude::*,
	traits::{Currency, IsSubType, IsType},
	weights::{Weight, WeightToFee},
	DefaultNoBound,
};
use frame_system::pallet_prelude::*;
use pallet_transaction_payment::OnChargeTransaction;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{DispatchInfoOf, PostDispatchInfoOf, SignedExtension, Zero},
	transaction_validity::{TransactionValidity, TransactionValidityError},
	FixedPointOperand, Saturating,
};
use sp_std::prelude::*;

use common_primitives::{
	capacity::{Nontransferable, Replenishable},
	node::UtilityProvider,
};
pub use pallet::*;
pub use weights::*;

mod payment;
pub use payment::*;

pub use types::GetStableWeight;
pub mod types;

pub mod capacity_stable_weights;

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

pub(crate) type ChargeCapacityBalanceOf<T> =
	<<T as Config>::OnChargeCapacityTransaction as OnChargeCapacityTransaction<T>>::Balance;

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

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

#[allow(dead_code)]
#[frame_support::pallet]
pub mod pallet {
	use super::*;

	// Simple declaration of the `Pallet` type. It is placeholder we use to implement traits and
	// method.
	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

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

		/// The type that replenishes and keeps capacity balances.
		type Capacity: Replenishable + Nontransferable;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// The type that checks what transactions are capacity with their stable weights.
		type CapacityCalls: GetStableWeight<<Self as Config>::RuntimeCall, Weight>;

		/// Charge Capacity for transaction payments.
		type OnChargeCapacityTransaction: OnChargeCapacityTransaction<Self>;

		/// The maxmimum number of capacity calls that can be batched together.
		#[pallet::constant]
		type MaximumCapacityBatchLength: Get<u8>;

		type BatchProvider: UtilityProvider<OriginFor<Self>, <Self as Config>::RuntimeCall>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {
		/// The maximum amount of requested batched calls was exceeded
		BatchedCallAmountExceedsMaximum,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Dispatch the given call as a sub_type of pay_with_capacity. Calls dispatched in this
		/// fashion, if allowed, will pay with Capacity.
		// The weight calculation is a temporary adjustment because overhead benchmarks do not account
		// for capacity calls.  We count reads and writes for a pay_with_capacity call,
		// then subtract one of each for regular transactions since overhead benchmarks account for these.
		#[pallet::call_index(0)]
		#[pallet::weight({
		let dispatch_info = call.get_dispatch_info();
		let capacity_overhead = Pallet::<T>::get_capacity_overhead_weight();
		let total = capacity_overhead.saturating_add(dispatch_info.weight);
		(< T as Config >::WeightInfo::pay_with_capacity().saturating_add(total), dispatch_info.class)
		})]
		pub fn pay_with_capacity(
			origin: OriginFor<T>,
			call: Box<<T as Config>::RuntimeCall>,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin.clone())?;

			call.dispatch(origin)
		}

		/// Dispatch the given call as a sub_type of pay_with_capacity_batch_all. Calls dispatched in this
		/// fashion, if allowed, will pay with Capacity.
		#[pallet::call_index(1)]
		#[pallet::weight({
		let dispatch_infos = calls.iter().map(|call| call.get_dispatch_info()).collect::<Vec<_>>();
		let dispatch_weight = dispatch_infos.iter()
				.map(|di| di.weight)
				.fold(Weight::zero(), |total: Weight, weight: Weight| total.saturating_add(weight));

		let capacity_overhead = Pallet::<T>::get_capacity_overhead_weight();
		let total = capacity_overhead.saturating_add(dispatch_weight);
		(< T as Config >::WeightInfo::pay_with_capacity_batch_all(calls.len() as u32).saturating_add(total), DispatchClass::Normal)
		})]
		pub fn pay_with_capacity_batch_all(
			origin: OriginFor<T>,
			calls: Vec<<T as Config>::RuntimeCall>,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin.clone())?;
			ensure!(
				calls.len() <= T::MaximumCapacityBatchLength::get().into(),
				Error::<T>::BatchedCallAmountExceedsMaximum
			);

			T::BatchProvider::batch_all(origin, calls)
		}
	}
}

impl<T: Config> Pallet<T> {
	// The weight calculation is a temporary adjustment because overhead benchmarks do not account
	// for capacity calls.  We count reads and writes for a pay_with_capacity call,
	// then subtract one of each for regular transactions since overhead benchmarks account for these.
	//   Storage: Msa PublicKeyToMsaId (r:1)
	//   Storage: Capacity CapacityLedger(r:1, w:2)
	//   Storage: Capacity CurrentEpoch(r:1) ? maybe cached in on_initialize
	//   Storage: System Account(r:1)
	//   Total (r: 4-1=3, w: 2-1=1)
	pub fn get_capacity_overhead_weight() -> Weight {
		T::DbWeight::get().reads(2).saturating_add(T::DbWeight::get().writes(1))
	}

	pub fn compute_capacity_fee(
		len: u32,
		class: DispatchClass,
		extrinsic_weight: Weight,
	) -> BalanceOf<T> {
		let weight_fee = Self::weight_to_fee(extrinsic_weight);

		let len_fee = Self::length_to_fee(len);
		let base_fee = Self::weight_to_fee(T::BlockWeights::get().get(class).base_extrinsic);

		let fee = base_fee.saturating_add(weight_fee).saturating_add(len_fee);
		fee
	}

	/// Compute the length portion of a fee by invoking the configured `LengthToFee` impl.
	pub fn length_to_fee(length: u32) -> BalanceOf<T> {
		T::LengthToFee::weight_to_fee(&Weight::from_ref_time(length as u64))
	}

	/// Compute the unadjusted portion of the weight fee by invoking the configured `WeightToFee`
	/// impl. Note that the input `weight` is capped by the maximum block weight before computation.
	pub fn weight_to_fee(weight: Weight) -> BalanceOf<T> {
		// cap the weight to the maximum defined in runtime, otherwise it will be the
		// `Bounded` maximum of its data type, which is not desired.
		let capped_weight = weight.min(T::BlockWeights::get().max_block);
		T::WeightToFee::weight_to_fee(&capped_weight)
	}
}

/// Custom Transaction Validity Errors for ChargeFrqTransactionPayment
pub enum ChargeFrqTransactionPaymentError {
	/// The call is not eligible to be paid for with Capacity
	CallIsNotCapacityEligible,
	/// The account key is not associated with an MSA
	InvalidMsaKey,
	/// The Capacity Target does not exist
	TargetCapacityNotFound,
	/// The minimum balance required for keys used to pay with Capacity
	BelowMinDeposit,
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

impl ChargeFrqTransactionPaymentError {
	pub fn into(self) -> TransactionValidityError {
		TransactionValidityError::from(InvalidTransaction::Custom(self as u8))
	}
}

impl<T: Config> ChargeFrqTransactionPayment<T>
where
	BalanceOf<T>: Send + Sync + FixedPointOperand + IsType<ChargeCapacityBalanceOf<T>>,
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
			Some(Call::pay_with_capacity { .. }) |
			Some(Call::pay_with_capacity_batch_all { .. }) => Zero::zero(),
			_ => self.0,
		}
	}

	/// Withdraws fee from either Capacity ledger or Token account.
	fn withdraw_fee(
		&self,
		who: &T::AccountId,
		call: &<T as frame_system::Config>::RuntimeCall,
		info: &DispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
		len: usize,
	) -> Result<(BalanceOf<T>, InitialPayment<T>), TransactionValidityError> {
		match call.is_sub_type() {
			Some(Call::pay_with_capacity { call }) =>
				self.withdraw_capacity_fee(who, &vec![*call.clone()], len),
			Some(Call::pay_with_capacity_batch_all { calls }) =>
				self.withdraw_capacity_fee(who, calls, len),
			_ => self.withdraw_token_fee(who, call, info, len, self.tip(call)),
		}
	}

	/// Withdraws the transaction fee paid in Capacity using a key associated to an MSA.
	fn withdraw_capacity_fee(
		&self,
		key: &T::AccountId,
		calls: &Vec<<T as Config>::RuntimeCall>,
		len: usize,
	) -> Result<(BalanceOf<T>, InitialPayment<T>), TransactionValidityError> {
		let mut calls_weight_sum = Weight::zero();

		for call in calls {
			let call_weight = T::CapacityCalls::get_stable_weight(call)
				.ok_or(ChargeFrqTransactionPaymentError::CallIsNotCapacityEligible.into())?;
			calls_weight_sum = calls_weight_sum.saturating_add(call_weight);
		}

		let fee =
			Pallet::<T>::compute_capacity_fee(len as u32, DispatchClass::Normal, calls_weight_sum);

		let fee = T::OnChargeCapacityTransaction::withdraw_fee(key, fee.into())?;

		Ok((fee.into(), InitialPayment::Capacity))
	}

	/// Withdraws transaction fee paid with tokens from an.
	fn withdraw_token_fee(
		&self,
		who: &T::AccountId,
		call: &<T as frame_system::Config>::RuntimeCall,
		info: &DispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
		len: usize,
		tip: BalanceOf<T>,
	) -> Result<(BalanceOf<T>, InitialPayment<T>), TransactionValidityError> {
		let fee = pallet_transaction_payment::Pallet::<T>::compute_fee(len as u32, info, tip);
		if fee.is_zero() {
			return Ok((fee, InitialPayment::Free))
		}

		<OnChargeTransactionOf<T> as OnChargeTransaction<T>>::withdraw_fee(
			who, call, info, fee, tip,
		)
		.map(|i| (fee, InitialPayment::Token(i)))
		.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })
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

	BalanceOf<T>: Send
		+ Sync
		+ FixedPointOperand
		+ From<u64>
		+ IsType<ChargeCapacityBalanceOf<T>>
		+ IsType<CapacityBalanceOf<T>>,
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
