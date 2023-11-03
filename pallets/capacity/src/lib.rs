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
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

use frame_support::{
	ensure,
	traits::{Currency, Get, Hooks, LockIdentifier, LockableCurrency, WithdrawReasons},
	weights::{constants::RocksDbWeight, Weight},
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

mod migration;
pub mod weights;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

const STAKING_ID: LockIdentifier = *b"netstkng";
use frame_system::pallet_prelude::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::pallet_prelude::{GetStorageVersion, StorageVersion};
	use frame_support::{pallet_prelude::*, Twox64Concat};
	use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeDisplay};

	/// the storage version for this pallet
	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Function that allows a balance to be locked.
		type Currency: LockableCurrency<Self::AccountId, Moment = BlockNumberFor<Self>>;

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
	/// - Value: [`StakingAccountDetailsV2`](types::StakingAccountDetailsV2)
	#[pallet::storage]
	#[pallet::getter(fn get_staking_account_for)]
	pub type StakingAccountLedger<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, StakingAccountDetailsV2<T>>;

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
		StakingTargetDetails<BalanceOf<T>>,
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
	pub type CurrentEpochInfo<T: Config> =
		StorageValue<_, EpochInfo<BlockNumberFor<T>>, ValueQuery>;

	#[pallet::type_value]
	/// EpochLength defaults to 100 blocks when not set
	pub fn EpochLengthDefault<T: Config>() -> BlockNumberFor<T> {
		100u32.into()
	}

	/// Storage for the epoch length
	#[pallet::storage]
	#[pallet::getter(fn get_epoch_length)]
	pub type EpochLength<T: Config> =
		StorageValue<_, BlockNumberFor<T>, ValueQuery, EpochLengthDefault<T>>;

	#[pallet::storage]
	#[pallet::getter(fn get_unstake_unlocking_for)]
	pub type UnstakeUnlocks<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, UnlockChunks<T>>;

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

		/// removes all thawed UnlockChunks from caller's StakingAccount and unlocks the sum of the thawed values
		/// in the caller's token account.
		///
		/// ### Errors
		///   - Returns `Error::NotAStakingAccount` if no StakingAccountDetailsV2 are found for `origin`.
		///   - Returns `Error::NoUnstakedTokensAvailable` if the account has no unstaking chunks or none are thawed.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::withdraw_unstaked())]
		pub fn withdraw_unstaked(origin: OriginFor<T>) -> DispatchResult {
			let staker = ensure_signed(origin)?;

			let mut staking_account =
				Self::get_staking_account_for(&staker).ok_or(Error::<T>::NotAStakingAccount)?;

			let current_epoch = Self::get_current_epoch();

			// TODO: call reap_thawed on unlock storage
			// let amount_withdrawn = staking_account.reap_thawed(current_epoch);
			let amount_withdrawn: BalanceOf<T> = Zero::zero();
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
	) -> Result<(StakingAccountDetailsV2<T>, BalanceOf<T>), DispatchError> {
		ensure!(amount > Zero::zero(), Error::<T>::ZeroAmountNotAllowed);
		ensure!(T::TargetValidator::validate(target), Error::<T>::InvalidTarget);

		let staking_account = Self::get_staking_account_for(&staker).unwrap_or_default();
		// let stakable_amount = staking_account.get_stakable_amount_for(&staker, amount);
		// TODO: correct amount
		let stakable_amount = amount.clone();

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
		staking_account: &mut StakingAccountDetailsV2<T>,
		target: MessageSourceId,
		amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		staking_account.deposit(amount).ok_or(ArithmeticError::Overflow)?;

		let capacity = Self::capacity_generated(amount);
		let mut target_details = Self::get_target_for(&staker, &target).unwrap_or_default();
		target_details.deposit(amount, capacity).ok_or(ArithmeticError::Overflow)?;

		let mut capacity_details = Self::get_capacity_for(target).unwrap_or_default();
		capacity_details.deposit(&amount, &capacity).ok_or(ArithmeticError::Overflow)?;

		Self::set_staking_account(&staker, staking_account);
		Self::set_target_details_for(&staker, target, target_details);
		Self::set_capacity_for(target, capacity_details);

		Ok(capacity)
	}

	/// Sets staking account details.
	fn set_staking_account(staker: &T::AccountId, staking_account: &StakingAccountDetailsV2<T>) {
		// TODO: adjust for storage in unlocks  (active is wrong)
		T::Currency::set_lock(STAKING_ID, &staker, staking_account.active, WithdrawReasons::all());
		StakingAccountLedger::<T>::insert(staker, staking_account);
	}

	/// Deletes staking account details
	fn delete_staking_account(staker: &T::AccountId) {
		T::Currency::remove_lock(STAKING_ID, &staker);
		StakingAccountLedger::<T>::remove(&staker);
	}

	/// If the staking account total is zero we reap storage, otherwise set the account to the new details.
	fn update_or_delete_staking_account(
		staker: &T::AccountId,
		staking_account: &StakingAccountDetailsV2<T>,
	) {
		// TODO: look for no key in unlock chunks
		if staking_account.active.is_zero() {
			Self::delete_staking_account(&staker);
		} else {
			Self::set_staking_account(&staker, &staking_account)
		}
	}

	/// Sets target account details.
	fn set_target_details_for(
		staker: &T::AccountId,
		target: MessageSourceId,
		target_details: StakingTargetDetails<BalanceOf<T>>,
	) {
		StakingTargetLedger::<T>::insert(staker, target, target_details);
	}

	/// Sets targets Capacity.
	pub fn set_capacity_for(
		target: MessageSourceId,
		capacity_details: CapacityDetails<BalanceOf<T>, T::EpochNumber>,
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
		ensure!(amount <= staking_account.active, Error::<T>::AmountToUnstakeExceedsAmountStaked);

		let current_epoch: T::EpochNumber = Self::get_current_epoch();
		let thaw_at =
			current_epoch.saturating_add(T::EpochNumber::from(T::UnstakingThawPeriod::get()));

		let unstake_result = staking_account.withdraw(amount, thaw_at)?;

		Self::set_staking_account(&unstaker, &staking_account);

		Ok(unstake_result)
	}

	/// Reduce available capacity of target and return the amount of capacity reduction.
	fn reduce_capacity(
		unstaker: &T::AccountId,
		target: MessageSourceId,
		amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let mut staking_target_details = Self::get_target_for(&unstaker, &target)
			.ok_or(Error::<T>::StakerTargetRelationshipNotFound)?;
		let mut capacity_details =
			Self::get_capacity_for(target).ok_or(Error::<T>::TargetCapacityNotFound)?;

		let capacity_to_withdraw = Self::calculate_capacity_reduction(
			amount,
			capacity_details.total_tokens_staked,
			capacity_details.total_capacity_issued,
		);
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
		if current_block.saturating_sub(Self::get_current_epoch_info().epoch_start)
			>= Self::get_epoch_length()
		{
			let current_epoch = Self::get_current_epoch();
			CurrentEpoch::<T>::set(current_epoch.saturating_add(1u32.into()));
			CurrentEpochInfo::<T>::set(EpochInfo { epoch_start: current_block });
			T::WeightInfo::on_initialize()
				.saturating_add(T::DbWeight::get().reads(1))
				.saturating_add(T::DbWeight::get().writes(2))
		} else {
			// 1 for get_current_epoch_info, 1 for get_epoch_length
			T::DbWeight::get().reads(2u64).saturating_add(RocksDbWeight::get().writes(1))
		}
	}
}

impl<T: Config> Nontransferable for Pallet<T> {
	type Balance = BalanceOf<T>;

	/// Return the remaining capacity for the Provider MSA Id
	fn balance(msa_id: MessageSourceId) -> Self::Balance {
		match Self::get_capacity_for(msa_id) {
			Some(capacity_details) => capacity_details.remaining_capacity,
			None => BalanceOf::<T>::zero(),
		}
	}

	/// Spend capacity: reduce remaining capacity by the given amount
	fn deduct(msa_id: MessageSourceId, amount: Self::Balance) -> Result<(), DispatchError> {
		let mut capacity_details =
			Self::get_capacity_for(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;

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
			Self::get_capacity_for(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;
		capacity_details.deposit(&amount, &Self::capacity_generated(amount));
		Self::set_capacity_for(msa_id, capacity_details);
		Ok(())
	}
}

impl<T: Config> Replenishable for Pallet<T> {
	type Balance = BalanceOf<T>;

	fn replenish_all_for(msa_id: MessageSourceId) -> Result<(), DispatchError> {
		let mut capacity_details =
			Self::get_capacity_for(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;

		capacity_details.replenish_all(&Self::get_current_epoch());

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
			Self::get_capacity_for(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;
		capacity_details.replenish_by_amount(amount, &Self::get_current_epoch());
		Ok(())
	}

	fn can_replenish(msa_id: MessageSourceId) -> bool {
		if let Some(capacity_details) = Self::get_capacity_for(msa_id) {
			return capacity_details.can_replenish(Self::get_current_epoch());
		}
		false
	}
}
