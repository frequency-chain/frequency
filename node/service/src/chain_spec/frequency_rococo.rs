#![allow(missing_docs)]
use hex_literal::hex;

use cumulus_primitives_core::ParaId;
use frequency_runtime::{AccountId, AuraId, SudoConfig, EXISTENTIAL_DEPOSIT};
use sc_service::ChainType;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<frequency_runtime::GenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

use super::{get_properties, Extensions};

// pub fn load_frequency_rococo_spec() -> Result<ChainSpec, String> {
// 	ChainSpec::from_json_bytes(&include_bytes!("../../specs/frequency_rococo.json")[..])
// }

pub fn frequency_rococo_testnet() -> ChainSpec {
	let properties = get_properties("UNIT", 12, 42);
	let para_id: ParaId = 4044.into();
	ChainSpec::from_genesis(
		// Name
		"Frequency",
		// ID
		"frequency_rococo",
		ChainType::Live,
		move || {
			frequency_rococo_genesis(
				// initial collators.
				vec![],
				Some(
					hex!["a4c16e3ce8b11a0e43102bc4a718e9f031df2d0f61b1b03031b2f45ed664f853"].into(),
				),
				vec![],
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
) -> frequency_runtime::GenesisConfig {
	frequency_runtime::GenesisConfig {
		system: frequency_runtime::SystemConfig {
			code: frequency_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: frequency_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		parachain_info: frequency_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: frequency_runtime::CollatorSelectionConfig {
			invulnerables: initial_authorities.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: frequency_runtime::SessionConfig {
			keys: initial_authorities
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                             // account id
						acc,                                     // validator id
						frequency_runtime::SessionKeys { aura }, // session keys
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
		polkadot_xcm: frequency_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
		schemas: Default::default(),
	}
}
