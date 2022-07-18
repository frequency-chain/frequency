use codec::{Decode, Encode};
use common_primitives::msa::MessageSourceId;
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_std::{
	cmp::{Ord, Ordering},
	prelude::*,
};

/// Graph Edge
#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Eq, PartialOrd)]
#[scale_info(skip_type_params(T))]
pub struct Edge {
	/// neighbor id
	pub static_id: MessageSourceId,
	/// connection permission
	pub permission: Permission,
}

impl Ord for Edge {
	fn cmp(&self, other: &Self) -> Ordering {
		self.static_id.cmp(&other.static_id)
	}
}

/// Graph Node
#[derive(Default, Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Eq)]
#[scale_info(skip_type_params(T))]
pub struct Node {
	// going to add node related fields here
}

/// Graph Edge Permission
#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Ord, Eq, PartialOrd)]
pub struct Permission {
	/// permission details
	pub data: u16,
}
