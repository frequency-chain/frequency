use crate::{Config, EpochLength, Pallet};
use frame_support::{
	pallet_prelude::{GetStorageVersion, Weight},
	traits::{Get, OnRuntimeUpgrade, StorageVersion},
};
use frame_system::pallet_prelude::BlockNumberFor;

const LOG_TARGET: &str = "runtime::capacity";

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

/// The OnRuntimeUpgrade implementation for this storage migration
pub struct MigrationToV4<T>(sp_std::marker::PhantomData<T>);
impl<T> MigrationToV4<T>
where
	T: Config,
{
	/// Update the epoch length to double the current value
	pub fn update_epoch_length() -> Weight {
		let current_epoch_length = EpochLength::<T>::get();
		let new_epoch_length: BlockNumberFor<T> = current_epoch_length * 2u32.into();
		log::info!(target: LOG_TARGET, "🔄 Capacity EpochLength update from {:?} to {:?}", current_epoch_length, new_epoch_length);

		EpochLength::<T>::put(new_epoch_length);

		T::DbWeight::get().reads_writes(1, 1)
	}
}

impl<T: Config> OnRuntimeUpgrade for MigrationToV4<T>
where
	T: Config,
{
	fn on_runtime_upgrade() -> Weight {
		let on_chain_version = Pallet::<T>::on_chain_storage_version(); // 1r

		if on_chain_version.ge(&4) {
			log::info!(target: LOG_TARGET, "Old Capacity EpochLength migration attempted to run. Please remove");
			return T::DbWeight::get().reads(1);
		}

		log::info!(target: LOG_TARGET, "🔄 Capacity EpochLength update migration started");
		// The migration started with 1r to get the STORAGE_VERSION
		let mut total_weight = T::DbWeight::get().reads_writes(1, 0);

		total_weight += Self::update_epoch_length();

		StorageVersion::new(4).put::<Pallet<T>>(); // 1 w

		total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(0, 1));

		log::info!(target: LOG_TARGET, "🔄 Capacity EpochLength second update migration finished");

		total_weight
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
		use frame_support::storage::generator::StorageValue;
		use sp_std::vec;
		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		if on_chain_version >= 4 {
			return Ok(Vec::new());
		}

		let pallet_prefix = EpochLength::<T>::pallet_prefix();
		let storage_prefix = EpochLength::<T>::storage_prefix();
		assert_eq!(&b"Capacity"[..], pallet_prefix);
		assert_eq!(&b"EpochLength"[..], storage_prefix);
		log::info!(target: LOG_TARGET, "Running pre_upgrade...");

		let current_epoch_length = EpochLength::<T>::get();

		#[cfg(feature = "frequency")]
		assert_eq!(current_epoch_length, 7_200u32.into());
		#[cfg(feature = "frequency-testnet")]
		assert_eq!(current_epoch_length, 100u32.into());

		log::info!(target: LOG_TARGET, "Finish pre_upgrade for with current epoch length {:?} to ", current_epoch_length,);
		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		let on_chain_version = Pallet::<T>::on_chain_storage_version();
		if on_chain_version >= 4 {
			return Ok(());
		}

		assert_eq!(on_chain_version, crate::pallet::STORAGE_VERSION);

		#[cfg(feature = "frequency")]
		{
			let post_upgrade_epoch_length = EpochLength::<T>::get();
			assert_eq!(post_upgrade_epoch_length, 14_400u32.into());
		}

		#[cfg(feature = "frequency-testnet")]
		{
			let post_upgrade_epoch_length = EpochLength::<T>::get();
			assert_eq!(post_upgrade_epoch_length, 200u32.into());
		}

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
