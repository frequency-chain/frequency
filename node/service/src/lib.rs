//! Frequency Client library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

#[cfg(not(any(
	feature = "frequency",
	feature = "frequency-rococo-local",
	feature = "frequency-no-relay",
	feature = "frequency-rococo-testnet"
)))]
compile_error!(
	r#"You must enable one of these features:
- Mainnet: "frequency"
- Frequency Rococo: "frequency-rococo-testnet"
- Local: "frequency-rococo-local"
- No Relay: "frequency-no-relay",
- All: "frequency-lint-check"#
);

pub mod chain_spec;
pub mod rpc;
pub mod service;
