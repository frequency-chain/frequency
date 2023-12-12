use crate::{
	BalanceOf, BlockNumberFor, Config, FreezeReason, Pallet, StakingAccountLedger, StakingType,
};
use frame_support::{
	pallet_prelude::{GetStorageVersion, IsType, Weight},
	traits::{
		fungible::MutateFreeze, Get, LockIdentifier, LockableCurrency, OnRuntimeUpgrade,
		StorageVersion,
	},
};
use parity_scale_codec::Encode;
use sp_core::hexdisplay::HexDisplay;

const LOG_TARGET: &str = "runtime::capacity";

#[cfg(feature = "try-runtime")]
use sp_std::{fmt::Debug, vec::Vec};

const STAKING_ID: LockIdentifier = *b"netstkng";

/// Only contains V2 storage format
pub mod v2 {
	use super::*;
	use frame_support::{storage_alias, Twox64Concat};
	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;

	#[derive(Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
	/// The StakingDetails struct
	pub struct StakingDetails<T: Config> {
		/// The amount a Staker has staked, minus the sum of all tokens in `unlocking`.
		pub active: BalanceOf<T>,
		/// The type of staking for this staking account
		pub staking_type: StakingType,
	}

	#[storage_alias]
	/// alias to StakingAccountLedger storage
	pub(crate) type StakingAccountLedger<T: Config> = StorageMap<
		Pallet<T>,
		Twox64Concat,
		<T as frame_system::Config>::AccountId,
		StakingDetails<T>,
	>;
}

/// The OnRuntimeUpgrade implementation for this storage migration
pub struct MigrationToV3<T, OldCurrency>(sp_std::marker::PhantomData<(T, OldCurrency)>);
impl<T, OldCurrency> MigrationToV3<T, OldCurrency>
where
	T: Config,
	OldCurrency: 'static + LockableCurrency<T::AccountId, Moment = BlockNumberFor<T>>,
	OldCurrency::Balance: IsType<BalanceOf<T>>,
{
	/// Translate capacity staked locked deposit to frozen deposit
	pub fn translate_lock_to_freeze(account_id: T::AccountId, amount: OldCurrency::Balance) {
		OldCurrency::remove_lock(STAKING_ID, &account_id);
		T::Currency::set_freeze(&FreezeReason::Staked.into(), &account_id, amount.into())
			.unwrap_or_else(|err| {
				log::error!(target: LOG_TARGET, "Failed to freeze {:?} from account 0x{:?}, reason: {:?}", amount, HexDisplay::from(&account_id.encode()), err);
			});
	}
}

impl<T: Config, OldCurrency> OnRuntimeUpgrade for MigrationToV3<T, OldCurrency>
where
	T: Config,
	OldCurrency: 'static + LockableCurrency<T::AccountId, Moment = BlockNumberFor<T>>,
	OldCurrency::Balance: IsType<BalanceOf<T>>,
{
	fn on_runtime_upgrade() -> Weight {
		// Self::migrate_to_v3()
		let on_chain_version = Pallet::<T>::on_chain_storage_version(); // 1r

		if on_chain_version.lt(&3) {
			log::info!(target: LOG_TARGET, "🔄 Locks->Freezes migration started");
			let mut maybe_count = 0u32;
			// translate_lock_to_freeze(staker, amount);
			v2::StakingAccountLedger::<T>::iter()
				.map(|(account_id, staking_details)| (account_id, staking_details.active))
				.for_each(|(staker, amount)| {
					MigrationToV3::<T, OldCurrency>::translate_lock_to_freeze(
						staker,
						amount.into(),
					);
					maybe_count += 1;
					log::info!(target: LOG_TARGET,"migrated {:?}", maybe_count);
				});

			StorageVersion::new(3).put::<Pallet<T>>(); // 1 w
			let reads = (maybe_count + 1) as u64;
			let writes = (maybe_count * 2 + 1) as u64;
			log::info!(target: LOG_TARGET, "🔄 migration finished");
			let weight = T::DbWeight::get().reads_writes(reads, writes);
			log::info!(target: LOG_TARGET, "Migration calculated weight = {:?}", weight);
			weight
		} else {
			// storage was already migrated.
			log::info!(target: LOG_TARGET, "Old Locks->Freezes migration attempted to run. Please remove");
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
		use frame_support::storage::generator::StorageMap;
		// use parity_scale_codec::Encode;
		let pallet_prefix = v2::StakingAccountLedger::<T>::module_prefix();
		let storage_prefix = v2::StakingAccountLedger::<T>::storage_prefix();
		assert_eq!(&b"Capacity"[..], pallet_prefix);
		assert_eq!(&b"StakingAccountLedger"[..], storage_prefix);
		log::info!(target: LOG_TARGET, "Running pre_upgrade...");

		let count = v2::StakingAccountLedger::<T>::iter().count() as u32;
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
	use frame_support::traits::WithdrawReasons;

	use super::*;
	use crate::{
		tests::{
			mock::{Test as T, *},
		},
		StakingAccountLedger as OldStakingAccountLedger, StakingDetails as OldStakingDetails,
		StakingType::MaximumCapacity,
	};

	type MigrationOf<T> = MigrationToV3<T, pallet_balances::Pallet<T>>;

	#[test]
	fn migration_works() {
		new_test_ext().execute_with(|| {
			StorageVersion::new(2).put::<Pallet<T>>();
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

			let old_record =
				OldStakingDetails::<Test> { active: amount, staking_type: MaximumCapacity };
			OldStakingAccountLedger::<Test>::insert(account, old_record);
			assert_eq!(OldStakingAccountLedger::<Test>::iter().count(), 1);

			// Run migration.
			let state = MigrationOf::<T>::pre_upgrade().unwrap();
			MigrationOf::<T>::on_runtime_upgrade();
			MigrationOf::<T>::post_upgrade(state).unwrap();

			// Check that the old staking locks are now freezes
		})
	}
}
