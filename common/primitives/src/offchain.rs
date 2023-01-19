use codec::{Decode, Encode};
use sp_io::offchain_index;
use sp_runtime::offchain::{
	storage::{StorageRetrievalError, StorageValueRef},
	storage_lock::{StorageLock, Time},
	Duration,
};
use sp_std::fmt::Debug;

const DB_LOCK: &[u8] = b"lock";

const LOCK_DEADLINE: u64 = 1000;

/// Locks the execution of the function
#[derive(Debug)]
pub enum LockStatus {
	/// Lock is acquired
	Locked,
	/// Lock is released
	Released,
}

/// Locks the execution of the function
pub fn lock<F>(prefix: &[u8], f: F) -> LockStatus
where
	F: Fn(),
{
	let locked_tx = [prefix, DB_LOCK].concat();
	let locked_tx_key = locked_tx.encode();
	let mut lock = StorageLock::<Time>::with_deadline(
		locked_tx_key.as_slice(),
		Duration::from_millis(LOCK_DEADLINE),
	);
	let lock_status = if let Ok(_guard) = lock.try_lock() {
		f();
		LockStatus::Released
	} else {
		LockStatus::Locked
	};
	lock_status
}

/// Wrapper for offchain get operations
pub fn get_index_value<V: Decode + Debug>(key: &[u8]) -> Result<V, StorageRetrievalError> {
	let indexed_value = get_impl::<V>(key);
	indexed_value
}

/// Wrapper for offchain_index set operations
pub fn set_offchain_index_value(key: &[u8], value: &[u8]) {
	offchain_index::clear(key);
	offchain_index::set(key, value);
}

/// Sets a value by the key to offchain index
pub fn set_index_value<V>(key: &[u8], value: V)
where
	V: Encode + Debug,
{
	let oci_mem = StorageValueRef::persistent(key);
	oci_mem.set(&value);
}

/// Gets a value by the key from persistent storage
fn get_impl<V: Decode + Debug>(key: &[u8]) -> Result<V, StorageRetrievalError> {
	let oci_mem = StorageValueRef::persistent(key);
	if let Ok(Some(data)) = oci_mem.get::<V>() {
		return Ok(data)
	} else {
		return Err(StorageRetrievalError::Undecodable)
	}
}
