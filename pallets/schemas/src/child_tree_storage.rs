use crate::{Config};
use codec::{Decode, Encode};
use common_primitives::{
	msa::MessageSourceId,
};
use frame_support::{storage::{child, child::ChildInfo, ChildTriePrefixIterator}, Identity, Blake2_256, StorageHasher, Blake2_128, Blake2_128Concat};
use sp_runtime::{traits::Hash};
use sp_std::{marker::PhantomData, prelude::*};

/// child tree access utility
pub struct ChildTreeStorage;
impl ChildTreeStorage
{
	/// Reads a child tree node
	///
	/// The read is performed from the `msa_id` only. The `key` is not required. If the
	/// data doesn't store under the given `key` `None` is returned.
	pub fn read<V: Decode + Sized>(msa_id: &MessageSourceId, key: &[u8]) -> Option<V> {
		let child = Self::get_child_tree(*msa_id);
		child::get::<V>(&child, &Identity::hash(key))
	}

	/// Prefix Iterator for a child tree
	///
	/// Allows getting all the keys having the same prefix
	/// Warning: This should not be used from any on-chain transaction!
	pub fn prefix_iterator<V: Decode + Sized>(
		msa_id: &MessageSourceId,
		key: &[u8]
	) -> ChildTriePrefixIterator<(Vec<u8>, V)> {
		let child = Self::get_child_tree(*msa_id);
		ChildTriePrefixIterator::<_>::with_prefix(
			&child,
			key,
		)
	}

	/// Writes directly into child tree node
	pub fn write<V: Encode + Sized>(
		msa_id: &MessageSourceId,
		key: &[u8],
		new_value: V,
	) {
		let child_trie_info = &Self::get_child_tree(*msa_id);
		child::put_raw(child_trie_info, &Identity::hash(key), new_value.encode().as_ref());
	}

	/// Kills a child tree node
	pub fn kill<V: Encode + Sized>(
		msa_id: &MessageSourceId,
		key: &[u8],
	) {
		let child_trie_info = &Self::get_child_tree(*msa_id);
		child::kill(child_trie_info, &Identity::hash(key));
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
