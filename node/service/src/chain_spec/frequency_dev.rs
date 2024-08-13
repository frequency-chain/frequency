#![allow(missing_docs)]
use common_runtime::constants::{FREQUENCY_LOCAL_TOKEN, TOKEN_DECIMALS};
use frequency_runtime::Ss58Prefix;
use sc_service::ChainType;
/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

use super::{get_properties, Extensions};

/// Generates the chain spec for a development (no relay)
pub fn development_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let properties =
		get_properties(FREQUENCY_LOCAL_TOKEN, TOKEN_DECIMALS as u32, Ss58Prefix::get().into());

	ChainSpec::builder(
		frequency_runtime::wasm_binary_unwrap(),
		Extensions {
			relay_chain: "dev".into(), // You MUST set this to the correct network!
			para_id: 1000,
		},
	)
	.with_name("Frequency Development (No Relay)")
	.with_id("dev")
	.with_properties(properties)
	.with_chain_type(ChainType::Development)
	.with_protocol_id("dev")
	.with_genesis_config_preset_name("development")
	.build()
}
