//! Transfer Funds as Frozen with a Time Delayed Release
//!
//! ## Quick Links
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
#![doc = include_str!("../README.md")]
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
			fungible::{Inspect as InspectFungible, InspectHold, Mutate, MutateFreeze, MutateHold},
			Balance,
			Fortitude::Polite,
			Precision::Exact,
			Preservation,
			Restriction::Free,
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
extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod types;
pub use types::*;

pub mod weights;
pub use weights::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use frame_support::{dispatch::PostDispatchInfo, traits::fungible::InspectHold};
	use sp_runtime::traits::Dispatchable;

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
		/// Funds are currently frozen and are not yet liquid.
		TimeReleaseVesting,
	}

	/// A reason for holding funds
	/// Create a hold reason for this pallet that is aggregated by 'contract_runtime'.
	#[pallet::composite_enum]
	pub enum HoldReason {
		/// Funds reserved/held for scheduled transfers
		TimeReleaseScheduledVesting,
	}

	/// the storage version for this pallet
	pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

	/// Custom origin to be used when scheduling a transfer
	#[derive(
		PartialEq,
		Eq,
		Clone,
		MaxEncodedLen,
		Encode,
		Decode,
		DecodeWithMemTracking,
		TypeInfo,
		RuntimeDebug,
	)]
	#[pallet::origin]
	pub enum Origin<T: Config> {
		/// A scheduled release triggered by the TimeRelease pallet.
		TimeRelease(T::AccountId),
	}

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The overarching freeze reason.
		type RuntimeFreezeReason: From<FreezeReason>;

		/// Overarching hold reason.
		type RuntimeHoldReason: From<HoldReason>;

		/// runtime origin
		type RuntimeOrigin: From<Origin<Self>> + From<<Self as frame_system::Config>::RuntimeOrigin>;

		/// We need MaybeSerializeDeserialize because of the genesis config.
		type Balance: Balance + MaybeSerializeDeserialize;

		/// The currency trait used to set a freeze on a balance.
		type Currency: MutateFreeze<Self::AccountId, Id = Self::RuntimeFreezeReason>
			+ InspectFungible<Self::AccountId, Balance = Self::Balance>
			+ Mutate<Self::AccountId>
			+ MutateHold<Self::AccountId, Reason = Self::RuntimeHoldReason>
			+ InspectHold<Self::AccountId>;

		#[pallet::constant]
		/// The minimum amount transferred to call `transfer`.
		type MinReleaseTransfer: Get<BalanceOf<Self>>;

		/// Required origin for time-release transfer.
		type TransferOrigin: EnsureOrigin<
			<Self as frame_system::Config>::RuntimeOrigin,
			Success = Self::AccountId,
		>;

		/// Timerelase origin
		type TimeReleaseOrigin: EnsureOrigin<
			<Self as frame_system::Config>::RuntimeOrigin,
			Success = Self::AccountId,
		>;

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;

		/// The maximum release schedules.
		type MaxReleaseSchedules: Get<u32>;

		/// The block-number provider.
		type BlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>;

		/// The aggregate call type
		type RuntimeCall: Parameter
			+ Dispatchable<
				RuntimeOrigin = <Self as frame_system::Config>::RuntimeOrigin,
				PostInfo = PostDispatchInfo,
			> + From<Call<Self>>
			+ IsType<<Self as frame_system::Config>::RuntimeCall>;

		/// The scheduler provider
		type SchedulerProvider: SchedulerProviderTrait<
			<Self as Config>::RuntimeOrigin,
			BlockNumberFor<Self>,
			<Self as Config>::RuntimeCall,
		>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Release period is zero
		ZeroReleasePeriod,
		/// Period-count is zero
		ZeroReleasePeriodCount,
		/// Insufficient amount of balance to freeze
		InsufficientBalanceToFreeze,
		/// This account have too many release schedules
		TooManyReleaseSchedules,
		/// The transfer amount is too low
		AmountLow,
		/// Failed because the maximum release schedules was exceeded
		MaxReleaseSchedulesExceeded,
		/// A scheduled transfer with the same identifier already exists.
		DuplicateScheduleName,
		/// No scheduled transfer found for the given identifier.
		NotFound,
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
	pub type ReleaseSchedules<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<ReleaseScheduleOf<T>, T::MaxReleaseSchedules>,
		ValueQuery,
	>;

	/// Tracks the amount of funds reserved for each scheduled transfer, keyed by the transfer's identifier.
	#[pallet::storage]
	pub type ScheduleReservedAmounts<T: Config> =
		StorageMap<_, Twox64Concat, ScheduleName, BalanceOf<T>>;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		/// Phantom data.
		#[serde(skip)]
		pub _config: core::marker::PhantomData<T>,
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
						"Account does not have enough balance."
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
		/// Claim thawed balances.
		///
		/// # Events
		/// * [`Event::Claimed`]
		///
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::claim(<T as Config>::MaxReleaseSchedules::get() / 2))]
		pub fn claim(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let frozen_amount = Self::claim_frozen_balance(&who)?;

			Self::deposit_event(Event::Claimed { who, amount: frozen_amount });
			Ok(())
		}

		/// Add a new release schedule for an account.
		///
		/// # Events
		/// * [`Event::ReleaseScheduleAdded]
		///
		/// # Errors
		///
		/// * [`Error::InsufficientBalanceToFreeze] - Insufficient amount of balance to freeze.
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

			let total_amount = schedule.total_amount().ok_or(ArithmeticError::Overflow)?;
			Self::ensure_sufficient_free_balance(&from, &to, total_amount)?;

			Self::finalize_vesting_transfer(
				&from,
				&to,
				schedule.clone(),
				Self::transfer_from_free_balance,
			)?;

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
		/// * [`Error::InsufficientBalanceToFreeze] - Insufficient amount of balance to freeze.
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

		/// Claim thawed balances on behalf for an account.
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
			let frozen_amount = Self::claim_frozen_balance(&who)?;

			Self::deposit_event(Event::Claimed { who, amount: frozen_amount });
			Ok(())
		}

		/// This function schedules a transfer by calling the `Scheduler` pallet's `schedule` function.
		/// The transfer will be executed at the specified block number.
		///
		/// * [`Error::InsufficientBalanceToFreeze`] - Insufficient amount of balance to freeze.
		/// * [`Error::MaxReleaseSchedulesExceeded`] - Failed because the maximum release schedules was exceeded-
		/// * [`ArithmeticError::Overflow] - Failed because of an overflow.
		///
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::schedule_named_transfer().saturating_add(T::WeightInfo::execute_scheduled_named_transfer()))]
		pub fn schedule_named_transfer(
			origin: OriginFor<T>,
			id: ScheduleName,
			dest: <T::Lookup as StaticLookup>::Source,
			schedule: ReleaseScheduleOf<T>,
			when: BlockNumberFor<T>,
		) -> DispatchResult {
			let from = T::TransferOrigin::ensure_origin(origin)?;
			let to = T::Lookup::lookup(dest.clone())?;

			ensure!(
				!ScheduleReservedAmounts::<T>::contains_key(id),
				Error::<T>::DuplicateScheduleName
			);

			let total_amount = Self::validate_and_get_schedule_amount(&schedule)?;
			Self::ensure_sufficient_free_balance(&from, &to, total_amount)?;

			Self::hold_funds_for_scheduled_vesting(
				&from,
				schedule.total_amount().ok_or(ArithmeticError::Overflow)?,
			)?;

			Self::insert_reserved_amount(
				id,
				schedule.total_amount().ok_or(ArithmeticError::Overflow)?,
			)?;

			Self::schedule_transfer_for(id, from.clone(), dest, schedule, when)?;

			Ok(())
		}

		/// Cancels a scheduled transfer by its unique identifier.
		/// Releases any funds held for the specified transfer.
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::cancel_scheduled_named_transfer(<T as Config>::MaxReleaseSchedules::get() / 2))]
		pub fn cancel_scheduled_named_transfer(
			origin: OriginFor<T>,
			id: ScheduleName,
		) -> DispatchResult {
			let who = T::TransferOrigin::ensure_origin(origin)?;

			T::SchedulerProvider::cancel(Origin::<T>::TimeRelease(who.clone()).into(), id)?;

			Self::release_reserved_funds_by_id(id, &who)?;

			Ok(())
		}

		/// Execute a scheduled transfer
		///
		/// This function is called to execute a transfer that was previously scheduled.
		/// It ensures that the origin is valid, the destination account exists, and the
		/// release schedule is valid. It then finalizes the vesting transfer from the
		/// hold balance and emits an event indicating that the release schedule was added.
		///
		/// # Arguments
		///
		/// * `origin` - The origin of the call, which must be the TimeRelease origin.
		/// * `dest` - The destination account to which the funds will be transferred.
		/// * `schedule` - The release schedule that specifies the amount and timing of the transfer.
		///
		/// # Errors
		///
		/// * [`ArithmeticError::Overflow`] - If the total amount in the schedule overflows.
		/// * [`Error::InsufficientBalanceToFreeze`] - If the hold balance is insufficient.
		/// * [`Error::ZeroReleasePeriod`] - If the release period is zero.
		/// * [`Error::ZeroReleasePeriodCount`] - If the release period count is zero.
		/// * [`Error::AmountLow`] - If the total amount is below the minimum release transfer amount.
		///
		/// # Events
		///
		/// * [`Event::ReleaseScheduleAdded`] - Indicates that a new release schedule was added.
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::execute_scheduled_named_transfer())]
		pub fn execute_scheduled_named_transfer(
			origin: OriginFor<T>,
			id: ScheduleName,
			dest: <T::Lookup as StaticLookup>::Source,
			schedule: ReleaseScheduleOf<T>,
		) -> DispatchResult {
			let from = T::TimeReleaseOrigin::ensure_origin(origin)?;
			let to = T::Lookup::lookup(dest)?;

			let total_amount = Self::validate_and_get_schedule_amount(&schedule)?;
			Self::ensure_sufficient_hold_balance(&from, total_amount)?;

			Self::finalize_vesting_transfer(
				&from,
				&to,
				schedule.clone(),
				Self::transfer_from_hold_balance,
			)?;

			ScheduleReservedAmounts::<T>::remove(id);

			Self::deposit_event(Event::ReleaseScheduleAdded {
				from,
				to,
				release_schedule: schedule,
			});

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn claim_frozen_balance(who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
		let frozen = Self::prune_and_get_frozen_balance(who);
		if frozen.is_zero() {
			Self::delete_freeze(who)?;
		} else {
			Self::update_freeze(who, frozen)?;
		}

		Ok(frozen)
	}

	/// Deletes schedules that have released all funds up to a block-number.
	fn prune_schedules_for(
		who: &T::AccountId,
		block_number: BlockNumberFor<T>,
	) -> BoundedVec<ReleaseScheduleOf<T>, T::MaxReleaseSchedules> {
		let mut schedules = ReleaseSchedules::<T>::get(who);
		schedules.retain(|schedule| !schedule.frozen_amount(block_number).is_zero());

		if schedules.is_empty() {
			ReleaseSchedules::<T>::remove(who);
		} else {
			Self::set_schedules_for(who, schedules.clone());
		}

		schedules
	}

	/// Returns frozen balance based on current block number.
	fn prune_and_get_frozen_balance(who: &T::AccountId) -> BalanceOf<T> {
		let now = T::BlockNumberProvider::current_block_number();

		let schedules = Self::prune_schedules_for(who, now);

		let total = schedules
			.iter()
			.fold(BalanceOf::<T>::zero(), |acc, schedule| acc + schedule.frozen_amount(now));

		total
	}

	fn schedule_transfer_for(
		id: ScheduleName,
		from: T::AccountId,
		dest: <T::Lookup as StaticLookup>::Source,
		schedule: ReleaseScheduleOf<T>,
		when: BlockNumberFor<T>,
	) -> DispatchResult {
		let schedule_call =
			<T as self::Config>::RuntimeCall::from(Call::<T>::execute_scheduled_named_transfer {
				id,
				dest,
				schedule,
			});

		T::SchedulerProvider::schedule(
			Origin::<T>::TimeRelease(from).into(),
			id,
			when,
			Box::new(schedule_call),
		)?;

		Ok(())
	}

	fn do_update_release_schedules(
		who: &T::AccountId,
		schedules: Vec<ReleaseScheduleOf<T>>,
	) -> DispatchResult {
		let bounded_schedules =
			BoundedVec::<ReleaseScheduleOf<T>, T::MaxReleaseSchedules>::try_from(schedules)
				.map_err(|_| Error::<T>::MaxReleaseSchedulesExceeded)?;

		// empty release schedules cleanup the storage and thaw the funds
		if bounded_schedules.is_empty() {
			Self::delete_release_schedules(who)?;
			return Ok(());
		}

		let total_amount =
			bounded_schedules.iter().try_fold::<_, _, Result<BalanceOf<T>, DispatchError>>(
				Zero::zero(),
				|acc_amount, schedule| {
					let amount = ensure_valid_release_schedule::<T>(schedule)?;
					acc_amount.checked_add(&amount).ok_or_else(|| ArithmeticError::Overflow.into())
				},
			)?;

		ensure!(T::Currency::balance(who) >= total_amount, Error::<T>::InsufficientBalanceToFreeze,);

		Self::update_freeze(who, total_amount)?;
		Self::set_schedules_for(who, bounded_schedules);

		Ok(())
	}

	fn update_freeze(who: &T::AccountId, frozen: BalanceOf<T>) -> DispatchResult {
		T::Currency::set_freeze(&FreezeReason::TimeReleaseVesting.into(), who, frozen)?;
		Ok(())
	}

	fn hold_funds_for_scheduled_vesting(
		who: &T::AccountId,
		hold_amount: BalanceOf<T>,
	) -> DispatchResult {
		T::Currency::hold(&HoldReason::TimeReleaseScheduledVesting.into(), who, hold_amount)?;
		Ok(())
	}

	fn release_reserved_funds_by_id(id: ScheduleName, who: &T::AccountId) -> DispatchResult {
		let amount = ScheduleReservedAmounts::<T>::take(id).ok_or(Error::<T>::NotFound)?;

		T::Currency::release(&HoldReason::TimeReleaseScheduledVesting.into(), who, amount, Exact)?;

		Ok(())
	}

	fn insert_reserved_amount(id: ScheduleName, amount: BalanceOf<T>) -> DispatchResult {
		ScheduleReservedAmounts::<T>::insert(id, amount);

		Ok(())
	}

	fn delete_freeze(who: &T::AccountId) -> DispatchResult {
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
		Self::delete_freeze(who)?;
		Ok(())
	}

	fn hold_balance_for(who: &T::AccountId) -> BalanceOf<T> {
		T::Currency::balance_on_hold(&HoldReason::TimeReleaseScheduledVesting.into(), who)
	}

	fn ensure_sufficient_hold_balance(from: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
		ensure!(Self::hold_balance_for(from) >= amount, Error::<T>::InsufficientBalanceToFreeze);

		Ok(())
	}

	fn validate_and_get_schedule_amount(
		schedule: &ReleaseScheduleOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		ensure_valid_release_schedule::<T>(schedule)
	}

	fn append_release_schedule(
		recipient: &T::AccountId,
		schedule: ReleaseScheduleOf<T>,
	) -> DispatchResult {
		<ReleaseSchedules<T>>::try_append(recipient, schedule)
			.map_err(|_| Error::<T>::MaxReleaseSchedulesExceeded)?;
		Ok(())
	}

	fn transfer_from_free_balance(
		from: &T::AccountId,
		to: &T::AccountId,
		schedule_amount: BalanceOf<T>,
	) -> DispatchResult {
		T::Currency::transfer(from, to, schedule_amount, Preservation::Expendable)?;

		Ok(())
	}

	/// Transfers the specified amount from the hold balance of the `from` account to the `to` account.
	///
	/// # Arguments
	///
	/// * `from` - The account from which the funds will be transferred.
	/// * `to` - The account to which the funds will be transferred.
	/// * `schedule_amount` - The amount to be transferred.
	///
	/// # Errors
	///
	/// * `ArithmeticError::Overflow` - If the transfer amount overflows.
	fn transfer_from_hold_balance(
		from: &T::AccountId,
		to: &T::AccountId,
		schedule_amount: BalanceOf<T>,
	) -> DispatchResult {
		// Transfer held funds into a destination account.
		//
		// If `mode` is `OnHold`, then the destination account must already exist and the assets
		// transferred will still be on hold in the destination account. If not, then the destination
		// account need not already exist, but must be creatable.
		//
		// If `precision` is `BestEffort`, then an amount less than `amount` may be transferred without
		// error.
		//
		// If `force` is `Force`, then other fund-locking mechanisms may be disregarded. It should be
		// left as `Polite` in most circumstances, but when you want the same power as a `slash`, it
		// may be `Force`.
		//
		// The actual amount transferred is returned, or `Err` in the case of error and nothing is
		// changed.
		T::Currency::transfer_on_hold(
			&HoldReason::TimeReleaseScheduledVesting.into(),
			from,
			to,
			schedule_amount,
			Exact,
			Free,
			Polite,
		)?;

		Ok(())
	}

	fn ensure_sufficient_free_balance(
		from: &T::AccountId,
		to: &T::AccountId,
		amount: BalanceOf<T>,
	) -> DispatchResult {
		if to == from {
			ensure!(T::Currency::balance(from) >= amount, Error::<T>::InsufficientBalanceToFreeze,);
		}

		Ok(())
	}

	fn finalize_vesting_transfer(
		from: &T::AccountId,
		to: &T::AccountId,
		schedule: ReleaseScheduleOf<T>,
		transfer_fn: fn(&T::AccountId, &T::AccountId, BalanceOf<T>) -> DispatchResult,
	) -> DispatchResult {
		let schedule_amount = Self::validate_and_get_schedule_amount(&schedule)?;

		let total_amount = Self::prune_and_get_frozen_balance(to)
			.checked_add(&schedule_amount)
			.ok_or(ArithmeticError::Overflow)?;

		transfer_fn(from, to, schedule_amount)?;
		Self::update_freeze(to, total_amount)?;

		Self::append_release_schedule(to, schedule)?;

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
