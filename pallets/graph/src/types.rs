use codec::{Decode, Encode};
use common_primitives::msa::MessageSenderId;
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_std::{cmp::Ord, prelude::*};

#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Ord, Eq, PartialOrd)]
#[scale_info(skip_type_params(T))]
pub struct Edge {
	pub static_id: MessageSenderId,
}

#[derive(Default, Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, Eq)]
#[scale_info(skip_type_params(T))]
pub struct Node {
	// going to add node related fields here
}
