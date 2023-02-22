#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::{DispatchInfo, GetDispatchInfo, PostDispatchInfo},
	pallet_prelude::*,
	traits::IsType,
};

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::prelude::*;

use sp_runtime::{
	traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SignedExtension},
	transaction_validity::{TransactionValidity, TransactionValidityError},
	FixedPointOperand,
};

use pallet_transaction_payment::OnChargeTransaction;

// Type aliases used for interaction with `OnChargeTransaction`.
pub(crate) type OnChargeTransactionOf<T> =
	<T as pallet_transaction_payment::Config>::OnChargeTransaction;

// Balance type alias.
pub(crate) type BalanceOf<T> = <OnChargeTransactionOf<T> as OnChargeTransaction<T>>::Balance;

// Liquidity info type alias (imbalances).
pub(crate) type LiquidityInfoOf<T> =
	<OnChargeTransactionOf<T> as OnChargeTransaction<T>>::LiquidityInfo;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod mock;

pub use pallet::*;

#[allow(dead_code)]
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;
	// Simple declaration of the `Pallet` type. It is placeholder we use to implement traits and
	// method.
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The overarching call type.
		type RuntimeCall: Parameter
			+ Dispatchable<RuntimeOrigin = Self::RuntimeOrigin>
			+ GetDispatchInfo
			+ From<frame_system::Call<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>;
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
	BalanceOf<T>: Send + Sync + FixedPointOperand,
{
	/// Utility construct from tip.
	pub fn from(tip: BalanceOf<T>) -> Self {
		Self(tip)
	}

	/// Return the tip as being chosen by the transaction sender.
	pub fn tip(&self) -> BalanceOf<T> {
		self.0
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
		Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
	BalanceOf<T>: Send + Sync + From<u64> + FixedPointOperand,
{
	const IDENTIFIER: &'static str = "ChargeTransactionPayment";
	type AccountId = T::AccountId;
	type Call = <T as frame_system::Config>::RuntimeCall;
	type AdditionalSigned = ();
	type Pre = (
		// tip
		BalanceOf<T>,
		Self::AccountId,
		LiquidityInfoOf<T>,
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
		let result = pallet_transaction_payment::ChargeTransactionPayment::<T>::from(self.0)
			.validate(who, call, info, len)?;

		Ok(result)
	}

	/// Do any pre-flight stuff for a signed transaction.
	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		let result = pallet_transaction_payment::ChargeTransactionPayment::<T>::from(self.tip())
			.pre_dispatch(&who, &call, &info, len)?;

		Ok(result)
	}

	/// Do any post-flight stuff for an extrinsic.
	fn post_dispatch(
		maybe_pre: Option<Self::Pre>,
		info: &DispatchInfoOf<Self::Call>,
		post_info: &PostDispatchInfoOf<Self::Call>,
		len: usize,
		result: &DispatchResult,
	) -> Result<(), TransactionValidityError> {
		if let Some((tip, who, imbalance)) = maybe_pre {
			pallet_transaction_payment::ChargeTransactionPayment::<T>::post_dispatch(
				Some((tip, who, imbalance)),
				info,
				post_info,
				len,
				result,
			)?;
		}

		Ok(())
	}
}
