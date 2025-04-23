#![allow(missing_docs)]
use polkadot_service::chain_spec::Extensions as RelayChainExtensions;

use super::Extensions;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

// Generic chain spec, in case when we don't have the native runtime.
pub type RelayChainSpec = sc_service::GenericChainSpec<RelayChainExtensions>;

#[allow(clippy::unwrap_used)]
/// Generates the Frequency Paseo chain spec from the raw json
pub fn load_frequency_paseo_spec() -> ChainSpec {
	ChainSpec::from_json_bytes(
		&include_bytes!("../../../../resources/frequency-paseo.raw.json")[..],
	)
	.unwrap()
}

// TODO: Remove once on a Polkadot-SDK with Paseo
#[allow(clippy::unwrap_used)]
/// Generates the Paseo Relay chain spec from the json
pub fn load_paseo_spec() -> RelayChainSpec {
	RelayChainSpec::from_json_bytes(&include_bytes!("../../../../resources/paseo.json")[..])
		.unwrap()
}
