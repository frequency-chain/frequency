extern crate alloc;
use alloc::vec::Vec;

/// A handle (base, canonical, or display)
// ORIGINAL is pub type Handle = BoundedVec<u8, ConstU32<HANDLE_BASE_BYTES_MAX>>;
pub type Handle = Vec<u8>;

/// The handle suffix
pub type HandleSuffix = u16;
