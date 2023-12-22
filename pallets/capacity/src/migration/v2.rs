use crate::{
	types::{StakingDetails, UnlockChunk},
	unlock_chunks_total, BalanceOf, BlockNumberFor, Config, FreezeReason, Pallet,
	StakingAccountLedger, StakingType, UnlockChunkList, UnstakeUnlocks,
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
use sp_runtime::traits::Saturating;

const LOG_TARGET: &str = "runtime::capacity";

#[cfg(feature = "try-runtime")]
use sp_std::{fmt::Debug, vec::Vec};

/// LockIdentifier for the old capacity staking lock
pub const STAKING_ID: LockIdentifier = *b"netstkng";
/// Only contains V1 storage format
pub mod v1 {
	use super::*;
	use frame_support::{storage_alias, BoundedVec, Twox64Concat};
	use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
	use scale_info::TypeInfo;

	#[derive(Default, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
	/// Old StakingAccountDetails struct
	pub struct StakingAccountDetails<T: Config> {
		/// The amount a Staker has staked, minus the sum of all tokens in `unlocking`.
		pub active: BalanceOf<T>,
		/// The total amount of tokens in `active` and `unlocking`
		pub total: BalanceOf<T>,
		/// Unstaked balances that are thawing or awaiting withdrawal.
		pub unlocking: BoundedVec<
			UnlockChunk<BalanceOf<T>, T::EpochNumber>,
			<T as crate::pallet::Config>::MaxUnlockingChunks,
		>,
	}

	#[storage_alias]
	/// alias to StakingAccountLedger storage
	pub(crate) type StakingAccountLedger<T: Config> = StorageMap<
		Pallet<T>,
		Twox64Concat,
		<T as frame_system::Config>::AccountId,
		StakingAccountDetails<T>,
	>;
}

/// The OnRuntimeUpgrade implementation for this storage migration
pub struct MigrateToV2<T, OldCurrency>(sp_std::marker::PhantomData<(T, OldCurrency)>);
impl<T, OldCurrency> MigrateToV2<T, OldCurrency>
where
	T: Config,
	OldCurrency: 'static + LockableCurrency<T::AccountId, Moment = BlockNumberFor<T>>,
	OldCurrency::Balance: IsType<BalanceOf<T>>,
{
	/// migrate StakingAccountLedger to use new StakingDetails
	pub fn migrate_to_v2() -> Weight {
		let on_chain_version = Pallet::<T>::on_chain_storage_version(); // 1r

		if on_chain_version.ge(&2) {
			// storage was already migrated.
			log::info!(target: LOG_TARGET, "Old StakingAccountLedger / Locks->Freezes migration attempted to run. Please remove");
			return T::DbWeight::get().reads(1)
		}

		log::info!(target: LOG_TARGET, "🔄 StakingAccountLedger / Locks->Freezes migration started");
		// The migration started with 1r to get the STORAGE_VERSION
		let mut total_weight = T::DbWeight::get().reads_writes(1, 0);
		let mut total_accounts_migrated = 0u32;

		StakingAccountLedger::<T>::translate(
			|key: <T as frame_system::Config>::AccountId,
			 old_details: v1::StakingAccountDetails<T>| {
				let new_account: StakingDetails<T> = StakingDetails {
					active: old_details.active,
					staking_type: StakingType::MaximumCapacity,
				};
				let new_unlocks: UnlockChunkList<T> = old_details.unlocking;
				UnstakeUnlocks::<T>::insert(key.clone(), new_unlocks);

				let total_unlocking = Self::get_unlocking_total_for(&key);
				let total_amount = new_account.active.saturating_add(total_unlocking);

				let trans_fn_weight = MigrateToV2::<T, OldCurrency>::translate_lock_to_freeze(
					key.clone(),
					total_amount.into(),
				);

				// add the reads and writes from this iteration
				total_weight = total_weight
					.saturating_add(trans_fn_weight)
					.saturating_add(T::DbWeight::get().reads_writes(1, 2));

				total_accounts_migrated += 1;
				// log::info!(target: LOG_TARGET,"migrated {:?}", total_accounts_migrated);
				Some(new_account)
			},
		);
		StorageVersion::new(2).put::<Pallet<T>>(); // 1 w
		log::info!(target: LOG_TARGET, "🔄 migration finished");
		total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));
		log::info!(target: LOG_TARGET, "Migration calculated weight = {:?}", total_weight);
		total_weight
	}

	/// Translate capacity staked locked deposit to frozen deposit
	pub fn translate_lock_to_freeze(
		account_id: T::AccountId,
		amount: OldCurrency::Balance,
	) -> Weight {
		// 1 read get Freezes: set_freeze: get locks
		// 1 read get Locks: update_freezes: Locks::get().iter()
		// 1 write set Account: update_freezes->mutate_account->try_mutate_account->ensure_upgraded: if account is *not* already upgraded.
		// 1 write set Account: update_freezes->mutate_account->try_mutate_account: set account data
		// 1 write set Freezes: update_freezes: Freezes::insert
		<T as Config>::Currency::set_freeze(
			&FreezeReason::CapacityStaking.into(),
			&account_id,
			amount.into(),
		)
		.unwrap_or_else(|err| {
			log::error!(target: LOG_TARGET, "Failed to freeze {:?} for account 0x{:?}, reason: {:?}", amount, HexDisplay::from(&account_id.encode()), err);
		});

		// 1 read get Locks: remove_lock: set locks
		// 1 read get Freezes
		// 1 read get Account
		// 1 write set Account: update_locks->try_mutate_account->ensure_upgraded: if account is *not* already upgraded.
		// 1 write set Account: update_locks->try_mutate_account: set account data
		// [Cached] 1 read get Locks: update_locks: set existed with `contains_key`
		// 1 write set Locks: update_locks->Locks::remove: remove existed
		OldCurrency::remove_lock(STAKING_ID, &account_id);

		log::info!(target: LOG_TARGET, "🔄 migrated account 0x{:?}, amount:{:?}", HexDisplay::from(&account_id.encode()), amount.into());
		T::DbWeight::get().reads_writes(5, 6)
	}

	/// Calculates the total amount of tokens that have been unstaked and are not thawed, for the given staker.
	pub fn get_unlocking_total_for(staker: &T::AccountId) -> BalanceOf<T> {
		let unlocks = UnstakeUnlocks::<T>::get(staker).unwrap_or_default();
		unlock_chunks_total::<T>(&unlocks)
	}
}

impl<T: Config, OldCurrency> OnRuntimeUpgrade for MigrateToV2<T, OldCurrency>
where
	T: Config,
	OldCurrency: 'static + LockableCurrency<T::AccountId, Moment = BlockNumberFor<T>>,
	OldCurrency::Balance: IsType<BalanceOf<T>>,
{
	fn on_runtime_upgrade() -> Weight {
		Self::migrate_to_v2()
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
		use frame_support::storage::generator::StorageMap;
		use parity_scale_codec::Encode;
		let pallet_prefix = v1::StakingAccountLedger::<T>::module_prefix();
		let storage_prefix = v1::StakingAccountLedger::<T>::storage_prefix();
		assert_eq!(&b"Capacity"[..], pallet_prefix);
		assert_eq!(&b"StakingAccountLedger"[..], storage_prefix);
		log::info!(target: LOG_TARGET, "Running pre_upgrade...");

		let count = v1::StakingAccountLedger::<T>::iter().count() as u32;
		log::info!(target: LOG_TARGET, "Finish pre_upgrade for {:?} records", count);
		Ok(count.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		use parity_scale_codec::Decode;
		let pre_upgrade_count: u32 = Decode::decode(&mut state.as_slice()).unwrap_or_default();
		let on_chain_version = Pallet::<T>::on_chain_storage_version();

		assert_eq!(on_chain_version, 2);
		assert_eq!(pre_upgrade_count as usize, StakingAccountLedger::<T>::iter().count());
		assert_eq!(pre_upgrade_count as usize, UnstakeUnlocks::<T>::iter().count());

		log::info!(target: LOG_TARGET, "✅ migration post_upgrade checks passed");
		Ok(())
	}
}
