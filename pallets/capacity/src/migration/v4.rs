use crate::{Config, EpochLength, Pallet};

use frame_support::{
	pallet_prelude::{GetStorageVersion, Weight},
	traits::{Get, OnRuntimeUpgrade, StorageVersion},
};
use sp_runtime::Saturating;

const LOG_TARGET: &str = "runtime::capacity";

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

/// The OnRuntimeUpgrade implementation for this storage migration
pub struct MigrationToV4<T>(sp_std::marker::PhantomData<T>);
impl<T> MigrationToV4<T>
where
	T: Config,
{
	/// Translate capacity staked locked deposit to frozen deposit
	pub fn update_epoch_length() -> Weight {
		let current_epoch_length = EpochLength::<T>::get();

		let new_epoch_length = current_epoch_length.saturating_mul(2u32.into());

		EpochLength::<T>::put(new_epoch_length);

		T::DbWeight::get().reads_writes(0, 1)
	}
}

impl<T: Config> OnRuntimeUpgrade for MigrationToV4<T>
where
	T: Config,
{
	fn on_runtime_upgrade() -> Weight {
		let on_chain_version = Pallet::<T>::on_chain_storage_version(); // 1r

		if on_chain_version >= 3 {
			log::info!(target: LOG_TARGET, "Old Capacity Locks->Freezes migration attempted to run. Please remove");
			return T::DbWeight::get().reads(1);
		}

		log::info!(target: LOG_TARGET, "🔄 Capacity 6 second update migration started");
		// The migration started with 1r to get the STORAGE_VERSION
		let mut total_weight = T::DbWeight::get().reads_writes(1, 0);

		total_weight += Self::update_epoch_length();

		StorageVersion::new(3).put::<Pallet<T>>(); // 1 w

		total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

		log::info!(target: LOG_TARGET, "🔄 Capacity 6 second update migration finished");

		total_weight
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
		use frame_support::storage::generator::StorageMap;
		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		if on_chain_version >= 3 {
			return Ok(Vec::new());
		}

		let pallet_prefix = EpochLength::<T>::pallet_prefix();
		let storage_prefix = EpochLength::<T>::storage_prefix();
		assert_eq!(&b"Capacity"[..], pallet_prefix);
		assert_eq!(&b"EpochLength"[..], storage_prefix);
		log::info!(target: LOG_TARGET, "Running pre_upgrade...");

		asset_eq!(EpochLength::<T>::get(), 7_200u32);

		log::info!(target: LOG_TARGET, "Finish pre_upgrade for {:?} records", count);
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		use parity_scale_codec::Decode;
		let pre_upgrade_count: u32 = Decode::decode(&mut state.as_slice()).unwrap_or_default();
		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		if on_chain_version >= 3 {
			return Ok(());
		}

		assert_eq!(on_chain_version, crate::pallet::STORAGE_VERSION);

		let post_upgrade_epoch_length = EpochLength::<T>::get();
		assert_eq!(post_upgrade_epoch_length, 7_200);

		log::info!(target: LOG_TARGET, "✅ migration post_upgrade checks passed");
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::tests::mock::{Test as T, *};

	type MigrationOf<T> = MigrationToV4<T>;

	#[test]
	fn migration_works() {
		new_test_ext().execute_with(|| {
			EpochLength::<T>::put(7_200u32);

			assert_eq!(EpochLength::<T>::get(), 7_200u32);

			MigrationOf::<T>::on_runtime_upgrade();

			let on_chain_version = Pallet::<T>::on_chain_storage_version();
			assert_eq!(on_chain_version, crate::pallet::STORAGE_VERSION);

			assert_eq!(EpochLength::<T>::get(), 14_400u32);
		})
	}
}
