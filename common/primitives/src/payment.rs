use frame_support::{pallet_prelude::TransactionValidityError, traits::tokens::Balance};
use frame_system::Config;

/// A trait used for the withdrawal of Capacity.
pub trait OnChargeCapacityTransaction<T: Config> {
	/// Scalar type for representing balance of an account.
	type Balance: Balance;

	/// Handles withdrawal of Capacity from an Account.
	fn withdraw_fee(
		key: &T::AccountId,
		fee: Self::Balance,
	) -> Result<Self::Balance, TransactionValidityError>;
}
