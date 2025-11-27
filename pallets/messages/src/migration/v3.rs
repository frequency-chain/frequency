//! Migration to add schema_id to `messages` pallet storage.
//! NOTE: Pallet activity is not suspended during this migration.
//! All writes go to the new storage with new keys, so there is no danger
//! of overlap with entries being migrated.
use crate::{
	migration::{
		v2,
		v2::{DoneV3Migration, K1Type, K2Type, K3Type, MessagesV2},
	},
	pallet::MessagesV3,
	weights, Config, Event, Message, MessageIndex, Pallet,
};
#[cfg(feature = "try-runtime")]
use alloc::vec::Vec;
use common_primitives::schema::SchemaId;
use core::marker::PhantomData;
use frame_support::{
	migrations::{MigrationId, SteppedMigration, SteppedMigrationError},
	pallet_prelude::{StorageVersion, Zero},
	storage::PrefixIterator,
	traits::{Get, GetStorageVersion},
	weights::WeightMeter,
};
use frame_system::pallet_prelude::BlockNumberFor;
#[cfg(feature = "try-runtime")]
use parity_scale_codec::Encode;

const LOG_TARGET: &str = "pallet::messages::migration::v3";

// Migration cursor: (storage index, cumulative records processed)
#[allow(type_alias_bounds)]
pub(crate) type MessagesCursor<T: Config> = (BlockNumberFor<T>, SchemaId, MessageIndex, u64);

pub(crate) fn migrate_single_record<T: Config>(
	iter: &mut PrefixIterator<(
		(K1Type<T>, K2Type, K3Type),
		v2::Message<T::MessagesMaxPayloadSizeBytes>,
	)>,
	cursor: &mut MessagesCursor<T>,
) -> bool {
	// If there's a next item in the iterator, perform the migration.
	let messages_remain = if let Some(((block_number, schema_id, index), value)) = iter.next() {
		// Migrate the inner value to the new structure
		let new_value = Message {
			schema_id, // schema_id is added to the struct
			payload: value.payload,
			provider_msa_id: value.provider_msa_id,
			msa_id: value.msa_id,
		};
		// We can just insert here since the old and the new map share the same key-space.
		MessagesV3::<T>::insert((block_number, schema_id, index), new_value);

		cursor.3 += 1;
		true
	} else {
		false
	};

	if !messages_remain || cursor.3 % <u64>::from(T::MigrateEmitEvery::get()) == 0 {
		Pallet::<T>::deposit_event(Event::<T>::MessagesMigrated {
			from_version: 2,
			to_version: 3,
			cumulative_total_migrated: cursor.3,
		});
	}

	messages_remain
}

/// Migrates the items of the [`v2::MessagesV2`] map to [`crate::MessagesV3`]
///
/// The `step` function will be called once per block. It is very important that this function
/// *never* panics and never uses more weight than it got in its meter. The migrations should also
/// try to make maximal progress per step, so that the total time it takes to migrate stays low.
pub struct MigrateV2ToV3<T: Config, W: weights::WeightInfo>(PhantomData<(T, W)>);
impl<T: Config, W: weights::WeightInfo> SteppedMigration for MigrateV2ToV3<T, W> {
	// Cursor type for migration. Not used for migration logic itself, other than
	// for emitting/tracking migration progress/status.
	// ((tuple of last message processed), cumulative message count)
	type Cursor = MessagesCursor<T>;
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
		#[cfg(feature = "try-runtime")]
		// Extra check for try-runtime, since pallet-migrations is not seeded with
		// the completed migrations.
		if StorageVersion::new(3) <= Pallet::<T>::on_chain_storage_version() {
			return Ok(None);
		}
		let required = W::v2_to_v3_step();
		// If there is not enough weight for a single step, return an error. This case can be
		// problematic if it is the first migration that ran in this block. But there is nothing
		// that we can do about it here.
		if meter.remaining().any_lt(required) {
			return Err(SteppedMigrationError::InsufficientWeight { required });
		}

		let mut step_count = 0u32;
		let mut iter = MessagesV2::<T>::drain();
		let mut last_cursor = cursor.unwrap_or((BlockNumberFor::<T>::zero(), 0, 0, 0));
		let mut messages_remain = true;

		// We loop here to do as much progress as possible per step.
		while meter.try_consume(required).is_ok() {
			// If there's a next item in the iterator, perform the migration.
			messages_remain = migrate_single_record::<T>(&mut iter, &mut last_cursor);
			if !messages_remain {
				break;
			} else {
				step_count += 1;
			}
		}

		if step_count > 0 {
			log::info!(target: LOG_TARGET, "Migrated {}{} messages", step_count, if messages_remain { "" } else { " final" });
		}

		if !messages_remain {
			meter.try_consume(T::DbWeight::get().writes(1)).map_err(|_| {
				SteppedMigrationError::InsufficientWeight { required: T::DbWeight::get().writes(1) }
			})?;
			v2::DoneV3Migration::<T>::put(true);
		}
		Ok(messages_remain.then_some(last_cursor))
	}
}

/// Finalize the migration of [`v2::MessagesV2`] map to [`crate::MessagesV3`]
/// by updating the pallet storage version.
pub struct FinalizeV3Migration<T: Config, W: weights::WeightInfo>(PhantomData<(T, W)>);
impl<T: Config, W: weights::WeightInfo> SteppedMigration for FinalizeV3Migration<T, W> {
	type Cursor = (BlockNumberFor<T>, SchemaId, MessageIndex);
	// Without the explicit length here the construction of the ID would not be infallible.
	type Identifier = MigrationId<40>;

	/// The identifier of this migration. Which should be globally unique.
	fn id() -> Self::Identifier {
		MigrationId {
			pallet_id: *b"pallet::messages::migration::v3-finalize",
			version_from: 2,
			version_to: 3,
		}
	}

	/// Final migration step
	fn step(
		_cursor: Option<Self::Cursor>,
		meter: &mut WeightMeter,
	) -> Result<Option<Self::Cursor>, SteppedMigrationError> {
		#[cfg(feature = "try-runtime")]
		// Extra check for try-runtime, since pallet-migrations is not seeded with
		// the completed migrations.
		if Pallet::<T>::on_chain_storage_version() >= StorageVersion::new(3) {
			return Ok(None);
		}
		let required = W::v2_to_v3_final_step();
		// If there is not enough weight for a single step, return an error. This case can be
		// problematic if it is the first migration that ran in this block. But there is nothing
		// that we can do about it here.
		if meter.try_consume(required).is_err() {
			return Err(SteppedMigrationError::InsufficientWeight { required });
		}

		// Make sure this migration is idempotent--don't set storage version if already at or higher then 3
		if Pallet::<T>::on_chain_storage_version() >= StorageVersion::new(3) {
			log::info!(target: LOG_TARGET, "Messages pallet migration finalization: storage version already set to 3");
		} else {
			StorageVersion::new(3).put::<Pallet<T>>();
			log::info!(target: LOG_TARGET, "Finalized messages pallet migration: storage version set to 3");
		}

		// Clean up ephemeral migration storage
		DoneV3Migration::<T>::kill();
		Ok(None)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, frame_support::sp_runtime::TryRuntimeError> {
		// pre-upgrade hook is really meant for single-block migrations, as the hook is called for
		// every block. For MBMs, just return empty until the SteppedMigration is complete
		if v2::DoneV3Migration::<T>::exists() {
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
			// Check the state of the storage after the migration.
			// Check the len of prev and post are the same.
			let target_version = StorageVersion::new(3);
			let current_version = StorageVersion::get::<Pallet<T>>();
			if current_version < target_version {
				return Err(frame_support::sp_runtime::TryRuntimeError::Other(
					"Migration failed: current storage version is not 3 or higher",
				))
			}

			v2::DoneV3Migration::<T>::kill();
		}

		Ok(())
	}
}
