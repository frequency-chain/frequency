use codec::{Decode, Encode, MaxEncodedLen};
use common_primitives::msa::MessageSenderId;
use frame_support::{pallet_prelude::ConstU32, BoundedVec, RuntimeDebug};
use scale_info::TypeInfo;
use sp_std::{
	cmp::{Ord, Ordering},
	prelude::*,
};

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type StorageKey = BoundedVec<u8, ConstU32<4096>>;
pub type TrieId = BoundedVec<u8, ConstU32<128>>;

#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Eq, PartialOrd)]
#[scale_info(skip_type_params(T))]
pub struct Edge {
	pub static_id: MessageSenderId,
	pub permission: Permission,
}

impl Ord for Edge {
	fn cmp(&self, other: &Self) -> Ordering {
		self.static_id.cmp(&other.static_id)
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Node {
	/// Unique ID for the subtree encoded as a bytes vector.
	pub trie_id: TrieId,
}

#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Ord, Eq, PartialOrd)]
pub struct Permission {
	pub data: u16,
}
