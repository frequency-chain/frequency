//! Migrations for the MSA Pallet

use super::*;
use frame_support::storage_alias;

/// Data Structures that were removed
pub mod v1 {
	use super::*;

	/// Replaced by [``]
	#[storage_alias]
	pub(super) type PayloadSignatureBucketCount<T: Config> = StorageMap<
		Pallet<T>,
		Twox64Concat,
		u64, // bucket number
		u32, // number of signatures
		ValueQuery,
	>;

	/// Replaced with [``]
	#[storage_alias]
	pub(super) type PayloadSignatureRegistry<T: Config> = StorageDoubleMap<
		Pallet<T>,      // prefix
		Twox64Concat,   // hasher for key1
		u64,            // Bucket number. Stored as BlockNumber because I'm done arguing with rust about it.
		Twox64Concat,   // hasher for key2
		MultiSignature, // An externally-created Signature for an external payload, provided by an extrinsic
		u64,            // An actual flipping block number.
		                // OptionQuery,    // The type for the query
		                // GetDefault,     // OnEmpty return type, defaults to None
		                // T::MaxSignaturesStored, // Maximum total signatures to store
	>;
}

/// MSA Pallet Migration Triggers
pub fn migrate<T: Config>() -> Weight {
	let version = StorageVersion::get::<Pallet<T>>();
	let mut weight: Weight = Weight::zero();

	if version < 2 {
		weight = weight.saturating_add(v2::migrate::<T>());
		// Updated version inside so we can drain the prefix if needed
	}

	weight
}

/// Migrating to remove old storage
mod v2 {
	use super::*;
	use frame_support::storage::generator::{StorageDoubleMap, StorageMap};

	/// Remove PayloadSignatureBucketCount and PayloadSignatureRegistry
	pub fn migrate<T: Config>() -> Weight {
		let registry_prefix = v1::PayloadSignatureRegistry::<T>::prefix_hash();
		let clear_reg =
			frame_support::storage::unhashed::clear_prefix(&registry_prefix, Some(100), None);

		if clear_reg.maybe_cursor.is_none() {
			// We know this one will only have 2
			let count_prefix = v1::PayloadSignatureBucketCount::<T>::prefix_hash();
			let clear_cnt =
				frame_support::storage::unhashed::clear_prefix(&count_prefix, Some(10), None);

			// Done. Migrate to v2
			StorageVersion::new(2).put::<Pallet<T>>();

			// Weight
			T::DbWeight::get()
				.reads((clear_reg.unique + clear_cnt.unique).into())
				.saturating_add(
					T::DbWeight::get().writes((clear_reg.loops + clear_cnt.loops + 1).into()),
				)
		} else {
			T::DbWeight::get()
				.reads(clear_reg.unique.into())
				.saturating_add(T::DbWeight::get().writes(clear_reg.loops.into()))
		}
	}
}
