use crate::msa::MessageSourceId;
use numtoa::NumToA;
use parity_scale_codec::{Decode, Encode};
use sp_io::offchain_index;
use sp_runtime::offchain::storage::{StorageRetrievalError, StorageValueRef};
use sp_std::{fmt::Debug, vec, vec::Vec};

/// Lock expiration timeout in in milli-seconds for msa pallet per msa account
pub const MSA_ACCOUNT_LOCK_TIMEOUT_EXPIRATION: u64 = 50;
/// Lock name prefix for msa account
pub const MSA_ACCOUNT_LOCK_NAME_PREFIX: &[u8; 16] = b"Msa::ofw::lock::";
/// Offchain storage prefix for msa account
pub const MSA_ACCOUNT_STORAGE_NAME_PREFIX: &[u8; 16] = b"Msa::ofw::keys::";
/// msa account lock name
pub fn get_msa_account_lock_name(msa_id: MessageSourceId) -> Vec<u8> {
	let mut buff = [0u8; 30];
	vec![MSA_ACCOUNT_LOCK_NAME_PREFIX, msa_id.numtoa(10, &mut buff)].concat()
}
/// msa account storage key name
pub fn get_msa_account_storage_key_name(msa_id: MessageSourceId) -> Vec<u8> {
	let mut buff = [0u8; 30];
	vec![MSA_ACCOUNT_STORAGE_NAME_PREFIX, msa_id.numtoa(10, &mut buff)].concat()
}

/// Locks the execution of the function
#[derive(Debug)]
pub enum LockStatus {
	/// Lock is acquired
	Locked,
	/// Lock is released
	Released,
}

/// Wrapper for offchain get operations
pub fn get_index_value<V: Decode + Debug>(key: &[u8]) -> Result<Option<V>, StorageRetrievalError> {
	get_impl::<V>(key)
}

/// Wrapper for offchain_index remove operations
pub fn remove_offchain_index_value(key: &[u8]) {
	offchain_index::clear(key);
}

/// Wrapper for offchain_index set operations
pub fn set_offchain_index_value(key: &[u8], value: &[u8]) {
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
fn get_impl<V: Decode + Debug>(key: &[u8]) -> Result<Option<V>, StorageRetrievalError> {
	let oci_mem = StorageValueRef::persistent(key);
	match oci_mem.get::<V>() {
		Ok(Some(data)) => Ok(Some(data)),
		Ok(None) => Ok(None),
		Err(_) => Err(StorageRetrievalError::Undecodable),
	}
}
