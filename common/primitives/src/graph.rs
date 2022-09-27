use crate::msa::MessageSourceId;
use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{traits::ConstU32, BoundedVec};

/// graph page max size
pub const GRAPH_PAGE_BYTES: u32 = 2048;
/// public page items
pub const PUBLIC_PAGE_ITEMS: u32 = GRAPH_PAGE_BYTES / 8;
/// Public Page type
pub type PublicPage = BoundedVec<MessageSourceId, ConstU32<PUBLIC_PAGE_ITEMS>>;
/// Private Page type
pub type PrivatePage = BoundedVec<u8, ConstU32<GRAPH_PAGE_BYTES>>;
/// connection permission
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Ord, Eq, PartialOrd)]
pub struct Permission {
	/// permission type
	pub data: u16,
}
/// type of graph
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo, PartialEq)]
pub enum GraphType {
	/// public graph
	Public,
	/// private graph
	Private,
}
/// graph key
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Ord, Eq, PartialOrd)]
pub struct GraphKey {
	/// permission type
	pub permission: Permission,
	/// page number
	pub page: u16,
}
