#![cfg_attr(not(feature = "std"), no_std)]

/// Pallet schema library.
/// This pallet is used to define, validate, store and access schemas for message passing
/// between the participants of the network.
pub use pallet::*;
use frame_support::{traits::Get};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use common_primitives::schema::SchemaId;
	use common_primitives::schema::Schema;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);
	
	#[pallet::storage]
	#[pallet::getter(fn get_schemas)]
	pub type SchemaRegistry<T: Config> = StorageMap<_, Blake2_128Concat, SchemaId, Schema, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
	}

	#[pallet::error]
	pub enum Error<T> {
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

	}
	impl<T: Config> Pallet<T> {
		pub fn get_latest_schema_id() -> Result<SchemaId, DispatchError> {
			Ok(0)
		}
	}
}
