//! Migrations for the MSA Pallet

use super::*;
use frame_support::storage_alias;

/// Data Structures that were removed
pub mod v0 {
	use super::*;

	/// Replaced by `PayloadSignatureRegistryRingPointer`
	#[storage_alias]
	pub(super) type PayloadSignatureBucketCount<T: Config> = StorageMap<
		Pallet<T>,
		Twox64Concat,
		u64, // bucket number
		u32, // number of signatures
		ValueQuery,
	>;

	/// Replaced with `PayloadSignatureRegistryRing`
	#[storage_alias]
	pub(super) type PayloadSignatureRegistry<T: Config> = StorageDoubleMap<
		Pallet<T>,      // prefix
		Twox64Concat,   // hasher for key1
		u64,            // Bucket number. Stored as BlockNumber because I'm done arguing with rust about it.
		Twox64Concat,   // hasher for key2
		MultiSignature, // An externally-created Signature for an external payload, provided by an extrinsic
		u64,            // An actual flipping block number.
	>;
}

/// MSA Pallet Migration Triggers
pub fn migrate<T: Config>() -> Weight {
	let version = StorageVersion::get::<Pallet<T>>();
	let mut weight: Weight = Weight::zero();

	if version < 1 {
		weight = weight.saturating_add(v1::migrate::<T>());
		// Updated version inside so we can drain the prefix if needed
	}

	weight
}

/// Migrating to remove old storage
mod v1 {
	use super::*;
	use frame_support::storage::generator::{StorageDoubleMap, StorageMap};

	/// Remove PayloadSignatureBucketCount and PayloadSignatureRegistry
	pub fn migrate<T: Config>() -> Weight {
		let registry_prefix = v0::PayloadSignatureRegistry::<T>::prefix_hash();
		let clear_reg =
			frame_support::storage::unhashed::clear_prefix(&registry_prefix, Some(100), None);

		if clear_reg.maybe_cursor.is_none() {
			// We know this one will only have 2
			let count_prefix = v0::PayloadSignatureBucketCount::<T>::prefix_hash();
			let clear_cnt =
				frame_support::storage::unhashed::clear_prefix(&count_prefix, Some(10), None);

			// Done. Migrate to v1
			StorageVersion::new(1).put::<Pallet<T>>();

			log::info!(target: "pallet_msa::migrations", "🟢        pallet_msa: Successful migration to StorageVersion(1)");

			// Weight
			T::DbWeight::get()
				.reads((clear_reg.unique + clear_cnt.unique).into())
				.saturating_add(
					T::DbWeight::get().writes((clear_reg.loops + clear_cnt.loops + 1).into()),
				)
		} else {
			log::info!(target: "pallet_msa::migrations", "⚠️        pallet_msa: Partial migration to StorageVersion(1)");
			T::DbWeight::get()
				.reads(clear_reg.unique.into())
				.saturating_add(T::DbWeight::get().writes(clear_reg.loops.into()))
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{Test as T, *};
	use sp_keyring::Sr25519Keyring::Alice;

	#[test]
	fn no_migrations_means_zero_weight() {
		new_test_ext().execute_with(|| {
			StorageVersion::new(100).put::<Pallet<T>>();
			let weight = migrate::<T>();
			assert_eq!(weight, Weight::zero());
		});
	}

	#[test]
	fn v1_can_migrate() {
		new_test_ext().execute_with(|| {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 0);
			v1::migrate::<T>();
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 1);
		});
	}

	#[test]
	fn v1_can_migrate_with_data() {
		new_test_ext().execute_with(|| {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 0);

			// Setup
			v0::PayloadSignatureBucketCount::<T>::set(0, 100);
			v0::PayloadSignatureBucketCount::<T>::set(1, 2);

			let iter = v0::PayloadSignatureBucketCount::<T>::iter();
			assert_eq!(iter.count(), 2);

			v0::PayloadSignatureRegistry::<T>::set(
				0,
				MultiSignature::Sr25519(Alice.sign(b"foo")),
				Some(12345),
			);

			let iter = v0::PayloadSignatureRegistry::<T>::iter();
			assert_eq!(iter.count(), 1);

			// Migrate and Check

			v1::migrate::<T>();
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 1);
			let iter = v0::PayloadSignatureBucketCount::<T>::iter();
			assert_eq!(iter.count(), 0);
		});
	}

	#[test]
	fn v1_can_migrate_with_lots_of_data() {
		new_test_ext().execute_with(|| {
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 0);

			// Setup
			for i in 0..102 {
				let msg = Alice.sign(format!("foo{}", i).as_bytes());
				v0::PayloadSignatureRegistry::<T>::set(0, MultiSignature::Sr25519(msg), Some(i));
			}

			let iter = v0::PayloadSignatureRegistry::<T>::iter();
			assert_eq!(iter.count(), 102);

			// Migrate and Check

			v1::migrate::<T>();
			assert_eq!(StorageVersion::get::<Pallet<T>>(), 1);
			let iter = v0::PayloadSignatureRegistry::<T>::iter();
			assert_eq!(iter.count(), 0);
		});
	}
}
