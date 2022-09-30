/// Trivial migration which moves MessageSourceIdOf -> MessageSourceMigratedIdOf
///
/// Note: The depositor is not optional since he can never change.
use crate::*;

use frame_support::{migration::StorageKeyIterator, traits::OnRuntimeUpgrade, weights::Weight};
use log;

/// Struct on which to implement OnRuntimeUpgrade trait
pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);

/// Version 1
pub mod v1 {
	use super::*;
	impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
		/// Perform a module upgrade.
		///
		/// # Warning
		///
		/// This function will be called before we initialized any runtime state, aka `on_initialize`
		/// wasn't called yet. So, information like the block number and any other
		/// block local data are not accessible.
		///
		/// Return the non-negotiable weight consumed for runtime upgrade.
		fn on_runtime_upgrade() -> Weight {
			let current = Pallet::<T>::current_storage_version();
			let onchain = Pallet::<T>::on_chain_storage_version();
			log::info!(
				"Running MSA migration with current storage version {:?} / onchain {:?}",
				current,
				onchain
			);

			// Execute the migration when upgrading MSA storage version from version 0 to version 1
			if current == 1 && onchain == 0 {
				// TODO: Iterate through AccountId in MessageSourceIdOf and
				// copy the key/value pair to MessageSourceMigratedIdOf

				// put the current storage version into storage
				current.put::<Pallet<T>>();
				log::info!("Migrated MessageSourceIdOf storage to MessageSourceMigratedIdOf");
				T::DbWeight::get().reads(1)
			} else {
				log::info!("MigrateToV1 has already been completed and can be removed.");
				T::DbWeight::get().reads(1)
			}
		}

		/// Execute some pre-checks prior to a runtime upgrade.
		///
		/// This hook is never meant to be executed on-chain but is meant to be used by testing tools.
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			Ok(())
		}

		/// Execute some post-checks after a runtime upgrade.
		///
		/// This hook is never meant to be executed on-chain but is meant to be used by testing tools.
		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			Ok(())
		}
	}
}
