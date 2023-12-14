/// Offchain Storage for MSA
use common_primitives::offchain as offchain_common;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::offchain::storage::StorageRetrievalError;
use sp_std::{
	fmt::{Debug, Formatter},
	vec::Vec,
};

/// Pallet MSA lock prefix
pub const MSA_LOCK_PREFIX: &[u8] = b"pallet::msa::";

/// Block event storage prefix
pub const BLOCK_EVENT_KEY: &[u8] = b"frequency::block_event::msa::";

/// Generic prefix for MSA index storage
pub const MSA_INDEX_KEY: &[u8] = b"frequency::msa::";

/// Offchain index for MSA events count
pub const BLOCK_EVENT_COUNT_KEY: &[u8] = b"frequency::block_event::msa::count::";

/// Derive storage key for MSA index
#[deny(clippy::clone_double_ref)]
pub(crate) fn derive_storage_key<K>(prefix: &[u8], suffix: K) -> Vec<u8>
where
	K: Encode + Clone + Ord + Decode,
{
	[prefix, suffix.encode().as_slice()].concat().encode().to_vec()
}

/// MSA Public Key Data
/// Support scale encoding and decoding for efficient storage
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default)]
pub struct MSAPublicKeyData<K, V, B>
where
	K: Encode + Clone + Ord + Decode + Eq + Debug,
	V: Encode + Clone + Decode + Eq + Debug,
	B: Encode + Clone + Decode + Eq + Debug + Default,
{
	/// msa_id is the key for the index
	pub msa_id: K,
	/// public_keys is the collection of public keys for the MSA
	pub public_keys: Vec<V>,
	/// block_number is the block number of the last update of this map
	pub block_number: B,
}

impl<K, V, B> Debug for MSAPublicKeyData<K, V, B>
where
	K: Encode + Clone + Ord + Decode + Eq + Debug,
	V: Encode + Clone + Decode + Eq + Debug,
	B: Encode + Clone + Decode + Eq + Debug + Default,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> sp_std::fmt::Result {
		write!(
			f,
			"MSA Public Key Data: {{ msa_id: {:?}, public_keys: {:?}, block_number: {:?} }}",
			self.msa_id, self.public_keys, self.block_number
		)
	}
}
/// Event Type on MSA Public Key Data
pub enum EventType {
	/// Add MSA Public Key
	Add,
	/// Remove MSA Public Key
	Remove,
}

/// Operations enum for MSA Offchain Storage
/// Add: Add MSA Public Key
/// Remove: Remove MSA Public Key. Note: if only one key is present, the entire MSA ID is removed from index
pub enum MSAPublicKeyDataOperation<K, V, B> {
	/// Add MSA Public Key
	Add(K, V, B),
	/// Remove MSA Public Key
	Remove(K, V, B),
}

/// Process MSA key event and update offchain storage
pub fn process_msa_key_event<K, V, B>(
	ops: MSAPublicKeyDataOperation<K, V, B>,
) -> Result<(), StorageRetrievalError>
where
	K: Encode + Clone + Ord + Decode + Eq + Debug,
	V: Encode + Clone + Decode + Eq + Debug,
	B: Encode + Clone + Decode + Eq + Debug + Default,
{
	match ops {
		MSAPublicKeyDataOperation::Add(msa_id, key, block) =>
			return add_msa_key(msa_id, key, block),
		MSAPublicKeyDataOperation::Remove(msa_id, key, block) =>
			return remove_msa_key(msa_id, key, block),
	}
}
/// Set offchain index value, used to store MSA Events to be process by offchain worker
pub fn set_offchain_index<V>(key: &[u8], value: V)
where
	V: Encode + Clone + Decode + Eq + Debug,
{
	offchain_common::set_offchain_index_value(key, value.encode().as_slice());
}

/// Get offchain index value, used to store MSA Events to be process by offchain worker
pub fn get_offchain_index<V>(key: &[u8]) -> Option<V>
where
	V: Encode + Clone + Decode + Eq + Debug,
{
	let value = offchain_common::get_index_value::<V>(key);
	match value {
		Ok(v) => Some(v),
		Err(e) => {
			log::error!("Error getting offchain index value: {:?}", e);
			None
		},
	}
}

/// Remove offchain index value, used to store MSA Events to be process by offchain worker
pub fn remove_offchain_index(key: &[u8]) {
	offchain_common::remove_offchain_index_value(key);
}

fn add_msa_key<K, V, B>(msa_id: K, key: V, block: B) -> Result<(), StorageRetrievalError>
where
	K: Encode + Clone + Ord + Decode + Eq + Debug,
	V: Encode + Clone + Decode + Eq + Debug,
	B: Encode + Clone + Decode + Eq + Debug + Default,
{
	let key_binding = derive_storage_key::<K>(MSA_INDEX_KEY, msa_id.clone());
	let derived_key = key_binding.as_slice();
	let msa_keys = offchain_common::get_index_value::<MSAPublicKeyData<K, V, B>>(derived_key);
	let mut msa_key_map = Vec::new();

	if let Ok(keys) = msa_keys {
		msa_key_map = keys.public_keys;
	}
	msa_key_map.push(key);
	let msa_key_data =
		MSAPublicKeyData::<K, V, B> { msa_id, public_keys: msa_key_map, block_number: block };
	offchain_common::set_index_value(derived_key, msa_key_data);
	Ok(())
}

fn remove_msa_key<K, V, B>(msa_id: K, key: V, block: B) -> Result<(), StorageRetrievalError>
where
	K: Encode + Clone + Ord + Decode + Eq + Debug,
	V: Encode + Clone + Decode + Eq + Debug,
	B: Encode + Clone + Decode + Eq + Debug + Default,
{
	let key_binding = derive_storage_key::<K>(MSA_INDEX_KEY, msa_id.clone());
	let derived_key = key_binding.as_slice();
	let msa_keys = offchain_common::get_index_value::<MSAPublicKeyData<K, V, B>>(derived_key);
	let mut msa_key_map = Vec::new();
	if let Ok(keys) = msa_keys {
		msa_key_map = keys.public_keys;
		msa_key_map.retain(|x| x != &key);
	}

	let msa_keys_updated =
		MSAPublicKeyData::<K, V, B> { msa_id, public_keys: msa_key_map, block_number: block };
	offchain_common::set_index_value(derived_key, msa_keys_updated);
	Ok(())
}

/// Get MSA public keys from offchain storage
pub fn get_msa_keys<K, V, B>(msa_id: K) -> Result<(B, Vec<V>), StorageRetrievalError>
where
	K: Encode + Clone + Ord + Decode + Eq + Debug,
	V: Encode + Clone + Decode + Eq + Debug,
	B: Encode + Clone + Decode + Eq + Debug + Default,
{
	let key_binding = derive_storage_key::<K>(MSA_INDEX_KEY, msa_id.clone());
	let derived_key = key_binding.as_slice();
	let msa_keys = offchain_common::get_index_value::<MSAPublicKeyData<K, V, B>>(derived_key);
	let mut msa_key_map = Vec::new();
	let mut block_number = B::default();
	match msa_keys {
		Ok(keys) => {
			msa_key_map = keys.public_keys;
			block_number = keys.block_number;
		},
		Err(e) => {
			log::error!("Error in getting MSA for msa_id: {:?}, error: {:?}", msa_id, e);
		},
	}
	Ok((block_number, msa_key_map))
}
