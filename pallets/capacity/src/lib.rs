//! # Capacity Pallet
//! The Capacity pallet provides functionality for staking tokens to the Frequency network.
//!
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
//!
//! ## Overview
//! Capacity is a refillable resource that can be used to make transactions on the network.
//! Tokens may be staked to the network targeting a provider MSA account to which the network will grant Capacity.
//!
//! The network generates Capacity based on a Capacity-generating function that considers usage and network congestion.
//! When token is staked, the targeted provider MSA receives the Capacity generated.
//!
//! This Capacity can be used to pay for transactions given that the key used to pay for those transactions have a minimum balance
//! of the existential deposit.
//!
//! The staked amount may be increased, targeting either the same or a different target to receive the newly generated Capacity.
//! As a result, every time the network is staked to, the staked tokens are locked until unstaked.
//!
//! Unstaking schedules some amount of token to be unlocked. Any amount up to the total staked amount may
//! be unstaked, however, there is a a limit on how many concurrently scheduled unstaking requests can exist.
//! After scheduling tokens to be unlocked, they must undergo a thaw period before being withdrawn.
//!
//! After thawing, the tokens may be withdrawn using the withdraw_unstaked extrinsic.
//! On success, the tokens are unlocked and free up space to submit more unstaking request.
//!
//! The Capacity pallet provides functions for:
//!
//! - staking and, updating,
//!
//! ## Terminology
//! * **Capacity:** A refillable and non-transferable resource that can be used to pay for transactions on the network.
//! * **Staker:** An account that stakes tokens to the Frequency network in exchange for Capacity.
//! * **Target** A provider MSA account that receives Capacity.
//! * **Epoch Period:** The length of an epoch in blocks, used for replenishment and thawing.
//! * **Replenishable:**  The ability of a provider MSA account to be refilled with Capacity after an epoch period.
//!
// Substrate macros are tripping the clippy::expect_used lint.
#![allow(clippy::expect_used)]
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(rustdoc_missing_doc_code_examples)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]
use sp_std::ops::{Add, Mul};

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{Currency, Get, Hooks, LockIdentifier, LockableCurrency, WithdrawReasons},
	weights::{constants::RocksDbWeight, Weight},
};

use sp_runtime::{
	traits::{CheckedAdd, CheckedDiv, One, Saturating, Zero},
	ArithmeticError, DispatchError, Perbill,
};

pub use common_primitives::{
	capacity::{Nontransferable, Replenishable, TargetValidator},
	msa::MessageSourceId,
	utils::wrap_binary_data,
};

#[cfg(feature = "runtime-benchmarks")]
use common_primitives::benchmarks::RegisterProviderBenchmarkHelper;
use common_primitives::{
	capacity::StakingType,
	node::{RewardEra},
};

pub use pallet::*;
pub use types::*;
pub use weights::*;
pub mod types;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod tests;
mod tests;

pub mod weights;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

const STAKING_ID: LockIdentifier = *b"netstkng";
#[frame_support::pallet]
pub mod pallet {
	use super::*;
    use codec::EncodeLike;

	use common_primitives::capacity::{StakingType};
	use frame_support::{pallet_prelude::*, Twox64Concat};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeDisplay};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Function that allows a balance to be locked.
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

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
		type MaxEpochLength: Get<Self::BlockNumber>;

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

		/// A period of `EraLength` blocks in which a Staking Pool applies and
		/// when Staking Rewards may be earned.
		type RewardEra: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ MaybeDisplay
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ sp_std::hash::Hash
			+ MaxEncodedLen
			+ EncodeLike
			+ Into<BalanceOf<Self>>
			+ TypeInfo;

		/// The number of blocks in a Staking Era
		#[pallet::constant]
		type EraLength: Get<u32>;

		/// The maximum number of eras over which one can claim rewards
		#[pallet::constant]
		type StakingRewardsPastErasMax: Get<Self::RewardEra>;

		/// The StakingRewardsProvider used by this pallet in a given runtime
		type RewardsProvider: StakingRewardsProvider<Self>;

		/// After calling change_staking_target, the thaw period  for the created
		/// thaw chunk to expire. If the staker has called change_staking_target MaxUnlockingChunks
		/// times, then at least one of the chunks must have expired before the next call
		/// will succeed.
		type ChangeStakingTargetThawEras: Get<Self::RewardEra>;
	}

	/// Storage for keeping a ledger of staked token amounts for accounts.
	/// - Keys: AccountId
	/// - Value: [`StakingAccountDetails`](types::StakingAccountDetails)
	#[pallet::storage]
	#[pallet::getter(fn get_staking_account_for)]
	pub type StakingAccountLedger<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, StakingAccountDetails<T>>;

	/// Storage to record how many tokens were targeted to an MSA.
	/// - Keys: AccountId, MSA Id
	/// - Value: [`StakingTargetDetails`](types::StakingTargetDetails)
	#[pallet::storage]
	#[pallet::getter(fn get_target_for)]
	pub type StakingTargetLedger<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		MessageSourceId,
		StakingTargetDetails<T>,
	>;

	/// Storage for target Capacity usage.
	/// - Keys: MSA Id
	/// - Value: [`CapacityDetails`](types::CapacityDetails)
	#[pallet::storage]
	#[pallet::getter(fn get_capacity_for)]
	pub type CapacityLedger<T: Config> =
		StorageMap<_, Twox64Concat, MessageSourceId, CapacityDetails<BalanceOf<T>, T::EpochNumber>>;

	/// Storage for the current epoch number
	#[pallet::storage]
	#[pallet::whitelist_storage]
	#[pallet::getter(fn get_current_epoch)]
	pub type CurrentEpoch<T: Config> = StorageValue<_, T::EpochNumber, ValueQuery>;

	/// Storage for the current epoch info
	#[pallet::storage]
	#[pallet::getter(fn get_current_epoch_info)]
	pub type CurrentEpochInfo<T: Config> = StorageValue<_, EpochInfo<T::BlockNumber>, ValueQuery>;

	#[pallet::type_value]
	/// EpochLength defaults to 100 blocks when not set
	pub fn EpochLengthDefault<T: Config>() -> T::BlockNumber {
		100u32.into()
	}

	/// Storage for the epoch length
	#[pallet::storage]
	#[pallet::getter(fn get_epoch_length)]
	pub type EpochLength<T: Config> =
		StorageValue<_, T::BlockNumber, ValueQuery, EpochLengthDefault<T>>;

	/// Information about the current staking reward era.
	#[pallet::storage]
	#[pallet::getter(fn get_current_era)]
	pub type CurrentEraInfo<T: Config> =
		StorageValue<_, RewardEraInfo<T::RewardEra, T::BlockNumber>, ValueQuery>;

	/// Reward Pool history
	#[pallet::storage]
	#[pallet::getter(fn get_reward_pool_for_era)]
	pub type StakingRewardPool<T: Config> =
		CountedStorageMap<_, Twox64Concat, T::RewardEra, RewardPoolInfo<BalanceOf<T>>>;

	#[pallet::pallet]
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
		/// Unsstaked token that has thawed was unlocked for the given account
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
			blocks: T::BlockNumber,
		},
		/// Capacity has been withdrawn from a MessageSourceId.
		CapacityWithdrawn {
			/// The MSA from which Capacity has been withdrawn.
			msa_id: MessageSourceId,
			/// The amount of Capacity withdrawn from MSA.
			amount: BalanceOf<T>,
		},
		/// The target of a staked amount was changed to a new MessageSourceId
		StakingTargetChanged {
			/// The account that retargeted the staking amount
			account: T::AccountId,
			/// The Provider MSA that the staking amount is taken from
			from_msa: MessageSourceId,
			/// The Provider MSA that the staking amount is retargeted to
			to_msa: MessageSourceId,
			/// The amount in token that was retargeted
			amount: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Staker attempted to stake to an invalid staking target.
		InvalidTarget,
		/// Capacity is not available for the given MSA.
		InsufficientCapacityBalance,
		/// Staker is attempting to stake an amount below the minimum amount.
		StakingAmountBelowMinimum,
		/// Staker is attempting to stake a zero amount.  DEPRECATED
		ZeroAmountNotAllowed,
		/// This AccountId does not have a staking account.
		NotAStakingAccount,
		/// No staked value is available for withdrawal; either nothing is being unstaked,
		/// or nothing has passed the thaw period.
		NoUnstakedTokensAvailable,
		/// Unstaking amount must be greater than zero.
		UnstakedAmountIsZero,
		/// Amount to unstake or change targets is greater than the amount staked.
		InsufficientStakingBalance,
		/// Attempted to get a staker / target relationship that does not exist.
		StakerTargetRelationshipNotFound,
		/// Attempted to get the target's capacity that does not exist.
		TargetCapacityNotFound,
		/// Staker has reached the limit of unlocking chunks and must wait for at least one thaw period
		/// to complete.
		MaxUnlockingChunksExceeded,
		/// Capacity increase exceeds the total available Capacity for target.
		IncreaseExceedsAvailable,
		/// Attempted to set the Epoch length to a value greater than the max Epoch length.
		MaxEpochLengthExceeded,
		/// Staker is attempting to stake an amount that leaves a token balance below the minimum amount.
		BalanceTooLowtoStake,
		/// Staker tried to change StakingType on an existing account
		CannotChangeStakingType,
		/// The Era specified is too far in the past or is in the future
		EraOutOfRange,
		/// Rewards were already paid out for the specified Era range
		IneligibleForPayoutInEraRange,
		/// Attempted to retarget but from and to Provider MSA Ids were the same
		CannotRetargetToSameProvider,
		/// Rewards were already paid out this era
		AlreadyClaimedRewardsThisEra,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(current: T::BlockNumber) -> Weight {
			Self::start_new_epoch_if_needed(current)
				.saturating_add(Self::start_new_reward_era_if_needed(current))
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Stakes some amount of tokens to the network and generates Capacity.
		///
		/// ### Errors
		///
		/// - Returns Error::InvalidTarget if attempting to stake to an invalid target.
		/// - Returns Error::InsufficientStakingAmount if attempting to stake an amount below the minimum amount.
		/// - Returns Error::CannotChangeStakingType if the staking account exists and staking_type is different
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::stake())]
		pub fn stake(
			origin: OriginFor<T>,
			target: MessageSourceId,
			amount: BalanceOf<T>,
			staking_type: StakingType,
		) -> DispatchResult {
			let staker = ensure_signed(origin)?;

			let (mut staking_account, actual_amount) =
				Self::ensure_can_stake(&staker, &target, &amount, &staking_type)?;

			let capacity = Self::increase_stake_and_issue_capacity(
				&staker,
				&mut staking_account,
				&target,
				&actual_amount,
				&staking_type,
			)?;

			Self::deposit_event(Event::Staked {
				account: staker,
				amount: actual_amount,
				target,
				capacity,
			});

			Ok(())
		}

		/// removes all thawed UnlockChunks from caller's StakingAccount and unlocks the sum of the thawed values
		/// in the caller's token account.
		///
		/// ### Errors
		///   - Returns `Error::NotAStakingAccount` if no StakingAccountDetails are found for `origin`.
		///   - Returns `Error::NoUnstakedTokensAvailable` if the account has no unstaking chunks or none are thawed.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::withdraw_unstaked())]
		pub fn withdraw_unstaked(origin: OriginFor<T>) -> DispatchResult {
			let staker = ensure_signed(origin)?;

			let mut staking_account =
				Self::get_staking_account_for(&staker).ok_or(Error::<T>::NotAStakingAccount)?;

			let current_epoch = Self::get_current_epoch();
			let amount_withdrawn = staking_account.reap_thawed(current_epoch);
			ensure!(!amount_withdrawn.is_zero(), Error::<T>::NoUnstakedTokensAvailable);

			Self::update_or_delete_staking_account(&staker, &mut staking_account);
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
		/// - Returns `Error::InvalidTarget` if `target` is not a valid staking target
		/// - Returns `Error:: NotAStakingAccount` if `origin` has nothing staked
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::unstake())]
		pub fn unstake(
			origin: OriginFor<T>,
			target: MessageSourceId,
			requested_amount: BalanceOf<T>,
		) -> DispatchResult {
			let unstaker = ensure_signed(origin)?;
			Self::ensure_can_unstake(&unstaker)?;

			ensure!(requested_amount > Zero::zero(), Error::<T>::UnstakedAmountIsZero);

			let actual_amount = Self::decrease_active_staking_balance(&unstaker, requested_amount)?;
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
		pub fn set_epoch_length(origin: OriginFor<T>, length: T::BlockNumber) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(length <= T::MaxEpochLength::get(), Error::<T>::MaxEpochLengthExceeded);

			EpochLength::<T>::set(length);

			Self::deposit_event(Event::EpochLengthUpdated { blocks: length });
			Ok(())
		}

		/// Sets the target of the staking capacity to a new target.
		/// This adds a chunk to `StakingAccountDetails.stake_change_unlocking chunks`, up to `T::MaxUnlockingChunks`.
		/// The staked amount and Capacity generated by `amount` originally targeted to the `from` MSA Id is reassigned to the `to` MSA Id.
		/// Does not affect unstaking process or additional stake amounts.
		/// Changing a staking target to a Provider when Origin has nothing staked them will retain the staking type.
		/// Changing a staking target to a Provider when Origin has any amount staked to them will error if the staking types are not the same.
		/// ### Errors
		/// - [`Error::NotAStakingAccount`] if origin does not have a staking account
		/// - [`Error::MaxUnlockingChunksExceeded`] if `stake_change_unlocking_chunks` == `T::MaxUnlockingChunks`
		/// - [`Error::StakerTargetRelationshipNotFound`] if `from` is not a target for Origin's staking account.
		/// - [`Error::StakingAmountBelowMinimum`] if `amount` to retarget is below the minimum staking amount.
		/// - [`Error::InsufficientStakingBalance`] if `amount` to retarget exceeds what the staker has targeted to `from` MSA Id.
		/// - [`Error::InvalidTarget`] if `to` does not belong to a registered Provider.
		/// - [`Error::CannotChangeStakingType`] if origin already has funds staked for `to` and the staking type for `from` is different.
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::unstake())]
		pub fn change_staking_target(
			origin: OriginFor<T>,
			from: MessageSourceId,
			to: MessageSourceId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let staker = ensure_signed(origin)?;

			ensure!(from.ne(&to), Error::<T>::CannotRetargetToSameProvider);
			ensure!(
				amount >= T::MinimumStakingAmount::get(),
				Error::<T>::StakingAmountBelowMinimum
			);
			ensure!(
				StakingAccountLedger::<T>::contains_key(&staker),
				Error::<T>::NotAStakingAccount
			);

			let current_target = Self::get_target_for(&staker, &from)
				.ok_or(Error::<T>::StakerTargetRelationshipNotFound)?;

			ensure!(T::TargetValidator::validate(to), Error::<T>::InvalidTarget);

			Self::do_retarget(&staker, &from, &to, &amount, &current_target.staking_type)?;

			Self::deposit_event(Event::StakingTargetChanged {
				account: staker,
				from_msa: from,
				to_msa: to,
				amount,
			});
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Checks to see if staker has sufficient free-balance to stake the minimum required staking amount,
	/// and leave the minimum required free balance after staking.
	///
	/// # Errors
	/// * [`Error::StakingAmountBelowMinimum`]
	/// * [`Error::InvalidTarget`]
	/// * [`Error::BalanceTooLowtoStake`]
	///
	fn ensure_can_stake(
		staker: &T::AccountId,
		target: &MessageSourceId,
		amount: &BalanceOf<T>,
		staking_type: &StakingType,
	) -> Result<(StakingAccountDetails<T>, BalanceOf<T>), DispatchError> {
		ensure!(amount > &Zero::zero(), Error::<T>::StakingAmountBelowMinimum);
		ensure!(T::TargetValidator::validate(*target), Error::<T>::InvalidTarget);

		let staking_account: StakingAccountDetails<T> =
			Self::get_staking_account_for(&staker).unwrap_or_default();

		let staking_target: StakingTargetDetails<T> =
			Self::get_target_for(&staker, target).unwrap_or_default();
		// if total > 0 the account exists already. Prevent the staking type from being changed.
		if staking_target.amount > Zero::zero() {
			ensure!(
				staking_target.staking_type == *staking_type,
				Error::<T>::CannotChangeStakingType
			);
		}

		let stakable_amount = staking_account.get_stakable_amount_for(&staker, *amount);

		ensure!(stakable_amount > Zero::zero(), Error::<T>::BalanceTooLowtoStake);

		let new_active_staking_amount = staking_account
			.active
			.checked_add(&stakable_amount)
			.ok_or(ArithmeticError::Overflow)?;

		ensure!(
			new_active_staking_amount >= T::MinimumStakingAmount::get(),
			Error::<T>::StakingAmountBelowMinimum
		);

		Ok((staking_account, stakable_amount))
	}

	/// Increase a staking account and target account balances by amount.
	/// Additionally, it issues Capacity to the MSA target.
	fn increase_stake_and_issue_capacity(
		staker: &T::AccountId,
		staking_account: &mut StakingAccountDetails<T>,
		target: &MessageSourceId,
		amount: &BalanceOf<T>,
		staking_type: &StakingType,
	) -> Result<BalanceOf<T>, DispatchError> {
		staking_account.deposit(*amount).ok_or(ArithmeticError::Overflow)?;

		let capacity = if staking_type.eq(&StakingType::ProviderBoost) {
			Self::capacity_generated(T::RewardsProvider::capacity_boost(*amount))
		} else {
			Self::capacity_generated(*amount)
		};

		let mut target_details = Self::get_target_for(staker, target).unwrap_or_default();
		target_details.deposit(*amount, capacity).ok_or(ArithmeticError::Overflow)?;
		target_details.staking_type = staking_type.clone();

		let mut capacity_details = Self::get_capacity_for(target).unwrap_or_default();
		capacity_details.deposit(amount, &capacity).ok_or(ArithmeticError::Overflow)?;

		Self::set_staking_account(staker, staking_account);
		Self::set_target_details_for(staker, target, &target_details);
		Self::set_capacity_for(target, &capacity_details);

		Ok(capacity)
	}

	/// Sets staking account details.
	fn set_staking_account(staker: &T::AccountId, staking_account: &StakingAccountDetails<T>) {
		T::Currency::set_lock(STAKING_ID, &staker, staking_account.total, WithdrawReasons::all());
		StakingAccountLedger::<T>::insert(staker, staking_account);
	}

	/// Deletes staking account details
	fn delete_staking_account(staker: &T::AccountId) {
		T::Currency::remove_lock(STAKING_ID, &staker);
		StakingAccountLedger::<T>::remove(&staker);
	}

	/// If the staking account total is zero we reap storage, otherwise set the acount to the new details.
	fn update_or_delete_staking_account(
		staker: &T::AccountId,
		staking_account: &StakingAccountDetails<T>,
	) {
		if staking_account.total.is_zero() {
			Self::delete_staking_account(&staker);
		} else {
			Self::set_staking_account(&staker, &staking_account)
		}
	}

	/// Sets target account details.
	fn set_target_details_for(
		staker: &T::AccountId,
		target: &MessageSourceId,
		target_details: &StakingTargetDetails<T>,
	) {
		if target_details.amount.is_zero() {
			StakingTargetLedger::<T>::remove(staker, target);
		} else {
			StakingTargetLedger::<T>::insert(staker, target, target_details);
		}
	}

	/// Sets targets Capacity.
	pub fn set_capacity_for(
		target: &MessageSourceId,
		capacity_details: &CapacityDetails<BalanceOf<T>, T::EpochNumber>,
	) {
		CapacityLedger::<T>::insert(target, capacity_details);
	}

	/// Decrease a staking account's active token and create an unlocking chunk to be thawed at some future block.
	fn decrease_active_staking_balance(
		unstaker: &T::AccountId,
		amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let mut staking_account =
			Self::get_staking_account_for(unstaker).ok_or(Error::<T>::NotAStakingAccount)?;
		ensure!(amount <= staking_account.active, Error::<T>::InsufficientStakingBalance);

		let unstake_result = staking_account.withdraw(amount, Self::get_thaw_at_epoch())?;

		Self::set_staking_account(&unstaker, &staking_account);

		Ok(unstake_result)
	}

	fn get_thaw_at_epoch() -> <T as Config>::EpochNumber {
		let current_epoch: T::EpochNumber = Self::get_current_epoch();
		let thaw_period = T::UnstakingThawPeriod::get();
		current_epoch.saturating_add(thaw_period.into())
	}

	fn ensure_can_unstake(unstaker: &T::AccountId) -> Result<(), DispatchError> {
		let staking_account: StakingAccountDetails<T> =
			Self::get_staking_account_for(unstaker).ok_or(Error::<T>::NotAStakingAccount)?;
		ensure!(
			staking_account.unlocking.len().lt(&(T::MaxUnlockingChunks::get() as usize)),
			Error::<T>::MaxUnlockingChunksExceeded
		);
		Ok(())
	}

	/// Reduce available capacity of target and return the amount of capacity reduction.
	/// If withdrawing `amount` would take the StakingTargetDetails below the minimum staking amount,
	/// the entire amount is transferred and the record will be deleted. CapacityDetails will not
	/// be checked.
	/// Capacity reduction amount also depends on staking type.
	fn reduce_capacity(
		unstaker: &T::AccountId,
		target: MessageSourceId,
		amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let mut staking_target_details = Self::get_target_for(&unstaker, &target)
			.ok_or(Error::<T>::StakerTargetRelationshipNotFound)?;

		ensure!(amount.le(&staking_target_details.amount), Error::<T>::InsufficientStakingBalance);

		let mut capacity_details =
			Self::get_capacity_for(target).ok_or(Error::<T>::TargetCapacityNotFound)?;

		let capacity_to_withdraw =
			if staking_target_details.staking_type.eq(&StakingType::ProviderBoost) {
				Perbill::from_rational(amount, staking_target_details.amount)
					.mul_ceil(staking_target_details.capacity)
			} else {
				Self::calculate_capacity_reduction(
					amount,
					capacity_details.total_tokens_staked,
					capacity_details.total_capacity_issued,
				)
			};
		// this call will return an amount > than requested if the resulting StakingTargetDetails balance
		// is below the minimum. This ensures we withdraw the same amounts as for staking_target_details.
		let (actual_amount, actual_capacity) =
			staking_target_details.withdraw(amount, capacity_to_withdraw);

		capacity_details.withdraw(actual_capacity, actual_amount);

		Self::set_capacity_for(&target, &capacity_details);
		Self::set_target_details_for(unstaker, &target, &staking_target_details);

		Ok(capacity_to_withdraw)
	}

	/// Calculates Capacity generated for given FRQCY regardless of staking type
	fn capacity_generated(amount: BalanceOf<T>) -> BalanceOf<T> {
		let cpt = T::CapacityPerToken::get();
		cpt.mul(amount).into()
	}

	/// Determine the capacity reduction when given total_capacity, unstaking_amount, and total_amount_staked,
	/// based on ratios
	fn calculate_capacity_reduction(
		unstaking_amount: BalanceOf<T>,
		total_amount_staked: BalanceOf<T>,
		total_capacity: BalanceOf<T>,
	) -> BalanceOf<T> {
		Perbill::from_rational(unstaking_amount, total_amount_staked).mul_ceil(total_capacity)
	}

	fn start_new_epoch_if_needed(current_block: T::BlockNumber) -> Weight {
		// Should we start a new epoch?
		if current_block.saturating_sub(Self::get_current_epoch_info().epoch_start) >=
			Self::get_epoch_length()
		{
			let current_epoch = Self::get_current_epoch();
			CurrentEpoch::<T>::set(current_epoch.saturating_add(One::one()));
			CurrentEpochInfo::<T>::set(EpochInfo { epoch_start: current_block });
			T::WeightInfo::on_initialize()
				.saturating_add(T::DbWeight::get().reads(1))
				.saturating_add(T::DbWeight::get().writes(2))
		} else {
			// 1 for get_current_epoch_info, 1 for get_epoch_length
			T::DbWeight::get().reads(2).saturating_add(RocksDbWeight::get().writes(1))
		}
	}

	/// Returns whether `account_id` may claim and and be paid token rewards.
	pub fn payout_eligible(account_id: T::AccountId) -> bool {
		let _staking_account =
			Self::get_staking_account_for(account_id).ok_or(Error::<T>::StakingAccountNotFound);
		false
	}

	fn start_new_reward_era_if_needed(current_block: T::BlockNumber) -> Weight {
		let current_era_info: RewardEraInfo<T::RewardEra, T::BlockNumber> = Self::get_current_era(); // 1r
		if current_block.saturating_sub(current_era_info.started_at) >= T::EraLength::get().into() {
			// 1r
			let new_era_info = RewardEraInfo {
				era_index: current_era_info.era_index.saturating_add(One::one()),
				started_at: current_block,
			};

			let current_reward_pool_info =
				Self::get_reward_pool_for_era(current_era_info.era_index).unwrap_or_default(); // 1r
			let past_eras_max = T::StakingRewardsPastErasMax::get();
			let entries: u32 = StakingRewardPool::<T>::count();
			if past_eras_max.eq(&entries.into()) {
				// 2r
				let current_era = Self::get_current_era().era_index;
				let earliest_era = current_era.saturating_sub(past_eras_max).add(One::one());
				StakingRewardPool::<T>::remove(earliest_era); // 1w
			}
			CurrentEraInfo::<T>::set(new_era_info); // 1w

			// 		let msa_handle = T::HandleProvider::get_handle_for_msa(msa_id);
			let total_reward_pool =
				T::RewardsProvider::reward_pool_size(current_reward_pool_info.total_staked_token);
			let new_reward_pool = RewardPoolInfo {
				total_staked_token: current_reward_pool_info.total_staked_token,
				total_reward_pool,
				unclaimed_balance: total_reward_pool,
			};
			StakingRewardPool::<T>::insert(new_era_info.era_index, new_reward_pool); // 1w

			T::WeightInfo::on_initialize()
				.saturating_add(T::DbWeight::get().reads(5))
				.saturating_add(T::DbWeight::get().writes(3))
		} else {
			T::DbWeight::get().reads(2)
		}
	}

	/// Returns whether `account_id` may claim and and be paid token rewards.
	pub fn payout_eligible(account_id: T::AccountId) -> bool {
		let _staking_account =
			Self::get_staking_account_for(account_id).ok_or(Error::<T>::NotAStakingAccount);
		false
	}

	/// adds a new chunk to StakingAccountDetails.stake_change_unlocking.  The
	/// call `update_stake_change_unlocking` garbage-collects thawed chunks before adding the new one.
	fn add_change_staking_target_unlock_chunk(
		staker: &T::AccountId,
		amount: &BalanceOf<T>,
	) -> Result<(), DispatchError> {
		let mut staking_account_details: StakingAccountDetails<T> =
			Self::get_staking_account_for(staker).ok_or(Error::<T>::NotAStakingAccount)?;

		let current_era: T::RewardEra = Self::get_current_era().era_index;
		let thaw_eras = T::ChangeStakingTargetThawEras::get();
		let thaw_at = current_era.saturating_add(thaw_eras);
		staking_account_details.update_stake_change_unlocking(amount, &thaw_at, &current_era)?;
		Self::set_staking_account(staker, &staking_account_details);
		Ok(())
	}

	/// Performs the work of withdrawing the requested amount from the old staker-provider target details, and
	/// from the Provider's capacity details, and depositing it into the new staker-provider target details.
	pub fn do_retarget(
		staker: &T::AccountId,
		from_msa: &MessageSourceId,
		to_msa: &MessageSourceId,
		amount: &BalanceOf<T>,
		staking_type: &StakingType,
	) -> Result<(), DispatchError> {
		let capacity_withdrawn = Self::reduce_capacity(staker, *from_msa, *amount)?;

		let mut to_msa_target = Self::get_target_for(staker, to_msa).unwrap_or_default();

		if to_msa_target.amount.is_zero() {
			to_msa_target.staking_type = staking_type.clone();
		} else {
			ensure!(
				to_msa_target.staking_type.eq(staking_type),
				Error::<T>::CannotChangeStakingType
			);
		}
		to_msa_target
			.deposit(*amount, capacity_withdrawn)
			.ok_or(ArithmeticError::Overflow)?;

		let mut capacity_details = Self::get_capacity_for(to_msa).unwrap_or_default();
		capacity_details
			.deposit(amount, &capacity_withdrawn)
			.ok_or(ArithmeticError::Overflow)?;

		Self::set_target_details_for(staker, to_msa, &to_msa_target);
		Self::set_capacity_for(to_msa, &capacity_details);
		Self::add_change_staking_target_unlock_chunk(staker, amount)
	}
}

/// Nontransferable functions are intended for capacity spend and recharge.
/// Implementations of Nontransferable MUST NOT be concerned with StakingType.
impl<T: Config> Nontransferable for Pallet<T> {
	type Balance = BalanceOf<T>;

	/// Return the remaining capacity for the Provider MSA Id
	fn balance(msa_id: MessageSourceId) -> Self::Balance {
		match Self::get_capacity_for(msa_id) {
			Some(capacity_details) => capacity_details.remaining_capacity,
			None => BalanceOf::<T>::zero(),
		}
	}

	/// Spend capacity: reduce remaining (i.e. available to spend on messages)
	/// capacity by the given capacity_amount
	fn deduct(
		msa_id: MessageSourceId,
		capacity_amount: Self::Balance,
	) -> Result<(), DispatchError> {
		let mut capacity_details =
			Self::get_capacity_for(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;

		capacity_details
			.deduct_capacity_by_amount(capacity_amount)
			.map_err(|_| Error::<T>::InsufficientCapacityBalance)?;

		Self::set_capacity_for(&msa_id, &capacity_details);

		Self::deposit_event(Event::CapacityWithdrawn { msa_id, amount: capacity_amount });
		Ok(())
	}

	/// Increase all totals for the MSA's CapacityDetails. Currently unused.
	/// Capacity increase relative to token amount must be calculated in advance;
	/// this knows nothing about StakingType!
	fn deposit(
		msa_id: MessageSourceId,
		token_amount: Self::Balance,
		capacity_amount: Self::Balance,
	) -> Result<(), DispatchError> {
		let mut capacity_details =
			Self::get_capacity_for(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;
		capacity_details.deposit(&token_amount, &capacity_amount);
		Self::set_capacity_for(&msa_id, &capacity_details);
		// TODO: deposit CapacityDeposited event, new event
		Ok(())
	}
}

impl<T: Config> Replenishable for Pallet<T> {
	type Balance = BalanceOf<T>;

	fn replenish_all_for(msa_id: MessageSourceId) -> Result<(), DispatchError> {
		let mut capacity_details =
			Self::get_capacity_for(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;

		capacity_details.replenish_all(&Self::get_current_epoch());

		Self::set_capacity_for(&msa_id, &capacity_details);

		Ok(())
	}

	/// Change: now calls new fn replenish_by_amount on the capacity_details,
	/// which does what this (actually Self::deposit) used to do.
	/// Currently unused.
	fn replenish_by_amount(
		msa_id: MessageSourceId,
		amount: Self::Balance,
	) -> Result<(), DispatchError> {
		let mut capacity_details =
			Self::get_capacity_for(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;
		capacity_details.replenish_by_amount(amount, &Self::get_current_epoch());
		Ok(())
	}

	fn can_replenish(msa_id: MessageSourceId) -> bool {
		if let Some(capacity_details) = Self::get_capacity_for(msa_id) {
			return capacity_details.can_replenish(Self::get_current_epoch())
		}
		false
	}
}

impl<T: Config> StakingRewardsProvider<T> for Pallet<T> {
	type AccountId = T::AccountId;
	type RewardEra = T::RewardEra;
	type Hash = T::Hash;

	// Calculate the size of the reward pool for the current era, based on current staked token
	// and the other determined factors of the current economic model
	fn reward_pool_size(total_staked: BalanceOf<T>) -> BalanceOf<T> {
		if total_staked.is_zero() {
			return BalanceOf::<T>::zero()
		}

		// For now reward pool size is set to 10% of total staked token
		total_staked.checked_div(&BalanceOf::<T>::from(10u8)).unwrap_or_default()
	}

	// Performs range checks plus a reward calculation based on economic model for the era range
	fn staking_reward_total(
		_account_id: T::AccountId,
		from_era: T::RewardEra,
		to_era: T::RewardEra,
	) -> Result<BalanceOf<T>, DispatchError> {
		let era_range = from_era.saturating_sub(to_era);
		ensure!(era_range.le(&T::StakingRewardsPastErasMax::get()), Error::<T>::EraOutOfRange);
		ensure!(from_era.le(&to_era), Error::<T>::EraOutOfRange);
		let current_era_info = Self::get_current_era();
		ensure!(to_era.lt(&current_era_info.era_index), Error::<T>::EraOutOfRange);

		// For now rewards 1 unit per era for a valid range since there is no history storage
		let per_era = BalanceOf::<T>::one();

		let num_eras = to_era.saturating_sub(from_era);
		Ok(per_era.saturating_mul(num_eras.into()))
	}

	fn validate_staking_reward_claim(
		_account_id: T::AccountId,
		_proof: T::Hash,
		_payload: StakingRewardClaim<T>,
	) -> bool {
		true
	}

	/// How much, as a percentage of staked token, to boost a targeted Provider when staking.
	fn capacity_boost(amount: BalanceOf<T>) -> BalanceOf<T> {
		Perbill::from_percent(5u32).mul(amount)
	}
}
