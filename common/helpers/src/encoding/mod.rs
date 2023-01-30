/// Encoding helpers
pub mod traits;
/// export encoding primitive types.
pub use traits::*;
/// Protocol buffer encoding
pub mod protocol_buf;

#[cfg(test)]
mod tests;
