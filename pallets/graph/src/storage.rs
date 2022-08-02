use crate::{Config, GraphKey, GraphType, PrivatePage, PublicPage, StorageKey};
use codec::Encode;
use common_primitives::msa::MessageSourceId;
use frame_support::{
	storage::{child, child::ChildInfo, ChildTriePrefixIterator},
	Identity,
};
use sp_runtime::{traits::Hash, DispatchError};
use sp_std::{marker::PhantomData, prelude::*};

/// child tree access utility
pub struct Storage<T>(PhantomData<T>);
impl<T> Storage<T>
where
	T: Config,
	// T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
	/// Reads a public graph storage
	///
	/// The read is performed from the `msa_id` only. The `address` is not necessary. If the
	/// contract doesn't store under the given `key` `None` is returned.
	pub fn read_public_graph(msa_id: &MessageSourceId, key: &StorageKey) -> Option<PublicPage> {
		let child = Self::get_child_tree(*msa_id, GraphType::Public);
		child::get::<PublicPage>(&child, key.as_slice())
	}

	/// iterator for public graph
	pub fn public_graph_iter(
		msa_id: &MessageSourceId,
	) -> ChildTriePrefixIterator<(GraphKey, PublicPage)> {
		let child = Self::get_child_tree(*msa_id, GraphType::Public);
		ChildTriePrefixIterator::<_>::with_prefix_over_key::<Identity>(&child, &[])
	}

	/// Reads a private graph storage
	///
	/// The read is performed from the `msa_id` only. The `address` is not necessary. If the
	/// contract doesn't store under the given `key` `None` is returned.
	pub fn read_private_graph(msa_id: &MessageSourceId, key: &StorageKey) -> Option<PrivatePage> {
		let child = Self::get_child_tree(*msa_id, GraphType::Private);
		child::get::<PrivatePage>(&child, key.as_slice())
	}

	/// iterator for public graph
	pub fn private_graph_iter(
		msa_id: &MessageSourceId,
	) -> ChildTriePrefixIterator<(GraphKey, PrivatePage)> {
		let child = Self::get_child_tree(*msa_id, GraphType::Private);
		ChildTriePrefixIterator::<_>::with_prefix_over_key::<Identity>(&child, &[])
	}

	/// write directly into child tree
	pub fn write_public(
		msa_id: &MessageSourceId,
		key: &StorageKey,
		new_value: Option<PublicPage>,
	) -> Result<bool, DispatchError> {
		let child_trie_info = &Self::get_child_tree(*msa_id, GraphType::Public);

		match &new_value {
			Some(new_value) => child::put_raw(child_trie_info, &key, new_value.encode().as_ref()),
			None => child::kill(child_trie_info, &key),
		}

		Ok(true)
	}

	/// write into graph
	pub fn write_private(
		msa_id: &MessageSourceId,
		key: &StorageKey,
		new_value: Option<PrivatePage>,
	) -> Result<bool, DispatchError> {
		let child_trie_info = &Self::get_child_tree(*msa_id, GraphType::Private);

		match &new_value {
			Some(new_value) => child::put_raw(child_trie_info, &key, new_value.encode().as_ref()),
			None => child::kill(child_trie_info, &key),
		}

		Ok(true)
	}

	fn get_child_tree(msa_id: MessageSourceId, graph: GraphType) -> ChildInfo {
		let trie_root = Self::get_graph_key_prefix(msa_id, graph);
		child::ChildInfo::new_default(T::Hashing::hash(&trie_root[..]).as_ref())
	}

	fn get_graph_key_prefix(msa_id: MessageSourceId, graph: GraphType) -> Vec<u8> {
		let mut k: Vec<u8> = vec![];
		k.extend(b"graph");
		k.extend_from_slice(&msa_id.encode()[..]);
		match graph {
			GraphType::Public => k.extend(b"P"),
			GraphType::Private => k.extend(b"S"),
		}
		k
	}
}
