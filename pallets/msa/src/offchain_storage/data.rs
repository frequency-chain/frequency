/// MSA Offchain Storage Data
use codec::{Decode, Encode};
use sp_std::collections::btree_map::BTreeMap;

/// MSA Public Key Data, stored in offchain storage
#[derive(Debug, Encode, Decode, Clone, PartialEq, Eq, Default)]
pub struct MSAPublicKeyData<K, V>(pub BTreeMap<K, Vec<V>>);

/// Operations enum for MSA Offchain Storage
pub enum MSAPublicKeyDataOperation<K, V> {
	/// Add MSA Public Key
	Add(K, V),
	/// Remove MSA Public Key
	Remove(K, V),
}
