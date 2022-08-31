#![allow(missing_docs)]
use cumulus_primitives_core::ParaId;
use frequency_runtime::{AccountId, AuraId, Balance, SudoConfig, CouncilConfig, TechnicalCommitteeConfig, EXISTENTIAL_DEPOSIT};
use sc_service::ChainType;
use sp_core::sr25519;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<frequency_runtime::GenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

use super::{get_account_id_from_seed, get_collator_keys_from_seed, get_properties, Extensions};

//TODO: Define various keys for frequency mainnet
pub mod frequency_mainnet_keys {}

// pub fn load_frequency_spec() -> Result<ChainSpec, String> {
// 	ChainSpec::from_json_bytes(&include_bytes!("../../specs/frequency.json")[..])
// }

pub fn frequency() -> ChainSpec {
	let properties = get_properties("UNIT", 12, 42);
	// TODO need a paraid here for the frequency chain
	let para_id: ParaId = 999.into();
	ChainSpec::from_genesis(
		// Name
		"Frequency",
		// ID
		"frequency",
		ChainType::Live,
		move || {
			frequency_genesis(
				// TODO: initial collators.
				vec![(
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_collator_keys_from_seed("Alice"),
				)],
				//TODO: sudo key(s) for mainnet
				Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
				// TODO:: endowed accounts with initial balance.
				vec![(get_account_id_from_seed::<sr25519::Public>("Alice"), 1 << 60)],
				// TODO: initial council members
				Default::default(),
				// TODO: initial technical committee members
				Default::default(),
				// TODO: candidacy bond (if needed)
				EXISTENTIAL_DEPOSIT * 16,
				// TODO: include council/democracy/staking related inputs
				para_id,
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("frequency"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			//TODO: set the correct network
			relay_chain: "TODO".into(),
			para_id: para_id.into(),
		},
	)
}

fn frequency_genesis(
	initial_authorities: Vec<(AccountId, AuraId)>,
	root_key: Option<AccountId>,
	endowed_accounts: Vec<(AccountId, Balance)>,
	council_members: Vec<AccountId>,
	technical_committee_members: Vec<AccountId>,
	candidacy_bond: Balance,
	id: ParaId,
) -> frequency_runtime::GenesisConfig {
	frequency_runtime::GenesisConfig {
		system: frequency_runtime::SystemConfig {
			code: frequency_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
		},
		balances: frequency_runtime::BalancesConfig { balances: endowed_accounts },
		parachain_info: frequency_runtime::ParachainInfoConfig { parachain_id: id },
		collator_selection: frequency_runtime::CollatorSelectionConfig {
			invulnerables: initial_authorities.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond,
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
		vesting: Default::default(),
		democracy: Default::default(),
		council: CouncilConfig { phantom: Default::default(), members: council_members },
		technical_committee: TechnicalCommitteeConfig { phantom: Default::default(), members: technical_committee_members },
	}
}
