//! Frequency Client library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

#[cfg(not(any(
	feature = "frequency",
	feature = "frequency-rococo-local",
	feature = "frequency-rococo-testnet"
)))]
compile_error!(
	r#"You must enable one of these features:
- Mainnet: "frequency"
- Frequency Rococo: "frequency-rococo-testnet"
- Local: "frequency-rococo-local"
- All: "all-frequency-features""#
);

pub mod chain_spec;
mod frequency_rpc;
pub mod rpc;
pub mod service;
