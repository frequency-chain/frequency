use crate::{BalanceOf, BlockNumberFor, Config, FreezeReason, Pallet, StakingAccountLedger};
use frame_support::{
	pallet_prelude::{GetStorageVersion, IsType, Weight},
	traits::{
		fungible::MutateFreeze, Get, LockIdentifier, LockableCurrency, OnRuntimeUpgrade,
		StorageVersion,
	},
};
use parity_scale_codec::Encode;
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::Saturating;

const LOG_TARGET: &str = "runtime::capacity";

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

const STAKING_ID: LockIdentifier = *b"netstkng";

/// The OnRuntimeUpgrade implementation for this storage migration
pub struct MigrationToV3<T, OldCurrency>(sp_std::marker::PhantomData<(T, OldCurrency)>);
impl<T, OldCurrency> MigrationToV3<T, OldCurrency>
where
	T: Config,
	OldCurrency: 'static + LockableCurrency<T::AccountId, Moment = BlockNumberFor<T>>,
	OldCurrency::Balance: IsType<BalanceOf<T>>,
{
	/// Translate capacity staked locked deposit to frozen deposit
	pub fn translate_lock_to_freeze(
		account_id: T::AccountId,
		amount: OldCurrency::Balance,
	) -> Weight {
		// 1 read get Locks: remove_lock: set locks
		// 1 read get Freezes
		// 1 read get Account
		//   1 write set Account: update_locks->try_mutate_account->ensure_upgraded: if account is *not* already upgraded.
		//     We could avoid this write by calling the upgrade script before running the migration,
		//     which would ensure that all accounts are upgraded.
		// 1 write set Account: update_locks->try_mutate_account: set account data
		// 1 read get Locks: update_locks: set existed with `contains_key`
		// 1 write set Locks: update_locks->Locks::remove: remove existed
		OldCurrency::remove_lock(STAKING_ID, &account_id);

		// 1 read get Freezes: set_freeze: set locks
		// 1 read get Locks: update_freezes: Locks::get().iter()
		//   1 write set Account: update_freezes->mutate_account->try_mutate_account->ensure_upgraded: if account is *not* already upgraded.
		// 1 write set Account: update_freezes->mutate_account->try_mutate_account: set account data
		// 1 write set Freezes: update_freezes: Freezes::insert

		<T as Config>::Currency::set_freeze(
			&FreezeReason::Staked.into(),
			&account_id,
			amount.into(),
		)
		.unwrap_or_else(|err| {
			log::error!(target: LOG_TARGET, "Failed to freeze {:?} for account 0x{:?}, reason: {:?}", amount, HexDisplay::from(&account_id.encode()), err);
		});

		log::info!(target: LOG_TARGET, "🔄 migrated account 0x{:?}, amount:{:?}", HexDisplay::from(&account_id.encode()), amount.into());
		T::DbWeight::get().reads_writes(6, 4)
	}
}

impl<T: Config, OldCurrency> OnRuntimeUpgrade for MigrationToV3<T, OldCurrency>
where
	T: Config,
	OldCurrency: 'static + LockableCurrency<T::AccountId, Moment = BlockNumberFor<T>>,
	OldCurrency::Balance: IsType<BalanceOf<T>>,
{
	fn on_runtime_upgrade() -> Weight {
		let on_chain_version = Pallet::<T>::on_chain_storage_version(); // 1r

		if on_chain_version.ge(&crate::pallet::STORAGE_VERSION) {
			log::info!(target: LOG_TARGET, "Old Capacity Locks->Freezes migration attempted to run. Please remove");
			return T::DbWeight::get().reads(1)
		}

		log::info!(target: LOG_TARGET, "🔄 Capacity Locks->Freezes migration started");
		// The migration started with 1r to get the STORAGE_VERSION
		let mut total_weight = T::DbWeight::get().reads_writes(1, 0);
		let mut total_accounts_migrated = 0u32;

		// Get all the keys(accounts) from the StakingAccountLedger
		StakingAccountLedger::<T>::iter()
			.map(|(account_id, staking_details)| (account_id, staking_details.active))
			.for_each(|(account_id, active_amount)| {
				let (total_unlocking, cal_fn_weight) =
					Pallet::<T>::get_unlocking_total_for(&account_id);
				let total_amount = active_amount.saturating_add(total_unlocking);

				let trans_fn_weight = MigrationToV3::<T, OldCurrency>::translate_lock_to_freeze(
					account_id.clone(),
					total_amount.into(),
				);

				total_weight =
					total_weight.saturating_add(cal_fn_weight).saturating_add(trans_fn_weight);

				total_accounts_migrated += 1;
			});

		log::info!(target: LOG_TARGET, "total accounts migrated from locks to freezes: {:?}", total_accounts_migrated);

		StorageVersion::new(3).put::<Pallet<T>>(); // 1 w
		total_weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

		log::info!(target: LOG_TARGET, "🔄 Capacity Locks->Freezes migration finished");
		log::info!(target: LOG_TARGET, "Capacity Migration calculated weight = {:?}", total_weight);

		total_weight
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
		use frame_support::storage::generator::StorageMap;
		let pallet_prefix = StakingAccountLedger::<T>::module_prefix();
		let storage_prefix = StakingAccountLedger::<T>::storage_prefix();
		assert_eq!(&b"Capacity"[..], pallet_prefix);
		assert_eq!(&b"StakingAccountLedger"[..], storage_prefix);
		log::info!(target: LOG_TARGET, "Running pre_upgrade...");

		let count = StakingAccountLedger::<T>::iter().count() as u32;
		log::info!(target: LOG_TARGET, "Finish pre_upgrade for {:?} records", count);
		Ok(count.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		use parity_scale_codec::Decode;
		let pre_upgrade_count: u32 = Decode::decode(&mut state.as_slice()).unwrap_or_default();
		let on_chain_version = Pallet::<T>::on_chain_storage_version();

		assert_eq!(on_chain_version, crate::pallet::STORAGE_VERSION);
		assert_eq!(pre_upgrade_count as usize, StakingAccountLedger::<T>::iter().count());

		log::info!(target: LOG_TARGET, "✅ migration post_upgrade checks passed");
		Ok(())
	}
}

#[cfg(test)]
#[cfg(feature = "try-runtime")]
mod test {
	use frame_support::traits::{tokens::fungible::InspectFreeze, WithdrawReasons};

	use super::*;
	use crate::{
		tests::mock::{Test as T, *},
		StakingAccountLedger, StakingDetails,
		StakingType::MaximumCapacity,
	};
	use pallet_balances::{BalanceLock, Reasons};

	type MigrationOf<T> = MigrationToV3<T, pallet_balances::Pallet<T>>;

	#[test]
	fn migration_works() {
		new_test_ext().execute_with(|| {
			// Create some data in the old format
			// Grab an account with a balance
			let account = 200;
			let locked_amount = 50;

			pallet_balances::Pallet::<T>::set_lock(
				STAKING_ID,
				&account,
				locked_amount,
				WithdrawReasons::all(),
			);
			// Confirm lock exists
			assert_eq!(
				pallet_balances::Pallet::<T>::locks(&account).get(0),
				Some(&BalanceLock { id: STAKING_ID, amount: locked_amount, reasons: Reasons::All })
			);

			let test_record =
				StakingDetails::<Test> { active: locked_amount, staking_type: MaximumCapacity };
			StakingAccountLedger::<Test>::insert(account, test_record);
			assert_eq!(StakingAccountLedger::<Test>::iter().count(), 1);

			// Run migration.
			let state = MigrationOf::<T>::pre_upgrade().unwrap();
			MigrationOf::<T>::on_runtime_upgrade();
			MigrationOf::<T>::post_upgrade(state).unwrap();

			// Check that the old staking locks are now freezes
			assert_eq!(pallet_balances::Pallet::<T>::locks(&account), vec![]);
			assert_eq!(
				<Test as Config>::Currency::balance_frozen(&FreezeReason::Staked.into(), &account),
				locked_amount
			);
		})
	}
}
