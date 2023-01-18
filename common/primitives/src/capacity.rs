use frame_support::traits::tokens::Balance;
use sp_runtime::DispatchError;

use crate::msa::MessageSourceId;

/// A trait for checking that a target MSA can be staked to.
pub trait TargetValidator {
	/// Checks if an MSA is a valid target.
	fn validate(target: MessageSourceId) -> bool;
}
/// A trait for Non-transferable asset.
pub trait Nontransferable {
	/// Scalar type for representing balance of an account.
	type Balance: Balance;

	/// The available Capacity for an MSA account.
	fn balance(msa_id: MessageSourceId) -> Result<Self::Balance, DispatchError>;

	/// Reduce the available Capacity of an MSA account.
	fn reduce_balance(msa_id: MessageSourceId, amount: Self::Balance) -> Result<(), DispatchError>;

	/// Reduce the available Capacity of an MSA account.
	fn increase_balance(
		msa_id: MessageSourceId,
		amount: Self::Balance,
	) -> Result<(), DispatchError>;

	/// Increase the available Capacity for an MSA account.
	fn issue(msa_id: MessageSourceId, amount: Self::Balance) -> Result<(), DispatchError>;
}

/// A trait for replenishing Capacity.
pub trait Replenishable {
	/// Scalar type for representing balance of an account.
	type Balance: Balance;

	/// Replenish an MSA's Capacity by an amount.
	fn replenish_by_account(
		msa_id: MessageSourceId,
		amount: Self::Balance,
	) -> Result<(), DispatchError>;

	/// Replenish all Capacity balance for an MSA.
	fn replenish_all_for_account(msa_id: MessageSourceId) -> Result<(), DispatchError>;

	/// Checks if an account can be replenished.
	fn can_replenish(msa_id: MessageSourceId) -> bool;
}
