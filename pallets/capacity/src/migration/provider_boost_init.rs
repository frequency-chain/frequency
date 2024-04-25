use crate::{Config, CurrentEraInfo, RewardEraInfo, RewardPoolInfo, StakingRewardPool};
use frame_support::{
	pallet_prelude::Weight,
	traits::{Get, OnRuntimeUpgrade},
};

#[cfg(feature = "try-runtime")]
use sp_std::vec::Vec;

/// Initialization during runtime upgrade for Provider Boost storage
pub struct ProviderBoostInit<T: Config>(sp_std::marker::PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for ProviderBoostInit<T> {
	fn on_runtime_upgrade() -> Weight {
		let current_era_info = CurrentEraInfo::<T>::get();
		if current_era_info.eq(&RewardEraInfo::default()) {
			// 1r
			let current_block = frame_system::Pallet::<T>::block_number(); // 1r
			let era_index: T::RewardEra = 0u32.into();
			CurrentEraInfo::<T>::set(RewardEraInfo { era_index, started_at: current_block }); // 1w
			StakingRewardPool::<T>::insert(era_index, RewardPoolInfo::default()); // 1w
			T::DbWeight::get().reads_writes(2, 2)
		} else {
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, TryRuntimeError> {
		if CurrentEraInfo::<T>::exists() {
			log::info!("CurrentEraInfo exists; initialization should be skipped.");
		} else {
			log::info!("CurrentEraInfo not found. Initialization should proceed.");
		}
		if StakingRewardPool::<T>::iter().count() == 0usize {
			log::info!("StakingRewardPool will be updated with Era 0");
		} else {
			log::info!("StakingRewardPool has already been initialized.")
		}
		Ok(Vec::default())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: Vec<u8>) -> Result<(), TryRuntimeError> {
		assert!(CurrentEraInfo::<T>::exists());
		let current_block = frame_system::Pallet::<T>::block_number();
		let info = CurrentEraInfo::<T>::get();
		assert_eq!(info.started_at, current_block);
		log::info!("CurrentEraInfo.started_at is set to {:?}.", info.started_at);
		assert_eq!(StakingRewardPool::<T>::iter().count(), 1);
		Ok(())
	}
}
