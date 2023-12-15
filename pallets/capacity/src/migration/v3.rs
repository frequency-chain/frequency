use crate::{
	BalanceOf, BlockNumberFor, Config, FreezeReason, Pallet, StakingAccountLedger, STORAGE_VERSION,
};
use frame_support::{
	pallet_prelude::{GetStorageVersion, IsType, Weight},
	traits::{
		fungible::{Inspect, MutateFreeze},
		Get, LockIdentifier, LockableCurrency, OnRuntimeUpgrade, StorageVersion,
	},
};
use parity_scale_codec::Encode;
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::{DispatchError, Saturating};

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
	) -> Result<Weight, DispatchError> {
		log::info!(
			"Attempting to migrate {:?} locks for account 0x{:?} total_balance: {:?}, reducible_balance {:?}",
			amount,
			HexDisplay::from(&account_id.encode()),
			<T as Config>::Currency::total_balance(&account_id),
			<T as Config>::Currency::reducible_balance(&account_id, frame_support::traits::tokens::Preservation::Expendable, frame_support::traits::tokens::Fortitude::Polite),
		);
		OldCurrency::remove_lock(STAKING_ID, &account_id); // 1r + 1w
		match <T as Config>::Currency::set_freeze(
			&FreezeReason::Staked.into(),
			&account_id,
			amount.into(),
		) {
			Ok(_) => {
				log::info!(target: LOG_TARGET, "🔄 migrated account 0x{:?}, amount:{:?}", HexDisplay::from(&account_id.encode()), amount.into());
				Ok(T::DbWeight::get().reads_writes(2, 1))
			},
			Err(err) => {
				log::error!(target: LOG_TARGET, "Failed to freeze {:?} for account 0x{:?}, reason: {:?}", amount, HexDisplay::from(&account_id.encode()), err);
				Err(err)
			},
		}
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

		if on_chain_version.lt(&STORAGE_VERSION) {
			log::info!(target: LOG_TARGET, "🔄 Capacity Locks->Freezes migration started");
			let mut accounts_migrated_count = 0u32;
			StakingAccountLedger::<T>::iter()
				.map(|(account_id, staking_details)| (account_id, staking_details.active))
				.try_for_each(|(staker, active_amount)| {
					let total_amount = active_amount.saturating_add(Pallet::<T>::get_unlocking_total_for(&staker)); // 1r
					match MigrationToV3::<T, OldCurrency>::translate_lock_to_freeze(
						staker.clone(),
						total_amount.into(),
					)  { // 1r + 2w
						Ok(_) => {
							accounts_migrated_count += 1;
							log::info!(target: LOG_TARGET, "migrated count: {:?}", accounts_migrated_count);
							Ok(())
						},
						Err(err) => {
							log::error!(target: LOG_TARGET, "Failed to migrate account 0x{:?}, reason: {:?}", HexDisplay::from(&staker.encode()), err);
							Err(err)
						}
					}
				});

			StorageVersion::new(3).put::<Pallet<T>>(); // 1 w
			let reads = (accounts_migrated_count * 2 + 1) as u64;
			let writes = (accounts_migrated_count * 2 + 1) as u64;
			log::info!(target: LOG_TARGET, "🔄 migration finished");
			let weight = T::DbWeight::get().reads_writes(reads, writes);
			log::info!(target: LOG_TARGET, "Migration calculated weight = {:?}", weight);
			weight
		} else {
			// storage was already migrated.
			log::info!(target: LOG_TARGET, "Old Capacity Locks->Freezes migration attempted to run. Please remove");
			T::DbWeight::get().reads(1)
		}
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
			let amount = 50;

			pallet_balances::Pallet::<T>::set_lock(
				STAKING_ID,
				&account,
				amount,
				WithdrawReasons::all(),
			);
			// Confirm lock exists
			assert_eq!(
				pallet_balances::Pallet::<T>::locks(&account).get(0),
				Some(&BalanceLock { id: STAKING_ID, amount: 50u64, reasons: Reasons::All })
			);

			let test_record =
				StakingDetails::<Test> { active: amount, staking_type: MaximumCapacity };
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
				50u64
			);
		})
	}
}
