use crate::{Config, StorageKey, TrieId};
use common_primitives::msa::MessageSourceId;
use frame_support::{
	storage::{child, child::ChildInfo},
	Blake2_128Concat, Hashable, ReversibleStorageHasher, StorageHasher,
};
// use log::log;
// use sp_core::crypto::UncheckedFrom;
use sp_io::hashing::twox_128;
use sp_runtime::{traits::Hash, DispatchError};
use sp_std::{marker::PhantomData, prelude::*};

/// Associated child trie unique id is built from the hash part of the trie id.
fn child_trie_info(trie_id: &[u8]) -> ChildInfo {
	ChildInfo::new_default(trie_id)
}

fn blake2_128_concat(key: &[u8]) -> Vec<u8> {
	Blake2_128Concat::hash(key)
}

/// type of graph
pub enum GraphType {
	/// public graph
	Public,
	/// private graph
	Private,
}

/// child tree access utility
pub struct Storage<T>(PhantomData<T>);
impl<T> Storage<T>
where
	T: Config,
	// T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
	/// Reads a storage kv pair of a contract.
	///
	/// The read is performed from the `trie_id` only. The `address` is not necessary. If the
	/// contract doesn't store under the given `key` `None` is returned.
	pub fn read(trie_id: &TrieId, key: &StorageKey) -> Option<Vec<u8>> {
		child::get_raw(&child_trie_info(trie_id), blake2_128_concat(key).as_slice())
	}

	/// Returns `Some(len)` (in bytes) if a storage item exists at `key`.
	///
	/// Returns `None` if the `key` wasn't previously set by `set_storage` or
	/// was deleted.
	pub fn size(trie_id: &TrieId, key: &StorageKey) -> Option<u32> {
		child::len(&child_trie_info(trie_id), blake2_128_concat(key).as_slice())
	}

	/// Generates a unique trie id by returning  `hash(account_id ++ nonce)`.
	pub fn generate_trie_id(node_id: MessageSourceId, nonce: u64) -> TrieId {
		let buf: Vec<_> =
			(&node_id.to_le_bytes()).iter().chain(&nonce.to_le_bytes()).cloned().collect();
		T::Hashing::hash(&buf)
			.as_ref()
			.to_vec()
			.try_into()
			.expect("Runtime uses a reasonable hash size. Hence sizeof(T::Hash) <= 128; qed")
	}

	/// write directly into child tree
	pub fn write(
		trie_id: &TrieId,
		key: &StorageKey,
		new_value: Option<Vec<u8>>,
	) -> Result<bool, DispatchError> {
		let hashed_key = blake2_128_concat(key);
		let child_trie_info = &child_trie_info(trie_id);

		match &new_value {
			Some(new_value) => child::put_raw(child_trie_info, &hashed_key, new_value),
			None => child::kill(child_trie_info, &hashed_key),
		}

		Ok(true)
	}

	/// write into graph
	pub fn write_graph(
		trie_id: &TrieId,
		graph: GraphType,
		key: &StorageKey,
		page: u16,
		new_value: Option<Vec<u8>>,
	) -> Result<bool, DispatchError> {
		let child_trie_info = &child_trie_info(trie_id);
		let whole_key = Self::get_whole_key(graph, Some(key), Some(page));

		log::info!(
			"trie_id: {:02x?} key: {:02x?}",
			trie_id.clone().into_inner(),
			whole_key.clone()
		);

		match &new_value {
			Some(new_value) => child::put_raw(child_trie_info, &whole_key, new_value),
			None => child::kill(child_trie_info, &whole_key),
		}

		Ok(true)
	}

	/// iterating keys
	pub fn iter_keys(trie_id: &TrieId) -> Vec<StorageKey> {
		let child_trie_info = &child_trie_info(trie_id);
		let location = child_trie_info.storage_key();

		#[cfg(test)]
		{
			let prefix = child_trie_info.prefixed_storage_key();
			println!("prefix {:X?}", prefix.into_inner());
			println!("location {:X?}", location);
		}

		let mut res = Vec::new();

		let mut key = Some(Vec::default());
		while key.is_some() {
			key = sp_io::default_child_storage::next_key(location, key.unwrap().as_slice());

			if let Some(arr) = key.clone() {
				let reversed = Blake2_128Concat::reverse(arr.as_slice()).to_vec();
				res.push(StorageKey::try_from(reversed.clone()).unwrap());

				#[cfg(test)]
				{
					println!("key {:X?}", key);
					println!("reversed {:X?}", reversed);
				}
			}
		}

		res
	}

	fn get_graph_key_prefix(graph: GraphType) -> Vec<u8> {
		let mut k: Vec<u8> = vec![];
		k.extend(&twox_128(b"G"));
		match graph {
			GraphType::Public => k.extend(&twox_128(b"B")),
			GraphType::Private => k.extend(&twox_128(b"R")),
		}
		k
	}

	fn get_whole_key(graph: GraphType, key: Option<&StorageKey>, page: Option<u16>) -> Vec<u8> {
		let prefix = Self::get_graph_key_prefix(graph);
		let mut whole_key = vec![];
		whole_key.extend_from_slice(prefix.as_ref());

		match (key, page) {
			(Some(k), Some(p)) => {
				let hashed_key = blake2_128_concat(k);
				whole_key.extend_from_slice(hashed_key.as_ref());
				whole_key.extend(&p.blake2_128_concat());
				whole_key
			},
			(Some(k), None) => {
				let hashed_key = blake2_128_concat(k);
				whole_key.extend_from_slice(hashed_key.as_ref());
				whole_key
			},
			(_, _) => whole_key,
		}
	}
}
