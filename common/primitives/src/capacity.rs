use crate::msa::MessageSourceId;
use codec::{Encode, MaxEncodedLen};
use frame_support::traits::tokens::Balance;
use scale_info::TypeInfo;
use sp_api::Decode;
use sp_runtime::DispatchError;

/// A trait for checking that a target MSA can be staked to.
pub trait TargetValidator {
	/// Checks if an MSA is a valid target.
	fn validate(target: MessageSourceId) -> bool;
}

/// A blanket implementation
impl TargetValidator for () {
	fn validate(_target: MessageSourceId) -> bool {
		false
	}
}

/// A trait for Non-transferable asset.
pub trait Nontransferable {
	/// Scalar type for representing balance of an account.
	type Balance: Balance;

	/// The balance Capacity for an MSA account.
	fn balance(msa_id: MessageSourceId) -> Self::Balance;

	/// Reduce Capacity of an MSA account by amount.
	fn deduct(msa_id: MessageSourceId, amount: Self::Balance) -> Result<(), DispatchError>;

	/// Increase Capacity of an MSA account by an amount. (unused)
	fn deposit(msa_id: MessageSourceId, amount: Self::Balance) -> Result<(), DispatchError>;
}

/// A trait for replenishing Capacity.
pub trait Replenishable {
	/// Scalar type for representing balance of an account.
	type Balance: Balance;

	/// Replenish an MSA's Capacity by an amount.
	fn replenish_by_amount(
		msa_id: MessageSourceId,
		amount: Self::Balance,
	) -> Result<(), DispatchError>;

	/// Replenish all Capacity balance for an MSA.
	fn replenish_all_for(msa_id: MessageSourceId) -> Result<(), DispatchError>;

	/// Checks if an account can be replenished.
	fn can_replenish(msa_id: MessageSourceId) -> bool;
}

#[derive(Clone, Debug, Decode, Encode, TypeInfo, Eq, MaxEncodedLen, PartialEq, PartialOrd)]
/// The type of staking a given Staking Account is doing.
pub enum StakingType {
	/// Staking account targets Providers for capacity only, no token reward
	MaximumCapacity,
	/// Staking account targets Providers and splits reward between capacity to the Provider
	/// and token for the account holder
	ProviderBoost,
}
