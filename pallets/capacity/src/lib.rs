//! Managages staking to the network for Capacity
//!
//! ## Quick Links
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
#![doc = include_str!("../README.md")]
//!
//! ## Lazy Capacity Refill
//!
//! Capacity is refilled on an as needed basis.
//! Thus, the provider's capacity balance retains the information of the last epoch.
//! Upon use, if the last Epoch is less than the current Epoch, the balance is assumed to be the maximum as the reload "has" happened.
//! Thus, the first use of Capacity in an Epoch will update the last Epoch number to match the current Epoch.
//! If a provider does not use any Capacity in an Epoch, the provider's capacity balance information is never updated for that Epoch.
//!
// Substrate macros are tripping the clippy::expect_used lint.
#![allow(clippy::expect_used)]
#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

use frame_support::{
	ensure,
	traits::{
		tokens::fungible::{Inspect as InspectFungible, InspectFreeze, Mutate, MutateFreeze},
		Get, Hooks,
	},
	weights::Weight,
};

use sp_runtime::{
	traits::{CheckedAdd, Saturating, Zero},
	ArithmeticError, DispatchError, Perbill,
};
use sp_std::ops::Mul;

pub use common_primitives::{
	capacity::{Nontransferable, Replenishable, TargetValidator},
	msa::MessageSourceId,
	utils::wrap_binary_data,
};

#[cfg(feature = "runtime-benchmarks")]
use common_primitives::benchmarks::RegisterProviderBenchmarkHelper;

pub use pallet::*;
pub use types::*;
pub use weights::*;
pub mod types;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod tests;

/// storage migrations
pub mod migration;
pub mod weights;
type BalanceOf<T> =
	<<T as Config>::Currency as InspectFungible<<T as frame_system::Config>::AccountId>>::Balance;

use frame_system::pallet_prelude::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{
		pallet_prelude::{StorageVersion, *},
		Twox64Concat,
	};
	use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeDisplay};

	/// A reason for freezing funds.
	/// Creates a freeze reason for this pallet that is aggregated by `construct_runtime`.
	#[pallet::composite_enum]
	pub enum FreezeReason {
		/// The account has staked tokens to the Frequency network.
		CapacityStaking,
	}

	/// the storage version for this pallet
	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(3);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The overarching freeze reason.
		type RuntimeFreezeReason: From<FreezeReason>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Functions that allow a fungible balance to be changed or frozen.
		type Currency: MutateFreeze<Self::AccountId, Id = Self::RuntimeFreezeReason>
			+ Mutate<Self::AccountId>
			+ InspectFreeze<Self::AccountId>
			+ InspectFungible<Self::AccountId>;

		/// Function that checks if an MSA is a valid target.
		type TargetValidator: TargetValidator;

		/// The minimum required token amount to stake. It facilitates cleaning dust when unstaking.
		#[pallet::constant]
		type MinimumStakingAmount: Get<BalanceOf<Self>>;

		/// The minimum required token amount to remain in the account after staking.
		#[pallet::constant]
		type MinimumTokenBalance: Get<BalanceOf<Self>>;

		/// The maximum number of unlocking chunks a StakingAccountLedger can have.
		/// It determines how many concurrent unstaked chunks may exist.
		#[pallet::constant]
		type MaxUnlockingChunks: Get<u32>;

		#[cfg(feature = "runtime-benchmarks")]
		/// A set of helper functions for benchmarking.
		type BenchmarkHelper: RegisterProviderBenchmarkHelper;

		/// The number of Epochs before you can unlock tokens after unstaking.
		#[pallet::constant]
		type UnstakingThawPeriod: Get<u16>;

		/// Maximum number of blocks an epoch can be
		#[pallet::constant]
		type MaxEpochLength: Get<BlockNumberFor<Self>>;

		/// A type that provides an Epoch number
		/// traits pulled from frame_system::Config::BlockNumber
		type EpochNumber: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ MaybeDisplay
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ sp_std::hash::Hash
			+ MaxEncodedLen
			+ TypeInfo;

		/// How much FRQCY one unit of Capacity costs
		#[pallet::constant]
		type CapacityPerToken: Get<Perbill>;
	}

	/// Storage for keeping a ledger of staked token amounts for accounts.
	/// - Keys: AccountId
	/// - Value: [`StakingDetails`](types::StakingDetails)
	#[pallet::storage]
	pub type StakingAccountLedger<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, StakingDetails<T>>;

	/// Storage to record how many tokens were targeted to an MSA.
	/// - Keys: AccountId, MSA Id
	/// - Value: [`StakingTargetDetails`](types::StakingTargetDetails)
	#[pallet::storage]
	pub type StakingTargetLedger<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		MessageSourceId,
		StakingTargetDetails<BalanceOf<T>>,
	>;

	/// Storage for target Capacity usage.
	/// - Keys: MSA Id
	/// - Value: [`CapacityDetails`](types::CapacityDetails)
	#[pallet::storage]
	pub type CapacityLedger<T: Config> =
		StorageMap<_, Twox64Concat, MessageSourceId, CapacityDetails<BalanceOf<T>, T::EpochNumber>>;

	/// Storage for the current epoch number
	#[pallet::storage]
	#[pallet::whitelist_storage]
	pub type CurrentEpoch<T: Config> = StorageValue<_, T::EpochNumber, ValueQuery>;

	/// Storage for the current epoch info
	#[pallet::storage]
	pub type CurrentEpochInfo<T: Config> =
		StorageValue<_, EpochInfo<BlockNumberFor<T>>, ValueQuery>;

	#[pallet::type_value]
	/// EpochLength defaults to 100 blocks when not set
	pub fn EpochLengthDefault<T: Config>() -> BlockNumberFor<T> {
		100u32.into()
	}

	/// Storage for the epoch length
	#[pallet::storage]
	pub type EpochLength<T: Config> =
		StorageValue<_, BlockNumberFor<T>, ValueQuery, EpochLengthDefault<T>>;

	#[pallet::storage]
	pub type UnstakeUnlocks<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, UnlockChunkList<T>>;

	// Simple declaration of the `Pallet` type. It is placeholder we use to implement traits and
	// method.
	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Tokens have been staked to the Frequency network.
		Staked {
			/// The token account that staked tokens to the network.
			account: T::AccountId,
			/// The MSA that a token account targeted to receive Capacity based on this staking amount.
			target: MessageSourceId,
			/// An amount that was staked.
			amount: BalanceOf<T>,
			/// The Capacity amount issued to the target as a result of the stake.
			capacity: BalanceOf<T>,
		},
		/// Unstaked token that has thawed was unlocked for the given account
		StakeWithdrawn {
			/// the account that withdrew its stake
			account: T::AccountId,
			/// the total amount withdrawn, i.e. put back into free balance.
			amount: BalanceOf<T>,
		},
		/// A token account has unstaked the Frequency network.
		UnStaked {
			/// The token account that unstaked tokens from the network.
			account: T::AccountId,
			/// The MSA target that will now have Capacity reduced as a result of unstaking.
			target: MessageSourceId,
			/// The amount that was unstaked.
			amount: BalanceOf<T>,
			/// The Capacity amount that was reduced from a target.
			capacity: BalanceOf<T>,
		},
		/// The Capacity epoch length was changed.
		EpochLengthUpdated {
			/// The new length of an epoch in blocks.
			blocks: BlockNumberFor<T>,
		},
		/// Capacity has been withdrawn from a MessageSourceId.
		CapacityWithdrawn {
			/// The MSA from which Capacity has been withdrawn.
			msa_id: MessageSourceId,
			/// The amount of Capacity withdrawn from MSA.
			amount: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Staker attempted to stake to an invalid staking target.
		InvalidTarget,
		/// Capacity is not available for the given MSA.
		InsufficientBalance,
		/// Staker is attempting to stake an amount below the minimum amount.
		InsufficientStakingAmount,
		/// Staker is attempting to stake a zero amount.
		ZeroAmountNotAllowed,
		/// This AccountId does not have a staking account.
		NotAStakingAccount,
		/// No staked value is available for withdrawal; either nothing is being unstaked,
		/// or nothing has passed the thaw period.
		NoUnstakedTokensAvailable,
		/// Unstaking amount should be greater than zero.
		UnstakedAmountIsZero,
		/// Amount to unstake is greater than the amount staked.
		AmountToUnstakeExceedsAmountStaked,
		/// Attempting to get a staker / target relationship that does not exist.
		StakerTargetRelationshipNotFound,
		/// Attempting to get the target's capacity that does not exist.
		TargetCapacityNotFound,
		/// Staker reached the limit number for the allowed amount of unlocking chunks.
		MaxUnlockingChunksExceeded,
		/// Increase Capacity increase exceeds the total available Capacity for target.
		IncreaseExceedsAvailable,
		/// Attempting to set the epoch length to a value greater than the max epoch length.
		MaxEpochLengthExceeded,
		/// Staker is attempting to stake an amount that leaves a token balance below the minimum amount.
		BalanceTooLowtoStake,
		/// None of the token amounts in UnlockChunks has thawed yet.
		NoThawedTokenAvailable,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(current: BlockNumberFor<T>) -> Weight {
			Self::start_new_epoch_if_needed(current)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Stakes some amount of tokens to the network and generates Capacity.
		///
		/// ### Errors
		///
		/// - Returns Error::ZeroAmountNotAllowed if the staker is attempting to stake a zero amount.
		/// - Returns Error::InvalidTarget if attempting to stake to an invalid target.
		/// - Returns Error::InsufficientStakingAmount if attempting to stake an amount below the minimum amount.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::stake())]
		pub fn stake(
			origin: OriginFor<T>,
			target: MessageSourceId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let staker = ensure_signed(origin)?;

			let (mut staking_account, actual_amount) =
				Self::ensure_can_stake(&staker, target, amount)?;

			let capacity = Self::increase_stake_and_issue_capacity(
				&staker,
				&mut staking_account,
				target,
				actual_amount,
			)?;

			Self::deposit_event(Event::Staked {
				account: staker,
				amount: actual_amount,
				target,
				capacity,
			});

			Ok(())
		}

		/// Removes all thawed UnlockChunks from caller's UnstakeUnlocks and unlocks the sum of the thawed values
		/// in the caller's token account.
		///
		/// ### Errors
		///   - Returns `Error::NoUnstakedTokensAvailable` if the account has no unstaking chunks.
		///   - Returns `Error::NoThawedTokenAvailable` if there are unstaking chunks, but none are thawed.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::withdraw_unstaked())]
		pub fn withdraw_unstaked(origin: OriginFor<T>) -> DispatchResult {
			let staker = ensure_signed(origin)?;
			let amount_withdrawn = Self::do_withdraw_unstaked(&staker)?;
			Self::deposit_event(Event::<T>::StakeWithdrawn {
				account: staker,
				amount: amount_withdrawn,
			});
			Ok(())
		}

		/// Schedules an amount of the stake to be unlocked.
		/// ### Errors
		///
		/// - Returns `Error::UnstakedAmountIsZero` if `amount` is not greater than zero.
		/// - Returns `Error::MaxUnlockingChunksExceeded` if attempting to unlock more times than config::MaxUnlockingChunks.
		/// - Returns `Error::AmountToUnstakeExceedsAmountStaked` if `amount` exceeds the amount currently staked.
		/// - Returns `Error::InvalidTarget` if `target` is not a valid staking target (not a Provider)
		/// - Returns `Error:: NotAStakingAccount` if `origin` has nothing staked at all
		/// - Returns `Error::StakerTargetRelationshipNotFound` if `origin` has nothing staked to `target`
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::unstake())]
		pub fn unstake(
			origin: OriginFor<T>,
			target: MessageSourceId,
			requested_amount: BalanceOf<T>,
		) -> DispatchResult {
			let unstaker = ensure_signed(origin)?;

			ensure!(requested_amount > Zero::zero(), Error::<T>::UnstakedAmountIsZero);

			let actual_amount = Self::decrease_active_staking_balance(&unstaker, requested_amount)?;
			Self::add_unlock_chunk(&unstaker, actual_amount)?;

			let capacity_reduction = Self::reduce_capacity(&unstaker, target, actual_amount)?;

			Self::deposit_event(Event::UnStaked {
				account: unstaker,
				target,
				amount: actual_amount,
				capacity: capacity_reduction,
			});
			Ok(())
		}

		/// Sets the epoch period length (in blocks).
		///
		/// # Requires
		/// * Root Origin
		///
		/// ### Errors
		/// - Returns `Error::MaxEpochLengthExceeded` if `length` is greater than T::MaxEpochLength.
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::set_epoch_length())]
		pub fn set_epoch_length(origin: OriginFor<T>, length: BlockNumberFor<T>) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(length <= T::MaxEpochLength::get(), Error::<T>::MaxEpochLengthExceeded);

			EpochLength::<T>::set(length);

			Self::deposit_event(Event::EpochLengthUpdated { blocks: length });
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Checks to see if staker has sufficient free-balance to stake the minimum required staking amount,
	/// and leave the minimum required free balance after staking.
	///
	/// # Errors
	/// * [`Error::ZeroAmountNotAllowed`]
	/// * [`Error::InvalidTarget`]
	/// * [`Error::BalanceTooLowtoStake`]
	///
	fn ensure_can_stake(
		staker: &T::AccountId,
		target: MessageSourceId,
		amount: BalanceOf<T>,
	) -> Result<(StakingDetails<T>, BalanceOf<T>), DispatchError> {
		ensure!(amount > Zero::zero(), Error::<T>::ZeroAmountNotAllowed);
		ensure!(T::TargetValidator::validate(target), Error::<T>::InvalidTarget);

		let staking_account = StakingAccountLedger::<T>::get(&staker).unwrap_or_default();
		let stakable_amount = Self::get_stakable_amount_for(&staker, amount);

		ensure!(stakable_amount > Zero::zero(), Error::<T>::BalanceTooLowtoStake);

		let new_active_staking_amount = staking_account
			.active
			.checked_add(&stakable_amount)
			.ok_or(ArithmeticError::Overflow)?;

		ensure!(
			new_active_staking_amount >= T::MinimumStakingAmount::get(),
			Error::<T>::InsufficientStakingAmount
		);

		Ok((staking_account, stakable_amount))
	}

	/// Increase a staking account and target account balances by amount.
	/// Additionally, it issues Capacity to the MSA target.
	fn increase_stake_and_issue_capacity(
		staker: &T::AccountId,
		staking_account: &mut StakingDetails<T>,
		target: MessageSourceId,
		amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		staking_account.deposit(amount).ok_or(ArithmeticError::Overflow)?;

		let capacity = Self::capacity_generated(amount);
		let mut target_details =
			StakingTargetLedger::<T>::get(&staker, &target).unwrap_or_default();
		target_details.deposit(amount, capacity).ok_or(ArithmeticError::Overflow)?;

		let mut capacity_details = CapacityLedger::<T>::get(target).unwrap_or_default();
		capacity_details.deposit(&amount, &capacity).ok_or(ArithmeticError::Overflow)?;

		Self::set_staking_account_and_lock(&staker, staking_account)?;

		Self::set_target_details_for(&staker, target, target_details);
		Self::set_capacity_for(target, capacity_details);

		Ok(capacity)
	}

	/// Sets staking account details after a deposit
	fn set_staking_account_and_lock(
		staker: &T::AccountId,
		staking_account: &StakingDetails<T>,
	) -> Result<(), DispatchError> {
		let unlocks = UnstakeUnlocks::<T>::get(staker).unwrap_or_default();
		let total_to_lock: BalanceOf<T> = staking_account
			.active
			.checked_add(&unlock_chunks_total::<T>(&unlocks))
			.ok_or(ArithmeticError::Overflow)?;
		T::Currency::set_freeze(&FreezeReason::CapacityStaking.into(), staker, total_to_lock)?;
		Self::set_staking_account(staker, staking_account);
		Ok(())
	}

	fn set_staking_account(staker: &T::AccountId, staking_account: &StakingDetails<T>) {
		if staking_account.active.is_zero() {
			StakingAccountLedger::<T>::set(staker, None);
		} else {
			StakingAccountLedger::<T>::insert(staker, staking_account);
		}
	}

	/// Sets target account details.
	fn set_target_details_for(
		staker: &T::AccountId,
		target: MessageSourceId,
		target_details: StakingTargetDetails<BalanceOf<T>>,
	) {
		if target_details.amount.is_zero() {
			StakingTargetLedger::<T>::remove(staker, target);
		} else {
			StakingTargetLedger::<T>::insert(staker, target, target_details);
		}
	}

	/// Sets targets Capacity.
	pub fn set_capacity_for(
		target: MessageSourceId,
		capacity_details: CapacityDetails<BalanceOf<T>, T::EpochNumber>,
	) {
		CapacityLedger::<T>::insert(target, capacity_details);
	}

	/// Decrease a staking account's active token.
	fn decrease_active_staking_balance(
		unstaker: &T::AccountId,
		amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let mut staking_account =
			StakingAccountLedger::<T>::get(unstaker).ok_or(Error::<T>::NotAStakingAccount)?;
		ensure!(amount <= staking_account.active, Error::<T>::AmountToUnstakeExceedsAmountStaked);

		let actual_unstaked_amount = staking_account.withdraw(amount)?;
		Self::set_staking_account(unstaker, &staking_account);
		Ok(actual_unstaked_amount)
	}

	fn add_unlock_chunk(
		unstaker: &T::AccountId,
		actual_unstaked_amount: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		let current_epoch: T::EpochNumber = CurrentEpoch::<T>::get();
		let thaw_at =
			current_epoch.saturating_add(T::EpochNumber::from(T::UnstakingThawPeriod::get()));
		let mut unlocks = UnstakeUnlocks::<T>::get(unstaker).unwrap_or_default();

		let unlock_chunk: UnlockChunk<BalanceOf<T>, T::EpochNumber> =
			UnlockChunk { value: actual_unstaked_amount, thaw_at };
		unlocks
			.try_push(unlock_chunk)
			.map_err(|_| Error::<T>::MaxUnlockingChunksExceeded)?;

		UnstakeUnlocks::<T>::set(unstaker, Some(unlocks));
		Ok(())
	}

	// Calculates a stakable amount from a proposed amount.
	pub(crate) fn get_stakable_amount_for(
		staker: &T::AccountId,
		proposed_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		let account_balance = T::Currency::balance(&staker);
		account_balance
			.saturating_sub(T::MinimumTokenBalance::get())
			.min(proposed_amount)
	}

	pub(crate) fn do_withdraw_unstaked(
		staker: &T::AccountId,
	) -> Result<BalanceOf<T>, DispatchError> {
		let current_epoch = CurrentEpoch::<T>::get();
		let mut total_unlocking: BalanceOf<T> = Zero::zero();

		let mut unlocks =
			UnstakeUnlocks::<T>::get(staker).ok_or(Error::<T>::NoUnstakedTokensAvailable)?;
		let amount_withdrawn = unlock_chunks_reap_thawed::<T>(&mut unlocks, current_epoch);
		ensure!(!amount_withdrawn.is_zero(), Error::<T>::NoThawedTokenAvailable);

		if unlocks.is_empty() {
			UnstakeUnlocks::<T>::set(staker, None);
		} else {
			total_unlocking = unlock_chunks_total::<T>(&unlocks);
			UnstakeUnlocks::<T>::set(staker, Some(unlocks));
		}

		let staking_account = StakingAccountLedger::<T>::get(staker).unwrap_or_default();
		let total_locked = staking_account.active.saturating_add(total_unlocking);
		if total_locked.is_zero() {
			T::Currency::thaw(&FreezeReason::CapacityStaking.into(), staker)?;
		} else {
			T::Currency::set_freeze(&FreezeReason::CapacityStaking.into(), staker, total_locked)?;
		}
		Ok(amount_withdrawn)
	}

	/// Reduce available capacity of target and return the amount of capacity reduction.
	fn reduce_capacity(
		unstaker: &T::AccountId,
		target: MessageSourceId,
		amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let mut staking_target_details = StakingTargetLedger::<T>::get(&unstaker, &target)
			.ok_or(Error::<T>::StakerTargetRelationshipNotFound)?;
		let mut capacity_details =
			CapacityLedger::<T>::get(target).ok_or(Error::<T>::TargetCapacityNotFound)?;

		let capacity_to_withdraw = if staking_target_details.amount.eq(&amount) {
			staking_target_details.capacity
		} else {
			Self::calculate_capacity_reduction(
				amount,
				capacity_details.total_tokens_staked,
				capacity_details.total_capacity_issued,
			)
		};

		staking_target_details.withdraw(amount, capacity_to_withdraw);
		capacity_details.withdraw(capacity_to_withdraw, amount);

		Self::set_capacity_for(target, capacity_details);
		Self::set_target_details_for(unstaker, target, staking_target_details);

		Ok(capacity_to_withdraw)
	}

	/// Calculates Capacity generated for given FRQCY
	fn capacity_generated(amount: BalanceOf<T>) -> BalanceOf<T> {
		let cpt = T::CapacityPerToken::get();
		cpt.mul(amount).into()
	}

	/// Determine the capacity reduction when given total_capacity, unstaking_amount, and total_amount_staked.
	fn calculate_capacity_reduction(
		unstaking_amount: BalanceOf<T>,
		total_amount_staked: BalanceOf<T>,
		total_capacity: BalanceOf<T>,
	) -> BalanceOf<T> {
		Perbill::from_rational(unstaking_amount, total_amount_staked).mul_ceil(total_capacity)
	}

	fn start_new_epoch_if_needed(current_block: BlockNumberFor<T>) -> Weight {
		// Should we start a new epoch?
		if current_block.saturating_sub(CurrentEpochInfo::<T>::get().epoch_start) >=
			EpochLength::<T>::get()
		{
			let current_epoch = CurrentEpoch::<T>::get();
			CurrentEpoch::<T>::set(current_epoch.saturating_add(1u32.into()));
			CurrentEpochInfo::<T>::set(EpochInfo { epoch_start: current_block });
			T::WeightInfo::on_initialize()
				.saturating_add(T::DbWeight::get().reads(1))
				.saturating_add(T::DbWeight::get().writes(2))
		} else {
			// 1 for get_current_epoch_info, 1 for get_epoch_length
			T::DbWeight::get().reads(2u64).saturating_add(T::DbWeight::get().writes(1))
		}
	}
}

impl<T: Config> Nontransferable for Pallet<T> {
	type Balance = BalanceOf<T>;

	/// Return the remaining capacity for the Provider MSA Id
	fn balance(msa_id: MessageSourceId) -> Self::Balance {
		match CapacityLedger::<T>::get(msa_id) {
			Some(capacity_details) => capacity_details.remaining_capacity,
			None => BalanceOf::<T>::zero(),
		}
	}

	/// Spend capacity: reduce remaining capacity by the given amount
	fn deduct(msa_id: MessageSourceId, amount: Self::Balance) -> Result<(), DispatchError> {
		let mut capacity_details =
			CapacityLedger::<T>::get(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;

		capacity_details
			.deduct_capacity_by_amount(amount)
			.map_err(|_| Error::<T>::InsufficientBalance)?;

		Self::set_capacity_for(msa_id, capacity_details);

		Self::deposit_event(Event::CapacityWithdrawn { msa_id, amount });
		Ok(())
	}

	/// Increase all totals for the MSA's CapacityDetails.
	fn deposit(msa_id: MessageSourceId, amount: Self::Balance) -> Result<(), DispatchError> {
		let mut capacity_details =
			CapacityLedger::<T>::get(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;
		capacity_details.deposit(&amount, &Self::capacity_generated(amount));
		Self::set_capacity_for(msa_id, capacity_details);
		Ok(())
	}
}

impl<T: Config> Replenishable for Pallet<T> {
	type Balance = BalanceOf<T>;

	fn replenish_all_for(msa_id: MessageSourceId) -> Result<(), DispatchError> {
		let mut capacity_details =
			CapacityLedger::<T>::get(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;

		capacity_details.replenish_all(&CurrentEpoch::<T>::get());

		Self::set_capacity_for(msa_id, capacity_details);

		Ok(())
	}

	/// Change: now calls new fn replenish_by_amount on the capacity_details,
	/// which does what this (actually Self::deposit) used to do
	fn replenish_by_amount(
		msa_id: MessageSourceId,
		amount: Self::Balance,
	) -> Result<(), DispatchError> {
		let mut capacity_details =
			CapacityLedger::<T>::get(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;
		capacity_details.replenish_by_amount(amount, &CurrentEpoch::<T>::get());
		Ok(())
	}

	fn can_replenish(msa_id: MessageSourceId) -> bool {
		if let Some(capacity_details) = CapacityLedger::<T>::get(msa_id) {
			return capacity_details.can_replenish(CurrentEpoch::<T>::get());
		}
		false
	}
}
