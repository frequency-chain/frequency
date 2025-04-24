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

// TODO: Remove once on a Polkadot-SDK with Paseo-Local
#[allow(clippy::unwrap_used)]
/// Generates the Paseo-Local Relay chain spec from the json
pub fn load_paseo_local_spec() -> RelayChainSpec {
	RelayChainSpec::from_json_bytes(&include_bytes!("../../../../resources/paseo-local.json")[..])
		.unwrap()
}

/// Generates the chain spec for a local testnet
pub fn local_paseo_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let properties =
		get_properties(FREQUENCY_TESTNET_TOKEN, TOKEN_DECIMALS as u32, Ss58Prefix::get().into());

	ChainSpec::builder(
		frequency_runtime::wasm_binary_unwrap(),
		Extensions {
			relay_chain: "paseo-local".into(), // You MUST set this to the correct network!
			para_id: 2000,
		},
	)
	.with_name("Frequency Local Testnet")
	.with_protocol_id("frequency-paseo-local")
	.with_properties(properties)
	.with_chain_type(ChainType::Local)
	.with_genesis_config_preset_name("frequency-local")
	.build()
}
