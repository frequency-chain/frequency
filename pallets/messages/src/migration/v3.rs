use crate::{migration::v2, pallet::MessagesV3, weights, Config, Message, MessageIndex, Pallet};
#[cfg(feature = "try-runtime")]
use alloc::vec::Vec;
use common_primitives::schema::SchemaId;
use core::marker::PhantomData;
use frame_support::{
	migrations::{MigrationId, SteppedMigration, SteppedMigrationError},
	pallet_prelude::StorageVersion,
	weights::WeightMeter,
};
use frame_system::pallet_prelude::BlockNumberFor;
#[cfg(feature = "try-runtime")]
use parity_scale_codec::{Decode, Encode};

const LOG_TARGET: &str = "pallet::messages::migration::v3";

fn convert_from_old<T: Config>(
	id: SchemaId,
	old_info: v2::Message<T::MessagesMaxPayloadSizeBytes>,
) -> Message<T::MessagesMaxPayloadSizeBytes> {
	Message {
		schema_id: id,
		payload: old_info.payload,
		provider_msa_id: old_info.provider_msa_id,
		msa_id: old_info.msa_id,
	}
}

/// Migrates the items of the [`v2::MessagesV2`] map to [`crate::MessagesV3`]
///
/// The `step` function will be called once per block. It is very important that this function
/// *never* panics and never uses more weight than it got in its meter. The migrations should also
/// try to make maximal progress per step, so that the total time it takes to migrate stays low.
pub struct MigrateV2ToV3<T: Config, W: weights::WeightInfo>(PhantomData<(T, W)>);
impl<T: Config, W: weights::WeightInfo> SteppedMigration for MigrateV2ToV3<T, W> {
	type Cursor = (BlockNumberFor<T>, SchemaId, MessageIndex);
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
		let required = W::v2_to_v3_step();
		// If there is not enough weight for a single step, return an error. This case can be
		// problematic if it is the first migration that ran in this block. But there is nothing
		// that we can do about it here.
		if meter.remaining().any_lt(required) {
			return Err(SteppedMigrationError::InsufficientWeight { required });
		}

		let mut count = 0u32;

		let mut iter = v2::MessagesV2::<T>::drain();
		let mut last_cursor = cursor;

		// We loop here to do as much progress as possible per step.
		loop {
			if meter.try_consume(required).is_err() {
				break;
			}

			// If there's a next item in the iterator, perform the migration.
			if let Some(((block_number, schema_id, index), value)) = iter.next() {
				count += 1;
				// Migrate the inner value to the new structure
				let value = convert_from_old::<T>(schema_id, value);
				// We can just insert here since the old and the new map share the same key-space.
				MessagesV3::<T>::insert((block_number, schema_id, index), value);
				last_cursor = Some((block_number, schema_id, index));
			} else {
				log::info!(target: LOG_TARGET, "Migrated final {} messages", count);
				return Ok(None);
			}
		}

		log::info!(target: LOG_TARGET, "Migrated {} messages", count);
		Ok(last_cursor)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, frame_support::sp_runtime::TryRuntimeError> {
		// Return the state of the storage before the migration.
		Ok((v2::MessagesV2::<T>::iter().count() as u32).encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(prev: Vec<u8>) -> Result<(), frame_support::sp_runtime::TryRuntimeError> {
		// Check the state of the storage after the migration.
		let prev_messages =
			<u32>::decode(&mut &prev[..]).expect("Failed to decode the previous storage state");

		// Check the len of prev and post are the same.
		assert_eq!(
			MessagesV3::<T>::iter().count() as u32,
			prev_messages,
			"Migration failed: the number of items in the storage after the migration is not the same as before"
		);

		Ok(())
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
		let required = W::v2_to_v3_final_step();
		// If there is not enough weight for a single step, return an error. This case can be
		// problematic if it is the first migration that ran in this block. But there is nothing
		// that we can do about it here.
		if meter.try_consume(required).is_err() {
			return Err(SteppedMigrationError::InsufficientWeight { required });
		}
		StorageVersion::new(3).put::<Pallet<T>>();

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
		assert!(
			StorageVersion::get::<Pallet<T>>() > prev_version,
			"Migration failed: current storage version is not greater than the previous storage version"
		);

		Ok(())
	}
}
