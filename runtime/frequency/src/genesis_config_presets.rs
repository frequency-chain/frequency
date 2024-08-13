use crate::development_genesis::*;
use scale_info::prelude::format;

fn development_genesis_config() -> serde_json::Value {
	development_genesis(
		development_invulnerables(),
		development_endowed_accounts(),
		development_council_members(),
		development_technical_committee_members(),
		1000.into(),
	)
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &sp_genesis_builder::PresetId) -> Option<sp_std::vec::Vec<u8>> {
	let genesis = match id.try_into() {
		Ok("development") => development_genesis_config(),
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
