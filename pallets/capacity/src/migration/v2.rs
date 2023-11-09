use frame_support::{pallet_prelude::*, traits::OnRuntimeUpgrade};
use sp_runtime::Saturating;

/// migration module
pub mod migration {
	use std::marker::PhantomData;
	use crate::{BalanceOf, Config, Pallet, StakingType, StakingAccountLedger, types::{StakingDetails, UnlockChunk}, UnstakeUnlocks, UnlockChunks};
	use frame_support::{
		traits::OnRuntimeUpgrade,
		pallet_prelude::{GetStorageVersion, Weight}
	};
	use frame_support::traits::StorageVersion;
	use sp_runtime::TryRuntimeError;

	const LOG_TARGET: &str ="runtime::messages";

	/// Only contains V1 storage format
	pub mod v1 {
		use super::*;
		use codec::{Decode, Encode, MaxEncodedLen};
		use frame_support::{BoundedVec, storage_alias, Twox64Concat, pallet_prelude::OptionQuery};
		use scale_info::TypeInfo;
		use sp_core::Get;
		use sp_runtime::traits::AtLeast32BitUnsigned;
		use sp_std::fmt::Debug;
		use log;

		#[derive(Default, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
		#[scale_info(skip_type_params(Balance, EpochNumber, MaxDataSize))]
		#[codec(mel_bound(MaxDataSize: MaxEncodedLen, StakingAccountDetails<Balance, EpochNumber, MaxDataSize>: Encode))]
		pub struct StakingAccountDetails<Balance, EpochNumber, MaxDataSize>
		where
			Balance: AtLeast32BitUnsigned + Copy + Debug + MaxEncodedLen,
			EpochNumber: AtLeast32BitUnsigned + Copy + Debug + MaxEncodedLen,
			MaxDataSize: Get<u32> + Debug,
		{
			/// The amount a Staker has staked, minus the sum of all tokens in `unlocking`.
			pub active: Balance,
			/// The total amount of tokens in `active` and `unlocking`
			pub total: Balance,
			/// Unstaked balances that are thawing or awaiting withdrawal.
			pub unlocking: BoundedVec<UnlockChunk<Balance, EpochNumber>, MaxDataSize>,
		}

		#[storage_alias]
		pub(crate) type StorageAccountLedger<T: Config> =
		StorageMap<
			Pallet<T>,
			Twox64Concat,
			<T as frame_system::Config>::AccountId,
			StakingAccountDetails<BalanceOf<T>, T::EpochNumber, T::MaxUnlockingChunks>,
			OptionQuery
		>;

	}


	/// migrate StakingAccountLedger to use new StakingAccountDetailsV2
	pub fn migrate_to_v2<T: Config>() -> Weight {
		let onchain_version = Pallet::<T>::on_chain_storage_version();
		// let current_version = Pallet::<T>::current_storage_version();

		type OldDetails =v1::StakingAccountDetails<BalanceOf<T>, T::EpochNumber, T::MaxUnlockingChunks>;

		if onchain_version.gt(&2) {
			log::info!(target: LOG_TARGET, "🔄 StakingAccountLedger migration started");
			StakingAccountLedger::<T>::translate::<OldDetails,_>(
					|key: <T as frame_system::Config>::AccountId, old_details: OldDetails| {
						let new_account: StakingDetails<T> = StakingDetails {
							active: old_details.active.clone(),
							staking_type: StakingType::MaximumCapacity,
						};
						let new_unlocks: UnlockChunks<T> = UnlockChunks {
							unlocking: old_details.unlocking.clone()
						};
						UnstakeUnlocks::<T>::insert(key, new_unlocks);
						log::info!(target: LOG_TARGET, "Migrated {:?}", key);
						Some(new_account)
					}
				);
			StorageVersion::new(2).put::<Pallet::<T>>();
			log::info!(target: LOG_TARGET, "🔄 migration finished");
			let count = (StakingAccountLedger::<T>::iter().count() + 1) as u64;
			T::DbWeight::get().reads_writes(count, count)
		} else {
			// storage was already migrated.
			Weight::zero()
		}
	}

	pub struct MigrateToV2<T>(PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV2<T> {
		fn on_runtime_upgrade() -> Weight {
			migrate_to_v2::<T>()
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
			use frame_support::storage::generator::StorageDoubleMap;
			log::info!(target: LOG_TARGET, "Running pre_upgrade...");


			let pallet_prefix = v1::StorageAccountLedger::<T>::module_prefix();
			let storage_prefix = v1::StorageAccountLedger::<T>::storage_prefix();
			assert_eq!(&b"StorageAccountLedger"[..], pallet_prefix);
			assert_eq!(&b"StorageAccountLedger"[..], storage_prefix);

			let count = v1::StorageAccountLedger::<T>::iter().count();
			log::info!(target: LOG_TARGET, "Finish pre_upgrade for {:?} records", count);
			Ok(count.into())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(state: Vec<u8>) -> Result<(), TryRuntimeError> {
			use codec::Decode;
			let pre_upgrade_count: u32 = Decode::decode::<Vec<u8>>(state.as_mut()).unwrap_or_default();
			let on_chain_version = Pallet::<T>::on_chain_storage_version();

			assert_eq!(on_chain_version, crate::pallet::STORAGE_VERSION);
			assert_eq!(pre_upgrade_count, StakingAccountLedger::<T>::iter().count());
			assert_eq!(pre_upgrade_count, UnstakeUnlocks::<T>::iter().count());

			log::info!(target: LOG_TARGET, "✅ migration post_upgrade checks passed");
		}
	}
}
