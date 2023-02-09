use core::marker::PhantomData;

use codec::{Decode, Encode};
use common_primitives::msa::MessageSourceId;
use frame_support::{
	storage::{child, child::ChildInfo, ChildTriePrefixIterator},
	Identity, ReversibleStorageHasher,
};
use sp_std::{fmt::Debug, prelude::*};

pub trait IsStatefulKey: Debug + Encode + Decode + Eq + PartialEq {}

impl IsStatefulKey for () {}
impl<T1: Debug + Decode + Encode + Eq + PartialEq> IsStatefulKey for (T1,) {}
impl<
		T1: Debug + Decode + Encode + Eq + PartialEq,
		T2: Debug + Decode + Encode + Eq + PartialEq,
	> IsStatefulKey for (T1, T2)
{
}
impl<
		T1: Debug + Decode + Encode + Eq + PartialEq,
		T2: Debug + Decode + Encode + Eq + PartialEq,
		T3: Debug + Decode + Encode + Eq + PartialEq,
	> IsStatefulKey for (T1, T2, T3)
{
}
impl<
		T1: Debug + Decode + Encode + Eq + PartialEq,
		T2: Debug + Decode + Encode + Eq + PartialEq,
		T3: Debug + Decode + Encode + Eq + PartialEq,
		T4: Debug + Decode + Encode + Eq + PartialEq,
	> IsStatefulKey for (T1, T2, T3, T4)
{
}

/// Paginated Stateful data access utility
pub struct StatefulChildTree<H: ReversibleStorageHasher = Identity> {
	hasher: PhantomData<H>,
}
impl<H: ReversibleStorageHasher> StatefulChildTree<H> {
	/// Reads a child tree node and tries to decode it
	///
	/// The read is performed from the `msa_id` only. The `key` is not required. If the
	/// data doesn't store under the given `key` `None` is returned.
	pub fn try_read<K: IsStatefulKey, V: Decode + Sized>(
		msa_id: &MessageSourceId,
		keys: &K,
	) -> Result<Option<V>, ()> {
		let child = Self::get_child_tree(*msa_id);
		let value = child::get_raw(&child, &H::hash(&keys.encode()).as_ref());
		match value {
			None => Ok(None),
			Some(v) => Ok(Decode::decode(&mut &v[..]).map(Some).map_err(|_| ())?),
		}
	}

	/// Prefix Iterator for a child tree
	///
	/// Allows getting all the keys having the same prefix
	/// Warning: This should not be used from any on-chain transaction!
	pub fn prefix_iterator<
		PrefixKey: IsStatefulKey,
		V: Decode + Sized,
		OutputKey: IsStatefulKey + Sized,
	>(
		msa_id: &MessageSourceId,
		keys: &PrefixKey,
	) -> ChildTriePrefixIterator<(OutputKey, V)> {
		let child = Self::get_child_tree(*msa_id);
		ChildTriePrefixIterator::with_prefix_over_key::<H>(
			&child,
			&H::hash(&keys.encode()).as_ref(),
		)
	}

	/// Writes directly into child tree node
	pub fn write<K: IsStatefulKey, V: Encode + Sized>(
		msa_id: &MessageSourceId,
		keys: &K,
		new_value: V,
	) {
		let child_trie_info = &Self::get_child_tree(*msa_id);
		child::put_raw(
			child_trie_info,
			&H::hash(&keys.encode()).as_ref(),
			new_value.encode().as_ref(),
		);
	}

	/// Kills a child tree node
	pub fn kill<K: IsStatefulKey>(msa_id: &MessageSourceId, keys: &K) {
		let child_trie_info = &Self::get_child_tree(*msa_id);
		child::kill(child_trie_info, &H::hash(&keys.encode()).as_ref());
	}

	fn get_child_tree(msa_id: MessageSourceId) -> ChildInfo {
		let trie_root = Self::get_tree_prefix(msa_id);
		child::ChildInfo::new_default(H::hash(&trie_root[..]).as_ref())
	}

	fn get_tree_prefix(msa_id: MessageSourceId) -> Vec<u8> {
		let arr = [&msa_id.encode()[..], b"::"].concat();
		arr.to_vec()
	}
}
