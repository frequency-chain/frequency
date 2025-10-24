// Old V1 storage types
pub mod types;

use crate::{Config, Pallet};
use frame_support::{pallet_prelude::ValueQuery, storage_alias};
pub use types::*;

/// Flag to indicate that Paginated migration is complete
#[storage_alias]
pub(crate) type DonePaginated<T: Config> = StorageValue<Pallet<T>, bool, ValueQuery>;

/// Flag to indicate that Itemized migration is complete
#[storage_alias]
pub(crate) type DoneItemized<T: Config> = StorageValue<Pallet<T>, bool, ValueQuery>;
