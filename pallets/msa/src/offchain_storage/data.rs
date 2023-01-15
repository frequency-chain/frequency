/// MSA Offchain Storage Data
use codec::{Decode, Encode};
use sp_std::collections::btree_map::BTreeMap;

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
