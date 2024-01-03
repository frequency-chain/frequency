use crate::{types, BalanceOf, BlockNumberFor, Config, FreezeReason, Pallet, ReleaseSchedules};
use frame_support::{
	pallet_prelude::{GetStorageVersion, IsType, Weight},
	traits::{
		fungible::MutateFreeze, Get, LockIdentifier, LockableCurrency, OnRuntimeUpgrade,
		StorageVersion,
	},
};
use parity_scale_codec::Encode;
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::{traits::Zero, Saturating};

const LOG_TARGET: &str = "runtime::time-release";

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

const RELEASE_LOCK_ID: LockIdentifier = *b"timeRels";

/// The OnRuntimeUpgrade implementation for this storage migration
pub struct MigrationToV2<T, OldCurrency>(sp_std::marker::PhantomData<(T, OldCurrency)>);
impl<T, OldCurrency> MigrationToV2<T, OldCurrency>
where
	T: Config,
	OldCurrency: 'static + LockableCurrency<T::AccountId, Moment = BlockNumberFor<T>>,
	OldCurrency::Balance: IsType<BalanceOf<T>>,
{
	/// Translate time release locked deposit to frozen deposit
	pub fn translate_lock_to_freeze(
		account_id: &T::AccountId,
		amount: OldCurrency::Balance,
	) -> Weight {
		// 1 read get Freezes: set_freeze: get locks
		// 1 read get Locks: update_freezes: Locks::get().iter()
		// 1 write set Account: update_freezes->mutate_account->try_mutate_account->ensure_upgraded: if account is *not* already upgraded.
		// 1 write set Account: update_freezes->mutate_account->try_mutate_account: set account data
		// 1 write set Freezes: update_freezes: Freezes::insert
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
		});

		// 1 read get Locks: remove_lock: get locks
		// 1 read get Freezes
		// 1 read get Account
		// 1 write set Account: update_locks->try_mutate_account->ensure_upgraded: if account is *not* already upgraded.
		// 1 write set Account: update_locks->try_mutate_account: set account data
		// [Cached] 1 read get Locks: update_locks: set existed with `contains_key`
		// 1 write set Locks: update_locks->Locks::remove: remove existed
		OldCurrency::remove_lock(RELEASE_LOCK_ID, &account_id);

		T::DbWeight::get().reads_writes(5, 6)
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

		// storage was already migrated.
		if on_chain_version.ge(&2) {
			log::info!(
				target: LOG_TARGET,
				"Old Time Release Locks->Freezes migration attempted to run. Please remove"
			);
			return T::DbWeight::get().reads(1)
		}

		log::info!(target: LOG_TARGET, "🔄 Time Release Locks->Freezes migration started");
		// The migration started with 1r to get the on_chain_storage_version
		let mut total_weight = T::DbWeight::get().reads_writes(1, 0);
		let mut total_accounts_migrated = 0u32;

		// Get all the keys(accounts) from the ReleaseSchedules storage
		ReleaseSchedules::<T>::iter()
			.map(|(account_id, _)| account_id)
			.for_each(|account_id| {
				let (total_amount, cal_fn_weight) = calculate_total_scheduled_frozen_for_account::<T>(&account_id);

				let trans_fn_weight = MigrationToV2::<T, OldCurrency>::translate_lock_to_freeze(
					&account_id,
					total_amount.into(),
				);

				total_weight = total_weight.saturating_add(cal_fn_weight).saturating_add(trans_fn_weight);

				total_accounts_migrated += 1;

				log::info!(target: LOG_TARGET, "🔄 migrated account 0x{:?}, amount:{:?}", HexDisplay::from(&account_id.encode()), total_amount);
			});

		log::info!(
			target: LOG_TARGET,
			"total accounts migrated from locks to frozen {:?}",
			total_accounts_migrated
		);

		StorageVersion::new(2).put::<Pallet<T>>();
		total_weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

		log::info!(target: LOG_TARGET, "🔄 Time Release migration finished");
		log::info!(
			target: LOG_TARGET,
			"Time Release Migration calculated weight = {:?}",
			total_weight
		);

		total_weight
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
		use frame_support::storage::generator::StorageMap;

		let on_chain_version = Pallet::<T>::on_chain_storage_version(); // 1r
		if on_chain_version >= 2 {
			return Ok(Vec::new())
		}

		let pallet_prefix = ReleaseSchedules::<T>::module_prefix();
		let storage_prefix = ReleaseSchedules::<T>::storage_prefix();
		assert_eq!(&b"TimeRelease"[..], pallet_prefix);
		assert_eq!(&b"ReleaseSchedules"[..], storage_prefix);
		log::info!(target: LOG_TARGET, "Running pre_upgrade...");

		let count = ReleaseSchedules::<T>::iter().count() as u32;
		log::info!(target: LOG_TARGET, "Finish pre_upgrade for {:?} records", count);
		Ok(count.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		use parity_scale_codec::Decode;

		let on_chain_version = Pallet::<T>::on_chain_storage_version(); // 1r
		if on_chain_version >= 2 {
			return Ok(())
		}
		let pre_upgrade_count: u32 = Decode::decode(&mut state.as_slice()).unwrap_or_default();
		let on_chain_version = Pallet::<T>::on_chain_storage_version();

		assert_eq!(on_chain_version, crate::module::STORAGE_VERSION);
		assert_eq!(pre_upgrade_count as usize, ReleaseSchedules::<T>::iter().count());

		log::info!(target: LOG_TARGET, "✅ migration post_upgrade checks passed");
		Ok(())
	}
}

fn calculate_total_scheduled_frozen_for_account<T: Config>(
	account_id: &T::AccountId,
) -> (BalanceOf<T>, Weight) {
	let total = ReleaseSchedules::<T>::get(&account_id) // 1r
		.iter()
		.map(|schedule: &types::ReleaseSchedule<BlockNumberFor<T>, BalanceOf<T>>| {
			schedule.total_amount()
		})
		.fold(Zero::zero(), |acc: BalanceOf<T>, amount| {
			acc.saturating_add(amount.unwrap_or(Zero::zero()))
		});

	(total, T::DbWeight::get().reads(1))
}

#[cfg(test)]
mod test {
	use frame_support::traits::{tokens::fungible::InspectFreeze, WithdrawReasons};

	use super::*;
	use crate::mock::{Test as T, *};
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

			create_schedules_and_set_freeze(schedules);

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
			let pre_upgrade_count = ReleaseSchedules::<T>::iter().count() as u32;
			MigrationOf::<Test>::on_runtime_upgrade();
			let on_chain_version = Pallet::<T>::on_chain_storage_version();

			assert_eq!(on_chain_version, crate::module::STORAGE_VERSION);
			assert_eq!(pre_upgrade_count as usize, ReleaseSchedules::<T>::iter().count());

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

	fn create_schedules_and_set_freeze(schedules: Vec<(AccountId, u32, u32, u32, u64)>) {
		schedules.iter().for_each(|(who, start, period, period_count, per_period)| {
			let mut bounded_schedules = ReleaseSchedules::<Test>::get(who);
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

			ReleaseSchedules::<Test>::insert(who, bounded_schedules);
		});
	}
}
