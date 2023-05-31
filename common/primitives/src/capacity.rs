use crate::msa::MessageSourceId;
use codec::{Encode, MaxEncodedLen};
use frame_support::traits::tokens::Balance;
use scale_info::TypeInfo;
use sp_api::Decode;
use sp_runtime::DispatchError;
use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeDisplay};
use crate::node::{AccountId, Era, Hash};

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
	/// Scalar type for representing staked token and capacity balances.
	type Balance: Balance;

	/// The balance Capacity for an MSA.
	fn balance(msa_id: MessageSourceId) -> Self::Balance;

	/// Reduce Capacity of an MSA by amount.
	fn deduct(msa_id: MessageSourceId, capacity_amount: Self::Balance)
		-> Result<(), DispatchError>;

	/// Increase Staked Token + Capacity amounts of an MSA. (unused)
	fn deposit(
		msa_id: MessageSourceId,
		token_amount: Self::Balance,
		capacity_amount: Self::Balance,
	) -> Result<(), DispatchError>;
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


pub trait StakingRewardsProvider {
	type Balance: Balance;
	/// Return the size of the reward pool for the given era, in token
	/// Errors:
	///     - EraOutOfRange when `era` is prior to the history retention limit, or greater than the current Era.
	fn reward_pool_size(era: Era) -> Self::Balance;

	/// Return the total unclaimed reward in token for `accountId` for `fromEra` --> `toEra`, inclusive
	/// Errors:
	///     - NotAStakingAccount
	///     - EraOutOfRange when fromEra or toEra are prior to the history retention limit, or greater than the current Era.
	fn staking_reward_total(accountId: AccountId, fromEra: Era, toEra: Era);

	/// Validate a payout claim for `accountId`, using `proof` and the provided `payload` StakingRewardClaim.
	/// Returns whether the claim passes validation.  Accounts must first pass `payoutEligible` test.
	/// Errors:
	///     - NotAStakingAccount
	///     - MaxUnlockingChunksExceeded
	///     - All other conditions that would prevent a reward from being claimed return 'false'
	fn validate_staking_reward_claim(accountId: AccountId, proof: Hash, payload: StakingRewardClaim) -> bool;

	/// Return whether `accountId` can claim a reward. Staking accounts may not claim a reward more than once
	/// per Era, may not claim rewards before a complete Era has been staked, and may not claim more rewards past
	/// the number of `MaxUnlockingChunks`.
	/// Errors:
	///     - NotAStakingAccount
	///     - MaxUnlockingChunksExceeded
	///     - All other conditions that would prevent a reward from being claimed return 'false'
	fn payout_eligible(accountId: AccountIdOf<T>) -> bool;
}
