use frame_support::DefaultNoBound;
use pallet_transaction_payment::{Config, OnChargeTransaction};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;

/// Type aliases used for interaction with `OnChargeTransaction`.
pub type OnChargeTransactionOf<T> = <T as pallet_transaction_payment::Config>::OnChargeTransaction;

/// Balance type alias.
pub type BalanceOf<T> = <OnChargeTransactionOf<T> as OnChargeTransaction<T>>::Balance;

/// Liquidity info type alias (imbalances).
pub type LiquidityInfoOf<T> = <OnChargeTransactionOf<T> as OnChargeTransaction<T>>::LiquidityInfo;

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
	/// Returns true if the initial payment is free.
	pub fn is_free(&self) -> bool {
		match *self {
			InitialPayment::Free => true,
			_ => false,
		}
	}

	/// Returns true if the initial payment is in capacity.
	pub fn is_capacity(&self) -> bool {
		match *self {
			InitialPayment::Capacity => true,
			_ => false,
		}
	}

	/// Returns true if the initial payment is in tokens.
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
