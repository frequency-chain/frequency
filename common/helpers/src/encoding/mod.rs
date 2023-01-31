/// Encoding helpers
pub mod traits;
/// export encoding primitive types.
pub use traits::*;
/// Avro encoding
pub mod avro_binary;
/// Protocol buffer encoding
pub mod protocol_buf;
/// Thrift encoding
pub mod thrift;

#[cfg(test)]
mod tests;
