#![allow(missing_docs)]
use polkadot_service::chain_spec::Extensions as RelayChainExtensions;

use super::Extensions;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

// Generic chain spec, in case when we don't have the native runtime.
pub type RelayChainSpec = sc_service::GenericChainSpec<RelayChainExtensions>;

#[allow(clippy::unwrap_used)]
/// Generates the Frequency Westend chain spec from the raw json
pub fn load_frequency_westend_spec() -> ChainSpec {
    ChainSpec::from_json_bytes(
        &include_bytes!("../../../../resources/frequency-westend.raw.json")[..],
    )
    .unwrap()
}

#[allow(clippy::unwrap_used)]
/// Generates the Westend Relay chain spec from the json
pub fn load_westend_spec() -> RelayChainSpec {
    RelayChainSpec::from_json_bytes(&include_bytes!("../../../../resources/westend.json")[..])
        .unwrap()
}
