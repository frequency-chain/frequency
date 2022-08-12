#![allow(missing_docs)]
use cumulus_primitives_core::ParaId;
use frequency_rococo_runtime::{AccountId, AuraId, SudoConfig, EXISTENTIAL_DEPOSIT};
use hex::FromHex;
use sc_service::ChainType;
use sp_core::ByteArray;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec =
	sc_service::GenericChainSpec<frequency_rococo_runtime::GenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

use super::{get_properties, Extensions};

pub mod public_testnet_keys {
	pub const COLLATOR_1_SR25519: &str =
		"0x42ec8b3541d5647e46711e8f109ed51838cdde115a0a7e56622c033d7e04a678";
	pub const COLLATOR_2_SR25519: &str =
		"0x58025fbd92bcc65fc2e054b623d10a8558662ae545a9718e26815024a2433545";
	pub const ROCOCO_FRQ_SUDO: &str =
		"0xa4c16e3ce8b11a0e43102bc4a718e9f031df2d0f61b1b03031b2f45ed664f853";
}

// pub fn load_frequency_rococo_spec() -> Result<ChainSpec, String> {
// 	ChainSpec::from_json_bytes(&include_bytes!("../../specs/frequency_rococo.json")[..])
// }

pub fn frequency_rococo_testnet() -> ChainSpec {
	let properties = get_properties("UNIT", 12, 42);
	let para_id: ParaId = 4044.into();
	ChainSpec::from_genesis(
		// Name
		"Frequency Rococo",
		// ID
		"frequency_rococo",
		ChainType::Live,
		move || {
			frequency_rococo_genesis(
				// initial collators.
				vec![
					(
						// 5E9QpPsz28HM1JM7AMnDS2aRN5unyf8EuNtz96eW4KvmxRrV
						public_testnet_keys::COLLATOR_1_SR25519
							.parse::<AccountId>()
							.unwrap()
							.into(),
						AuraId::from_slice(
							&<[u8; 32]>::from_hex(
								public_testnet_keys::COLLATOR_1_SR25519.strip_prefix("0x").unwrap(),
							)
							.unwrap(),
						)
						.unwrap(),
					),
					(
						// 5CntRvAGYzzorsvN3UKotz5gpFd5BgMwUzALKtbWGn3JsQAu
						public_testnet_keys::COLLATOR_2_SR25519
							.parse::<AccountId>()
							.unwrap()
							.into(),
						AuraId::from_slice(
							&<[u8; 32]>::from_hex(
								public_testnet_keys::COLLATOR_2_SR25519.strip_prefix("0x").unwrap(),
							)
							.unwrap(),
						)
						.unwrap(),
					),
				],
				Some(public_testnet_keys::ROCOCO_FRQ_SUDO.parse::<AccountId>().unwrap().into()),
				// endowed accounts.
				vec![
					// 5E9QpPsz28HM1JM7AMnDS2aRN5unyf8EuNtz96eW4KvmxRrV
					public_testnet_keys::COLLATOR_1_SR25519.parse::<AccountId>().unwrap().into(),
					// 5CntRvAGYzzorsvN3UKotz5gpFd5BgMwUzALKtbWGn3JsQAu
					public_testnet_keys::COLLATOR_2_SR25519.parse::<AccountId>().unwrap().into(),
				],
				para_id,
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("frequency-rococo"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "rococo".into(), // You MUST set this to the correct network!
			para_id: para_id.into(),
		},
	)
}

fn frequency_rococo_genesis(
	initial_authorities: Vec<(AccountId, AuraId)>,
	root_key: Option<AccountId>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> frequency_rococo_runtime::GenesisConfig {
	frequency_rococo_runtime::GenesisConfig {
		system: frequency_rococo_runtime::SystemConfig {
			code: frequency_rococo_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: frequency_rococo_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		parachain_info: frequency_rococo_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: frequency_rococo_runtime::CollatorSelectionConfig {
			invulnerables: initial_authorities.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: frequency_rococo_runtime::SessionConfig {
			keys: initial_authorities
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                                    // account id
						acc,                                            // validator id
						frequency_rococo_runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		sudo: SudoConfig {
			// Assign network admin rights.
			key: root_key,
		},
		polkadot_xcm: frequency_rococo_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
		schemas: Default::default(),
		vesting: Default::default(),
	}
}
