//! Types for the Capacity Pallet
use super::*;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{BoundedVec, RuntimeDebug, RuntimeDebugNoBound};
use scale_info::TypeInfo;
use sp_runtime::traits::Zero;

/// Details about the total token amount targeted to an MSA.
/// The Capacity that the target will receive.
#[derive(PartialEq, Eq, Default, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct StakingTargetDetails<Balance> {
	/// The total amount of tokens that have been targeted to the MSA.
	pub amount: Balance,
	/// The total Capacity that an MSA received.
	pub capacity: Balance,
}

/// The type used for storing information about staking details.
#[derive(TypeInfo, RuntimeDebugNoBound, Clone, Decode, Encode, PartialEq, Eq, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct StakingAccountDetails<T: Config> {
	/// The amount a Staker has staked, minus the sum of all tokens in `unlocking`.
	pub active: BalanceOf<T>,
	/// The total amount of tokens in `active` and `unlocking`
	pub total: BalanceOf<T>,
	/// Unstaked balances that are thawing or awaiting withdrawal.
	pub unlocking: BoundedVec<UnlockChunk<BalanceOf<T>, T::BlockNumber>, T::MaxUnlockingChunks>,
}

/// The type that is used to record a single request for a number of tokens to be unlocked.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct UnlockChunk<Balance, BlockNumber> {
	/// Amount to be unlocked.
	value: Balance,
	/// Block number at which point funds are unlocked.
	thaw_at: BlockNumber,
}
impl<T: Config> Default for StakingAccountDetails<T> {
	fn default() -> Self {
		Self { active: Zero::zero(), total: Zero::zero(), unlocking: Default::default() }
	}
}

/// The type for storing Registered Provider Capacity balance:
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CapacityDetails<Balance, BlockNumber> {
	/// The Capacity remaining for the `last_replenished_epoch`.
	pub remaining: Balance,
	/// The total Capacity issued to an MSA.
	pub total_available: Balance,
	/// The last block-number that an MSA was replenished with Capacity.
	pub last_replenished_epoch: BlockNumber,
}
