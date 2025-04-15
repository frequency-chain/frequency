//! Types for the Capacity Pallet
use super::*;
use common_primitives::capacity::RewardEra;
use frame_support::{
	pallet_prelude::PhantomData, BoundedVec, EqNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
use parity_scale_codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, Get, Saturating, Zero},
	BoundedBTreeMap, RuntimeDebug,
};
#[cfg(any(feature = "runtime-benchmarks", test))]
extern crate alloc;
#[cfg(any(feature = "runtime-benchmarks", test))]
use alloc::vec::Vec;

/// How much, as a percentage of staked token, to boost a targeted Provider when staking.
/// this value should be between 0 and 100
pub const STAKED_PERCENTAGE_TO_BOOST: u32 = 50;

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
	/// Amount to be unfrozen.
	pub value: Balance,
	/// Block number at which point funds are unfrozen.
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
			return (entire_amount, entire_capacity);
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
	BlockNumber: AtLeast32BitUnsigned + EncodeLike,
{
	/// the index of this era
	pub era_index: RewardEra,
	/// the starting block of this era
	pub started_at: BlockNumber,
}

/// A chunk of Reward Pool history items consists of a BoundedBTreeMap,
/// RewardEra is the key and the total stake for the RewardPool is the value.
/// the map has up to T::RewardPoolChunkLength items, however, the chunk storing the current era
/// has only that one.
#[derive(PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct RewardPoolHistoryChunk<T: Config>(
	BoundedBTreeMap<RewardEra, BalanceOf<T>, T::RewardPoolChunkLength>,
);
impl<T: Config> Default for RewardPoolHistoryChunk<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T: Config> Clone for RewardPoolHistoryChunk<T> {
	fn clone(&self) -> Self {
		Self(self.0.clone())
	}
}

impl<T: Config> RewardPoolHistoryChunk<T> {
	/// Constructs a new empty RewardPoolHistoryChunk
	pub fn new() -> Self {
		RewardPoolHistoryChunk(BoundedBTreeMap::new())
	}

	/// A wrapper for retrieving how much was provider_boosted in the given era
	/// from the  BoundedBTreeMap
	pub fn total_for_era(&self, reward_era: &RewardEra) -> Option<&BalanceOf<T>> {
		self.0.get(reward_era)
	}

	/// returns the range of 		eras in this chunk
	#[cfg(test)]
	pub fn era_range(&self) -> (RewardEra, RewardEra) {
		let zero_reward_era: RewardEra = Zero::zero();
		let zero_balance: BalanceOf<T> = Zero::zero();
		let (first, _vf) = self.0.first_key_value().unwrap_or((&zero_reward_era, &zero_balance));
		let (last, _vl) = self.0.last_key_value().unwrap_or((&zero_reward_era, &zero_balance));
		(*first, *last)
	}

	/// A wrapper for adding a new reward_era_entry to the BoundedBTreeMap
	pub fn try_insert(
		&mut self,
		reward_era: RewardEra,
		total: BalanceOf<T>,
	) -> Result<Option<BalanceOf<T>>, (RewardEra, BalanceOf<T>)> {
		self.0.try_insert(reward_era, total)
	}

	/// Get the earliest reward era stored in this BoundedBTreeMap
	#[cfg(test)]
	pub fn earliest_era(&self) -> Option<&RewardEra> {
		if let Some((first_era, _first_total)) = self.0.first_key_value() {
			return Some(first_era);
		}
		None
	}

	/// Is this chunk full?  It should always be yes once there is enough RewardPool history.
	pub fn is_full(&self) -> bool {
		self.0.len().eq(&(T::RewardPoolChunkLength::get() as usize))
	}
}

/// A record of staked amounts for a complete RewardEra
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct ProviderBoostHistory<T: Config>(
	BoundedBTreeMap<RewardEra, BalanceOf<T>, T::ProviderBoostHistoryLimit>,
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
		reward_era: &RewardEra,
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
	/// Returns Some(0) if they unstaked everything and this is the only entry
	/// Otherwise returns Some(history_count)
	pub fn subtract_era_balance(
		&mut self,
		reward_era: &RewardEra,
		subtract_amount: &BalanceOf<T>,
	) -> Option<usize> {
		if self.count().is_zero() {
			return None;
		};

		let current_staking_amount = self.get_last_staking_amount();
		if current_staking_amount.eq(subtract_amount) && self.count().eq(&1usize) {
			// Should not get here unless rewards have all been claimed, and provider boost history was
			// correctly updated. This && condition is to protect stakers against loss of rewards in the
			// case of some bug with payouts and boost history.
			return Some(0usize);
		}

		if let Some(entry) = self.0.get_mut(reward_era) {
			*entry = entry.saturating_sub(*subtract_amount);
		} else {
			self.remove_oldest_entry_if_full();
			if self
				.0
				.try_insert(*reward_era, current_staking_amount.saturating_sub(*subtract_amount))
				.is_err()
			{
				return None;
			}
		}
		Some(self.count())
	}

	/// A wrapper for the key/value retrieval of the BoundedBTreeMap.
	pub(crate) fn get_entry_for_era(&self, reward_era: &RewardEra) -> Option<&BalanceOf<T>> {
		self.0.get(reward_era)
	}

	/// Returns how much was staked during the given era, even if there is no explicit entry for that era.
	/// If there is no history entry for `reward_era`, returns the next earliest entry's staking balance.
	///
	/// Note there is no sense of what the current era is; subsequent calls could return a different result
	/// if 'reward_era' is the current era and there has been a boost or unstake.
	pub(crate) fn get_amount_staked_for_era(&self, reward_era: &RewardEra) -> BalanceOf<T> {
		// this gives an ordered-by-key Iterator
		let mut bmap_iter = self.0.iter();
		let mut eligible_amount: BalanceOf<T> = Zero::zero();
		while let Some((era, balance)) = bmap_iter.next() {
			if era.eq(reward_era) {
				return *balance;
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
		self.count().eq(&(T::ProviderBoostHistoryLimit::get() as usize))
	}
}

/// Struct with utilities for storing and updating unlock chunks
#[derive(Debug, TypeInfo, PartialEqNoBound, EqNoBound, Clone, Decode, Encode, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct RetargetInfo<T: Config> {
	/// How many times the account has retargeted this RewardEra
	pub retarget_count: u32,
	/// The last RewardEra they retargeted
	pub last_retarget_at: RewardEra,
	_marker: PhantomData<T>,
}

impl<T: Config> Default for RetargetInfo<T> {
	fn default() -> Self {
		Self { retarget_count: 0u32, last_retarget_at: Zero::zero(), _marker: Default::default() }
	}
}

impl<T: Config> RetargetInfo<T> {
	/// Constructor
	pub fn new(retarget_count: u32, last_retarget_at: RewardEra) -> Self {
		Self { retarget_count, last_retarget_at, _marker: Default::default() }
	}
	/// Increment retarget count and return Some() or
	/// If there are too many, return None
	pub fn update(&mut self, current_era: RewardEra) -> Option<()> {
		let max_retargets = T::MaxRetargetsPerRewardEra::get();
		if self.retarget_count.ge(&max_retargets) && self.last_retarget_at.eq(&current_era) {
			return None;
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

/// A trait that provides the Economic Model for Provider Boosting.
pub trait ProviderBoostRewardsProvider<T: Config> {
	/// The type for currency
	type Balance;

	/// Return the size of the reward pool using the current economic model
	fn reward_pool_size(total_staked: BalanceOf<T>) -> BalanceOf<T>;

	/// Calculate the reward for a single era.  We don't care about the era number,
	/// just the values.
	fn era_staking_reward(
		era_amount_staked: BalanceOf<T>, // how much individual staked for a specific era
		era_total_staked: BalanceOf<T>,  // how much everyone staked for the era
		era_reward_pool_size: BalanceOf<T>, // how much token in the reward pool that era
	) -> BalanceOf<T>;

	/// Return the effective amount when staked for a Provider Boost
	/// The amount is multiplied by a factor > 0 and < 1.
	fn capacity_boost(amount: BalanceOf<T>) -> BalanceOf<T>;
}
