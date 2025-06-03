//! Managages staking to the network for Capacity
//!
//! ## Quick Links
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Runtime API: `CapacityRuntimeApi`](../pallet_capacity_runtime_api/trait.CapacityRuntimeApi.html)
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

use core::ops::Mul;

use frame_support::{
	ensure,
	traits::{
		tokens::fungible::{Inspect as InspectFungible, InspectFreeze, Mutate, MutateFreeze},
		Get, Hooks,
	},
	weights::Weight,
};

use sp_runtime::{
	traits::{CheckedAdd, CheckedDiv, One, Saturating, Zero},
	ArithmeticError, BoundedVec, DispatchError, Perbill, Permill,
};

pub use common_primitives::{
	capacity::*,
	msa::MessageSourceId,
	node::{AccountId, Balance, BlockNumber},
	utils::wrap_binary_data,
};

use frame_system::pallet_prelude::*;

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

pub mod weights;
type BalanceOf<T> =
	<<T as Config>::Currency as InspectFungible<<T as frame_system::Config>::AccountId>>::Balance;
type ChunkIndex = u32;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use crate::StakingType::*;
	use common_primitives::capacity::RewardEra;
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
	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

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
			+ core::hash::Hash
			+ MaxEncodedLen
			+ TypeInfo;

		/// How much FRQCY one unit of Capacity costs
		#[pallet::constant]
		type CapacityPerToken: Get<Perbill>;

		/// The number of blocks in a RewardEra
		#[pallet::constant]
		type EraLength: Get<u32>;

		/// The maximum number of eras over which one can claim rewards
		/// Note that you can claim rewards even if you no longer are boosting, because you
		/// may claim rewards for past eras up to the history limit.
		/// MUST be a multiple of [`Self::RewardPoolChunkLength`]
		#[pallet::constant]
		type ProviderBoostHistoryLimit: Get<u32>;

		/// The ProviderBoostRewardsProvider used by this pallet in a given runtime
		type RewardsProvider: ProviderBoostRewardsProvider<Self>;

		/// A staker may not retarget more than MaxRetargetsPerRewardEra
		#[pallet::constant]
		type MaxRetargetsPerRewardEra: Get<u32>;

		/// The fixed size of the reward pool in each Reward Era.
		#[pallet::constant]
		type RewardPoolPerEra: Get<BalanceOf<Self>>;

		/// the percentage cap per era of an individual Provider Boost reward
		#[pallet::constant]
		type RewardPercentCap: Get<Permill>;

		/// The number of chunks of Reward Pool history we expect to store
		/// Is a divisor of [`Self::ProviderBoostHistoryLimit`]
		#[pallet::constant]
		type RewardPoolChunkLength: Get<u32>;

		/// Max differece between PTE and current block number
		#[pallet::constant]
		type MaxPteDifferenceFromCurrentBlock: Get<u32>;

		/// The origin that is allowed to set Precipitating Tokenomic Event (PTE) via governance
		type PteGovernanceOrigin: EnsureOrigin<Self::RuntimeOrigin>;
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

	/// stores how many times an account has retargeted, and when it last retargeted.
	#[pallet::storage]
	pub type Retargets<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, RetargetInfo<T>>;

	/// Information about the current reward era. Checked every block.
	#[pallet::storage]
	#[pallet::whitelist_storage]
	pub type CurrentEraInfo<T: Config> =
		StorageValue<_, RewardEraInfo<RewardEra, BlockNumberFor<T>>, ValueQuery>;

	/// Reward Pool history is divided into chunks of size RewardPoolChunkLength.
	/// ProviderBoostHistoryLimit is the total number of items, the key is the
	/// chunk number.
	#[pallet::storage]
	pub type ProviderBoostRewardPools<T: Config> =
		StorageMap<_, Twox64Concat, ChunkIndex, RewardPoolHistoryChunk<T>>;

	/// How much is staked this era
	#[pallet::storage]
	pub type CurrentEraProviderBoostTotal<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// Individual history for each account that has Provider-Boosted.
	#[pallet::storage]
	pub type ProviderBoostHistories<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, ProviderBoostHistory<T>>;

	// Governance-triggered event that starts unlock schedule for Committed Boosting.
	#[pallet::storage]
	pub type PrecipitatingEventBlockNumber<T: Config> =
		StorageValue<_, BlockNumberFor<T>, OptionQuery>;

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
		/// Provider Boost Token Rewards have been minted and transferred to the staking account.
		ProviderBoostRewardClaimed {
			/// The token account claiming and receiving the reward from ProviderBoost staking
			account: T::AccountId,
			/// The reward amount
			reward_amount: BalanceOf<T>,
		},
		/// Precipitating Tokenomic Event value is set
		PrecipitatingTokenomicEventSet {
			/// The block number of the event
			at: BlockNumberFor<T>,
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
		/// #[deprecated(since = "1.13.0", note = "Use StakingAmountBelowMinimum instead")]
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
		/// There are no unstaked token amounts that have passed their thaw period.
		NoThawedTokenAvailable,
		/// Staker tried to change StakingType on an existing account
		CannotChangeStakingType,
		/// The Era specified is too far in the past or is in the future (15)
		EraOutOfRange,
		/// Attempted to retarget but from and to Provider MSA Ids were the same
		CannotRetargetToSameProvider,
		/// There are no rewards eligible to claim.  Rewards have expired, have already been
		/// claimed, or boosting has never been done before the current era.
		NoRewardsEligibleToClaim,
		/// Caller must claim rewards before unstaking.
		MustFirstClaimRewards,
		/// Too many change_staking_target calls made in this RewardEra. (20)
		MaxRetargetsExceeded,
		/// Tried to exceed bounds of a some Bounded collection
		CollectionBoundExceeded,
		/// This origin has nothing staked for ProviderBoost.
		NotAProviderBoostAccount,
		/// Pte value is not in the valid window
		InvalidPteValue,
		/// Pte value is alrerady set
		PteValueAlreadySet,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(current: BlockNumberFor<T>) -> Weight {
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

		/// Removes all thawed UnlockChunks from caller's UnstakeUnlocks and thaws(unfreezes) the sum of the thawed values
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
		/// - Returns `Error::NotAStakingAccount` if `origin` has nothing staked at all
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

			ensure!(!Self::has_unclaimed_rewards(&unstaker), Error::<T>::MustFirstClaimRewards);

			let (actual_amount, staking_type) =
				Self::decrease_active_staking_balance(&unstaker, requested_amount)?;
			Self::add_unlock_chunk(&unstaker, actual_amount)?;

			let capacity_reduction =
				Self::reduce_capacity(&unstaker, target, actual_amount, staking_type)?;

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
		/// This adds a chunk to `StakingDetails.stake_change_unlocking chunks`, up to `T::MaxUnlockingChunks`.
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
		#[pallet::weight(T::WeightInfo::change_staking_target())]
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
		/// - Error::InvalidTarget if attempting to stake to an invalid target.
		/// - Error::StakingAmountBelowMinimum if attempting to stake an amount below the minimum amount.
		/// - Error::CannotChangeStakingType if the staking account exists and staking_type is MaximumCapacity
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

			let capacity = Self::increase_stake_and_issue_boost_capacity(
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

		/// Claim all outstanding Provider Boost rewards, up to ProviderBoostHistoryLimit Reward Eras
		/// in the past.  Accounts should check for unclaimed rewards before calling this extrinsic
		/// to avoid needless transaction fees.
		/// ### Errors:
		/// - NotAProviderBoostAccount:  if Origin has nothing staked for ProviderBoost
		/// - NoRewardsEligibleToClaim:  if Origin has no unclaimed rewards to pay out.
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::claim_staking_rewards())]
		pub fn claim_staking_rewards(origin: OriginFor<T>) -> DispatchResult {
			let staker = ensure_signed(origin)?;
			ensure!(
				ProviderBoostHistories::<T>::contains_key(staker.clone()),
				Error::<T>::NotAProviderBoostAccount
			);
			let total_to_mint = Self::do_claim_rewards(&staker)?;
			Self::deposit_event(Event::ProviderBoostRewardClaimed {
				account: staker.clone(),
				reward_amount: total_to_mint,
			});
			Ok(())
		}

		/// Sets the Precipitating Tokenomic Event value via Governance
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::set_pte_via_governance())]
		pub fn set_pte_via_governance(
			origin: OriginFor<T>,
			pte_block_number: BlockNumberFor<T>,
		) -> DispatchResult {
			T::PteGovernanceOrigin::ensure_origin(origin)?;

			let current_block = frame_system::Pallet::<T>::block_number();
			let bigger = pte_block_number.max(current_block);
			let smaller = pte_block_number.min(current_block);
			ensure!(
				bigger - smaller < T::MaxPteDifferenceFromCurrentBlock::get().into(),
				Error::<T>::InvalidPteValue
			);
			ensure!(
				PrecipitatingEventBlockNumber::<T>::get().is_none(),
				Error::<T>::PteValueAlreadySet
			);

			PrecipitatingEventBlockNumber::<T>::set(Some(pte_block_number));
			Self::deposit_event(Event::PrecipitatingTokenomicEventSet { at: pte_block_number });
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
	/// * [`Error::CannotChangeStakingType`]
	/// * [`Error::BalanceTooLowtoStake`]
	/// * [`Error::StakingAmountBelowMinimum`]
	///
	fn ensure_can_stake(
		staker: &T::AccountId,
		target: MessageSourceId,
		amount: BalanceOf<T>,
		staking_type: StakingType,
	) -> Result<(StakingDetails<T>, BalanceOf<T>), DispatchError> {
		ensure!(amount > Zero::zero(), Error::<T>::ZeroAmountNotAllowed);
		ensure!(T::TargetValidator::validate(target), Error::<T>::InvalidTarget);

		let staking_details = StakingAccountLedger::<T>::get(staker).unwrap_or_default();
		if !staking_details.active.is_zero() {
			ensure!(
				staking_details.staking_type.eq(&staking_type),
				Error::<T>::CannotChangeStakingType
			);
		}

		let stakable_amount = Self::get_stakable_amount_for(staker, amount);

		ensure!(stakable_amount > Zero::zero(), Error::<T>::BalanceTooLowtoStake);
		ensure!(
			stakable_amount >= T::MinimumStakingAmount::get(),
			Error::<T>::StakingAmountBelowMinimum
		);

		Ok((staking_details, stakable_amount))
	}

	fn ensure_can_boost(
		staker: &T::AccountId,
		target: &MessageSourceId,
		amount: &BalanceOf<T>,
	) -> Result<(StakingDetails<T>, BalanceOf<T>), DispatchError> {
		let (mut staking_details, stakable_amount) =
			Self::ensure_can_stake(staker, *target, *amount, StakingType::ProviderBoost)?;
		staking_details.staking_type = StakingType::ProviderBoost;
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
		let mut target_details = StakingTargetLedger::<T>::get(staker, target).unwrap_or_default();
		target_details.deposit(amount, capacity).ok_or(ArithmeticError::Overflow)?;

		let mut capacity_details = CapacityLedger::<T>::get(target).unwrap_or_default();
		capacity_details.deposit(&amount, &capacity).ok_or(ArithmeticError::Overflow)?;

		Self::set_staking_account_and_lock(staker, staking_account)?;

		Self::set_target_details_for(staker, target, target_details);
		Self::set_capacity_for(target, capacity_details);

		Ok(capacity)
	}

	fn increase_stake_and_issue_boost_capacity(
		staker: &T::AccountId,
		staking_details: &mut StakingDetails<T>,
		target: &MessageSourceId,
		amount: &BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		staking_details.deposit(*amount).ok_or(ArithmeticError::Overflow)?;
		Self::set_staking_account_and_lock(staker, staking_details)?;

		// get the capacity generated by a Provider Boost
		let capacity = Self::capacity_generated(T::RewardsProvider::capacity_boost(*amount));

		let mut target_details = StakingTargetLedger::<T>::get(staker, target).unwrap_or_default();

		target_details.deposit(*amount, capacity).ok_or(ArithmeticError::Overflow)?;
		Self::set_target_details_for(staker, *target, target_details);

		let mut capacity_details = CapacityLedger::<T>::get(target).unwrap_or_default();
		capacity_details.deposit(amount, &capacity).ok_or(ArithmeticError::Overflow)?;
		Self::set_capacity_for(*target, capacity_details);

		let era = CurrentEraInfo::<T>::get().era_index;
		Self::upsert_boost_history(staker, era, *amount, true)?;

		let reward_pool_total = CurrentEraProviderBoostTotal::<T>::get();
		CurrentEraProviderBoostTotal::<T>::set(reward_pool_total.saturating_add(*amount));

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

	/// Decrease a staking account's active token and reap if it goes below the minimum.
	/// Returns: actual amount unstaked, plus the staking type + StakingDetails,
	/// since StakingDetails may be reaped and staking type must be used to calculate the
	/// capacity reduction later.
	fn decrease_active_staking_balance(
		unstaker: &T::AccountId,
		amount: BalanceOf<T>,
	) -> Result<(BalanceOf<T>, StakingType), DispatchError> {
		let mut staking_account =
			StakingAccountLedger::<T>::get(unstaker).ok_or(Error::<T>::NotAStakingAccount)?;
		ensure!(amount <= staking_account.active, Error::<T>::InsufficientStakingBalance);

		let actual_unstaked_amount = staking_account.withdraw(amount)?;
		Self::set_staking_account(unstaker, &staking_account);

		let staking_type = staking_account.staking_type;
		if staking_type == StakingType::ProviderBoost {
			let era = CurrentEraInfo::<T>::get().era_index;
			Self::upsert_boost_history(unstaker, era, actual_unstaked_amount, false)?;
			let reward_pool_total = CurrentEraProviderBoostTotal::<T>::get();
			CurrentEraProviderBoostTotal::<T>::set(
				reward_pool_total.saturating_sub(actual_unstaked_amount),
			);
		}
		Ok((actual_unstaked_amount, staking_type))
	}

	fn add_unlock_chunk(
		unstaker: &T::AccountId,
		actual_unstaked_amount: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		let current_epoch: T::EpochNumber = CurrentEpoch::<T>::get();
		let thaw_at =
			current_epoch.saturating_add(T::EpochNumber::from(T::UnstakingThawPeriod::get()));
		let mut unlocks = UnstakeUnlocks::<T>::get(unstaker).unwrap_or_default();

		match unlocks.iter_mut().find(|chunk| chunk.thaw_at == thaw_at) {
			Some(chunk) => {
				chunk.value += actual_unstaked_amount;
			},
			None => {
				let unlock_chunk: UnlockChunk<BalanceOf<T>, T::EpochNumber> =
					UnlockChunk { value: actual_unstaked_amount, thaw_at };
				unlocks
					.try_push(unlock_chunk)
					.map_err(|_| Error::<T>::MaxUnlockingChunksExceeded)?;
			},
		}

		UnstakeUnlocks::<T>::set(unstaker, Some(unlocks));
		Ok(())
	}

	// Calculates a stakable amount from a proposed amount.
	pub(crate) fn get_stakable_amount_for(
		staker: &T::AccountId,
		proposed_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		let freezable_balance = T::Currency::balance_freezable(staker);
		let current_staking_balance =
			StakingAccountLedger::<T>::get(staker).unwrap_or_default().active;
		let stakable_amount = freezable_balance
			.saturating_sub(current_staking_balance)
			.saturating_sub(T::MinimumTokenBalance::get());
		if stakable_amount >= proposed_amount {
			proposed_amount
		} else {
			Zero::zero()
		}
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

	#[allow(unused)]
	fn get_thaw_at_epoch() -> <T as Config>::EpochNumber {
		let current_epoch: T::EpochNumber = CurrentEpoch::<T>::get();
		let thaw_period = T::UnstakingThawPeriod::get();
		current_epoch.saturating_add(thaw_period.into())
	}

	/// Reduce available capacity of target and return the amount of capacity reduction.
	fn reduce_capacity(
		unstaker: &T::AccountId,
		target: MessageSourceId,
		amount: BalanceOf<T>,
		staking_type: StakingType,
	) -> Result<BalanceOf<T>, DispatchError> {
		let mut staking_target_details = StakingTargetLedger::<T>::get(unstaker, target)
			.ok_or(Error::<T>::StakerTargetRelationshipNotFound)?;

		ensure!(amount.le(&staking_target_details.amount), Error::<T>::InsufficientStakingBalance);

		let mut capacity_details =
			CapacityLedger::<T>::get(target).ok_or(Error::<T>::TargetCapacityNotFound)?;

		let capacity_to_withdraw = if staking_target_details.amount.eq(&amount) {
			staking_target_details.capacity
		} else if staking_type.eq(&StakingType::ProviderBoost) {
			Perbill::from_rational(amount, staking_target_details.amount)
				.mul_ceil(staking_target_details.capacity)
		} else {
			// this call will return an amount > than requested if the resulting StakingTargetDetails balance
			// is below the minimum. This ensures we withdraw the same amounts as for staking_target_details.
			Self::calculate_capacity_reduction(
				amount,
				capacity_details.total_tokens_staked,
				capacity_details.total_capacity_issued,
			)
		};

		let (actual_amount, actual_capacity) = staking_target_details.withdraw(
			amount,
			capacity_to_withdraw,
			T::MinimumStakingAmount::get(),
		);

		capacity_details.withdraw(actual_capacity, actual_amount);

		Self::set_capacity_for(target, capacity_details);
		Self::set_target_details_for(unstaker, target, staking_target_details);

		Ok(capacity_to_withdraw)
	}

	/// Calculates Capacity generated for given FRQCY
	fn capacity_generated(amount: BalanceOf<T>) -> BalanceOf<T> {
		let cpt = T::CapacityPerToken::get();
		cpt.mul(amount)
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
		if current_block.saturating_sub(CurrentEpochInfo::<T>::get().epoch_start) >=
			EpochLength::<T>::get()
		{
			let current_epoch = CurrentEpoch::<T>::get();
			CurrentEpoch::<T>::set(current_epoch.saturating_add(1u32.into()));
			CurrentEpochInfo::<T>::set(EpochInfo { epoch_start: current_block });
			T::WeightInfo::start_new_epoch_if_needed()
		} else {
			// 1 for get_current_epoch_info, 1 for get_epoch_length
			T::DbWeight::get().reads(2u64).saturating_add(T::DbWeight::get().writes(1))
		}
	}

	fn start_new_reward_era_if_needed(current_block: BlockNumberFor<T>) -> Weight {
		let current_era_info: RewardEraInfo<RewardEra, BlockNumberFor<T>> =
			CurrentEraInfo::<T>::get(); // 1r

		if current_block.saturating_sub(current_era_info.started_at) >= T::EraLength::get().into() {
			// 1r
			let new_era_info = RewardEraInfo {
				era_index: current_era_info.era_index.saturating_add(One::one()),
				started_at: current_block,
			};
			CurrentEraInfo::<T>::set(new_era_info); // 1w

			// carry over the current reward pool total
			let current_reward_pool_total: BalanceOf<T> = CurrentEraProviderBoostTotal::<T>::get(); // 1
			Self::update_provider_boost_reward_pool(
				current_era_info.era_index,
				current_reward_pool_total,
			);
			T::WeightInfo::start_new_reward_era_if_needed()
		} else {
			T::DbWeight::get().reads(1)
		}
	}

	/// attempts to increment number of retargets this RewardEra
	/// Returns:
	///     Error::MaxRetargetsExceeded if they try to retarget too many times in one era.
	fn update_retarget_record(staker: &T::AccountId) -> Result<(), DispatchError> {
		let current_era: RewardEra = CurrentEraInfo::<T>::get().era_index;
		let mut retargets = Retargets::<T>::get(staker).unwrap_or_default();
		ensure!(retargets.update(current_era).is_some(), Error::<T>::MaxRetargetsExceeded);
		Retargets::<T>::set(staker, Some(retargets));
		Ok(())
	}

	/// Performs the work of withdrawing the requested amount from the old staker-provider target details, and
	/// from the Provider's capacity details, and depositing it into the new staker-provider target details.
	pub(crate) fn do_retarget(
		staker: &T::AccountId,
		from_msa: &MessageSourceId,
		to_msa: &MessageSourceId,
		amount: &BalanceOf<T>,
	) -> Result<(), DispatchError> {
		let staking_type = StakingAccountLedger::<T>::get(staker).unwrap_or_default().staking_type;
		let capacity_withdrawn = Self::reduce_capacity(staker, *from_msa, *amount, staking_type)?;

		let mut to_msa_target = StakingTargetLedger::<T>::get(staker, to_msa).unwrap_or_default();

		to_msa_target
			.deposit(*amount, capacity_withdrawn)
			.ok_or(ArithmeticError::Overflow)?;

		let mut capacity_details = CapacityLedger::<T>::get(to_msa).unwrap_or_default();
		capacity_details
			.deposit(amount, &capacity_withdrawn)
			.ok_or(ArithmeticError::Overflow)?;

		Self::set_target_details_for(staker, *to_msa, to_msa_target);
		Self::set_capacity_for(*to_msa, capacity_details);
		Ok(())
	}

	/// updates or inserts a new boost history record for current_era.   Pass 'add' = true for an increase (provider_boost),
	/// pass 'false' for a decrease (unstake)
	pub(crate) fn upsert_boost_history(
		account: &T::AccountId,
		current_era: RewardEra,
		boost_amount: BalanceOf<T>,
		add: bool,
	) -> Result<(), DispatchError> {
		let mut boost_history = ProviderBoostHistories::<T>::get(account).unwrap_or_default();

		let upsert_result = if add {
			boost_history.add_era_balance(&current_era, &boost_amount)
		} else {
			boost_history.subtract_era_balance(&current_era, &boost_amount)
		};
		match upsert_result {
			Some(0usize) => ProviderBoostHistories::<T>::remove(account),
			None => return Err(DispatchError::from(Error::<T>::EraOutOfRange)),
			_ => ProviderBoostHistories::<T>::set(account, Some(boost_history)),
		}
		Ok(())
	}

	pub(crate) fn has_unclaimed_rewards(account: &T::AccountId) -> bool {
		let current_era = CurrentEraInfo::<T>::get().era_index;
		match ProviderBoostHistories::<T>::get(account) {
			Some(provider_boost_history) => {
				match provider_boost_history.count() {
					0usize => false,
					1usize => {
						// if there is just one era entry and:
						// it's for the previous era, it means we've already paid out rewards for that era, or they just staked in the last era.
						// or if it's for the current era, they only just started staking.
						provider_boost_history
							.get_entry_for_era(&current_era.saturating_sub(1u32))
							.is_none() && provider_boost_history
							.get_entry_for_era(&current_era)
							.is_none()
					},
					_ => true,
				}
			},
			None => false,
		} // 1r
	}

	/// Get all unclaimed rewards information for each eligible Reward Era.
	/// If no unclaimed rewards, returns empty list.
	pub fn list_unclaimed_rewards(
		account: &T::AccountId,
	) -> Result<
		BoundedVec<
			UnclaimedRewardInfo<BalanceOf<T>, BlockNumberFor<T>>,
			T::ProviderBoostHistoryLimit,
		>,
		DispatchError,
	> {
		if !Self::has_unclaimed_rewards(account) {
			return Ok(BoundedVec::new());
		}

		let staking_history = ProviderBoostHistories::<T>::get(account)
			.ok_or(Error::<T>::NotAProviderBoostAccount)?; // cached read

		let current_era_info = CurrentEraInfo::<T>::get(); // cached read, ditto
		let max_history: u32 = T::ProviderBoostHistoryLimit::get();

		let start_era = current_era_info.era_index.saturating_sub(max_history);
		let end_era = current_era_info.era_index.saturating_sub(One::one()); // stop at previous era

		// start with how much was staked in the era before the earliest for which there are eligible rewards.
		let mut previous_amount: BalanceOf<T> = match start_era {
			0 => 0u32.into(),
			_ => staking_history.get_amount_staked_for_era(&(start_era.saturating_sub(1u32))),
		};
		let mut unclaimed_rewards: BoundedVec<
			UnclaimedRewardInfo<BalanceOf<T>, BlockNumberFor<T>>,
			T::ProviderBoostHistoryLimit,
		> = BoundedVec::new();
		for reward_era in start_era..=end_era {
			let staked_amount = staking_history.get_amount_staked_for_era(&reward_era);
			if !staked_amount.is_zero() {
				let expires_at_era = reward_era.saturating_add(max_history);
				let expires_at_block = Self::block_at_end_of_era(expires_at_era);
				let eligible_amount = staked_amount.min(previous_amount);
				let total_for_era =
					Self::get_total_stake_for_past_era(reward_era, current_era_info.era_index)?;
				let earned_amount = <T>::RewardsProvider::era_staking_reward(
					eligible_amount,
					total_for_era,
					T::RewardPoolPerEra::get(),
				);
				unclaimed_rewards
					.try_push(UnclaimedRewardInfo {
						reward_era,
						expires_at_block,
						staked_amount,
						eligible_amount,
						earned_amount,
					})
					.map_err(|_e| Error::<T>::CollectionBoundExceeded)?;
				// ^^ there's no good reason for this ever to fail in production but it must be handled.
				previous_amount = staked_amount;
			}
		} // 1r * up to ProviderBoostHistoryLimit-1, if they staked every RewardEra.
		Ok(unclaimed_rewards)
	}

	// Returns the block number for the end of the provided era. Assumes `era` is at least this
	// era or in the future
	pub(crate) fn block_at_end_of_era(era: RewardEra) -> BlockNumberFor<T> {
		let current_era_info = CurrentEraInfo::<T>::get();
		let era_length: BlockNumberFor<T> = T::EraLength::get().into();

		let era_diff = if current_era_info.era_index.eq(&era) {
			1u32
		} else {
			era.saturating_sub(current_era_info.era_index).saturating_add(1u32)
		};
		current_era_info.started_at + era_length.mul(era_diff.into()) - 1u32.into()
	}

	// Figure out the history chunk that a given era is in and pull out the total stake for that era.
	pub(crate) fn get_total_stake_for_past_era(
		reward_era: RewardEra,
		current_era: RewardEra,
	) -> Result<BalanceOf<T>, DispatchError> {
		// Make sure that the past era is not too old
		let era_range = current_era.saturating_sub(reward_era);
		ensure!(
			current_era.gt(&reward_era) && era_range.le(&T::ProviderBoostHistoryLimit::get()),
			Error::<T>::EraOutOfRange
		);

		let chunk_idx: ChunkIndex = Self::get_chunk_index_for_era(reward_era);
		let reward_pool_chunk = ProviderBoostRewardPools::<T>::get(chunk_idx).unwrap_or_default(); // 1r
		let total_for_era =
			reward_pool_chunk.total_for_era(&reward_era).ok_or(Error::<T>::EraOutOfRange)?;
		Ok(*total_for_era)
	}

	/// Get the index of the chunk for a given era, history limit, and chunk length
	/// Example with history limit of 6 and chunk length 3:
	/// - Arrange the chunks such that we overwrite a complete chunk only when it is not needed
	/// - The cycle is thus era modulo (history limit + chunk length)
	/// - `[0,1,2],[3,4,5],[6,7,8],[]`
	///
	/// Note Chunks stored = (History Length / Chunk size) + 1
	/// - The second step is which chunk to add to:
	/// - Divide the cycle by the chunk length and take the floor
	/// - Floor(5 / 3) = 1
	///
	/// Chunk Index = Floor((era % (History Length + chunk size)) / chunk size)
	pub(crate) fn get_chunk_index_for_era(era: RewardEra) -> u32 {
		let history_limit: u32 = T::ProviderBoostHistoryLimit::get();
		let chunk_len = T::RewardPoolChunkLength::get();
		let era_u32: u32 = era;

		// Add one chunk so that we always have the full history limit in our chunks
		let cycle: u32 = era_u32 % history_limit.saturating_add(chunk_len);
		cycle.saturating_div(chunk_len)
	}

	// This is where the reward pool gets updated.
	pub(crate) fn update_provider_boost_reward_pool(era: RewardEra, boost_total: BalanceOf<T>) {
		// Current era is this era
		let chunk_idx: ChunkIndex = Self::get_chunk_index_for_era(era);
		let mut new_chunk = ProviderBoostRewardPools::<T>::get(chunk_idx).unwrap_or_default(); // 1r

		// If it is full we are resetting.
		// This assumes that the chunk length is a divisor of the history limit
		if new_chunk.is_full() {
			new_chunk = RewardPoolHistoryChunk::new();
		};

		if new_chunk.try_insert(era, boost_total).is_err() {
			// Handle the error case that should never happen
			log::warn!("could not insert a new chunk into provider boost reward pool")
		}
		ProviderBoostRewardPools::<T>::set(chunk_idx, Some(new_chunk)); // 1w
	}
	fn do_claim_rewards(staker: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
		let rewards = Self::list_unclaimed_rewards(staker)?;
		ensure!(!rewards.len().is_zero(), Error::<T>::NoRewardsEligibleToClaim);
		let zero_balance: BalanceOf<T> = 0u32.into();
		let total_to_mint: BalanceOf<T> = rewards
			.iter()
			.fold(zero_balance, |acc, reward_info| acc.saturating_add(reward_info.earned_amount));
		ensure!(total_to_mint.gt(&Zero::zero()), Error::<T>::NoRewardsEligibleToClaim);
		let _minted_unused = T::Currency::mint_into(staker, total_to_mint)?;

		let mut new_history: ProviderBoostHistory<T> = ProviderBoostHistory::new();
		let last_staked_amount =
			rewards.last().unwrap_or(&UnclaimedRewardInfo::default()).staked_amount;
		let current_era = CurrentEraInfo::<T>::get().era_index;
		// We have already paid out for the previous era. Put one entry for the previous era as if that is when they staked,
		// so they will be credited for current_era.
		ensure!(
			new_history
				.add_era_balance(&current_era.saturating_sub(1u32), &last_staked_amount)
				.is_some(),
			Error::<T>::CollectionBoundExceeded
		);
		ProviderBoostHistories::<T>::set(staker, Some(new_history));

		Ok(total_to_mint)
	}
}

/// Nontransferable functions are intended for capacity spend and recharge.
/// Implementations of Nontransferable MUST NOT be concerned with StakingType.
impl<T: Config> Nontransferable for Pallet<T> {
	type Balance = BalanceOf<T>;

	/// Return the remaining capacity for the Provider MSA Id
	fn balance(msa_id: MessageSourceId) -> Self::Balance {
		match CapacityLedger::<T>::get(msa_id) {
			Some(capacity_details) => capacity_details.remaining_capacity,
			None => BalanceOf::<T>::zero(),
		}
	}

	fn replenishable_balance(msa_id: MessageSourceId) -> Self::Balance {
		match CapacityLedger::<T>::get(msa_id) {
			Some(capacity_details) => capacity_details.total_capacity_issued,
			None => BalanceOf::<T>::zero(),
		}
	}

	/// Spend capacity: reduce remaining capacity by the given amount
	fn deduct(msa_id: MessageSourceId, amount: Self::Balance) -> Result<(), DispatchError> {
		let mut capacity_details =
			CapacityLedger::<T>::get(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;

		capacity_details
			.deduct_capacity_by_amount(amount)
			.map_err(|_| Error::<T>::InsufficientCapacityBalance)?;

		Self::set_capacity_for(msa_id, capacity_details);

		Self::deposit_event(Event::CapacityWithdrawn { msa_id, amount });
		Ok(())
	}

	/// Increase all totals for the MSA's CapacityDetails.
	fn deposit(
		msa_id: MessageSourceId,
		token_amount: Self::Balance,
		capacity_amount: Self::Balance,
	) -> Result<(), DispatchError> {
		let mut capacity_details =
			CapacityLedger::<T>::get(msa_id).ok_or(Error::<T>::TargetCapacityNotFound)?;
		capacity_details.deposit(&token_amount, &capacity_amount);
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
	/// Currently unused.
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

impl<T: Config> ProviderBoostRewardsProvider<T> for Pallet<T> {
	type Balance = BalanceOf<T>;

	fn reward_pool_size(_total_staked: Self::Balance) -> Self::Balance {
		T::RewardPoolPerEra::get()
	}

	/// Calculate the reward for a single era.  We don't care about the era number,
	/// just the values.
	fn era_staking_reward(
		era_amount_staked: Self::Balance,
		era_total_staked: Self::Balance,
		era_reward_pool_size: Self::Balance,
	) -> Self::Balance {
		let capped_reward = T::RewardPercentCap::get().mul(era_amount_staked);
		let proportional_reward = era_reward_pool_size
			.saturating_mul(era_amount_staked)
			.checked_div(&era_total_staked)
			.unwrap_or_else(Zero::zero);
		proportional_reward.min(capped_reward)
	}

	/// How much, as a percentage of staked token, to boost a targeted Provider when staking.
	fn capacity_boost(amount: Self::Balance) -> Self::Balance {
		Perbill::from_percent(STAKED_PERCENTAGE_TO_BOOST).mul(amount)
	}
}
