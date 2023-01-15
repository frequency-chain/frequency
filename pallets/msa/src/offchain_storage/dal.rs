use crate::offchain_storage::{
	data::{MSAPublicKeyData, MSAPublicKeyDataOperation},
	keys::derive_storage_key,
};
use codec::{Decode, Encode};
use common_helpers::offchain as offchain_common;
use sp_runtime::offchain::{storage::StorageRetrievalError, StorageKind};

/// Process MSA key event and update offchain storage
pub fn process_msa_key_event<K, V>(
	ops: MSAPublicKeyDataOperation<K, V>,
) -> Result<(), StorageRetrievalError>
where
	K: Encode + Clone + Ord + Decode + Eq,
	V: Encode + Clone + Decode + Eq,
{
	match ops {
		MSAPublicKeyDataOperation::Add(msa_id, key) => return add_msa_key(msa_id, key),
		MSAPublicKeyDataOperation::Remove(msa_id, key) => return remove_msa_key(msa_id, key),
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
