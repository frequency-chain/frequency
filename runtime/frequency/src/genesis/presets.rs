use scale_info::prelude::format;

#[cfg(any(
	feature = "frequency-no-relay",
	feature = "frequency-local",
	feature = "frequency-lint-check"
))]
use crate::genesis::helpers::{
	default_council_members, default_endowed_accounts, default_invulnerables, default_session_keys,
	default_technical_committee_members,
};

#[cfg(any(
	feature = "frequency-no-relay",
	feature = "frequency-local",
	feature = "frequency-lint-check"
))]
fn development_genesis_config() -> serde_json::Value {
	super::helpers::build_genesis(
		default_invulnerables(),
		crate::EXISTENTIAL_DEPOSIT * 16,
		default_endowed_accounts(),
		default_session_keys(),
		default_council_members(),
		default_technical_committee_members(),
		super::helpers::load_genesis_schemas(),
		1000.into(),
	)
}

#[cfg(any(
	feature = "frequency-no-relay",
	feature = "frequency-local",
	feature = "frequency-lint-check"
))]
fn frequency_local_genesis_config() -> serde_json::Value {
	super::helpers::build_genesis(
		default_invulnerables(),
		crate::EXISTENTIAL_DEPOSIT * 16,
		default_endowed_accounts(),
		default_session_keys(),
		default_council_members(),
		default_technical_committee_members(),
		super::helpers::load_genesis_schemas(),
		2000.into(),
	)
}

#[cfg(feature = "frequency-testnet")]
#[allow(clippy::unwrap_used)]
fn frequency_testnet_genesis_config() -> serde_json::Value {
	let output: serde_json::Value = serde_json::from_slice(
		include_bytes!("../../../../resources/frequency-paseo.json").as_slice(),
	)
	.map_err(|e| format!("Invalid JSON blob {:?}", e))
	.unwrap();

	let runtime = &output["genesis"]["runtime"];
	runtime.clone()
}

#[allow(clippy::unwrap_used)]
fn frequency_genesis_config() -> serde_json::Value {
	#[cfg(not(feature = "runtime-benchmarks"))]
	{
		let output: serde_json::Value = serde_json::from_slice(
			include_bytes!("../../../../resources/frequency.json").as_slice(),
		)
		.map_err(|e| format!("Invalid JSON blob {:?}", e))
		.unwrap();

		// Return the unmodified output when `runtime-benchmarks` is not enabled
		return output["genesis"]["runtime"].clone();
	}

	#[cfg(feature = "runtime-benchmarks")]
	{
		let mut output: serde_json::Value = serde_json::from_slice(
			include_bytes!("../../../../resources/frequency.json").as_slice(),
		)
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

		return output["genesis"]["runtime"].clone();
	}
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &sp_genesis_builder::PresetId) -> Option<sp_std::vec::Vec<u8>> {
	let genesis = match id.as_str() {
		#[cfg(any(
			feature = "frequency-no-relay",
			feature = "frequency-local",
			feature = "frequency-lint-check"
		))]
		"development" => development_genesis_config(),
		#[cfg(any(
			feature = "frequency-no-relay",
			feature = "frequency-local",
			feature = "frequency-lint-check"
		))]
		"frequency-local" => frequency_local_genesis_config(),
		#[cfg(feature = "frequency-testnet")]
		"frequency-testnet" => frequency_testnet_genesis_config(),
		"frequency" => frequency_genesis_config(),
		_ => return None,
	};
	Some(
		serde_json::to_string(&genesis)
			.unwrap_or_else(|err| {
				format!("Serialization to json is expected to work. Error: {:?}", err)
			})
			.into_bytes(),
	)
}
