//! Types for the Capacity Pallet
use super::*;
use frame_support::{BoundedVec, EqNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use parity_scale_codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, Get, Saturating, Zero},
	BoundedBTreeMap, RuntimeDebug,
};

#[cfg(any(feature = "runtime-benchmarks", test))]
use sp_std::vec::Vec;

#[derive(
	Clone, Copy, Debug, Decode, Encode, TypeInfo, Eq, MaxEncodedLen, PartialEq, PartialOrd,
)]
/// The type of staking a given Staking Account is doing.
pub enum StakingType {
	/// Staking account targets Providers for capacity only, no token reward
	MaximumCapacity,
	/// Staking account targets Providers and splits reward between capacity to the Provider
	/// and token for the account holder
	ProviderBoost,
}

/// The type used for storing information about staking details.
#[derive(
	TypeInfo, RuntimeDebugNoBound, PartialEqNoBound, EqNoBound, Clone, Decode, Encode, MaxEncodedLen,
)]
#[scale_info(skip_type_params(T))]
pub struct StakingDetails<T: Config> {
	/// The amount a Staker has staked, minus the sum of all tokens in `unlocking`.
	pub active: BalanceOf<T>,
	/// The type of staking for this staking account
	pub staking_type: StakingType,
}

/// The type that is used to record a single request for a number of tokens to be unlocked.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct UnlockChunk<Balance, EpochNumber> {
	/// Amount to be unlocked.
	pub value: Balance,
	/// Block number at which point funds are unlocked.
	pub thaw_at: EpochNumber,
}

impl<T: Config> StakingDetails<T> {
	/// Increases total and active balances by an amount.
	pub fn deposit(&mut self, amount: BalanceOf<T>) -> Option<()> {
		self.active = amount.checked_add(&self.active)?;
		Some(())
	}

	/// Decrease the amount of active stake by an amount and create an UnlockChunk.
	pub fn withdraw(&mut self, amount: BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError> {
		let current_active = self.active;

		let mut new_active = self.active.saturating_sub(amount);
		let mut actual_unstaked: BalanceOf<T> = amount;

		if new_active.le(&T::MinimumStakingAmount::get()) {
			actual_unstaked = current_active;
			new_active = Zero::zero();
		}

		self.active = new_active;
		Ok(actual_unstaked)
	}
}

impl<T: Config> Default for StakingDetails<T> {
	fn default() -> Self {
		Self { active: Zero::zero(), staking_type: StakingType::MaximumCapacity }
	}
}

/// Details about the total token amount targeted to an MSA.
/// The Capacity that the target will receive.
#[derive(Default, PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct StakingTargetDetails<Balance>
where
	Balance: Default + Saturating + Copy + CheckedAdd + CheckedSub,
{
	/// The total amount of tokens that have been targeted to the MSA.
	pub amount: Balance,
	/// The total Capacity that an MSA received.
	pub capacity: Balance,
}

impl<Balance: Default + Saturating + Copy + CheckedAdd + CheckedSub + PartialOrd>
	StakingTargetDetails<Balance>
{
	/// Increase an MSA target Staking total and Capacity amount.
	pub fn deposit(&mut self, amount: Balance, capacity: Balance) -> Option<()> {
		self.amount = amount.checked_add(&self.amount)?;
		self.capacity = capacity.checked_add(&self.capacity)?;
		Some(())
	}

	/// Decrease an MSA target Staking total and Capacity amount.
	/// If the amount would put you below the minimum, zero out the amount.
	/// Return the actual amounts withdrawn.
	pub fn withdraw(
		&mut self,
		amount: Balance,
		capacity: Balance,
		minimum: Balance,
	) -> (Balance, Balance) {
		let entire_amount = self.amount;
		let entire_capacity = self.capacity;
		self.amount = self.amount.saturating_sub(amount);
		if self.amount.lt(&minimum) {
			*self = Self::default();
			return (entire_amount, entire_capacity)
		} else {
			self.capacity = self.capacity.saturating_sub(capacity);
		}
		(amount, capacity)
	}
}

/// The type for storing Registered Provider Capacity balance:
#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CapacityDetails<Balance, EpochNumber> {
	/// The Capacity remaining for the `last_replenished_epoch`.
	pub remaining_capacity: Balance,
	/// The amount of tokens staked to an MSA.
	pub total_tokens_staked: Balance,
	/// The total Capacity issued to an MSA.
	pub total_capacity_issued: Balance,
	/// The last Epoch that an MSA was replenished with Capacity.
	pub last_replenished_epoch: EpochNumber,
}

impl<Balance, EpochNumber> CapacityDetails<Balance, EpochNumber>
where
	Balance: Saturating + Copy + CheckedAdd + CheckedSub,
	EpochNumber: Clone + PartialOrd + PartialEq,
{
	/// Increase a targets total Tokens staked and Capacity total issuance by an amount.
	/// To be called on a stake
	pub fn deposit(&mut self, amount: &Balance, capacity: &Balance) -> Option<()> {
		self.total_tokens_staked = amount.checked_add(&self.total_tokens_staked)?;
		self.remaining_capacity = capacity.checked_add(&self.remaining_capacity)?;
		self.total_capacity_issued = capacity.checked_add(&self.total_capacity_issued)?;

		// We do not touch last_replenished epoch here, because it would create a DoS vulnerability.
		// Since capacity is lazily replenished, an attacker could stake
		// a minimum amount, then at the very beginning of each epoch, stake a tiny additional amount,
		// thus preventing replenishment when "last_replenished_at" is checked on the next provider's
		// message.
		Some(())
	}

	/// Return whether capacity can be replenished, given the current epoch.
	pub fn can_replenish(&self, current_epoch: EpochNumber) -> bool {
		self.last_replenished_epoch.lt(&current_epoch)
	}

	/// Completely refill all available capacity.
	/// To be called lazily when a Capacity message is sent in a new epoch.
	pub fn replenish_all(&mut self, current_epoch: &EpochNumber) {
		self.remaining_capacity = self.total_capacity_issued;
		self.last_replenished_epoch = current_epoch.clone();
	}

	/// Replenish remaining capacity by the provided amount and
	/// touch last_replenished_epoch with the current epoch.
	pub fn replenish_by_amount(&mut self, amount: Balance, current_epoch: &EpochNumber) {
		self.remaining_capacity = amount.saturating_add(self.remaining_capacity);
		self.last_replenished_epoch = current_epoch.clone();
	}

	/// Deduct the given amount from the remaining capacity that can be used to pay for messages.
	/// To be called when a message is paid for with capacity.
	pub fn deduct_capacity_by_amount(&mut self, amount: Balance) -> Result<(), ArithmeticError> {
		let new_remaining =
			self.remaining_capacity.checked_sub(&amount).ok_or(ArithmeticError::Underflow)?;
		self.remaining_capacity = new_remaining;
		Ok(())
	}

	/// Decrease a target's total available capacity.
	/// To be called on an unstake.
	pub fn withdraw(&mut self, capacity_deduction: Balance, tokens_staked_deduction: Balance) {
		self.total_tokens_staked = self.total_tokens_staked.saturating_sub(tokens_staked_deduction);
		self.total_capacity_issued = self.total_capacity_issued.saturating_sub(capacity_deduction);
		self.remaining_capacity = self.remaining_capacity.saturating_sub(capacity_deduction);
	}
}

/// The type for storing details about an epoch.
/// May evolve to store other needed data such as epoch_end.
#[derive(
	PartialEq, Eq, Clone, Default, PartialOrd, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
pub struct EpochInfo<BlockNumber> {
	/// The block number when this epoch started.
	pub epoch_start: BlockNumber,
}

/// A BoundedVec containing UnlockChunks
pub type UnlockChunkList<T> = BoundedVec<
	UnlockChunk<BalanceOf<T>, <T as Config>::EpochNumber>,
	<T as Config>::MaxUnlockingChunks,
>;

/// Computes and returns the total token held in an UnlockChunkList.
pub fn unlock_chunks_total<T: Config>(unlock_chunks: &UnlockChunkList<T>) -> BalanceOf<T> {
	unlock_chunks
		.iter()
		.fold(Zero::zero(), |acc: BalanceOf<T>, chunk| acc.saturating_add(chunk.value))
}

/// Deletes thawed chunks
/// Caller is responsible for updating free/locked balance on the token account.
/// Returns: the total amount reaped from `unlocking`
pub fn unlock_chunks_reap_thawed<T: Config>(
	unlock_chunks: &mut UnlockChunkList<T>,
	current_epoch: <T>::EpochNumber,
) -> BalanceOf<T> {
	let mut total_reaped: BalanceOf<T> = 0u32.into();
	unlock_chunks.retain(|chunk| {
		if current_epoch.ge(&chunk.thaw_at) {
			total_reaped = total_reaped.saturating_add(chunk.value);
			false
		} else {
			true
		}
	});
	total_reaped
}
#[cfg(any(feature = "runtime-benchmarks", test))]
#[allow(clippy::unwrap_used)]
/// set unlock chunks with (balance, thaw_at).  Does not check BoundedVec limit.
/// returns true on success, false on failure (?)
/// For testing and benchmarks ONLY, note possible panic via BoundedVec::try_from + unwrap
pub fn unlock_chunks_from_vec<T: Config>(chunks: &Vec<(u32, u32)>) -> UnlockChunkList<T> {
	let result: Vec<UnlockChunk<BalanceOf<T>, <T>::EpochNumber>> = chunks
		.into_iter()
		.map(|chunk| UnlockChunk { value: chunk.0.into(), thaw_at: chunk.1.into() })
		.collect();
	// CAUTION
	BoundedVec::try_from(result).unwrap()
}

/// The information needed to track a Reward Era
#[derive(
	PartialEq,
	Eq,
	Clone,
	Copy,
	Default,
	PartialOrd,
	Encode,
	Decode,
	RuntimeDebug,
	TypeInfo,
	MaxEncodedLen,
)]
pub struct RewardEraInfo<RewardEra, BlockNumber>
where
	RewardEra: AtLeast32BitUnsigned + EncodeLike,
{
	/// the index of this era
	pub era_index: RewardEra,
	/// the starting block of this era
	pub started_at: BlockNumber,
}

/// Needed data about a RewardPool for a given RewardEra.
/// The total_reward_pool balance for the previous era is set when a new era starts,
/// based on total staked token at the end of the previous era, and remains unchanged.
/// The unclaimed_balance is initialized to total_reward_pool and deducted whenever a
/// valid claim occurs.
#[derive(
	PartialEq, Eq, Clone, Default, PartialOrd, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen,
)]
pub struct RewardPoolInfo<Balance> {
	/// the total staked for rewards in the associated RewardEra
	pub total_staked_token: Balance,
	/// the reward pool for this era
	pub total_reward_pool: Balance,
	/// the remaining rewards balance to be claimed
	pub unclaimed_balance: Balance,
}

// TODO: Store retarget unlock chunks in their own storage but can be for any stake type
// TODO: Store unstake unlock chunks in their own storage but can be for any stake type (at first do only boosted unstake)

/// A record of staked amounts for a complete RewardEra
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct ProviderBoostHistory<T: Config>(
	BoundedBTreeMap<T::RewardEra, BalanceOf<T>, T::StakingRewardsPastErasMax>,
);

impl<T: Config> Default for ProviderBoostHistory<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T: Config> ProviderBoostHistory<T> {
	/// Constructs a new empty ProviderBoostHistory
	pub fn new() -> Self {
		ProviderBoostHistory(BoundedBTreeMap::new())
	}

	/// Adds `add_amount` to the entry for `reward_era`.
	/// Updates entry, or creates a new entry if it does not exist
	/// returns the total number of history items
	pub fn add_era_balance(
		&mut self,
		reward_era: &T::RewardEra,
		add_amount: &BalanceOf<T>,
	) -> Option<usize> {
		if let Some(entry) = self.0.get_mut(&reward_era) {
			// update
			*entry = entry.saturating_add(*add_amount);
		} else {
			// insert
			self.remove_oldest_entry_if_full(); // this guarantees a try_insert never fails
			let current_staking_amount = self.get_last_staking_amount();
			if self
				.0
				.try_insert(*reward_era, current_staking_amount.saturating_add(*add_amount))
				.is_err()
			{
				return None;
			};
		}

		Some(self.count())
	}

	/// Subtracts `subtract_amount` from the entry for `reward_era`. Zero values are still retained.
	/// Returns None if there is no entry for the reward era.
	/// Otherwise returns Some(history_count)
	pub fn subtract_era_balance(
		&mut self,
		reward_era: &T::RewardEra,
		subtract_amount: &BalanceOf<T>,
	) -> Option<usize> {
		if self.count().is_zero() {
			return None;
		};
		if let Some(entry) = self.0.get_mut(reward_era) {
			*entry = entry.saturating_sub(*subtract_amount);
		} else {
			self.remove_oldest_entry_if_full();
			let current_staking_amount = self.get_last_staking_amount();
			if self
				.0
				.try_insert(*reward_era, current_staking_amount.saturating_sub(*subtract_amount))
				.is_err()
			{
				return None
			}
		}
		Some(self.count())
	}

	/// Return how much is staked for the given era.  If there is no entry for that era, return None.
	pub fn get_staking_amount_for_era(&self, reward_era: &T::RewardEra) -> Option<&BalanceOf<T>> {
		self.0.get(reward_era)
	}

	/// Returns how much was staked during the given era. If there is no history entry for `reward_era`,
	/// returns the next earliest entry's staking balance. Note there is no sense of what the current
	/// era is; subsequent calls could return a different result if 'reward_era' is the current era and
	/// there has been a boost or unstake.
	pub fn get_amount_staked_for_era(&self, reward_era: &T::RewardEra) -> BalanceOf<T> {
		// this gives an ordered-by-key Iterator
		let mut bmap_iter = self.0.iter();
		let mut eligible_amount: BalanceOf<T> = Zero::zero();
		while let Some((era, balance)) = bmap_iter.next() {
			if era.eq(reward_era) {
				return *balance
			}
			// there was a boost or unstake in this era.
			else if era.gt(reward_era) {
				return eligible_amount;
			} // eligible_amount has been staked through reward_era
			eligible_amount = *balance;
		}
		eligible_amount
	}

	/// Returns the number of history items
	pub fn count(&self) -> usize {
		self.0.len()
	}

	fn remove_oldest_entry_if_full(&mut self) {
		if self.is_full() {
			// compiler errors with unwrap
			if let Some((earliest_key, _earliest_val)) = self.0.first_key_value() {
				self.0.remove(&earliest_key.clone());
			}
		}
	}

	fn get_last_staking_amount(&self) -> BalanceOf<T> {
		// compiler errors with unwrap
		if let Some((_last_key, last_value)) = self.0.last_key_value() {
			return *last_value;
		};
		Zero::zero()
	}

	fn is_full(&self) -> bool {
		self.count().eq(&(T::StakingRewardsPastErasMax::get() as usize))
	}
}

/// Struct with utilities for storing and updating unlock chunks
#[derive(Debug, TypeInfo, PartialEqNoBound, EqNoBound, Clone, Decode, Encode, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct RetargetInfo<T: Config> {
	/// How many times the account has retargeted this RewardEra
	pub retarget_count: u32,
	/// The last RewardEra they retargeted
	pub last_retarget_at: T::RewardEra,
}

impl<T: Config> Default for RetargetInfo<T> {
	fn default() -> Self {
		Self { retarget_count: 0u32, last_retarget_at: Zero::zero() }
	}
}

impl<T: Config> RetargetInfo<T> {
	/// Increment retarget count and return Some() or
	/// If there are too many, return None
	pub fn update(&mut self, current_era: T::RewardEra) -> Option<()> {
		let max_retargets = T::MaxRetargetsPerRewardEra::get();
		if self.retarget_count.ge(&max_retargets) && self.last_retarget_at.eq(&current_era) {
			return None
		}
		if self.last_retarget_at.lt(&current_era) {
			self.last_retarget_at = current_era;
			self.retarget_count = 1;
		} else {
			self.retarget_count = self.retarget_count.saturating_add(1u32);
		}
		Some(())
	}
}

/// A claim to be rewarded `claimed_reward` in token value
pub struct StakingRewardClaim<T: Config> {
	/// How much is claimed, in token
	pub claimed_reward: BalanceOf<T>,
	/// The end state of the staking account if the operations are valid
	pub staking_account_end_state: StakingDetails<T>,
	/// The starting era for the claimed reward period, inclusive
	pub from_era: T::RewardEra,
	/// The ending era for the claimed reward period, inclusive
	pub to_era: T::RewardEra,
}

/// A trait that provides the Economic Model for Provider Boosting.
pub trait StakingRewardsProvider<T: Config> {
	/// the AccountId this provider is using
	type AccountId;

	/// the range of blocks over which a Reward Pool is determined and rewards are paid out
	type RewardEra;

	///  The hasher to use for proofs
	type Hash;

	/// The type for currency
	type Balance;

	/// Calculate the size of the reward pool using the current economic model
	fn reward_pool_size(total_staked: BalanceOf<T>) -> BalanceOf<T>;

	/// Return the total unclaimed reward in token for `accountId` for `from_era` --> `to_era`, inclusive
	/// Errors:
	///     - EraOutOfRange when from_era or to_era are prior to the history retention limit, or greater than the current Era.
	fn staking_reward_total(
		account_id: Self::AccountId,
		from_era: Self::RewardEra,
		to_era: Self::RewardEra,
	) -> Result<BalanceOf<T>, DispatchError>;

	/// Calculate the staking reward for a single era.  We don't care about the era number,
	/// just the values.
	fn era_staking_reward(
		era_amount_staked: BalanceOf<T>, // how much individual staked for a specific era
		era_total_staked: BalanceOf<T>,  // how much everyone staked for the era
		era_reward_pool_size: BalanceOf<T>, // how much token in the reward pool that era
	) -> BalanceOf<T>;

	/// Validate a payout claim for `accountId`, using `proof` and the provided `payload` StakingRewardClaim.
	/// Returns whether the claim passes validation.  Accounts must first pass `payoutEligible` test.
	/// Errors:
	///     - NotAStakingAccount
	///     - MaxUnlockingChunksExceeded
	///     - All other conditions that would prevent a reward from being claimed return 'false'
	fn validate_staking_reward_claim(
		account_id: Self::AccountId,
		proof: Self::Hash,
		payload: StakingRewardClaim<T>,
	) -> bool;

	/// Return the effective amount when staked for a Provider Boost
	/// The amount is multiplied by a factor > 0 and < 1.
	fn capacity_boost(amount: BalanceOf<T>) -> BalanceOf<T>;
}

/// Result of checking a Boost History item to see if it's eligible for a reward.
#[derive(Copy, Clone, Encode, Decode, Default, RuntimeDebug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct UnclaimedRewardInfo<T: Config> {
	/// The Reward Era for which this reward was earned
	pub reward_era: T::RewardEra,
	/// When this reward expires, i.e. can no longer be claimed
	pub expires_at_block: BlockNumberFor<T>,
	/// The amount staked in this era that is eligible for rewards.  Does not count additional amounts
	/// staked in this era.
	pub staked_amount: BalanceOf<T>,
	/// The amount in token of the reward (only if it can be calculated using only on chain data)
	pub earned_amount: BalanceOf<T>,
}
