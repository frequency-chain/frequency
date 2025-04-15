use core::marker::PhantomData;

use common_primitives::msa::MessageSourceId;
use core::fmt::Debug;
use frame_support::{
	storage::{child, child::ChildInfo, ChildTriePrefixIterator},
	Blake2_128, Blake2_256, Hashable, StorageHasher, Twox128, Twox256,
};
use parity_scale_codec::{Decode, Encode};
use sp_core::{ConstU8, Get};
use sp_io::hashing::twox_64;
extern crate alloc;
use alloc::{boxed::Box, vec::Vec};

/// Hasher to use to hash keys to insert to storage.
pub trait MultipartKeyStorageHasher: StorageHasher {
	type HashSize: Get<u8>;
	/// Split the hash part out of the input.
	///
	/// I.e. for input `&[hash ++ hash ++ key]` returns `&[key]`
	fn reverse(x: &[u8], num_key_parts: u8) -> &[u8] {
		let hash_len: usize = (Self::HashSize::get() * num_key_parts).into();
		if x.len() < hash_len {
			log::error!("Invalid reverse: hash length too short");
			return &[]
		}
		&x[hash_len..]
	}
}
/// 128-bit Blake2 hash.
impl MultipartKeyStorageHasher for Blake2_128 {
	type HashSize = ConstU8<16>;
}
/// 256-bit Blake2 hash.
impl MultipartKeyStorageHasher for Blake2_256 {
	type HashSize = ConstU8<32>;
}
/// 128-bit XX hash.
impl MultipartKeyStorageHasher for Twox128 {
	type HashSize = ConstU8<16>;
}
/// 256-bit XX hash.
impl MultipartKeyStorageHasher for Twox256 {
	type HashSize = ConstU8<32>;
}

pub trait MultipartStorageKeyPart:
	Clone + Debug + Default + Decode + Encode + Eq + PartialEq + Hashable
{
}
impl<T> MultipartStorageKeyPart for T where
	T: Clone + Debug + Default + Decode + Encode + Eq + PartialEq + Hashable
{
}

/// A trait that defines operations for a multi-part key in child storage that allows to retrieve
/// all different prefix combinations
pub trait MultipartKey<H: MultipartKeyStorageHasher>: MultipartStorageKeyPart {
	type Arity: Get<u8>;

	fn hash(&self) -> Vec<u8>;
	fn hash_prefix_only(&self) -> Vec<u8>;

	#[allow(dead_code)]
	fn decode(hash: &[u8]) -> Result<Self, parity_scale_codec::Error> {
		let mut key_material = H::reverse(hash, Self::Arity::get());
		<Self as Decode>::decode(&mut key_material)
	}

	fn decode_without_prefix(
		hash: &[u8],
		prefix_len: u8,
	) -> Result<Self, parity_scale_codec::Error> {
		if prefix_len > Self::Arity::get() {
			return Err("Prefix longer than total key length".into())
		}

		let mut key_material = H::reverse(hash, Self::Arity::get() - prefix_len);
		<Self as Decode>::decode(&mut key_material)
	}
}

impl<H: MultipartKeyStorageHasher> MultipartKey<H> for () {
	type Arity = ConstU8<0>;

	fn hash(&self) -> Vec<u8> {
		Vec::new()
	}

	fn hash_prefix_only(&self) -> Vec<u8> {
		Vec::new()
	}

	fn decode(_hash: &[u8]) -> Result<Self, parity_scale_codec::Error> {
		Ok(())
	}
}

impl<H: MultipartKeyStorageHasher, T1: MultipartStorageKeyPart> MultipartKey<H> for (T1,) {
	type Arity = ConstU8<1>;

	fn hash(&self) -> Vec<u8> {
		let encoded_1 = self.0.encode();
		<H as StorageHasher>::hash(&encoded_1)
			.as_ref()
			.iter()
			.chain(self.encode().iter())
			.cloned()
			.collect::<Vec<_>>()
	}

	fn hash_prefix_only(&self) -> Vec<u8> {
		let encoded_1 = self.0.encode();
		<H as StorageHasher>::hash(&encoded_1)
			.as_ref()
			.iter()
			.cloned()
			.collect::<Vec<_>>()
	}
}

impl<H: MultipartKeyStorageHasher, T1: MultipartStorageKeyPart, T2: MultipartStorageKeyPart>
	MultipartKey<H> for (T1, T2)
{
	type Arity = ConstU8<2>;

	fn hash(&self) -> Vec<u8> {
		let encoded_1 = self.0.encode();
		let encoded_2 = self.1.encode();
		H::hash(&encoded_1)
			.as_ref()
			.iter()
			.chain(H::hash(&encoded_2).as_ref().iter())
			.chain(self.encode().iter())
			.cloned()
			.collect::<Vec<_>>()
	}

	fn hash_prefix_only(&self) -> Vec<u8> {
		let encoded_1 = self.0.encode();
		let encoded_2 = self.1.encode();
		<H as StorageHasher>::hash(&encoded_1)
			.as_ref()
			.iter()
			.chain(H::hash(&encoded_2).as_ref().iter())
			.cloned()
			.collect::<Vec<_>>()
	}
}

impl<
		H: MultipartKeyStorageHasher,
		T1: MultipartStorageKeyPart,
		T2: MultipartStorageKeyPart,
		T3: MultipartStorageKeyPart,
	> MultipartKey<H> for (T1, T2, T3)
{
	type Arity = ConstU8<3>;

	fn hash(&self) -> Vec<u8> {
		let encoded_1 = self.0.encode();
		let encoded_2 = self.1.encode();
		let encoded_3 = self.2.encode();
		H::hash(&encoded_1)
			.as_ref()
			.iter()
			.chain(H::hash(&encoded_2).as_ref().iter())
			.chain(H::hash(&encoded_3).as_ref().iter())
			.chain(self.encode().iter())
			.cloned()
			.collect::<Vec<_>>()
	}

	fn hash_prefix_only(&self) -> Vec<u8> {
		let encoded_1 = self.0.encode();
		let encoded_2 = self.1.encode();
		let encoded_3 = self.2.encode();
		<H as StorageHasher>::hash(&encoded_1)
			.as_ref()
			.iter()
			.chain(H::hash(&encoded_2).as_ref().iter())
			.chain(H::hash(&encoded_3).as_ref().iter())
			.cloned()
			.collect::<Vec<_>>()
	}
}

impl<
		H: MultipartKeyStorageHasher,
		T1: MultipartStorageKeyPart,
		T2: MultipartStorageKeyPart,
		T3: MultipartStorageKeyPart,
		T4: MultipartStorageKeyPart,
	> MultipartKey<H> for (T1, T2, T3, T4)
{
	type Arity = ConstU8<4>;

	fn hash(&self) -> Vec<u8> {
		let encoded_1 = self.0.encode();
		let encoded_2 = self.1.encode();
		let encoded_3 = self.2.encode();
		let encoded_4 = self.3.encode();
		H::hash(&encoded_1)
			.as_ref()
			.iter()
			.chain(H::hash(&encoded_2).as_ref().iter())
			.chain(H::hash(&encoded_3).as_ref().iter())
			.chain(H::hash(&encoded_4).as_ref().iter())
			.chain(self.encode().iter())
			.cloned()
			.collect::<Vec<_>>()
	}

	fn hash_prefix_only(&self) -> Vec<u8> {
		let encoded_1 = self.0.encode();
		let encoded_2 = self.1.encode();
		let encoded_3 = self.2.encode();
		let encoded_4 = self.3.encode();
		<H as StorageHasher>::hash(&encoded_1)
			.as_ref()
			.iter()
			.chain(H::hash(&encoded_2).as_ref().iter())
			.chain(H::hash(&encoded_3).as_ref().iter())
			.chain(H::hash(&encoded_4).as_ref().iter())
			.cloned()
			.collect::<Vec<_>>()
	}
}

/// A trait that allows to iterate on the prefix of a multi-part key
pub trait IsTuplePrefix<H: MultipartKeyStorageHasher, T: MultipartStorageKeyPart>:
	MultipartKey<H>
{
}
impl<H: MultipartKeyStorageHasher, T1: MultipartStorageKeyPart> IsTuplePrefix<H, (T1,)> for () {}
impl<H: MultipartKeyStorageHasher, T1: MultipartStorageKeyPart, T2: MultipartStorageKeyPart>
	IsTuplePrefix<H, (T1, T2)> for ()
{
}
impl<H: MultipartKeyStorageHasher, T1: MultipartStorageKeyPart, T2: MultipartStorageKeyPart>
	IsTuplePrefix<H, (T1, T2)> for (T1,)
{
}
impl<
		H: MultipartKeyStorageHasher,
		T1: MultipartStorageKeyPart,
		T2: MultipartStorageKeyPart,
		T3: MultipartStorageKeyPart,
	> IsTuplePrefix<H, (T1, T2, T3)> for ()
{
}
impl<
		H: MultipartKeyStorageHasher,
		T1: MultipartStorageKeyPart,
		T2: MultipartStorageKeyPart,
		T3: MultipartStorageKeyPart,
	> IsTuplePrefix<H, (T1, T2, T3)> for (T1,)
{
}
impl<
		H: MultipartKeyStorageHasher,
		T1: MultipartStorageKeyPart,
		T2: MultipartStorageKeyPart,
		T3: MultipartStorageKeyPart,
	> IsTuplePrefix<H, (T1, T2, T3)> for (T1, T2)
{
}
impl<
		H: MultipartKeyStorageHasher,
		T1: MultipartStorageKeyPart,
		T2: MultipartStorageKeyPart,
		T3: MultipartStorageKeyPart,
		T4: MultipartStorageKeyPart,
	> IsTuplePrefix<H, (T1, T2, T3, T4)> for ()
{
}
impl<
		H: MultipartKeyStorageHasher,
		T1: MultipartStorageKeyPart,
		T2: MultipartStorageKeyPart,
		T3: MultipartStorageKeyPart,
		T4: MultipartStorageKeyPart,
	> IsTuplePrefix<H, (T1, T2, T3, T4)> for (T1,)
{
}
impl<
		H: MultipartKeyStorageHasher,
		T1: MultipartStorageKeyPart,
		T2: MultipartStorageKeyPart,
		T3: MultipartStorageKeyPart,
		T4: MultipartStorageKeyPart,
	> IsTuplePrefix<H, (T1, T2, T3, T4)> for (T1, T2)
{
}
impl<
		H: MultipartKeyStorageHasher,
		T1: MultipartStorageKeyPart,
		T2: MultipartStorageKeyPart,
		T3: MultipartStorageKeyPart,
		T4: MultipartStorageKeyPart,
	> IsTuplePrefix<H, (T1, T2, T3, T4)> for (T1, T2, T3)
{
}

/// Paginated Stateful data access utility
pub struct StatefulChildTree<H: MultipartKeyStorageHasher = Twox128> {
	hasher: PhantomData<H>,
}
impl<H: MultipartKeyStorageHasher> StatefulChildTree<H> {
	/// Reads a child tree node and tries to decode it
	///
	/// The read is performed from the `msa_id` only. The `key` is not required. If the
	/// data doesn't store under the given `key` `None` is returned.
	pub fn try_read<K: MultipartKey<H>, V: Decode + Sized>(
		msa_id: &MessageSourceId,
		pallet_name: &[u8],
		storage_name: &[u8],
		keys: &K,
	) -> Result<Option<V>, ()> {
		let child = Self::get_child_tree_for_storage(*msa_id, pallet_name, storage_name);
		let value = child::get_raw(&child, &keys.hash());
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
		V: Decode + Sized,
		K: MultipartKey<H> + Sized,
		PrefixKey: IsTuplePrefix<H, K>,
	>(
		msa_id: &MessageSourceId,
		pallet_name: &[u8],
		storage_name: &[u8],
		prefix_keys: &PrefixKey,
	) -> Box<impl Iterator<Item = (K, V)>> {
		let child = Self::get_child_tree_for_storage(*msa_id, pallet_name, storage_name);
		let result = ChildTriePrefixIterator::<(Vec<u8>, V)>::with_prefix(
			&child,
			&prefix_keys.hash_prefix_only(),
		)
		.filter_map(|(k, v)| {
			if let Ok(key) =
				<K as MultipartKey<H>>::decode_without_prefix(&k, PrefixKey::Arity::get())
			{
				Some((key, v))
			} else {
				None
			}
		});
		Box::new(result)
	}

	/// Writes directly into child tree node
	pub fn write<K: MultipartKey<H>, V: Encode + Sized>(
		msa_id: &MessageSourceId,
		pallet_name: &[u8],
		storage_name: &[u8],
		keys: &K,
		new_value: V,
	) {
		let child_trie_info = &Self::get_child_tree_for_storage(*msa_id, pallet_name, storage_name);
		child::put_raw(child_trie_info, &keys.hash(), new_value.encode().as_ref());
	}

	/// Kills a child tree node
	pub fn kill<K: MultipartKey<H>>(
		msa_id: &MessageSourceId,
		pallet_name: &[u8],
		storage_name: &[u8],
		keys: &K,
	) {
		let child_trie_info = &Self::get_child_tree_for_storage(*msa_id, pallet_name, storage_name);
		child::kill(child_trie_info, &keys.hash());
	}

	/// These hashes should be consistent across the chain so we are hardcoding them
	fn get_child_tree_for_storage(
		msa_id: MessageSourceId,
		pallet_name: &[u8],
		storage_name: &[u8],
	) -> ChildInfo {
		let trie_root = Self::get_tree_prefix(msa_id);
		// child tree root should be hashed by Blake128 to avoid probability of conflict
		let hashed_keys: Vec<u8> = [
			Blake2_128::hash(&trie_root[..]).as_ref(),
			twox_64(pallet_name).as_ref(),
			twox_64(storage_name).as_ref(),
		]
		.concat();
		child::ChildInfo::new_default(&hashed_keys)
	}

	/// Storage Prefix for a given MSA Id
	fn get_tree_prefix(msa_id: MessageSourceId) -> Vec<u8> {
		let arr = [&msa_id.encode()[..], b"::"].concat();
		arr.to_vec()
	}
}
