use crate::{
	migration::v1,
	stateful_child_tree::{MultipartKey, StatefulChildTree},
	types::{ItemHeader, ITEMIZED_STORAGE_PREFIX, PAGINATED_STORAGE_PREFIX, PALLET_STORAGE_PREFIX},
	weights, Config, Event, Pallet,
};
use alloc::vec::Vec;
use common_primitives::{
	msa::{MessageSourceId, MsaLookup},
	schema::PayloadLocation,
};
use core::marker::PhantomData;
use frame_support::{
	migrations::{MigrationId, SteppedMigration, SteppedMigrationError},
	pallet_prelude::{ConstU32, Get, GetStorageVersion, RuntimeDebug, StorageVersion},
	weights::WeightMeter,
	BoundedVec,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_core::storage::ChildInfo;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

const LOG_TARGET: &str = "pallet::stateful-storage::migration::v2";

/// The length of a PaginatedKey (twox_128, twox_128, u16, u16)
pub type PaginatedKeyLength = ConstU32<72>;
/// The length of an ItemizedKey (twox_128, u16)
pub type ItemizedKeyLength = ConstU32<36>;

/// Type to encapsulate a child key of a certain size, or no key.
/// Necessary because we need MaxEncodedLen, which `Vec<u8>` doesn't give us.
/// Cursor struct for tracking migration progress
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug)]
pub struct ChildCursor<N: Get<u32>> {
	/// Current MSA ID data being migrated
	pub id: MessageSourceId,
	/// Last inner key processed; None means start from first key
	pub last_key: BoundedVec<u8, N>,
	/// Cumulative count of migrated pages
	pub cumulative_pages: u64,
}

impl<N: Get<u32>> Default for ChildCursor<N> {
	fn default() -> Self {
		Self { id: 1, last_key: BoundedVec::<u8, N>::default(), cumulative_pages: u64::default() }
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
	let Some(k) = next_key(child, &cur.last_key).unwrap_or(None) else {
		if cur.id % <u64>::from(T::MigrateEmitEvery::get()) == 0 {
			Pallet::<T>::deposit_event(Event::<T>::StatefulPagesMigrated {
				last_trie: (cur.id, PayloadLocation::Paginated),
				total_page_count: cur.cumulative_pages,
			});
		}
		// Finished this child → next id
		cur.last_key = BoundedVec::default();
		cur.id += 1;
		return Ok(false);
	};

	if let Some(old) =
		StatefulChildTree::<T::KeyHasher>::try_read_raw::<v1::PaginatedPage<T>>(child, &k)
			.map_err(|_| SteppedMigrationError::Failed)?
	{
		let (schema_id, _page_index) =
			<v1::PaginatedKey as MultipartKey<T::KeyHasher>>::decode(&k[..])
				.map_err(|_| SteppedMigrationError::Failed)?;
		let page_parts = (Some(schema_id), old);
		let mut new_page: crate::PaginatedPage<T> = page_parts.into();
		new_page.nonce = new_page.nonce.wrapping_add(1);

		StatefulChildTree::<T::KeyHasher>::write_raw(child, &k, new_page);
	}

	cur.last_key = k;
	cur.cumulative_pages += 1;

	if cur.cumulative_pages % <u64>::from(T::MigrateEmitEvery::get()) == 0 {
		Pallet::<T>::deposit_event(Event::<T>::StatefulPagesMigrated {
			last_trie: (cur.id, PayloadLocation::Paginated),
			total_page_count: cur.cumulative_pages,
		});
	}
	Ok(true)
}

/// Migrates a single ItemizedPage
pub fn process_itemized_page<T: Config, N: Get<u32>>(
	child: &ChildInfo,
	cur: &mut ChildCursor<N>,
) -> Result<bool, SteppedMigrationError> {
	let Some(k) = next_key(child, &cur.last_key).unwrap_or(None) else {
		if cur.id % <u64>::from(T::MigrateEmitEvery::get()) == 0 {
			Pallet::<T>::deposit_event(Event::<T>::StatefulPagesMigrated {
				last_trie: (cur.id, PayloadLocation::Paginated),
				total_page_count: cur.cumulative_pages,
			});
		}
		// Finished this child → next id
		cur.last_key = BoundedVec::default();
		cur.id += 1;
		return Ok(false);
	};

	if let Some(old) =
		StatefulChildTree::<T::KeyHasher>::try_read_raw::<v1::ItemizedPage<T>>(child, &k)
			.map_err(|_| SteppedMigrationError::Failed)
			.expect("failed to read raw itemized page")
	{
		let (schema_id,) = <v1::ItemizedKey as MultipartKey<T::KeyHasher>>::decode(&k[..])
			.map_err(|_| SteppedMigrationError::Failed)
			.expect("failed to decode itemized key");

		// Parse old page into old items
		let parsed_page = v1::ItemizedOperations::<T>::try_parse(&old)
			.map_err(|_| SteppedMigrationError::Failed)?;
		let min_expected_size = parsed_page.items.len() *
			(crate::types::ItemHeader::max_encoded_len() - v1::ItemHeader::max_encoded_len()) +
			parsed_page.page_size;

		// Migrate each old item to the new format and add to a new page buffer
		let mut updated_page_buffer = Vec::with_capacity(min_expected_size);
		parsed_page.items.into_iter().for_each(|(_item_index, parsed_item)| {
			let header = ItemHeader::V2 { schema_id, payload_len: parsed_item.header.payload_len };
			let mut encoded_item = header.encode();
			encoded_item.extend_from_slice(&parsed_item.data);
			updated_page_buffer.extend_from_slice(&encoded_item);
		});

		let bounded_page_buffer: BoundedVec<u8, T::MaxItemizedPageSizeBytes> =
			updated_page_buffer.try_into().map_err(|_| SteppedMigrationError::Failed)?;
		let mut new_page: crate::ItemizedPage<T> = bounded_page_buffer.clone().into();
		new_page.schema_id = None;
		new_page.nonce = old.nonce.wrapping_add(1);

		StatefulChildTree::<T::KeyHasher>::write_raw(child, &k, new_page);
	}

	cur.last_key = k;
	cur.cumulative_pages += 1;

	if cur.cumulative_pages % <u64>::from(T::MigrateEmitEvery::get()) == 0 {
		Pallet::<T>::deposit_event(Event::<T>::StatefulPagesMigrated {
			last_trie: (cur.id, PayloadLocation::Itemized),
			total_page_count: cur.cumulative_pages,
		});
	}
	Ok(true)
}

/// The `step` function will be called once per block. It is very important that this function
/// *never* panics and never uses more weight than it got in its meter. The migrations should also
/// try to make maximal progress per step, so that the total time it takes to migrate stays low.
pub struct MigratePaginatedV1ToV2<T: Config, W: weights::WeightInfo>(PhantomData<(T, W)>);
impl<T: Config, W: weights::WeightInfo> SteppedMigration for MigratePaginatedV1ToV2<T, W> {
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
		meter.try_consume(T::DbWeight::get().reads(2)).map_err(|_| {
			SteppedMigrationError::InsufficientWeight { required: T::DbWeight::get().reads(1) }
		})?;
		if StorageVersion::new(2) <= Pallet::<T>::on_chain_storage_version() {
			log::info!(target: LOG_TARGET, "Skipping migrating paginated storage: storage version already set to 2");
			return Ok(None);
		}
		let max_id = <T::MsaInfoProvider>::get_max_msa_id();
		let mut cur = cursor.unwrap_or_else(|| {
			log::info!(target: LOG_TARGET, "Starting migrating paginated storage, max MSA: {max_id}");
			Self::Cursor::default()
		});
		let required = W::paginated_v1_to_v2();

		if meter.remaining().any_lt(required) {
			return Err(SteppedMigrationError::InsufficientWeight { required });
		}

		while cur.id <= max_id {
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

		v1::DonePaginated::<T>::put(true);
		log::info!(target: LOG_TARGET, "Finished migrating paginated storage");
		Ok(None) // done
	}
}

/// The `step` function will be called once per block. It is very important that this function
/// *never* panics and never uses more weight than it got in its meter. The migrations should also
/// try to make maximal progress per step, so that the total time it takes to migrate stays low.
pub struct MigrateItemizedV1ToV2<T: Config, W: weights::WeightInfo>(PhantomData<(T, W)>);
impl<T: Config, W: weights::WeightInfo> SteppedMigration for MigrateItemizedV1ToV2<T, W> {
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
		meter.try_consume(T::DbWeight::get().reads(2)).map_err(|_| {
			SteppedMigrationError::InsufficientWeight { required: T::DbWeight::get().reads(1) }
		})?;
		if StorageVersion::new(2) <= Pallet::<T>::on_chain_storage_version() {
			log::info!(target: LOG_TARGET, "Skipping migrating itemized storage: storage version already set to 2");
			return Ok(None);
		}
		let max_id = <T::MsaInfoProvider>::get_max_msa_id();
		let mut cur = cursor.unwrap_or_else(|| {
			log::info!(target: LOG_TARGET, "Starting migrating itemized storage, max MSA: {max_id}");
			Self::Cursor::default()
		});
		let required = W::itemized_v1_to_v2();

		if meter.remaining().any_lt(required) {
			return Err(SteppedMigrationError::InsufficientWeight { required });
		}

		while cur.id <= max_id {
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

		v1::DoneItemized::<T>::put(true);
		log::info!(target: LOG_TARGET, "Finished migrating itemized storage");
		Ok(None) // done
	}
}

/// Finalize the migration by updating the pallet storage version.
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
		let required = T::DbWeight::get().reads(1).saturating_add(T::DbWeight::get().writes(1));
		if meter.try_consume(required).is_err() {
			return Err(SteppedMigrationError::InsufficientWeight { required });
		}
		if StorageVersion::new(2) <= Pallet::<T>::on_chain_storage_version() {
			log::info!(target: LOG_TARGET, "Skipping finalization of stateful-storage pallet migration: storage version already set to 2 or higher");
			return Ok(None);
		}
		StorageVersion::new(2).put::<Pallet<T>>();

		log::info!(target: LOG_TARGET, "Finalized stateful-storage pallet migration: storage version set to 2");
		Ok(None)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, frame_support::sp_runtime::TryRuntimeError> {
		// pre-upgrade hook is really meant for single-block migrations, as the hook is called for
		// every block. For MBMs, just return empty until the SteppedMigration is complete
		if v1::DonePaginated::<T>::exists() && v1::DoneItemized::<T>::exists() {
			// Return the storage version before the migration
			Ok(Pallet::<T>::on_chain_storage_version().encode())
		} else {
			Ok(Vec::new())
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(prev: Vec<u8>) -> Result<(), frame_support::sp_runtime::TryRuntimeError> {
		// post-upgrade hook is really meant for single-block migrations, as the hook is called
		// after every block. For MBMs, we'll set the pre-upgrade to generate an empty Vec<_>,
		// so here we check for that and only perform our validation if the input is non-empty.
		if !prev.is_empty() {
			// Check the len of prev and post are the same.
			let cur_version = StorageVersion::get::<Pallet<T>>();
			let target_version = StorageVersion::new(2);
			if cur_version < target_version {
				return Err(TryRuntimeError::Other(
					"Migration failed: current storage version is not 2 or higher",
				));
			} else {
				v1::DonePaginated::<T>::kill();
				v1::DoneItemized::<T>::kill();
			}
		}

		Ok(())
	}
}
