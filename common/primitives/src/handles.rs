use crate::msa::MessageSourceId;
#[cfg(feature = "std")]
use crate::utils::*;
use frame_support::BoundedVec;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::ConstU32;
use sp_std::vec::Vec;

/// The minimum base and canonical handle (not including suffix or delimiter) length in characters
pub const HANDLE_CHARS_MIN: u32 = 3;
/// The minimum base and canonical handle (not including suffix or delimiter) length in bytes
pub const HANDLE_BYTES_MIN: u32 = 1 * HANDLE_CHARS_MIN;
/// The maximum base and canonical handle (not including suffix or delimiter) length in characters
pub const HANDLE_CHARS_MAX: u32 = 20;
/// The maximum base and canonical handle (not including suffix or delimiter) length in bytes
pub const HANDLE_BYTES_MAX: u32 = 32; // Hard limit of 32 bytes
/// The maximum number of digits in a suffix
pub const SUFFIX_MAX_DIGITS: usize = 5; // The max value of a HandleSuffix (u16) is 65535 which is 5 digits.
/// The maximum count of suffixes allowed to be requested at once
pub const MAX_SUFFIXES_COUNT: u16 = 100;
/// The default count of suffixes to request if none is provided
pub const DEFAULT_SUFFIX_COUNT: u16 = 1;
/// A generic handle type  (base, canonical, or display)
type Handle = BoundedVec<u8, ConstU32<HANDLE_BYTES_MAX>>;
/// A base handle, which is chosen by the user
pub type BaseHandle = Handle;
/// A canonical base, which is a reduced/translated version of the base handle
pub type CanonicalBase = Handle;
/// A display handle, which is a base handle with suffix separated by a delimiter
pub type DisplayHandle =
	BoundedVec<u8, ConstU32<{ HANDLE_BYTES_MAX + SUFFIX_MAX_DIGITS as u32 + 1u32 }>>;
/// The handle suffix
pub type HandleSuffix = u16;

/// The handle suffix range type
pub type SuffixRangeType = u16;

/// The cursor into the shuffled suffix sequence
pub type SequenceIndex = u16;

/// Claim handle payload
#[derive(TypeInfo, Clone, Debug, Decode, Encode, PartialEq, Eq)]
pub struct ClaimHandlePayload<BlockNumber> {
	/// The desired base handle
	pub base_handle: Vec<u8>,
	/// The block number at which the proof for grant_delegation expires.
	pub expiration: BlockNumber,
}

impl<BlockNumber> ClaimHandlePayload<BlockNumber> {
	/// Create a new ClaimHandlePayload
	pub fn new(base_handle: Vec<u8>, expiration: BlockNumber) -> Self {
		ClaimHandlePayload { base_handle, expiration }
	}
}

/// RPC Response form for a Handle
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct HandleResponse {
	/// Base handle (without delimiter or suffix)
	#[cfg_attr(feature = "std", serde(with = "as_string"))]
	pub base_handle: Vec<u8>,
	/// Canonical handle (reduced/translated version of base)
	#[cfg_attr(feature = "std", serde(with = "as_string"))]
	pub canonical_base: Vec<u8>,
	/// Suffix
	pub suffix: HandleSuffix,
}

/// A behavior that allows for retrieving a `Handle` for a given `MessageSourceAccount`
pub trait HandleProvider {
	/// Validate a handle for a given `MessageSourceAccount`
	fn get_handle_for_msa(key: MessageSourceId) -> Option<HandleResponse>;
}

/// Blanket implementation for testing.
impl HandleProvider for () {
	fn get_handle_for_msa(_key: MessageSourceId) -> Option<HandleResponse> {
		None
	}
}

/// Output response for retrieving the next suffixes for a given handle
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct PresumptiveSuffixesResponse {
	/// The base handle
	#[cfg_attr(feature = "std", serde(with = "as_string"))]
	pub base_handle: Vec<u8>,
	/// The suffixes
	pub suffixes: Vec<HandleSuffix>,
}

/// Output response for retrieving the next suffixes for a given handle
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct CheckHandleResponse {
	/// The base handle
	#[cfg_attr(feature = "std", serde(with = "as_string"))]
	pub base_handle: Vec<u8>,
	/// The canonical handle
	#[cfg_attr(feature = "std", serde(with = "as_string"))]
	pub canonical_base: Vec<u8>,
	/// The current suffix index
	pub suffix_index: u16,
	/// Are additional suffixes available?
	pub suffixes_available: bool,
	/// Validity
	pub valid: bool,
}

impl Default for CheckHandleResponse {
	fn default() -> Self {
		Self {
			base_handle: Vec::new(),
			canonical_base: Vec::new(),
			suffix_index: 0,
			suffixes_available: false,
			valid: false,
		}
	}
}
