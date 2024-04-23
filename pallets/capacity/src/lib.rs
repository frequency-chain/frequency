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
use sp_std::ops::{Add, Mul};

use frame_support::{
	ensure,
	traits::{
		tokens::fungible::{Inspect as InspectFungible, InspectFreeze, Mutate, MutateFreeze},
		Get, Hooks,
	},
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
pub(crate) type BalanceOf<T> =
	<<T as Config>::Currency as InspectFungible<<T as frame_system::Config>::AccountId>>::Balance;

use frame_system::pallet_prelude::*;
use crate::StakingType::{MaximumCapacity, ProviderBoost};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{pallet_prelude::*, Twox64Concat};
	use parity_scale_codec::EncodeLike;
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
		type MaxUnlockingChunks: Get<u32> + Clone;

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

		/// The number of blocks in a RewardEra
		#[pallet::constant]
		type EraLength: Get<u32>;

		/// The maximum number of eras over which one can claim rewards
		#[pallet::constant]
		type StakingRewardsPastErasMax: Get<u32>;

		/// The StakingRewardsProvider used by this pallet in a given runtime
		type RewardsProvider: StakingRewardsProvider<Self>;

		/// A staker may not retarget more than MaxRetargetsPerRewardEra
		#[pallet::constant]
		type MaxRetargetsPerRewardEra: Get<u32>;
	}

	/// Storage for keeping a ledger of staked token amounts for accounts.
	/// - Keys: AccountId
	/// - Value: [`StakingDetails`](types::StakingDetails)
	#[pallet::storage]
	#[pallet::getter(fn get_staking_account_for)]
	pub type StakingAccountLedger<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, StakingDetails<T>>;

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
	pub type UnstakeUnlocks<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, UnlockChunkList<T>>;

	/// Information about the current staking reward era. Checked every block.
	#[pallet::storage]
	#[pallet::whitelist_storage]
	#[pallet::getter(fn get_current_era)]
	pub type CurrentEraInfo<T: Config> =
	StorageValue<_, RewardEraInfo<T::RewardEra, BlockNumberFor<T>>, ValueQuery>;

	/// Reward Pool history
	#[pallet::storage]
	#[pallet::getter(fn get_reward_pool_for_era)]
	pub type StakingRewardPool<T: Config> =
	CountedStorageMap<_, Twox64Concat, T::RewardEra, RewardPoolInfo<BalanceOf<T>>>;

	// TODO: storage for staking history

	/// stores how many times an account has retargeted, and when it last retargeted.
	#[pallet::storage]
	#[pallet::getter(fn get_retargets_for)]
	pub type Retargets<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, RetargetInfo<T>>;

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
		/// Tokens have been staked on the network for Provider Boosting
		ProviderBoosted {
			/// The token account that staked tokens to the network.
			account: T::AccountId,
			/// The MSA that a token account targeted to receive Capacity based on this staking amount.
			target: MessageSourceId,
			/// An amount that was staked.
			amount: BalanceOf<T>,
			/// The Capacity amount issued to the target as a result of the stake.
			capacity: BalanceOf<T>,
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
		/// or nothing has passed the thaw period.  (5)
		NoUnstakedTokensAvailable,
		/// Unstaking amount should be greater than zero.
		UnstakedAmountIsZero,
		/// Amount to unstake or change targets is greater than the amount staked.
		InsufficientStakingBalance,
		/// Attempted to get a staker / target relationship that does not exist.
		StakerTargetRelationshipNotFound,
		/// Attempted to get the target's capacity that does not exist.
		TargetCapacityNotFound,
		/// Staker has reached the limit of unlocking chunks and must wait for at least one thaw period
		/// to complete. (10)
		MaxUnlockingChunksExceeded,
		/// Capacity increase exceeds the total available Capacity for target.
		IncreaseExceedsAvailable,
		/// Attempted to set the Epoch length to a value greater than the max Epoch length.
		MaxEpochLengthExceeded,
		/// Staker is attempting to stake an amount that leaves a token balance below the minimum amount.
		BalanceTooLowtoStake,
		/// Staker tried to change StakingType on an existing account
		CannotChangeStakingType,
		/// The Era specified is too far in the past or is in the future (15)
		EraOutOfRange,
		/// Rewards were already paid out for the specified Era range
		IneligibleForPayoutInEraRange,
		/// Attempted to retarget but from and to Provider MSA Ids were the same
		CannotRetargetToSameProvider,
		/// Rewards were already paid out this era
		AlreadyClaimedRewardsThisEra,
		/// Caller must wait until the next RewardEra, claim rewards and then can boost again.
		MustFirstClaimRewards,
		/// Too many change_staking_target calls made in this RewardEra. (20)
		MaxRetargetsExceeded,
		/// There are no unstaked token amounts that have passed their thaw period.
		NoThawedTokenAvailable
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(current: BlockNumberFor<T>) -> Weight {
			Self::start_new_epoch_if_needed(current)
				.saturating_add(
					Self::start_new_reward_era_if_needed(current)
			)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Stakes some amount of tokens to the network and generates Capacity.
		///
		/// ### Errors
		///
		/// - Returns Error::InvalidTarget if attempting to stake to an invalid target.
		/// - Returns Error::StakingAmountBelowMinimum if attempting to stake an amount below the minimum amount.
		/// - Returns Error::CannotChangeStakingType if the staking account is a ProviderBoost account
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::stake())]
		pub fn stake(
			origin: OriginFor<T>,
			target: MessageSourceId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let staker = ensure_signed(origin)?;

			let (mut staking_account, actual_amount) =
				Self::ensure_can_stake(&staker, target, amount, MaximumCapacity)?;

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

			let (actual_amount, staking_type) = Self::decrease_active_staking_balance(&unstaker, requested_amount)?;
			Self::add_unlock_chunk(&unstaker, actual_amount)?;

			let capacity_reduction = Self::reduce_capacity(&unstaker, target, actual_amount, staking_type)?;

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

		/// Sets the target of the staking capacity to a new target.
		/// This adds a chunk to `StakingAccountDetails.stake_change_unlocking chunks`, up to `T::MaxUnlockingChunks`.
		/// The staked amount and Capacity generated by `amount` originally targeted to the `from` MSA Id is reassigned to the `to` MSA Id.
		/// Does not affect unstaking process or additional stake amounts.
		/// Changing a staking target to a Provider when Origin has nothing staked them will retain the staking type.
		/// Changing a staking target to a Provider when Origin has any amount staked to them will error if the staking types are not the same.
		/// ### Errors
		/// - [`Error::MaxUnlockingChunksExceeded`] if `stake_change_unlocking_chunks` == `T::MaxUnlockingChunks`
		/// - [`Error::StakerTargetRelationshipNotFound`] if `from` is not a target for Origin's staking account.
		/// - [`Error::StakingAmountBelowMinimum`] if `amount` to retarget is below the minimum staking amount.
		/// - [`Error::InsufficientStakingBalance`] if `amount` to retarget exceeds what the staker has targeted to `from` MSA Id.
		/// - [`Error::InvalidTarget`] if `to` does not belong to a registered Provider.
		/// - [`Error::MaxRetargetsExceeded`] if origin has reached the maximimum number of retargets for the current RewardEra.
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::unstake())]
		pub fn change_staking_target(
			origin: OriginFor<T>,
			from: MessageSourceId,
			to: MessageSourceId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let staker = ensure_signed(origin)?;
			// This will bounce immediately if they've tried to do this too many times.
			Self::update_retarget_record(&staker)?;
			ensure!(from.ne(&to), Error::<T>::CannotRetargetToSameProvider);
			ensure!(
				amount >= T::MinimumStakingAmount::get(),
				Error::<T>::StakingAmountBelowMinimum
			);

			ensure!(T::TargetValidator::validate(to), Error::<T>::InvalidTarget);

			Self::do_retarget(&staker, &from, &to, &amount)?;

			Self::deposit_event(Event::StakingTargetChanged {
				account: staker,
				from_msa: from,
				to_msa: to,
				amount,
			});
			Ok(())
		}
		/// Stakes some amount of tokens to the network and generates a comparatively small amount of Capacity
		/// for the target, and gives periodic rewards to origin.
		/// ### Errors
		///
		/// - Returns Error::InvalidTarget if attempting to stake to an invalid target.
		/// - Returns Error::StakingAmountBelowMinimum if attempting to stake an amount below the minimum amount.
		/// - Returns Error::CannotChangeStakingType if the staking account exists and staking_type is MaximumCapacity
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::provider_boost())]
		pub fn provider_boost(
			origin: OriginFor<T>,
			target: MessageSourceId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let staker = ensure_signed(origin)?;
			let (mut boosting_details, actual_amount) =
				Self::ensure_can_boost(&staker, &target, &amount)?;

			let capacity = Self::increase_stake_and_issue_boost(
				&staker,
				&mut boosting_details,
				&target,
				&actual_amount,
			)?;

			Self::deposit_event(Event::ProviderBoosted {
				account: staker,
				amount: actual_amount,
				target,
				capacity,
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
	/// * [`Error::ZeroAmountNotAllowed`]
	/// * [`Error::InvalidTarget`]
	/// * [`Error::BalanceTooLowtoStake`]
	///
	fn ensure_can_stake(
		staker: &T::AccountId,
		target: MessageSourceId,
		amount: BalanceOf<T>,
		staking_type: StakingType,
	) -> Result<(StakingDetails<T>, BalanceOf<T>), DispatchError> {
		ensure!(amount > Zero::zero(), Error::<T>::ZeroAmountNotAllowed);
		ensure!(T::TargetValidator::validate(target), Error::<T>::InvalidTarget);

		let staking_details = Self::get_staking_account_for(&staker).unwrap_or_default();
		if !staking_details.active.is_zero() {
			ensure!(staking_details.staking_type.eq(&staking_type), Error::<T>::CannotChangeStakingType);
		}

		let stakable_amount = Self::get_stakable_amount_for(&staker, amount);

		ensure!(stakable_amount > Zero::zero(), Error::<T>::BalanceTooLowtoStake);

		let new_active_staking_amount = staking_details
			.active
			.checked_add(&stakable_amount)
			.ok_or(ArithmeticError::Overflow)?;

		ensure!(
			new_active_staking_amount >= T::MinimumStakingAmount::get(),
			Error::<T>::StakingAmountBelowMinimum
		);

		Ok((staking_details, stakable_amount))
	}

	fn ensure_can_boost(
		staker: &T::AccountId,
		target: &MessageSourceId,
		amount: &BalanceOf<T>,
	) -> Result<(StakingDetails<T>, BalanceOf<T>), DispatchError> {
		// FIXME: if one is boosting additional amounts, this will fail.
		let (mut staking_details, stakable_amount) = 
			Self::ensure_can_stake(staker, *target, *amount, ProviderBoost)?;
		staking_details.staking_type = ProviderBoost;
		// TODO: update when boost history is implemented fully
		// let boost_history = Self::get_boost_history_for(staker).unwrap_or_default();
		Ok((staking_details, stakable_amount))
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
		let mut target_details = Self::get_target_for(&staker, &target).unwrap_or_default();
		target_details.deposit(amount, capacity).ok_or(ArithmeticError::Overflow)?;

		let mut capacity_details = Self::get_capacity_for(target).unwrap_or_default();
		capacity_details.deposit(&amount, &capacity).ok_or(ArithmeticError::Overflow)?;

		Self::set_staking_account_and_lock(&staker, staking_account)?;

		Self::set_target_details_for(&staker, target, target_details);
		Self::set_capacity_for(target, capacity_details);

		Ok(capacity)
	}

	fn increase_stake_and_issue_boost(
		staker: &T::AccountId,
		staking_details: &mut StakingDetails<T>,
		target: &MessageSourceId,
		amount: &BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		staking_details
			.deposit(*amount)
			.ok_or(ArithmeticError::Overflow)?;

		// get the capacity generated by a Provider Boost
		let capacity = Self::capacity_generated(T::RewardsProvider::capacity_boost(*amount));

		let mut target_details = Self::get_target_for(staker, target).unwrap_or_default();

		target_details.deposit(*amount, capacity).ok_or(ArithmeticError::Overflow)?;

		let mut capacity_details = Self::get_capacity_for(target).unwrap_or_default();
		capacity_details.deposit(amount, &capacity).ok_or(ArithmeticError::Overflow)?;

		// TODO:  add boost history record for era when boost history is implemented
		// let era = Self::get_current_era().era_index;
		Self::set_staking_account_and_lock(staker, staking_details)?;
		Self::set_target_details_for(staker, *target, target_details);
		Self::set_capacity_for(*target, capacity_details);

		Ok(capacity)
	}

	/// Sets staking account details after a deposit
	fn set_staking_account_and_lock(
		staker: &T::AccountId,
		staking_account: &StakingDetails<T>,
	) -> Result<(), DispatchError> {
		let unlocks = Self::get_unstake_unlocking_for(staker).unwrap_or_default();
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

	/// If the staking account total is zero we reap storage, otherwise set the account to the new details.
	// TODO: do not remove this lock unless both types of staking details are cleared.
	// fn delete_boosting_account(staker: &T::AccountId) {
	// 	// otherwise call set_lock for the new value containing only the other type of staking account.
	// 	T::Currency::remove_lock(STAKING_ID, &staker);
	// 	BoostingAccountLedger::<T>::remove(&staker);
	// }

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

	/// Decrease a staking account's active token and reap if it goes below the minimum.
	/// Returns: actual amount unstaked, plus the staking type + StakingDetails,
	/// since StakingDetails may be reaped and staking type must be used to calculate the
	/// capacity reduction later.
	fn decrease_active_staking_balance(
		unstaker: &T::AccountId,
		amount: BalanceOf<T>,
	) -> Result<(BalanceOf<T>, StakingType), DispatchError> {
		let mut staking_account =
			Self::get_staking_account_for(unstaker).ok_or(Error::<T>::NotAStakingAccount)?;
		ensure!(amount <= staking_account.active, Error::<T>::InsufficientStakingBalance);

		let actual_unstaked_amount = staking_account.withdraw(amount)?;
		let staking_type = staking_account.staking_type;
		Self::set_staking_account(unstaker, &staking_account);
		Ok((actual_unstaked_amount, staking_type))
	}

	fn add_unlock_chunk(
		unstaker: &T::AccountId,
		actual_unstaked_amount: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		let current_epoch: T::EpochNumber = Self::get_current_epoch();
		let thaw_at =
			current_epoch.saturating_add(T::EpochNumber::from(T::UnstakingThawPeriod::get()));
		let mut unlocks = Self::get_unstake_unlocking_for(unstaker).unwrap_or_default();

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
		let current_epoch = Self::get_current_epoch();
		let mut total_unlocking: BalanceOf<T> = Zero::zero();

		let mut unlocks =
			Self::get_unstake_unlocking_for(staker).ok_or(Error::<T>::NoUnstakedTokensAvailable)?;
		let amount_withdrawn = unlock_chunks_reap_thawed::<T>(&mut unlocks, current_epoch);
		ensure!(!amount_withdrawn.is_zero(), Error::<T>::NoThawedTokenAvailable);

		if unlocks.is_empty() {
			UnstakeUnlocks::<T>::set(staker, None);
		} else {
			total_unlocking = unlock_chunks_total::<T>(&unlocks);
			UnstakeUnlocks::<T>::set(staker, Some(unlocks));
		}

		let staking_account = Self::get_staking_account_for(staker).unwrap_or_default();
		let total_locked = staking_account.active.saturating_add(total_unlocking);
		if total_locked.is_zero() {
			T::Currency::thaw(&FreezeReason::CapacityStaking.into(), staker)?;
		} else {
			T::Currency::set_freeze(&FreezeReason::CapacityStaking.into(), staker, total_locked)?;
		}
		Ok(amount_withdrawn)
	}

	fn get_thaw_at_epoch() -> <T as Config>::EpochNumber {
		let current_epoch: T::EpochNumber = Self::get_current_epoch();
		let thaw_period = T::UnstakingThawPeriod::get();
		current_epoch.saturating_add(thaw_period.into())
	}

	/// Reduce available capacity of target and return the amount of capacity reduction.
	fn reduce_capacity(
		unstaker: &T::AccountId,
		target: MessageSourceId,
		amount: BalanceOf<T>,
		staking_type: StakingType
	) -> Result<BalanceOf<T>, DispatchError> {
		let mut staking_target_details = Self::get_target_for(&unstaker, &target)
			.ok_or(Error::<T>::StakerTargetRelationshipNotFound)?;

		ensure!(amount.le(&staking_target_details.amount), Error::<T>::InsufficientStakingBalance);

		let mut capacity_details =
			Self::get_capacity_for(target).ok_or(Error::<T>::TargetCapacityNotFound)?;

		let capacity_to_withdraw = if staking_target_details.amount.eq(&amount) {
			staking_target_details.capacity
		} else {
            if staking_type.eq(&StakingType::ProviderBoost) {
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
		};

        // TODO: update this function
        let (actual_amount, actual_capacity) = staking_target_details.withdraw(
            amount,
            capacity_to_withdraw,
            T::MinimumStakingAmount::get(),
        );
        staking_target_details.withdraw(amount, actual_capacity);
        capacity_details.withdraw(actual_capacity, actual_amount);

		Self::set_capacity_for(target, capacity_details);
		Self::set_target_details_for(unstaker, target, staking_target_details);

		Ok(capacity_to_withdraw)
	}

	/// Calculates Capacity generated for given FRQCY
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

	fn start_new_epoch_if_needed(current_block: BlockNumberFor<T>) -> Weight {
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

	fn start_new_reward_era_if_needed(current_block: BlockNumberFor<T>) -> Weight {
		let current_era_info: RewardEraInfo<T::RewardEra, BlockNumberFor<T>> = Self::get_current_era(); // 1r

		if current_block.saturating_sub(current_era_info.started_at) >= T::EraLength::get().into() {
			let new_era_info = RewardEraInfo {
				era_index: current_era_info.era_index.saturating_add(One::one()),
				started_at: current_block,
			};

			let current_reward_pool_info =
				Self::get_reward_pool_for_era(current_era_info.era_index).unwrap_or_default(); // 1r

			let past_eras_max = T::StakingRewardsPastErasMax::get();
			let entries: u32 = StakingRewardPool::<T>::count(); // 1r

			if past_eras_max.eq(&entries) {
				let earliest_era =
					current_era_info.era_index.saturating_sub(past_eras_max.into()).add(One::one());
				StakingRewardPool::<T>::remove(earliest_era); // 1w
			}
			CurrentEraInfo::<T>::set(new_era_info); // 1w

			let total_reward_pool =
				T::RewardsProvider::reward_pool_size(current_reward_pool_info.total_staked_token);
			let new_reward_pool = RewardPoolInfo {
				total_staked_token: current_reward_pool_info.total_staked_token,
				total_reward_pool,
				unclaimed_balance: total_reward_pool,
			};
			StakingRewardPool::<T>::insert(new_era_info.era_index, new_reward_pool); // 1w

			T::WeightInfo::on_initialize()
				.saturating_add(T::DbWeight::get().reads(3))
				.saturating_add(T::DbWeight::get().writes(3))
		} else {
			T::DbWeight::get().reads(1)
		}
	}

	/// Returns whether `account_id` may claim and and be paid token rewards.
	pub fn payout_eligible(account_id: T::AccountId) -> bool {
		let _staking_account =
			Self::get_staking_account_for(account_id).ok_or(Error::<T>::NotAStakingAccount);
		false
	}

	/// attempts to increment number of retargets this RewardEra
	/// Returns:
	///     Error::MaxRetargetsExceeded if they try to retarget too many times in one era.
	fn update_retarget_record(staker: &T::AccountId) -> Result<(), DispatchError> {
		let current_era: T::RewardEra = Self::get_current_era().era_index;
		let mut retargets = Self::get_retargets_for(staker).unwrap_or_default();
		ensure!(retargets.update(current_era).is_some(), Error::<T>::MaxRetargetsExceeded);
		Retargets::<T>::set(staker, Some(retargets));
		Ok(())
	}

	/// Performs the work of withdrawing the requested amount from the old staker-provider target details, and
	/// from the Provider's capacity details, and depositing it into the new staker-provider target details.
	pub fn do_retarget(
		staker: &T::AccountId,
		from_msa: &MessageSourceId,
		to_msa: &MessageSourceId,
		amount: &BalanceOf<T>,
	) -> Result<(), DispatchError> {
		let staking_type = Self::get_staking_account_for(staker).unwrap_or_default().staking_type;
		let capacity_withdrawn = Self::reduce_capacity(staker, *from_msa, *amount, staking_type)?;

		let mut to_msa_target = Self::get_target_for(staker, to_msa).unwrap_or_default();

		to_msa_target
			.deposit(*amount, capacity_withdrawn)
			.ok_or(ArithmeticError::Overflow)?;

		let mut capacity_details = Self::get_capacity_for(to_msa).unwrap_or_default();
		capacity_details
			.deposit(amount, &capacity_withdrawn)
			.ok_or(ArithmeticError::Overflow)?;

		Self::set_target_details_for(staker, *to_msa, to_msa_target);
		Self::set_capacity_for(*to_msa, capacity_details);
		Ok(())
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

	/// Spend capacity: reduce remaining capacity by the given amount
	fn deduct(msa_id: MessageSourceId, amount: Self::Balance) -> Result<(), DispatchError> {
		let mut capacity_details =
			Self::get_capacity_for(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;

		capacity_details
			.deduct_capacity_by_amount(amount)
			.map_err(|_| Error::<T>::InsufficientCapacityBalance)?;

		Self::set_capacity_for(msa_id, capacity_details);

		Self::deposit_event(Event::CapacityWithdrawn { msa_id, amount });
		Ok(())
	}

	/// Increase all totals for the MSA's CapacityDetails.
	fn deposit(msa_id: MessageSourceId, token_amount: Self::Balance, capacity_amount: Self::Balance) -> Result<(), DispatchError> {
		let mut capacity_details =
			Self::get_capacity_for(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;
		capacity_details.deposit(&token_amount, &capacity_amount);
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
		ensure!(
			era_range.le(&T::StakingRewardsPastErasMax::get().into()),
			Error::<T>::EraOutOfRange
		);
		ensure!(from_era.le(&to_era), Error::<T>::EraOutOfRange);
		let current_era_info = Self::get_current_era();
		ensure!(to_era.lt(&current_era_info.era_index), Error::<T>::EraOutOfRange);

		// TODO: update when staking history is implemented
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
		Perbill::from_percent(50u32).mul(amount)
	}
}
