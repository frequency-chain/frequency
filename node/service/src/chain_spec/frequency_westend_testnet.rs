#![allow(missing_docs)]
use common_runtime::constants::{FREQUENCY_TESTNET_TOKEN, TOKEN_DECIMALS};
use frequency_runtime::Ss58Prefix;
use polkadot_service::chain_spec::Extensions as RelayChainExtensions;
use sc_service::ChainType;

use super::{get_properties, Extensions};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

// Generic chain spec, in case when we don't have the native runtime.
pub type RelayChainSpec = sc_service::GenericChainSpec<RelayChainExtensions>;

/// Generates the chain spec for a local testnet
pub fn local_westend_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let properties =
		get_properties(FREQUENCY_TESTNET_TOKEN, TOKEN_DECIMALS as u32, Ss58Prefix::get().into());

	ChainSpec::builder(
		frequency_runtime::wasm_binary_unwrap(),
		Extensions {
			relay_chain: "westend-local".into(), // You MUST set this to the correct network!
			para_id: 2313,
		},
	)
	.with_name("Frequency Westend Testnet")
	.with_id("frequency-westend-testnet")
	.with_protocol_id("frequency-westend-testnet")
	.with_properties(properties)
	.with_chain_type(ChainType::Live)
	.with_genesis_config_preset_name("frequency-westend-testnet")
	.build()
}
