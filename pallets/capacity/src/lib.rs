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
//! The staked amount may be increased, targeting either the same or a different target to receive the newly generated Capacity.
//! As a result, every time the network is staked to, the staked tokens are locked until unstaked.
//!
//! Unstaking schedules some amount of token to be unlocked. There is no limit on the amount of token that can be unstaked.
//! However, there is a a limit on how many concurrently scheduled unstaking requests can exist.
//! After scheduling tokens to be unlocked, they must undergo a thaw period before being withdrawn.
//!
//! After thawing, the tokens may be withdrawn using the withdraw_unstaked extrinsic.
//! On success, the tokens are unlocked and free up space to submit more unstaking request.
//!
//! The MSA pallet provides functions for:
//!
//! - staking and, updating,
//!
//! ## Terminology
//! * **Staker:** [insert description].
//! * **Target** [insert description].
//! * **Epoch Period:** [insert description here].
//! * **Capacity:** [insert description here].
//! * **Replenishable:** [insert description here].
//!

#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

use frame_support::{
	dispatch::DispatchResult,
	traits::{Currency, LockableCurrency},
};

pub use common_primitives::{msa::MessageSourceId, utils::wrap_binary_data};
pub use pallet::*;
pub use types::*;
pub use weights::*;

pub mod types;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, Twox64Concat};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Function that allows a balance to be locked.
		type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

		/// The minimum required token amount to stake. It facilitates cleaning dust when unstaking.
		#[pallet::constant]
		type MinimumStakingAmount: Get<BalanceOf<Self>>;

		/// The maximum number of unlocking chunks a StakingAccountLedger can have. It determines how many concurrent unstaked chunks may exist.
		#[pallet::constant]
		type MaxUnlockingChunks: Get<u32>;
	}

	/// Storage for keeping a ledger of staked token amounts for accounts.
	#[pallet::storage]
	#[pallet::getter(fn get_staking_ledger)]
	pub type StakingAccountLedger<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, StakingAccountDetails<T>>;

	/// Storage to record how many tokens were targeted to an MSA.
	#[pallet::storage]
	#[pallet::getter(fn get_target_ledger)]
	pub type StakingTargetLedger<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		MessageSourceId,
		StakingTargetDetails<BalanceOf<T>>,
	>;

	/// Storage for target Capacity usage.
	#[pallet::storage]
	pub type CapacityOf<T: Config> =
		StorageMap<_, Twox64Concat, MessageSourceId, CapacityDetails<BalanceOf<T>, T::BlockNumber>>;

	// Simple declaration of the `Pallet` type. It is placeholder we use to implement traits and
	// method.
	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Stakes some amount of tokens to the network and generates Capacity.
		///
		/// ### Errors
		///
		/// - Returns Error::InsufficientBalance if the sender does not have free balance amount needed to stake.
		/// - Returns Error::InvalidTarget if attempting to stake to an invalid target.
		/// - Returns Error::InsufficientStakingAmount if attempting to stake an amount below the minimum amount.
		#[pallet::weight(T::WeightInfo::stake())]
		pub fn stake(
			_origin: OriginFor<T>,
			_target: MessageSourceId,
			_amount: BalanceOf<T>,
		) -> DispatchResult {
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {}
