//! # Time-Release Module
//!
//! This pallet is a fork of the [ORML-vesting]( [vesting](https://github.com/open-web3-stack/open-runtime-module-library/tree/master/vesting)) polkadot-v0.9.39.
//!
//! ## Overview
//!
//! Time-release module provides a means of scheduled balance lock on an account. It
//! uses the *graded release* way, which unlocks a specific amount of balance
//! every period of time, until all balance unlocked.
//!
//! ### Release Schedule
//!
//! The schedule of a release is described by data structure `ReleaseSchedule`:
//! from the block number of `start`, for every `period` amount of blocks,
//! `per_period` amount of balance would unlocked, until number of periods
//! `period_count` reached. Note in release schedules, *time* is measured by
//! block number. All `ReleaseSchedule`s under an account could be queried in
//! chain state.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `transfer` - Add a new release schedule for an account.
//! - `claim` - Claim unlocked balances.
//! - `update_release_schedules` - Update all release schedules under an
//!   account, `root` origin required.
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
	dispatch::DispatchResult,
	ensure,
	pallet_prelude::*,
	traits::{
		tokens::{
			fungible::{Inspect as InspectFungible, Mutate, MutateFreeze},
			Balance, Preservation,
		},
		BuildGenesisConfig, EnsureOrigin, Get,
	},
	BoundedVec,
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use sp_runtime::{
	traits::{BlockNumberProvider, CheckedAdd, StaticLookup, Zero},
	ArithmeticError,
};
use sp_std::vec::Vec;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// storage migrations
pub mod migration;

mod types;
pub use types::*;

pub mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	pub(crate) type BalanceOf<T> = <<T as Config>::Currency as InspectFungible<
		<T as frame_system::Config>::AccountId,
	>>::Balance;
	pub(crate) type ReleaseScheduleOf<T> = ReleaseSchedule<BlockNumberFor<T>, BalanceOf<T>>;

	/// Scheduled item used for configuring genesis.
	pub type ScheduledItem<T> = (
		<T as frame_system::Config>::AccountId,
		BlockNumberFor<T>,
		BlockNumberFor<T>,
		u32,
		BalanceOf<T>,
	);

	/// A reason for freezing funds.
	/// Creates a freeze reason for this pallet that is aggregated by `construct_runtime`.
	#[pallet::composite_enum]
	pub enum FreezeReason {
		/// Funds are currently locked and are not yet liquid.
		TimeReleaseVesting,
	}

	/// the storage version for this pallet
	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The overarching freeze reason.
		type RuntimeFreezeReason: From<FreezeReason>;

		/// We need MaybeSerializeDeserialize because of the genesis config.
		type Balance: Balance + MaybeSerializeDeserialize;

		/// The currency trait used to set a lock on a balance.
		type Currency: MutateFreeze<Self::AccountId, Id = Self::RuntimeFreezeReason>
			+ InspectFungible<Self::AccountId, Balance = Self::Balance>
			+ Mutate<Self::AccountId>;

		#[pallet::constant]
		/// The minimum amount transferred to call `transfer`.
		type MinReleaseTransfer: Get<BalanceOf<Self>>;

		/// Required origin for time-release transfer.
		type TransferOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;

		/// The maximum release schedules.
		type MaxReleaseSchedules: Get<u32>;

		/// The block-number provider.
		type BlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Release period is zero
		ZeroReleasePeriod,
		/// Period-count is zero
		ZeroReleasePeriodCount,
		/// Insufficient amount of balance to lock
		InsufficientBalanceToLock,
		/// This account have too many release schedules
		TooManyReleaseSchedules,
		/// The transfer amount is too low
		AmountLow,
		/// Failed because the maximum release schedules was exceeded
		MaxReleaseSchedulesExceeded,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Added new release schedule.
		ReleaseScheduleAdded {
			/// The account-id of the sender.
			from: T::AccountId,
			/// The account-id of the receiver.
			to: T::AccountId,
			/// The schedule indicating when transfer releases happen.
			release_schedule: ReleaseScheduleOf<T>,
		},
		/// Claimed release.
		Claimed {
			/// The account-id of the claimed amount.
			who: T::AccountId,
			/// The amount claimed.
			amount: BalanceOf<T>,
		},
		/// Updated release schedules.
		ReleaseSchedulesUpdated {
			/// The account-id for which a release schedule updated was made for.
			who: T::AccountId,
		},
	}

	/// Release schedules of an account.
	///
	/// ReleaseSchedules: `map AccountId => Vec<ReleaseSchedule>`
	#[pallet::storage]
	#[pallet::getter(fn release_schedules)]
	pub type ReleaseSchedules<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<ReleaseScheduleOf<T>, T::MaxReleaseSchedules>,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		/// Phantom data.
		#[serde(skip)]
		pub _config: sp_std::marker::PhantomData<T>,
		/// A list of schedules to include.
		pub schedules: Vec<ScheduledItem<T>>,
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			self.schedules
				.iter()
				.for_each(|(who, start, period, period_count, per_period)| {
					let mut bounded_schedules = ReleaseSchedules::<T>::get(who);
					bounded_schedules
						.try_push(ReleaseSchedule {
							start: *start,
							period: *period,
							period_count: *period_count,
							per_period: *per_period,
						})
						.expect("Max release schedules exceeded");
					let total_amount = bounded_schedules
						.iter()
						.try_fold::<_, _, Result<BalanceOf<T>, DispatchError>>(
							Zero::zero(),
							|acc_amount, schedule| {
								let amount = ensure_valid_release_schedule::<T>(schedule)?;
								acc_amount
									.checked_add(&amount)
									.ok_or_else(|| ArithmeticError::Overflow.into())
							},
						)
						.expect("Invalid release schedule");

					assert!(
						T::Currency::balance(who) >= total_amount,
						"Account do not have enough balance"
					);

					T::Currency::set_freeze(
						&FreezeReason::TimeReleaseVesting.into(),
						who,
						total_amount,
					)
					.expect("Failed to set freeze");
					ReleaseSchedules::<T>::insert(who, bounded_schedules);
				});
		}
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Claim unlocked balances.
		///
		/// # Events
		/// * [`Event::Claimed`]
		///
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::claim(<T as Config>::MaxReleaseSchedules::get() / 2))]
		pub fn claim(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let locked_amount = Self::do_claim(&who)?;

			Self::deposit_event(Event::Claimed { who, amount: locked_amount });
			Ok(())
		}

		/// Add a new release schedule for an account.
		///
		/// # Events
		/// * [`Event::ReleaseScheduleAdded]
		///
		/// # Errors
		///
		/// * [`Error::InsufficientBalanceToLock] - Insuffience amount of balance to lock.
		/// * [`Error::MaxReleaseSchedulesExceeded] - Failed because the maximum release schedules was exceeded-
		/// * [`ArithmeticError::Overflow] - Failed because of an overflow.
		///
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			schedule: ReleaseScheduleOf<T>,
		) -> DispatchResult {
			let from = T::TransferOrigin::ensure_origin(origin)?;
			let to = T::Lookup::lookup(dest)?;

			if to == from {
				ensure!(
					T::Currency::balance(&from) >=
						schedule.total_amount().ok_or(ArithmeticError::Overflow)?,
					Error::<T>::InsufficientBalanceToLock,
				);
			}

			Self::do_transfer(&from, &to, schedule.clone())?;

			Self::deposit_event(Event::ReleaseScheduleAdded {
				from,
				to,
				release_schedule: schedule,
			});
			Ok(())
		}

		/// Update all release schedules under an account, `root` origin required.
		///
		/// # Events
		/// * [`Event::ReleaseSchedulesUpdated] - Updated release schedules.
		///
		/// # Errors
		///
		/// * [`Error::InsufficientBalanceToLock] - Insuffience amount of balance to lock.
		/// * [`Error::MaxReleaseSchedulesExceeded] - Failed because the maximum release schedules was exceeded-
		/// * [`ArithmeticError::Overflow] - Failed because of an overflow.
		///
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::update_release_schedules(release_schedules.len() as u32))]
		pub fn update_release_schedules(
			origin: OriginFor<T>,
			who: <T::Lookup as StaticLookup>::Source,
			release_schedules: Vec<ReleaseScheduleOf<T>>,
		) -> DispatchResult {
			ensure_root(origin)?;

			let account = T::Lookup::lookup(who)?;
			Self::do_update_release_schedules(&account, release_schedules)?;

			Self::deposit_event(Event::ReleaseSchedulesUpdated { who: account });
			Ok(())
		}

		/// Claim unlocked balances on behalf for an account.
		///
		/// # Events
		/// * [`Event::Claimed`]
		///
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::claim(<T as Config>::MaxReleaseSchedules::get() / 2))]
		pub fn claim_for(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			ensure_signed(origin)?;
			let who = T::Lookup::lookup(dest)?;
			let locked_amount = Self::do_claim(&who)?;

			Self::deposit_event(Event::Claimed { who, amount: locked_amount });
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn do_claim(who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
		let locked = Self::prune_and_get_locked_balance(who);
		if locked.is_zero() {
			Self::delete_lock(who)?;
		} else {
			Self::update_lock(who, locked)?;
		}

		Ok(locked)
	}

	/// Deletes schedules that have released all funds up to a block-number.
	fn prune_schedules_for(
		who: &T::AccountId,
		block_number: BlockNumberFor<T>,
	) -> BoundedVec<ReleaseScheduleOf<T>, T::MaxReleaseSchedules> {
		let mut schedules = Self::release_schedules(who);
		schedules.retain(|schedule| !schedule.locked_amount(block_number).is_zero());

		if schedules.is_empty() {
			ReleaseSchedules::<T>::remove(who);
		} else {
			Self::set_schedules_for(who, schedules.clone());
		}

		schedules
	}

	/// Returns locked balance based on current block number.
	fn prune_and_get_locked_balance(who: &T::AccountId) -> BalanceOf<T> {
		let now = T::BlockNumberProvider::current_block_number();

		let schedules = Self::prune_schedules_for(&who, now);

		let total = schedules
			.iter()
			.fold(BalanceOf::<T>::zero(), |acc, schedule| acc + schedule.locked_amount(now));

		total
	}

	fn do_transfer(
		from: &T::AccountId,
		to: &T::AccountId,
		schedule: ReleaseScheduleOf<T>,
	) -> DispatchResult {
		let schedule_amount = ensure_valid_release_schedule::<T>(&schedule)?;

		let total_amount = Self::prune_and_get_locked_balance(to)
			.checked_add(&schedule_amount)
			.ok_or(ArithmeticError::Overflow)?;

		T::Currency::transfer(from, to, schedule_amount, Preservation::Expendable)?;

		Self::update_lock(&to, total_amount)?;

		<ReleaseSchedules<T>>::try_append(to, schedule)
			.map_err(|_| Error::<T>::MaxReleaseSchedulesExceeded)?;

		Ok(())
	}

	fn do_update_release_schedules(
		who: &T::AccountId,
		schedules: Vec<ReleaseScheduleOf<T>>,
	) -> DispatchResult {
		let bounded_schedules =
			BoundedVec::<ReleaseScheduleOf<T>, T::MaxReleaseSchedules>::try_from(schedules)
				.map_err(|_| Error::<T>::MaxReleaseSchedulesExceeded)?;

		// empty release schedules cleanup the storage and unlock the fund
		if bounded_schedules.is_empty() {
			Self::delete_release_schedules(who)?;
			return Ok(())
		}

		let total_amount =
			bounded_schedules.iter().try_fold::<_, _, Result<BalanceOf<T>, DispatchError>>(
				Zero::zero(),
				|acc_amount, schedule| {
					let amount = ensure_valid_release_schedule::<T>(schedule)?;
					acc_amount.checked_add(&amount).ok_or_else(|| ArithmeticError::Overflow.into())
				},
			)?;

		ensure!(T::Currency::balance(who) >= total_amount, Error::<T>::InsufficientBalanceToLock,);

		Self::update_lock(&who, total_amount)?;
		Self::set_schedules_for(who, bounded_schedules);

		Ok(())
	}

	fn update_lock(who: &T::AccountId, locked: BalanceOf<T>) -> DispatchResult {
		T::Currency::set_freeze(&FreezeReason::TimeReleaseVesting.into(), who, locked)?;
		Ok(())
	}

	fn delete_lock(who: &T::AccountId) -> DispatchResult {
		T::Currency::thaw(&FreezeReason::TimeReleaseVesting.into(), who)?;
		Ok(())
	}

	fn set_schedules_for(
		who: &T::AccountId,
		schedules: BoundedVec<ReleaseScheduleOf<T>, T::MaxReleaseSchedules>,
	) {
		ReleaseSchedules::<T>::insert(who, schedules);
	}

	fn delete_release_schedules(who: &T::AccountId) -> DispatchResult {
		<ReleaseSchedules<T>>::remove(who);
		Self::delete_lock(who)?;
		Ok(())
	}
}

/// Returns `Ok(total_total)` if valid schedule, or error.
fn ensure_valid_release_schedule<T: Config>(
	schedule: &ReleaseScheduleOf<T>,
) -> Result<BalanceOf<T>, DispatchError> {
	ensure!(!schedule.period.is_zero(), Error::<T>::ZeroReleasePeriod);
	ensure!(!schedule.period_count.is_zero(), Error::<T>::ZeroReleasePeriodCount);
	ensure!(schedule.end().is_some(), ArithmeticError::Overflow);

	let total_total = schedule.total_amount().ok_or(ArithmeticError::Overflow)?;

	ensure!(total_total >= T::MinReleaseTransfer::get(), Error::<T>::AmountLow);

	Ok(total_total)
}
