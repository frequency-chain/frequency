#![allow(missing_docs)]
use common_runtime::constants::{Ss58Prefix, FREQUENCY_TOKEN, TOKEN_DECIMALS};
use cumulus_primitives_core::ParaId;

use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

use super::{get_properties, Extensions};

#[allow(clippy::unwrap_used)]
pub fn load_frequency_spec() -> ChainSpec {
	ChainSpec::from_json_bytes(&include_bytes!("../../../../resources/frequency.raw.json")[..])
		.unwrap()
}

#[allow(clippy::unwrap_used)]
fn frequency_genesis_config() -> serde_json::Value {
	let mut output: serde_json::Value =
		serde_json::from_slice(include_bytes!("../../../../resources/frequency.json").as_slice())
			.map_err(|e| format!("Invalid JSON blob {:?}", e))
			.unwrap();

	if let Some(runtime) = output["genesis"]["runtime"].as_object_mut() {
		runtime.remove("vesting");
		runtime["parachainSystem"] = serde_json::json!({});
		runtime["treasury"] = serde_json::json!({});
		runtime["auraExt"] = serde_json::json!({});
	}

	if let Some(system) = output["genesis"]["runtime"]["system"].as_object_mut() {
		system.remove("code");
	}

	let runtime = &output["genesis"]["runtime"];
	runtime.clone()
}

#[allow(clippy::unwrap_used)]
pub fn benchmark_mainnet_config() -> ChainSpec {
	let properties =
		get_properties(FREQUENCY_TOKEN, TOKEN_DECIMALS as u32, Ss58Prefix::get().into());
	let para_id: ParaId = 2091.into();
	ChainSpec::builder(
		frequency_runtime::wasm_binary_unwrap(),
		Extensions { relay_chain: "polkadot".into(), para_id: para_id.into() },
	).with_name(
		"Frequency",
	).with_protocol_id(
		"frequency",
	).with_properties(
		properties,
	).with_chain_type(
		ChainType::Live
	).with_telemetry_endpoints(
		TelemetryEndpoints::new(vec![("wss://telemetry.polkadot.io/submit/".into(), 0), ("wss://telemetry.frequency.xyz/submit/".into(), 0)]).unwrap()	
	).with_boot_nodes(vec![
		"/dns4/0.boot.frequency.xyz/tcp/30333/ws/p2p/12D3KooWBd4aEArNvXECtt2JHQACBdFmeafpyfre3q81iM1xCcpP".parse().unwrap(),
		"/dns4/1.boot.frequency.xyz/tcp/30333/ws/p2p/12D3KooWCW8d7Yz2d3Jcb49rWcNppRNEs1K2NZitCpPtrHSQb6dw".parse().unwrap(),
	])
	.with_genesis_config_patch(frequency_genesis_config())
	.build()
}
