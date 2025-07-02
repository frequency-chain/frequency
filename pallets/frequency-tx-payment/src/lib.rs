//! Allows transactions in alternative payment methods such as capacity
//!
//! ## Quick Links
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Runtime API: `CapacityTransactionPaymentRuntimeApi`](../pallet_frequency_tx_payment_runtime_api/trait.CapacityTransactionPaymentRuntimeApi.html)
//! - [Custom RPC API: `CapacityPaymentApiServer`](../pallet_frequency_tx_payment_rpc/trait.CapacityPaymentApiServer.html)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
#![doc = include_str!("../README.md")]
// Substrate macros are tripping the clippy::expect_used lint.
#![allow(clippy::expect_used)]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::{DispatchInfo, GetDispatchInfo, PostDispatchInfo},
	pallet_prelude::*,
	traits::{IsSubType, IsType},
	weights::{Weight, WeightToFee},
	DefaultNoBound,
};
use frame_system::pallet_prelude::*;
use pallet_transaction_payment::{FeeDetails, InclusionFee, OnChargeTransaction};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[allow(deprecated)]
use sp_runtime::{
	traits::{
		AsSystemOriginSigner, DispatchInfoOf, Dispatchable, PostDispatchInfoOf, RefundWeight,
		TransactionExtension, Zero,
	},
	transaction_validity::{
		InvalidTransaction, TransactionSource, TransactionValidityError, ValidTransaction,
	},
	DispatchResult, FixedPointOperand, Saturating,
};
extern crate alloc;
use alloc::{boxed::Box, vec, vec::Vec};
use common_primitives::{
	capacity::{Nontransferable, Replenishable},
	msa::MsaKeyProvider,
	node::UtilityProvider,
};
use core::ops::Mul;
pub use pallet::*;
use sp_runtime::Permill;
pub use weights::*;

mod payment;
pub use payment::*;

pub use types::GetStableWeight;
pub mod types;

pub mod capacity_stable_weights;

use crate::types::GetAddKeyData;
use capacity_stable_weights::CAPACITY_EXTRINSIC_BASE_WEIGHT;

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
	/// The initial fee was paid in the native currency.
	Token(LiquidityInfoOf<T>),
	/// The initial fee was paid in an asset.
	Capacity,
}

#[cfg(feature = "std")]
impl<T: Config> InitialPayment<T> {
	pub fn is_free(&self) -> bool {
		matches!(*self, InitialPayment::Free)
	}

	pub fn is_capacity(&self) -> bool {
		matches!(*self, InitialPayment::Capacity)
	}

	pub fn is_token(&self) -> bool {
		matches!(*self, InitialPayment::Token(_))
	}
}

impl<T: Config> core::fmt::Debug for InitialPayment<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		match *self {
			InitialPayment::Free => write!(f, "Nothing"),
			InitialPayment::Capacity => write!(f, "Token"),
			InitialPayment::Token(_) => write!(f, "Imbalance"),
		}
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut core::fmt::Formatter) -> core::fmt::Result {
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
	use common_primitives::msa::{MessageSourceId, MsaKeyProvider};

	// Simple declaration of the `Pallet` type. It is placeholder we use to implement traits and
	// method.
	#[pallet::pallet]
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

		type MsaKeyProvider: MsaKeyProvider<AccountId = Self::AccountId>;
		type MsaCallFilter: GetAddKeyData<
			<Self as Config>::RuntimeCall,
			Self::AccountId,
			MessageSourceId,
		>;
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
		let total = capacity_overhead.saturating_add(dispatch_info.call_weight);
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
				.map(|di| di.call_weight)
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
	/// The weight calculation is a temporary adjustment because overhead benchmarks do not account
	/// for capacity calls.  We count reads and writes for a pay_with_capacity call,
	/// then subtract one of each for regular transactions since overhead benchmarks account for these.
	///   Storage: Msa PublicKeyToMsaId (r:1)
	///   Storage: Capacity CapacityLedger(r:1, w:2)
	///   Storage: Capacity CurrentEpoch(r:1) ? maybe cached in on_initialize
	///   Storage: System Account(r:1)
	///   Total (r: 4-1=3, w: 2-1=1)
	pub fn get_capacity_overhead_weight() -> Weight {
		T::DbWeight::get().reads(2).saturating_add(T::DbWeight::get().writes(1))
	}

	/// Compute the capacity fee for a transaction.
	/// The fee is computed as the sum of the following:
	/// - the weight fee, which is proportional to the weight of the transaction.
	/// - the length fee, which is proportional to the length of the transaction;
	/// - the base fee, which accounts for the overhead of an extrinsic.
	///
	/// NOTE: Changing CAPACITY_EXTRINSIC_BASE_WEIGHT will also change static capacity weights.
	pub fn compute_capacity_fee(len: u32, extrinsic_weight: Weight) -> BalanceOf<T> {
		let weight_fee = Self::weight_to_fee(extrinsic_weight);

		let len_fee = Self::length_to_fee(len);
		let base_fee = Self::weight_to_fee(CAPACITY_EXTRINSIC_BASE_WEIGHT);

		base_fee.saturating_add(weight_fee).saturating_add(len_fee)
	}

	/// Compute the capacity fee details for a transaction.
	/// # Arguments
	/// * `runtime_call` - The runtime call to be dispatched.
	/// * `weight` - The weight of the transaction.
	/// * `len` - The length of the transaction.
	///
	/// # Returns
	/// `FeeDetails` - The fee details for the transaction.
	pub fn compute_capacity_fee_details(
		runtime_call: &<T as Config>::RuntimeCall,
		dispatch_weight: &Weight,
		len: u32,
	) -> FeeDetails<BalanceOf<T>> {
		let calls = T::CapacityCalls::get_inner_calls(runtime_call)
			.expect("A collection of calls is expected at minimum one.");

		let mut calls_weight_sum = Weight::zero();
		for inner_call in calls {
			let call_weight = T::CapacityCalls::get_stable_weight(inner_call).unwrap_or_default();
			calls_weight_sum = calls_weight_sum.saturating_add(call_weight);
		}

		let mut fees = FeeDetails { inclusion_fee: None, tip: Zero::zero() };
		if !calls_weight_sum.is_zero() {
			if let Some(weight) = calls_weight_sum.checked_add(dispatch_weight) {
				let weight_fee = Self::weight_to_fee(weight);
				let len_fee = Self::length_to_fee(len);
				let base_fee = Self::weight_to_fee(CAPACITY_EXTRINSIC_BASE_WEIGHT);

				let tip = Zero::zero();
				fees = FeeDetails {
					inclusion_fee: Some(InclusionFee {
						base_fee,
						len_fee,
						adjusted_weight_fee: weight_fee,
					}),
					tip,
				};
			}
		}
		fees
	}
	/// Compute the length portion of a fee by invoking the configured `LengthToFee` impl.
	pub fn length_to_fee(length: u32) -> BalanceOf<T> {
		T::LengthToFee::weight_to_fee(&Weight::from_parts(length as u64, 0))
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
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo)]
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

	// simulates fee calculation and withdrawal without applying any changes
	fn dryrun_withdraw_fee(
		&self,
		who: &T::AccountId,
		call: &<T as frame_system::Config>::RuntimeCall,
		info: &DispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
		len: usize,
	) -> Result<BalanceOf<T>, TransactionValidityError> {
		match call.is_sub_type() {
			Some(Call::pay_with_capacity { call }) =>
				self.dryrun_withdraw_capacity_fee(who, &vec![*call.clone()], len),

			Some(Call::pay_with_capacity_batch_all { calls }) =>
				self.dryrun_withdraw_capacity_fee(who, calls, len),

			_ => self.dryrun_withdraw_token_fee(who, call, info, len, self.tip(call)),
		}
	}

	fn dryrun_withdraw_capacity_fee(
		&self,
		who: &T::AccountId,
		calls: &Vec<<T as Config>::RuntimeCall>,
		len: usize,
	) -> Result<BalanceOf<T>, TransactionValidityError> {
		let mut calls_weight_sum = Weight::zero();
		for call in calls {
			let call_weight = T::CapacityCalls::get_stable_weight(call)
				.ok_or(ChargeFrqTransactionPaymentError::CallIsNotCapacityEligible.into())?;
			calls_weight_sum = calls_weight_sum.saturating_add(call_weight);
		}
		let fee = Pallet::<T>::compute_capacity_fee(len as u32, calls_weight_sum);
		T::OnChargeCapacityTransaction::can_withdraw_fee(who, fee.into())?;
		Ok(fee)
	}

	fn dryrun_withdraw_token_fee(
		&self,
		who: &T::AccountId,
		call: &<T as frame_system::Config>::RuntimeCall,
		info: &DispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
		len: usize,
		tip: BalanceOf<T>,
	) -> Result<BalanceOf<T>, TransactionValidityError> {
		let fee = pallet_transaction_payment::Pallet::<T>::compute_fee(len as u32, info, tip);
		if fee.is_zero() {
			return Ok(Default::default());
		}
		T::OnChargeTransaction::can_withdraw_fee(who, call, info, fee, tip)?;
		Ok(fee)
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
		let mut subsidized_calls_weight_sum = Weight::zero();

		for call in calls {
			let call_weight = T::CapacityCalls::get_stable_weight(call)
				.ok_or(ChargeFrqTransactionPaymentError::CallIsNotCapacityEligible.into())?;
			calls_weight_sum = calls_weight_sum.saturating_add(call_weight);

			if self.call_is_adding_eligible_key_to_msa(call) {
				subsidized_calls_weight_sum =
					subsidized_calls_weight_sum.saturating_add(call_weight);
			}
		}
		let capacity_fee = Pallet::<T>::compute_capacity_fee(len as u32, calls_weight_sum)
			.saturating_sub(Self::subsidized_calls_reduction(len, subsidized_calls_weight_sum));
		let fee = T::OnChargeCapacityTransaction::withdraw_fee(key, capacity_fee.into())?;

		Ok((fee.into(), InitialPayment::Capacity))
	}

	// Give a 70% discount for eligible calls
	fn subsidized_calls_reduction(len: usize, eligible_call_weight: Weight) -> BalanceOf<T> {
		if eligible_call_weight.is_zero() {
			0u32.into()
		} else {
			let reduction: Permill = Permill::from_percent(70u32);
			reduction.mul(Pallet::<T>::compute_capacity_fee(len as u32, eligible_call_weight))
		}
	}

	fn call_is_adding_eligible_key_to_msa(&self, call: &<T as Config>::RuntimeCall) -> bool {
		if let Some((owner_account_id, new_account_id, msa_id)) =
			T::MsaCallFilter::get_add_key_data(call)
		{
			return T::MsaKeyProvider::key_eligible_for_subsidized_addition(
				owner_account_id,
				new_account_id,
				msa_id,
			);
		}
		false
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
			return Ok((fee, InitialPayment::Free));
		}

		<OnChargeTransactionOf<T> as OnChargeTransaction<T>>::withdraw_fee(
			who, call, info, fee, tip,
		)
		.map(|i| (fee, InitialPayment::Token(i)))
		.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })
	}
}

impl<T: Config> core::fmt::Debug for ChargeFrqTransactionPayment<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		write!(f, "ChargeFrqTransactionPayment<{:?}>", self.0)
	}
	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut core::fmt::Formatter) -> core::fmt::Result {
		Ok(())
	}
}

/// The info passed between the validate and prepare steps for the `ChargeFrqTransactionPayment` extension.
pub enum Val<T: Config> {
	Charge { tip: BalanceOf<T>, who: T::AccountId, fee: BalanceOf<T> },
	NoCharge,
}

/// The info passed between the prepare and post-dispatch steps for the `ChargeFrqTransactionPayment` extension.
pub enum Pre<T: Config> {
	Charge {
		tip: BalanceOf<T>,
		who: T::AccountId,
		initial_payment: InitialPayment<T>,
		weight: Weight,
	},
	NoCharge {
		refund: Weight,
	},
}

impl<T: Config> TransactionExtension<<T as frame_system::Config>::RuntimeCall>
	for ChargeFrqTransactionPayment<T>
where
	<T as frame_system::Config>::RuntimeCall:
		IsSubType<Call<T>> + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
	BalanceOf<T>: Send
		+ Sync
		+ FixedPointOperand
		+ From<u64>
		+ IsType<ChargeCapacityBalanceOf<T>>
		+ IsType<CapacityBalanceOf<T>>,
	<T as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<T::AccountId> + Clone,
{
	const IDENTIFIER: &'static str = "ChargeTransactionPayment";
	type Implicit = ();
	type Val = Val<T>;
	type Pre = Pre<T>;

	fn weight(&self, _call: &<T as frame_system::Config>::RuntimeCall) -> Weight {
		// TODO: Benchmark this
		Weight::zero()
	}

	fn validate(
		&self,
		origin: <T as frame_system::Config>::RuntimeOrigin,
		call: &<T as frame_system::Config>::RuntimeCall,
		info: &DispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
		len: usize,
		_self_implicit: Self::Implicit,
		_inherited_implication: &impl Encode,
		_source: TransactionSource,
	) -> sp_runtime::traits::ValidateResult<Self::Val, <T as frame_system::Config>::RuntimeCall> {
		let Some(who) = origin.as_system_origin_signer() else {
			return Ok((
				sp_runtime::transaction_validity::ValidTransaction::default(),
				Val::NoCharge,
				origin,
			));
		};
		let fee = self.dryrun_withdraw_fee(&who, call, info, len)?;
		let priority = pallet_transaction_payment::ChargeTransactionPayment::<T>::get_priority(
			info,
			len,
			self.tip(call),
			fee,
		);
		let val = Val::Charge { tip: self.tip(call), who: who.clone(), fee };
		let validity = ValidTransaction { priority, ..Default::default() };
		Ok((validity, val, origin))
	}

	fn prepare(
		self,
		val: Self::Val,
		_origin: &<T as frame_system::Config>::RuntimeOrigin,
		call: &<T as frame_system::Config>::RuntimeCall,
		info: &DispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
		len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		match val {
			Val::Charge { tip, who, .. } => {
				let (_fee, initial_payment) = self.withdraw_fee(&who, call, info, len)?;
				Ok(Pre::Charge { tip, who, initial_payment, weight: self.weight(call) })
			},
			Val::NoCharge => Ok(Pre::NoCharge { refund: self.weight(call) }),
		}
	}

	fn post_dispatch_details(
		pre: Self::Pre,
		info: &DispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
		post_info: &PostDispatchInfoOf<<T as frame_system::Config>::RuntimeCall>,
		len: usize,
		result: &DispatchResult,
	) -> Result<Weight, TransactionValidityError> {
		match pre {
			Pre::Charge { tip, who, initial_payment, weight } => {
				match initial_payment {
					InitialPayment::Token(already_withdrawn) => {
						let actual_ext_weight = Weight::zero(); // You may want to use a more precise weight function
						let unspent_weight = weight.saturating_sub(actual_ext_weight);
						let mut actual_post_info = *post_info;
						actual_post_info.refund(unspent_weight);
						pallet_transaction_payment::ChargeTransactionPayment::<T>::post_dispatch_details(
							pallet_transaction_payment::Pre::Charge { tip, who, imbalance: already_withdrawn },
							info,
							&actual_post_info,
							len,
							result,
						)?;
						Ok(unspent_weight)
					},
					InitialPayment::Capacity => {
						debug_assert!(tip.is_zero(), "tip should be zero for Capacity tx.");
						Ok(weight)
					},
					InitialPayment::Free => {
						debug_assert!(tip.is_zero(), "tip should be zero if initial fee was zero.");
						Ok(weight)
					},
				}
			},
			Pre::NoCharge { refund } => Ok(refund),
		}
	}
}
