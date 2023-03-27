use crate::msa::MessageSourceId;
use codec::{Decode, Encode};
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use sp_core::ConstU32;

/// The minimum base handle (not including suffix or delimiter) length in characters
pub const HANDLE_BASE_CHARS_MIN: u32 = 3;
/// The minimum base handle (not including suffix or delimiter) length in bytes
pub const HANDLE_BASE_BYTES_MIN: u32 = 1 * HANDLE_BASE_CHARS_MIN;
/// The maximum base handle (not including suffix or delimiter) length in characters
pub const HANDLE_BASE_CHARS_MAX: u32 = 20;
/// The maximum base handle (not including suffix or delimiter) length in bytes
pub const HANDLE_BASE_BYTES_MAX: u32 = 32; // Hard limit of 32 bytes

/// A handle (base, canonical, or display)
pub type Handle = BoundedVec<u8, ConstU32<HANDLE_BASE_BYTES_MAX>>;

/// The handle suffix
pub type HandleSuffix = u16;

/// The cursor into the shuffled suffix sequence
pub type SequenceIndex = u16;

/// Claim handle payload
#[derive(TypeInfo, Clone, Debug, Decode, Encode, PartialEq, Eq)]
pub struct ClaimHandlePayload {
	/// MSA id of the delegator claiming the handle
	pub owner_msa_id: MessageSourceId,
	/// The desired base handle
	pub base_handle: Handle,
}

impl ClaimHandlePayload {
	/// Create a new payload for claiming a handle
	pub fn new(owner_msa_id: MessageSourceId, base_handle: Handle) -> Self {
		Self { owner_msa_id, base_handle }
	}
}
