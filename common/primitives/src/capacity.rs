use crate::{msa::MessageSourceId, node::BlockNumber};
use frame_support::traits::tokens::Balance;
use scale_info::TypeInfo;
use sp_core::{Decode, Encode, MaxEncodedLen, RuntimeDebug};
use sp_runtime::{DispatchError, Permill};

/// The type of a Reward Era
pub type RewardEra = u32;

/// A trait for checking that a target MSA can be staked to.
pub trait TargetValidator {
	/// Checks if an MSA is a valid target.
	fn validate(target: MessageSourceId) -> bool;
}

#[derive(
	Clone, Copy, Debug, Decode, Encode, TypeInfo, Eq, MaxEncodedLen, PartialEq, PartialOrd,
)]
/// The type of staking a given Staking Account is doing.
pub enum StakingType {
	/// Staking account targets Providers for capacity only, no token reward
	MaximumCapacity,
	/// New reward program with new rules.
	/// Defines a mechanism for locking tokens that is both time-sensitive and event-driven, with immutable thawing behavior.
	CommittedBoost,
	/// Staking account targets Providers and splits reward between Capacity to the Provider
	/// and token for the account holder
	FlexibleBoost,
}

#[derive(
	Clone, Copy, Debug, Decode, Encode, TypeInfo, Eq, MaxEncodedLen, PartialEq, PartialOrd,
)]
/// The phase of Committed Boosting at a given block number
pub enum CommittmentPhase {
	/// The PTE block has not been set, nor has the failsafe block been exceeded
	PreCommitment,
	/// The PTE block has been set, but the initial commitment period has not yet elapsed
	InitialCommitment,
	/// The PTE has been set & the initial commitment period has elapsed, but the release period has not yet elapsed`
	StagedRelease,
	/// The release period has elapsed, and the commitment can be released
	RewardProgramEnded,
	/// The PTE block has not been set, and the failsafe block has been exceeded
	Failsafe,
}

// A trait defining the attributes for calculating freeze/release and reward values
/// associated with a particular `StakingType`
pub trait StakingConfigProvider {
	/// Scalar type for representing balance of an account.
	// type Balance: Balance;
	/// returns the configuration for the stake type
	fn get(staking_type: StakingType) -> StakingConfig;
}

/// A blanket implementation
impl TargetValidator for () {
	fn validate(_target: MessageSourceId) -> bool {
		false
	}
}

/// A trait for Non-transferable asset
pub trait Nontransferable {
	/// Scalar type for representing balance of an account.
	type Balance: Balance;

	/// The available Capacity for an MSA.
	fn balance(msa_id: MessageSourceId) -> Self::Balance;

	/// The replenishable Capacity for an MSA (the total capacity currently issued)
	fn replenishable_balance(msa_id: MessageSourceId) -> Self::Balance;

	/// Reduce Capacity of an MSA by amount.
	fn deduct(msa_id: MessageSourceId, capacity_amount: Self::Balance)
		-> Result<(), DispatchError>;

	/// Increase Staked Token and Capacity amounts of an MSA. (unused)
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

/// Result of checking a Boost History item to see if it's eligible for a reward.
#[derive(
	Copy, Clone, Default, Encode, Eq, Decode, RuntimeDebug, MaxEncodedLen, PartialEq, TypeInfo,
)]

pub struct UnclaimedRewardInfo<Balance, BlockNumber> {
	/// The Reward Era for which this reward was earned
	pub reward_era: RewardEra,
	/// When this reward expires, i.e., can no longer be claimed
	pub expires_at_block: BlockNumber,
	/// The total staked in this era as of the current block
	pub staked_amount: Balance,
	/// The amount staked in this era that is eligible for rewards.  Does not count additional amounts
	/// staked in this era.
	pub eligible_amount: Balance,
	/// The amount in token of the reward (only if it can be calculated using only on chain data)
	pub earned_amount: Balance,
}

/// Staking configuration details
pub struct StakingConfig {
	/// the percentage cap per era of an individual Provider Boost reward
	pub reward_percent_cap: Permill,
	/// the number of blocks a stake is initially frozen for
	pub initial_commitment_blocks: BlockNumber,
	/// the length in blocks of a commitment release stage
	pub commitment_release_stage_blocks: BlockNumber,
	/// the number of release stages that must elapse before the entire commitment can be released
	pub commitment_release_stages: u32,
}
