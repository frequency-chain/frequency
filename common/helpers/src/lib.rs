pub mod avro;
#[cfg(test)]
mod avro_tests;
/// Structs and traits specifically for RPC calls.
#[cfg(feature = "std")]
/// export rpc primitive types.
pub mod rpc;
pub mod types;
