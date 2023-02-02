use codec::{Decode, Encode};
use common_primitives::msa::MessageSourceId;
use frame_support::{
	storage::{child, child::ChildInfo, ChildTriePrefixIterator},
	Identity, StorageHasher,
};
use sp_core::hexdisplay::AsBytesRef;
use sp_std::prelude::*;

pub type StatefulPageKeyPart = Vec<u8>;

/// Paginated Stateful data access utility
pub struct StatefulChildTree;
impl StatefulChildTree {
	pub fn concat_keys(keys: &[StatefulPageKeyPart]) -> Vec<u8> {
		let mut key = Vec::<u8>::new();
		for k in keys {
			let mut k2 = k.to_owned();
			key.append(&mut k2);
		}

		key
	}

	/// Reads a child tree node
	///
	/// The read is performed from the `msa_id` only. The `key` is not required. If the
	/// data doesn't store under the given `key` `None` is returned.
	pub fn read<V: Decode + Sized>(
		msa_id: &MessageSourceId,
		keys: &[StatefulPageKeyPart],
	) -> Option<V> {
		let child = Self::get_child_tree(*msa_id);
		child::get::<V>(&child, &Identity::hash(Self::concat_keys(keys).as_bytes_ref()))
	}

	/// Prefix Iterator for a child tree
	///
	/// Allows getting all the keys having the same prefix
	/// Warning: This should not be used from any on-chain transaction!
	pub fn prefix_iterator<V: Decode + Sized>(
		msa_id: &MessageSourceId,
		keys: &[StatefulPageKeyPart],
	) -> ChildTriePrefixIterator<(Vec<u8>, V)> {
		let child = Self::get_child_tree(*msa_id);
		ChildTriePrefixIterator::<_>::with_prefix(&child, Self::concat_keys(keys).as_bytes_ref())
	}

	/// Writes directly into child tree node
	pub fn write<V: Encode + Sized>(
		msa_id: &MessageSourceId,
		keys: &[StatefulPageKeyPart],
		new_value: V,
	) {
		let child_trie_info = &Self::get_child_tree(*msa_id);
		child::put_raw(
			child_trie_info,
			&Identity::hash(Self::concat_keys(keys).as_bytes_ref()),
			new_value.encode().as_ref(),
		);
	}

	/// Kills a child tree node
	pub fn kill<V: Encode + Sized>(msa_id: &MessageSourceId, keys: &[StatefulPageKeyPart]) {
		let child_trie_info = &Self::get_child_tree(*msa_id);
		child::kill(child_trie_info, &Identity::hash(Self::concat_keys(keys).as_bytes_ref()));
	}

	fn get_child_tree(msa_id: MessageSourceId) -> ChildInfo {
		let trie_root = Self::get_tree_prefix(msa_id);
		child::ChildInfo::new_default(Identity::hash(&trie_root[..]).as_ref())
	}

	fn get_tree_prefix(msa_id: MessageSourceId) -> Vec<u8> {
		let arr = [&msa_id.encode()[..], b"::"].concat();
		arr.to_vec()
	}
}
