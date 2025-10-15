use crate::{
	migration::v1,
	stateful_child_tree::StatefulChildTree,
	types::{PAGINATED_STORAGE_PREFIX, PALLET_STORAGE_PREFIX},
	weights, Config, Pallet,
};
use common_primitives::msa::MessageSourceId;
use core::marker::PhantomData;
use frame_support::{
	migrations::{MigrationId, SteppedMigration, SteppedMigrationError},
	pallet_prelude::{Get, RuntimeDebug, StorageVersion},
	weights::WeightMeter,
};
use pallet_msa::{Config as MsaConfig, CurrentMsaIdentifierMaximum};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_core::storage::ChildInfo;
#[cfg(feature = "try-runtime")]
use sp_runtime::TryRuntimeError;

const LOG_TARGET: &str = "pallet::stateful-storage::migration::v2";

const PAGINATED_KEY_LENGTH: usize = 72;
const ITEMIZED_KEY_LENGTH: usize = 36;

/// Cursor struct for tracking migration progress
#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug)]
pub struct ChildCursor<const N: usize> {
	/// Current MSA ID data being migrated
	pub id: MessageSourceId,
	/// Last inner key processed; None means start from first key
	pub last_key: Option<[u8; N]>,
}

impl<const N: usize> Default for ChildCursor<N> {
	fn default() -> Self {
		Self { id: 1, last_key: None }
	}
}

fn next_key<const N: usize>(
	child: &ChildInfo,
	after: &Option<[u8; N]>,
) -> Result<Option<[u8; N]>, SteppedMigrationError> {
	let start: &[u8] = match after {
		Some(k) => k,
		None => &[],
	};
	match sp_io::default_child_storage::next_key(child.storage_key(), start) {
		Some(k) => Ok(Some(<[u8; N]>::try_from(k).map_err(|_| SteppedMigrationError::Failed)?)),
		None => Ok(None),
	}
}

fn process_single_page<T: Config, W: weights::WeightInfo, const N: usize>(
	child: &ChildInfo,
	cur: &mut ChildCursor<N>,
) -> Result<bool, SteppedMigrationError> {
	let Some(k) = next_key(&child, &cur.last_key)? else {
		// Finished this child → next id
		cur.last_key = None;
		cur.id += 1;
		return Ok(false);
	};

	if let Some(old) =
		StatefulChildTree::<T::KeyHasher>::try_read_raw::<v1::PaginatedPage<T>>(&child, &k)
			.map_err(|_| SteppedMigrationError::Failed)?
	{
		let (schema_id, _page_index) =
			v1::PaginatedKey::decode(&mut &k[..]).map_err(|_| SteppedMigrationError::Failed)?;
		let new_page: crate::PaginatedPage<T> = ((Some(schema_id), old)).into();
		StatefulChildTree::<T::KeyHasher>::write_raw(&child, &k, new_page);
	}

	cur.last_key = Some(k);
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
	type Cursor = ChildCursor<{ PAGINATED_KEY_LENGTH }>;
	// Without the explicit length here the construction of the ID would not be infallible.
	type Identifier = MigrationId<31>;

	/// The identifier of this migration. Which should be globally unique.
	fn id() -> Self::Identifier {
		MigrationId {
			pallet_id: *b"pallet::messages::migration::v3",
			version_from: 2,
			version_to: 3,
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
		let single_loop_weight = W::paginated_v1_to_v2();

		if meter.remaining().any_lt(single_loop_weight) {
			return Err(SteppedMigrationError::InsufficientWeight {
				required: W::paginated_v1_to_v2(),
			});
		}

		while cur.id < max_id {
			let child = StatefulChildTree::<T::KeyHasher>::get_child_tree_for_storage(
				cur.id,
				PALLET_STORAGE_PREFIX,
				PAGINATED_STORAGE_PREFIX,
			);

			'inner: loop {
				if meter.try_consume(single_loop_weight).is_err() {
					return Ok(Some(cur));
				}
				if !process_single_page::<T, W, { PAGINATED_KEY_LENGTH }>(&child, &mut cur)? {
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
		let required = W::v1_to_v2_final_step();
		// If there is not enough weight for a single step, return an error. This case can be
		// problematic if it is the first migration that ran in this block. But there is nothing
		// that we can do about it here.
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
