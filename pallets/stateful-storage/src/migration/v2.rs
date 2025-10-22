use crate::{
	migration::v1,
	stateful_child_tree::{MultipartKey, StatefulChildTree},
	types::{PAGINATED_STORAGE_PREFIX, PALLET_STORAGE_PREFIX},
	weights, Config, Pallet,
};
use common_primitives::msa::MessageSourceId;
use core::marker::PhantomData;
use frame_support::{
	migrations::{MigrationId, SteppedMigration, SteppedMigrationError},
	pallet_prelude::{ConstU32, Get, RuntimeDebug, StorageVersion},
	weights::WeightMeter,
	BoundedVec,
};
use alloc::vec::Vec;
use pallet_msa::{Config as MsaConfig, CurrentMsaIdentifierMaximum};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_core::storage::ChildInfo;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;
use crate::migration::v1::ItemizedOperations;
use crate::types::{ItemHeader, ItemVersion, ITEMIZED_STORAGE_PREFIX};

const LOG_TARGET: &str = "pallet::stateful-storage::migration::v2";

/// The length of a PaginatedKey (twox_128, twox_128, u16, u16)
pub type PaginatedKeyLength = ConstU32<72>;
/// The length of an ItemizedKey (twox_128, u16)
pub type ItemizedKeyLength = ConstU32<36>;

/// Type to encapsulate a child key of a certain size, or no key.
/// Necessary because we need MaxEncodedLen, which Vec<u8> doesn't give us.
/// Cursor struct for tracking migration progress
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug)]
pub struct ChildCursor<N: Get<u32>> {
	/// Current MSA ID data being migrated
	pub id: MessageSourceId,
	/// Last inner key processed; None means start from first key
	pub last_key: BoundedVec<u8, N>,
}

impl<N: Get<u32>> Default for ChildCursor<N> {
	fn default() -> Self {
		Self { id: 1, last_key: BoundedVec::<u8, N>::default() }
	}
}

fn next_key<N: Get<u32>>(
	child: &ChildInfo,
	after: &BoundedVec<u8, N>,
) -> Result<Option<BoundedVec<u8, N>>, SteppedMigrationError> {
	match sp_io::default_child_storage::next_key(child.storage_key(), after.as_slice()) {
		Some(k) => Ok(Some(k.try_into().map_err(|_| SteppedMigrationError::Failed)?)),
		None => Ok(None),
	}
}

/// Migrates a single PaginatedPage
pub fn process_paginated_page<T: Config, N: Get<u32>>(
	child: &ChildInfo,
	cur: &mut ChildCursor<N>,
) -> Result<bool, SteppedMigrationError> {
	let Some(k) = next_key(&child, &cur.last_key).unwrap_or(None) else {
		// Finished this child → next id
		cur.last_key = BoundedVec::default();
		cur.id += 1;
		return Ok(false);
	};

	if let Some(old) =
		StatefulChildTree::<T::KeyHasher>::try_read_raw::<v1::PaginatedPage<T>>(&child, &k)
			.map_err(|_| SteppedMigrationError::Failed)?
	{
		let (schema_id, _page_index) =
			<v1::PaginatedKey as MultipartKey<T::KeyHasher>>::decode(&mut &k[..])
				.map_err(|_| SteppedMigrationError::Failed)?;
		let page_parts = (Some(schema_id), old);
		let mut new_page: crate::PaginatedPage<T> = page_parts.into();
		new_page.nonce = new_page.nonce.wrapping_add(1);

		StatefulChildTree::<T::KeyHasher>::write_raw(&child, &k, new_page);
	}

	cur.last_key = k;
	Ok(true)
}

/// Migrates a single ItemizedPage
pub fn process_itemized_page<T: Config, N: Get<u32>>(child: &ChildInfo, cur: &mut ChildCursor<N>) -> Result<bool, SteppedMigrationError> {
	let Some(k) = next_key(&child, &cur.last_key).unwrap_or(None) else {
		// Finished this child → next id
		cur.last_key = BoundedVec::default();
		cur.id += 1;
		return Ok(false);
	};

	if let Some(old) =
		StatefulChildTree::<T::KeyHasher>::try_read_raw::<v1::ItemizedPage<T>>(&child, &k)
			.map_err(|_| SteppedMigrationError::Failed).expect("failed to read raw itemized page")
	{
		let (schema_id,) =
			<v1::ItemizedKey as MultipartKey<T::KeyHasher>>::decode(&mut &k[..])
				.map_err(|_| SteppedMigrationError::Failed).expect("failed to decode itemized key");

		// Parse old page into old items
		let parsed_page = ItemizedOperations::<T>::try_parse(&old).map_err(|_| SteppedMigrationError::Failed)?;
		let min_expected_size = parsed_page.items.len() * (crate::types::ItemHeader::max_encoded_len() - v1::ItemHeader::max_encoded_len()) + parsed_page.page_size;

		// Migrate each old item to the new format and add to a new page buffer
		let mut updated_page_buffer = Vec::with_capacity(min_expected_size);
		parsed_page.items.into_iter().for_each(|(_item_index, parsed_item)| {
			let header = ItemHeader {
				item_version: ItemVersion::V2,
				schema_id,
				payload_len: parsed_item.header.payload_len,
			};
			let mut encoded_item = header.encode();
			encoded_item.extend_from_slice(&parsed_item.data);
			updated_page_buffer.extend_from_slice(&encoded_item);
		});

		let bounded_page_buffer: BoundedVec<u8, T::MaxItemizedPageSizeBytes> = updated_page_buffer.try_into().map_err(|_| {
			SteppedMigrationError::Failed
		})?;
		let mut new_page: crate::ItemizedPage<T> = bounded_page_buffer.clone().into();
		new_page.page_version = crate::types::PageVersion::V2;
		new_page.schema_id = None;
		new_page.nonce = old.nonce.wrapping_add(1);

		StatefulChildTree::<T::KeyHasher>::write_raw(&child, &k, new_page);
	}

	cur.last_key = k;
	Ok(true)
}

/// The `step` function will be called once per block. It is very important that this function
/// *never* panics and never uses more weight than it got in its meter. The migrations should also
/// try to make maximal progress per step, so that the total time it takes to migrate stays low.
pub struct MigratePaginatedV1ToV2<T: Config + MsaConfig, W: weights::WeightInfo>(
	PhantomData<(T, W)>,
);
impl<T: Config + MsaConfig, W: weights::WeightInfo> SteppedMigration
	for MigratePaginatedV1ToV2<T, W>
{
	type Cursor = ChildCursor<PaginatedKeyLength>;
	// Without the explicit length here the construction of the ID would not be infallible.
	type Identifier = MigrationId<50>;

	/// The identifier of this migration. Which should be globally unique.
	fn id() -> Self::Identifier {
		MigrationId {
			pallet_id: *b"pallet::stateful-storage::migration::paginated::v2",
			version_from: 1,
			version_to: 2,
		}
	}

	/// The actual logic of the migration.
	///
	/// This function is called repeatedly until it returns `Ok(None)`, indicating that the
	/// migration is complete. Ideally, the migration should be designed in such a way that each
	/// step consumes as much weight as possible.
	fn step(
		cursor: Option<Self::Cursor>,
		meter: &mut WeightMeter,
	) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
		meter.try_consume(T::DbWeight::get().reads(1)).map_err(|_| {
			SteppedMigrationError::InsufficientWeight { required: T::DbWeight::get().reads(1) }
		})?;
		let max_id = CurrentMsaIdentifierMaximum::<T>::get();
		let mut cur = cursor.unwrap_or_default();
		let required = W::paginated_v1_to_v2();

		if meter.remaining().any_lt(required) {
			return Err(SteppedMigrationError::InsufficientWeight {
				required,
			});
		}

		while cur.id < max_id {
			let child = StatefulChildTree::<T::KeyHasher>::get_child_tree_for_storage(
				cur.id,
				PALLET_STORAGE_PREFIX,
				PAGINATED_STORAGE_PREFIX,
			);

			'inner: loop {
				if meter.try_consume(required).is_err() {
					return Ok(Some(cur));
				}
				if !process_paginated_page::<T, PaginatedKeyLength>(&child, &mut cur)? {
					break 'inner;
				}
			}
		}

		Ok(None) // done
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, frame_support::sp_runtime::TryRuntimeError> {
		// Return the state of the storage before the migration.
		let s = crate::Pallet::<T>::on_chain_storage_version();
		if let StorageVersion(version) = s {
			if version != 1 {
				return Err(frame_support::sp_runtime::TryRuntimeError::Other(
					"Migration failed: the storage version is not 1",
				));
			}
		}
		Ok(s.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(prev: Vec<u8>) -> Result<(), frame_support::sp_runtime::TryRuntimeError> {
		// Check the storage version has not been bumped yet
		let old_version =
			StorageVersion::decode(&mut &prev[..]).map_err(|_| TryRuntimeError::Corruption)?;
		if let StorageVersion(version) =
			crate::Pallet::<T>::on_chain_storage_version() != old_version
		{
			return Err(frame_support::sp_runtime::TryRuntimeError::Other(
				"Migration failed: the storage version has been prematurely bumped",
			));
		}

		Ok(())
	}
}

/// The `step` function will be called once per block. It is very important that this function
/// *never* panics and never uses more weight than it got in its meter. The migrations should also
/// try to make maximal progress per step, so that the total time it takes to migrate stays low.
pub struct MigrateItemizedV1ToV2<T: Config + MsaConfig, W: weights::WeightInfo>(
	PhantomData<(T, W)>,
);
impl<T: Config + MsaConfig, W: weights::WeightInfo> SteppedMigration
for MigrateItemizedV1ToV2<T, W>
{
	type Cursor = ChildCursor<ItemizedKeyLength>;
	// Without the explicit length here the construction of the ID would not be infallible.
	type Identifier = MigrationId<49>;

	/// The identifier of this migration. Which should be globally unique.
	fn id() -> Self::Identifier {
		MigrationId {
			pallet_id: *b"pallet::stateful-storage::migration::itemized::v2",
			version_from: 1,
			version_to: 2,
		}
	}

	/// The actual logic of the migration.
	///
	/// This function is called repeatedly until it returns `Ok(None)`, indicating that the
	/// migration is complete. Ideally, the migration should be designed in such a way that each
	/// step consumes as much weight as possible.
	fn step(
		cursor: Option<Self::Cursor>,
		meter: &mut WeightMeter,
	) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
		meter.try_consume(T::DbWeight::get().reads(1)).map_err(|_| {
			SteppedMigrationError::InsufficientWeight { required: T::DbWeight::get().reads(1) }
		})?;
		let max_id = CurrentMsaIdentifierMaximum::<T>::get();
		let mut cur = cursor.unwrap_or_default();
		let required = W::itemized_v1_to_v2();

		if meter.remaining().any_lt(required) {
			return Err(SteppedMigrationError::InsufficientWeight {
				required,
			});
		}

		while cur.id < max_id {
			let child = StatefulChildTree::<T::KeyHasher>::get_child_tree_for_storage(
				cur.id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
			);

			'inner: loop {
				if meter.try_consume(required).is_err() {
					return Ok(Some(cur));
				}
				if !process_itemized_page::<T, ItemizedKeyLength>(&child, &mut cur)? {
					break 'inner;
				}
			}
		}

		Ok(None) // done
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, frame_support::sp_runtime::TryRuntimeError> {
		// Return the state of the storage before the migration.
		let s = crate::Pallet::<T>::on_chain_storage_version();
		if let StorageVersion(version) = s {
			if version != 1 {
				return Err(frame_support::sp_runtime::TryRuntimeError::Other(
					"Migration failed: the storage version is not 1",
				));
			}
		}
		Ok(s.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(prev: Vec<u8>) -> Result<(), frame_support::sp_runtime::TryRuntimeError> {
		// Check the storage version has not been bumped yet
		let old_version =
			StorageVersion::decode(&mut &prev[..]).map_err(|_| TryRuntimeError::Corruption)?;
		if let StorageVersion(version) =
			crate::Pallet::<T>::on_chain_storage_version() != old_version
		{
			return Err(frame_support::sp_runtime::TryRuntimeError::Other(
				"Migration failed: the storage version has been prematurely bumped",
			));
		}

		Ok(())
	}
}

/// Finalize the migration of [`v2::MessagesV2`] map to [`crate::MessagesV3`]
/// by updating the pallet storage version.
pub struct FinalizeV2Migration<T: Config, W: weights::WeightInfo>(PhantomData<(T, W)>);
impl<T: Config, W: weights::WeightInfo> SteppedMigration for FinalizeV2Migration<T, W> {
	type Cursor = ();
	// Without the explicit length here the construction of the ID would not be infallible.
	type Identifier = MigrationId<48>;

	/// The identifier of this migration. Which should be globally unique.
	fn id() -> Self::Identifier {
		MigrationId {
			pallet_id: *b"pallet::stateful-storage::migration::v3-finalize",
			version_from: 1,
			version_to: 2,
		}
	}

	/// Final migration step
	fn step(
		_cursor: Option<Self::Cursor>,
		meter: &mut WeightMeter,
	) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
		// If there is not enough weight for a single step, return an error. This case can be
		// problematic if it is the first migration that ran in this block. But there is nothing
		// that we can do about it here.
		let required = T::DbWeight::get().writes(1);
		if meter.try_consume(required).is_err() {
			return Err(SteppedMigrationError::InsufficientWeight { required });
		}
		StorageVersion::new(2).put::<Pallet<T>>();

		log::info!(target: LOG_TARGET, "Finalized messages pallet migration: storage version set to 3");
		Ok(None)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, frame_support::sp_runtime::TryRuntimeError> {
		// Return the storage version before the migration
		Ok(StorageVersion::get::<Pallet<T>>().encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(prev: Vec<u8>) -> Result<(), frame_support::sp_runtime::TryRuntimeError> {
		// Check the state of the storage after the migration.
		let prev_version = <StorageVersion>::decode(&mut &prev[..])
			.expect("Failed to decode the previous storage state");

		// Check the len of prev and post are the same.
		if StorageVersion::get::<Pallet<T>>() <= prev_version {
			return TryRuntimeError::Other("Migration failed: current storage version is not greater than the previous storage version");
		}

		Ok(())
	}
}
