use codec::{Decode, Encode, MaxEncodedLen};
use common_primitives::msa::MessageSourceId;
use frame_support::{pallet_prelude::ConstU32, BoundedVec, RuntimeDebug};
use scale_info::TypeInfo;
use sp_std::{
	cmp::{Ord, Ordering},
	prelude::*,
};

/// account if type
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
/// storage key type
pub type StorageKey = BoundedVec<u8, ConstU32<4096>>;
/// trie id type
pub type TrieId = BoundedVec<u8, ConstU32<128>>;

/// graph edge
#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Eq, PartialOrd)]
#[scale_info(skip_type_params(T))]
pub struct Edge {
	/// target node id
	pub static_id: MessageSourceId,
	/// connection permission
	pub permission: Permission,
}

impl Ord for Edge {
	fn cmp(&self, other: &Self) -> Ordering {
		self.static_id.cmp(&other.static_id)
	}
}

/// graph node
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub struct Node {
	/// Unique ID for the subtree encoded as a bytes vector.
	pub trie_id: TrieId,
}

/// connection permission
#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Ord, Eq, PartialOrd)]
pub struct Permission {
	/// permission type
	pub data: u16,
}
