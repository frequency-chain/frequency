//! Frequency Client library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

// // Don't allow both frequency and all-frequency-features so that we always have a good mainnet runtime
// #[cfg(all(feature = "frequency", feature = "all-frequency-features"))]
// compile_error!("feature \"frequency\" and feature \"all-frequency-features\" cannot be enabled at the same time");

// // Don't allow both frequency-no-relay and all-frequency-features so that we always have a good mainnet runtime
// #[cfg(all(feature = "frequency-no-relay", feature = "all-frequency-features"))]
// compile_error!("feature \"frequency-no-relay\" and feature \"all-frequency-features\" cannot be enabled at the same time");

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
- All: "all-frequency-features""#
);

pub mod chain_spec;
pub mod rpc;
pub mod service;
