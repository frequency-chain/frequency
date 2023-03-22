use frame_support::BoundedVec;
use sp_core::ConstU32;

/// The minimum base handle (not including suffix or delimiter) length in characters
pub const HANDLE_BASE_CHARS_MIN: u32 = 2;
/// The minimum base handle (not including suffix or delimiter) length in bytes
pub const HANDLE_BASE_BYTES_MIN: u32 = 4 * HANDLE_BASE_CHARS_MIN;
/// The maximum base handle (not including suffix or delimiter) length in characters
pub const HANDLE_BASE_CHARS_MAX: u32 = 30;
/// The maximum base handle (not including suffix or delimiter) length in bytes
pub const HANDLE_BASE_BYTES_MAX: u32 = 4 * HANDLE_BASE_CHARS_MAX;

/// The base handle in canonical form
pub type CanonicalBaseHandle = BoundedVec<u8, ConstU32<HANDLE_BASE_BYTES_MAX>>;
/// The handle suffix
pub type HandleSuffix = u16;

/// The cursor into the shuffled suffix sequence
pub type SequenceCursor = u32;

/// The handle that is displayed in a user interface
pub type HandleDisplayName = BoundedVec<u8, ConstU32<HANDLE_BASE_BYTES_MAX>>;
