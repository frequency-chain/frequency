/// Offchain Storage for MSA
use codec::{Decode, Encode};
use common_primitives::offchain as offchain_common;
use sp_runtime::offchain::{storage::StorageRetrievalError, StorageKind};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

/// Generic prefix for MSA index storage
pub const MSA_INDEX_KEY: &[u8] = b"frequency::msa::";

/// Derive storage key for MSA index
#[deny(clippy::clone_double_ref)]
pub(crate) fn derive_storage_key<K>(msa_id: K) -> Vec<u8>
where
	K: Encode + Clone + Ord + Decode,
{
	[MSA_INDEX_KEY, msa_id.encode().as_slice()].concat()
}

/// MSA Public Key Data, stored in offchain storage
/// Key: MSA ID
/// Value: Public Key
/// Note: MSA ID is the same as the MSA Account ID
/// Note: Public Key is the same as the MSA Account Public Key
#[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, Default)]
pub struct MSAPublicKeyData<K, V>(pub BTreeMap<K, Vec<V>>);

/// Operations enum for MSA Offchain Storage
/// Add: Add MSA Public Key
/// Remove: Remove MSA Public Key. Note: if only one key is present, the entire MSA ID is removed from index
pub enum MSAPublicKeyDataOperation<K, V> {
	/// Add MSA Public Key
	Add(K, V),
	/// Remove MSA Public Key
	Remove(K, V),
}

/// Process MSA key event and update offchain storage
pub fn process_msa_key_event<K, V>(
	ops: MSAPublicKeyDataOperation<K, V>,
) -> Result<(), StorageRetrievalError>
where
	K: Encode + Clone + Ord + Decode + Eq,
	V: Encode + Clone + Decode + Eq,
{
	match ops {
		MSAPublicKeyDataOperation::Add(msa_id, key) => return add_msa_key(msa_id, key.clone()),
		MSAPublicKeyDataOperation::Remove(msa_id, key) =>
			return remove_msa_key(msa_id, key.clone()),
	}
}

fn add_msa_key<K, V>(msa_id: K, key: V) -> Result<(), StorageRetrievalError>
where
	K: Encode + Clone + Ord + Decode + Eq,
	V: Encode + Clone + Decode + Eq,
{
	let key_binding = derive_storage_key::<K>(msa_id.clone()).encode();
	let derived_key = key_binding.as_slice();
	let msa_keys = offchain_common::get_index_value::<MSAPublicKeyData<K, V>>(
		StorageKind::PERSISTENT,
		derived_key,
	);

	let msa_key_data = msa_keys.unwrap();
	let mut msa_key_map = msa_key_data.0;
	msa_key_map.get_mut(&msa_id).unwrap().push(key);
	let msa_keys_updated = MSAPublicKeyData(msa_key_map);

	offchain_common::set_index_value(derived_key, msa_keys_updated.encode().as_slice());
	Ok(())
}

fn remove_msa_key<K, V>(msa_id: K, key: V) -> Result<(), StorageRetrievalError>
where
	K: Encode + Clone + Ord + Decode + Eq,
	V: Encode + Clone + Decode + Eq,
{
	let key_binding = derive_storage_key::<K>(msa_id.clone()).encode();
	let derived_key = key_binding.as_slice();
	let msa_keys = offchain_common::get_index_value::<MSAPublicKeyData<K, V>>(
		StorageKind::PERSISTENT,
		derived_key,
	);

	let msa_key_data = msa_keys.unwrap();
	let mut msa_key_map = msa_key_data.0;
	if let Some(keys) = msa_key_map.get_mut(&msa_id.clone()) {
		if let Some(index) = keys.iter().position(|k| k == &key) {
			keys.remove(index);
		}
	}
	if msa_key_map.get(&msa_id).unwrap().is_empty() {
		msa_key_map.remove(&msa_id);
		return Ok(())
	}
	let msa_keys_updated = MSAPublicKeyData(msa_key_map);
	offchain_common::set_index_value(derived_key, msa_keys_updated.encode().as_slice());
	Ok(())
}

/// Get MSA public keys from offchain storage
pub fn get_msa_keys<K, V>(msa_id: K) -> Result<Vec<V>, StorageRetrievalError>
where
	K: Encode + Clone + Ord + Decode + Eq,
	V: Encode + Clone + Decode + Eq,
{
	let key_binding = derive_storage_key::<K>(msa_id.clone()).encode();
	let derived_key = key_binding.as_slice();
	let msa_keys = offchain_common::get_index_value::<MSAPublicKeyData<K, V>>(
		StorageKind::PERSISTENT,
		derived_key,
	);

	let msa_key_data = msa_keys.unwrap();
	let msa_key_map = msa_key_data.0;
	let keys = msa_key_map.get(&msa_id).unwrap();
	Ok(keys.clone())
}
