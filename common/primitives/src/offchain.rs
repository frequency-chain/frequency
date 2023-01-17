use codec::{Decode, Encode};
use frame_support::log::error as log_err;
use sp_io::offchain;
use sp_runtime::offchain::{
	storage::{StorageRetrievalError, StorageValueRef},
	StorageKind,
};
use sp_std::fmt::Debug;

/// Storage keys for offchain worker
/// THe tx_id is used to identify the transaction
const TX_ID: &[u8] = b"tx_id";
/// The lock is used to prevent the execution of the function
const DB_LOCK: &[u8] = b"lock";
/// The tx_id is used to identify the transaction in persistent storage
const DB_TX_ID: &[u8] = b"tx-id/";

/// Locks the execution of the function
#[derive(Debug)]
pub enum LockStatus {
	/// Lock is acquired
	Locked,
	/// Lock is released
	Release,
}

/// Acquires a lock on the execution of the function
pub fn lock<F>(prefix: &[u8], f: F) -> LockStatus
where
	F: Fn(),
{
	let lock_key = [prefix, DB_LOCK].concat();
	let mut lock_storage = StorageValueRef::persistent(&lock_key);

	let exec_id_opt = StorageValueRef::persistent(DB_TX_ID).get();
	if let Ok(Some(exec_id)) = exec_id_opt {
		let id_key = [prefix, TX_ID].concat();
		let id_storage = StorageValueRef::persistent(&id_key);
		let need_to_clear_lock =
			id_storage.mutate(|id: Result<Option<[u8; 32]>, StorageRetrievalError>| match id {
				Ok(Some(val)) => {
					if val != exec_id {
						// new id we need to clear lock because of first launch
						Ok(exec_id)
					} else {
						Err(())
					}
				},
				_ => {
					// no id we need to clear lock because of first launch
					Ok(exec_id)
				},
			});

		if need_to_clear_lock.is_ok() {
			lock_storage.clear();
		}
	}

	let can_process = lock_storage.mutate(
		|is_locked: Result<Option<bool>, StorageRetrievalError>| match is_locked {
			Ok(Some(true)) => Err(()),
			_ => Ok(true),
		},
	);

	match can_process {
		Ok(true) => {
			f();
			lock_storage.clear();
			LockStatus::Release
		},
		_ => LockStatus::Locked,
	}
}

/// Wrapper for offchain get operations
pub fn get_index_value<V: Decode + Debug>(
	kind: StorageKind,
	key: &[u8],
) -> Result<V, StorageRetrievalError> {
	match kind {
		StorageKind::PERSISTENT => {
			let indexed_value = get_impl::<V>(key);
			log_err!("key: {:?}", key);
			log_err!("indexed_value: {:?}", indexed_value);
			indexed_value
		},
		StorageKind::LOCAL => {
			let indexed_value = offchain::local_storage_get(kind, key);
			match indexed_value {
				Some(value) =>
					V::decode(&mut &value[..]).map_err(|_| StorageRetrievalError::Undecodable),
				None => Err(StorageRetrievalError::Undecodable),
			}
		},
	}
}

/// Sets a value by the key to offchain index
pub fn set_index_value<V>(key: &[u8], value: V)
where
	V: Encode + Debug,
{
	log_err!("key: {:?}", key);
	log_err!("set_index value: {:?}", value);
	let oci_mem = StorageValueRef::persistent(key);
	oci_mem.set(&value.encode());
}

/// Gets a value by the key from persistent storage
fn get_impl<V: Decode + Debug>(key: &[u8]) -> Result<V, StorageRetrievalError> {
	let oci_mem = StorageValueRef::persistent(key);
	log_err!("key from get_impl: {:?}", key);
	if let Ok(Some(data)) = oci_mem.get::<V>() {
		log_err!("data from get_impl: {:?}", data);
		return Ok(data)
	} else {
		return Err(StorageRetrievalError::Undecodable)
	}
}
