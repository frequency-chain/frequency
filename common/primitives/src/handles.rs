#[cfg(feature = "std")]
use crate::utils::*;
use codec::{Decode, Encode};
use frame_support::BoundedVec;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::ConstU32;
use sp_std::vec::Vec;

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
	/// The desired base handle
	pub base_handle: Vec<u8>,
}

impl ClaimHandlePayload {
	/// Create a new payload for claiming a handle
	pub fn new(base_handle: Vec<u8>) -> Self {
		Self { base_handle }
	}
}

/// RPC Response form for a Handle
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct HandleResponse {
	/// Base handle (without delimiter or suffix
	#[cfg_attr(feature = "std", serde(with = "as_string"))]
	pub base_handle: Vec<u8>,
	/// Canonical handle (reduced/translated version of base)
	#[cfg_attr(feature = "std", serde(with = "as_string"))]
	pub canonical_handle: Vec<u8>,
	/// Suffix
	pub suffix: HandleSuffix,
}
