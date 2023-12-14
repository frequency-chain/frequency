use crate::{types, BalanceOf, BlockNumberFor, Config, FreezeReason, Pallet};
use frame_support::{
	pallet_prelude::{GetStorageVersion, IsType, Weight},
	traits::{
		fungible::MutateFreeze, Get, LockIdentifier, LockableCurrency, OnRuntimeUpgrade,
		StorageVersion,
	},
	Blake2_128Concat,
};
use parity_scale_codec::Encode;
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::{traits::Zero, Saturating};

const LOG_TARGET: &str = "runtime::capacity";

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

const RELEASE_LOCK_ID: LockIdentifier = *b"timeRels";

/// Only contains V1 storage format
pub mod v1 {
	use super::*;
	use frame_support::{pallet_prelude::ValueQuery, storage_alias, BoundedVec};

	pub(crate) type ReleaseScheduleOf<T> = types::ReleaseSchedule<BlockNumberFor<T>, BalanceOf<T>>;

	/// Release schedules of an account.
	///
	/// ReleaseSchedules: `map AccountId => Vec<ReleaseSchedule>`
	/// alias to ReleaseSchedules storage
	#[storage_alias]
	pub(crate) type ReleaseSchedules<T: Config> = StorageMap<
		Pallet<T>,
		Blake2_128Concat,
		<T as frame_system::Config>::AccountId,
		BoundedVec<ReleaseScheduleOf<T>, <T as Config>::MaxReleaseSchedules>,
		ValueQuery,
	>;
}

/// The OnRuntimeUpgrade implementation for this storage migration
pub struct MigrationToV2<T, OldCurrency>(sp_std::marker::PhantomData<(T, OldCurrency)>);
impl<T, OldCurrency> MigrationToV2<T, OldCurrency>
where
	T: Config,
	OldCurrency: 'static + LockableCurrency<T::AccountId, Moment = BlockNumberFor<T>>,
	OldCurrency::Balance: IsType<BalanceOf<T>>,
{
	/// Translate capacity staked locked deposit to frozen deposit
	pub fn translate_lock_to_freeze(account_id: T::AccountId, amount: OldCurrency::Balance) {
		OldCurrency::remove_lock(RELEASE_LOCK_ID, &account_id); // 1r + 1w
														// TODO: Can we do anything if set_freeze fails?
		T::Currency::set_freeze(
			&FreezeReason::TimeReleaseVesting.into(),
			&account_id,
			amount.into(),
		)
		.unwrap_or_else(|err| {
			log::error!(
				target: LOG_TARGET,
				"Failed to freeze {:?} from account 0x{:?}, reason: {:?}",
				amount,
				HexDisplay::from(&account_id.encode()),
				err
			);
		}); // 1w
	}
}

impl<T: Config, OldCurrency> OnRuntimeUpgrade for MigrationToV2<T, OldCurrency>
where
	T: Config,
	OldCurrency: 'static + LockableCurrency<T::AccountId, Moment = BlockNumberFor<T>>,
	OldCurrency::Balance: IsType<BalanceOf<T>>,
{
	fn on_runtime_upgrade() -> Weight {
		log::info!(target: LOG_TARGET, "Running storage migration...");

		let on_chain_version = Pallet::<T>::on_chain_storage_version();

		if on_chain_version.lt(&crate::module::STORAGE_VERSION) {
			log::info!(target: LOG_TARGET, "🔄 Time Release Locks->Freezes migration started");
			let mut maybe_count = 0u32;

			// Get all the keys(accounts) from the ReleaseSchedules storage
			v1::ReleaseSchedules::<T>::iter().map(|(account_id, _)| account_id).for_each(
				|account_id| {
					// Get the total amount of tokens in the account's ReleaseSchedules
					let total_amount = v1::ReleaseSchedules::<T>::get(&account_id) // 1r
						.iter()
						.map(
							|schedule: &types::ReleaseSchedule<BlockNumberFor<T>, BalanceOf<T>>| {
								schedule.total_amount()
							},
						)
						.fold(Zero::zero(), |acc: BalanceOf<T>, amount| {
							acc.saturating_add(amount.unwrap_or(Zero::zero()))
						});

					// Translate the lock to a freeze
					MigrationToV2::<T, OldCurrency>::translate_lock_to_freeze(
						account_id,
						total_amount.into(),
					);
					maybe_count += 1;

					log::info!(target: LOG_TARGET, "total_amount {:?}", total_amount);
					log::info!(target: LOG_TARGET, "migrated {:?}", maybe_count);
				},
			);

			StorageVersion::new(2).put::<Pallet<T>>(); // 1 w
			let reads = (maybe_count * 2 + 1) as u64;
			// REVIEW: Are we doing 2 writes per account?
			let writes = (maybe_count * 2 + 1) as u64;
			log::info!(target: LOG_TARGET, "🔄 Time Release migration finished");
			let weight = T::DbWeight::get().reads_writes(reads, writes);
			log::info!(
				target: LOG_TARGET,
				"Time Release Migration calculated weight = {:?}",
				weight
			);
			weight
		} else {
			// storage was already migrated.
			log::info!(
				target: LOG_TARGET,
				"Old Time Release Locks->Freezes migration attempted to run. Please remove"
			);
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
		use frame_support::storage::generator::StorageMap;
		let pallet_prefix = v1::ReleaseSchedules::<T>::module_prefix();
		let storage_prefix = v1::ReleaseSchedules::<T>::storage_prefix();
		assert_eq!(&b"TimeRelease"[..], pallet_prefix);
		assert_eq!(&b"ReleaseSchedules"[..], storage_prefix);
		log::info!(target: LOG_TARGET, "Running pre_upgrade...");

		let count = v1::ReleaseSchedules::<T>::iter().count() as u32;
		log::info!(target: LOG_TARGET, "Finish pre_upgrade for {:?} records", count);
		Ok(count.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		use crate::ReleaseSchedules;
		use parity_scale_codec::Decode;
		let pre_upgrade_count: u32 = Decode::decode(&mut state.as_slice()).unwrap_or_default();
		let on_chain_version = Pallet::<T>::on_chain_storage_version();

		assert_eq!(on_chain_version, crate::module::STORAGE_VERSION);
		assert_eq!(pre_upgrade_count as usize, ReleaseSchedules::<T>::iter().count());

		log::info!(target: LOG_TARGET, "✅ migration post_upgrade checks passed");
		Ok(())
	}
}

#[cfg(test)]
#[cfg(feature = "try-runtime")]
mod test {
	use frame_support::traits::{tokens::fungible::InspectFreeze, WithdrawReasons};

	use super::*;
	use crate::mock::{Test, *};
	use pallet_balances::{BalanceLock, Reasons};

	type MigrationOf<T> = MigrationToV2<T, pallet_balances::Pallet<T>>;

	#[test]
	fn migration_works() {
		ExtBuilder::build().execute_with(|| {
			assert_eq!(pallet_balances::Pallet::<Test>::free_balance(&DAVE), DAVE_BALANCE);

			let schedules = vec![
				// who, start, period, period_count, per_period
				(DAVE, 2, 3, 1, 5),
			];

			create_schedules_and_set_lock(schedules);

			assert_eq!(
				pallet_balances::Pallet::<Test>::locks(&DAVE)
					.iter()
					.find(|l| l.id == RELEASE_LOCK_ID),
				Some(&BalanceLock {
					id: RELEASE_LOCK_ID,
					amount: 5u64.into(),
					reasons: Reasons::All
				})
			);

			// Run migration.
			let state = MigrationOf::<Test>::pre_upgrade().unwrap();
			MigrationOf::<Test>::on_runtime_upgrade();
			MigrationOf::<Test>::post_upgrade(state).unwrap();

			assert_eq!(
				pallet_balances::Pallet::<Test>::locks(&DAVE),
				vec![],
				"Locks should be empty"
			);
			assert_eq!(
				<Test as Config>::Currency::balance_frozen(
					&FreezeReason::TimeReleaseVesting.into(),
					&DAVE
				),
				5u64,
				"Frozen balance should be 5 for account {:?}",
				DAVE
			);
		})
	}

	fn create_schedules_and_set_lock(schedules: Vec<(AccountId, u32, u32, u32, u64)>) {
		schedules.iter().for_each(|(who, start, period, period_count, per_period)| {
			let mut bounded_schedules = v1::ReleaseSchedules::<Test>::get(who);
			let new_schedule = types::ReleaseSchedule {
				start: *start,
				period: *period,
				period_count: *period_count,
				per_period: *per_period,
			};

			bounded_schedules
				.try_push(new_schedule.clone())
				.expect("Max release schedules exceeded");

			let total_amount = bounded_schedules.iter().fold(
				Zero::zero(),
				|acc_amount: BalanceOf<Test>, schedule| {
					acc_amount.saturating_add(schedule.total_amount().unwrap_or(Zero::zero()))
				},
			);

			assert_eq!(total_amount, new_schedule.total_amount().unwrap_or(Zero::zero()));

			assert!(
				pallet_balances::Pallet::<Test>::free_balance(who) >= total_amount,
				"Account does not have enough balance"
			);

			pallet_balances::Pallet::<Test>::set_lock(
				RELEASE_LOCK_ID,
				&who,
				total_amount,
				WithdrawReasons::all(),
			);

			v1::ReleaseSchedules::<Test>::insert(who, bounded_schedules);
		});
	}
}
