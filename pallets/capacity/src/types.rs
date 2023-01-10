//! Types for the Capacity Pallet
use super::*;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{BoundedVec, EqNoBound, PartialEqNoBound, RuntimeDebug, RuntimeDebugNoBound};
use scale_info::TypeInfo;
use sp_runtime::traits::{CheckedAdd, Saturating, Zero};

/// The type used for storing information about staking details.
#[derive(
	TypeInfo, RuntimeDebugNoBound, PartialEqNoBound, EqNoBound, Clone, Decode, Encode, MaxEncodedLen,
)]
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

impl<T: Config> StakingAccountDetails<T> {
	/// Increases total and active balances by an amount.
	pub fn increase_by(&mut self, amount: BalanceOf<T>) -> Option<()> {
		self.total = amount.checked_add(&self.total)?;
		self.active = amount.checked_add(&self.active)?;

		Some(())
	}

	/// Calculates a stakable amount from a propose amount.
	pub fn get_stakable_amount_for(
		&self,
		staker: &T::AccountId,
		proposed_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		let account_balance = T::Currency::free_balance(&staker);
		let available_staking_balance = account_balance.saturating_sub(self.total);
		available_staking_balance.min(proposed_amount)
	}
}

impl<T: Config> Default for StakingAccountDetails<T> {
	fn default() -> Self {
		Self { active: Zero::zero(), total: Zero::zero(), unlocking: Default::default() }
	}
}

/// Details about the total token amount targeted to an MSA.
/// The Capacity that the target will receive.
#[derive(PartialEq, Eq, Default, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct StakingTargetDetails<Balance> {
	/// The total amount of tokens that have been targeted to the MSA.
	pub amount: Balance,
	/// The total Capacity that an MSA received.
	pub capacity: Balance,
}

impl<Balance: Saturating + Copy + CheckedAdd> StakingTargetDetails<Balance> {
	/// Increase an MSA target Staking total and Capacity amount.
	pub fn increase_by(&mut self, amount: Balance, capacity: Balance) -> Option<()> {
		self.amount = amount.checked_add(&self.amount)?;
		self.capacity = capacity.checked_add(&self.capacity)?;

		Some(())
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

impl<Balance: Saturating + Copy + CheckedAdd, BlockNumber> CapacityDetails<Balance, BlockNumber> {
	/// Increase a targets Capacity balance by an amount.
	pub fn increase_by(&mut self, amount: Balance, replenish_at: BlockNumber) -> Option<()> {
		self.remaining = amount.checked_add(&self.remaining)?;
		self.total_available = amount.checked_add(&self.total_available)?;
		self.last_replenished_epoch = replenish_at;

		Some(())
	}
}
