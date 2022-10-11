#![allow(missing_docs)]
use common_primitives::node::{AccountId, Balance};
use common_runtime::constants::{FREQUENCY_TOKEN, UNIT};
use cumulus_primitives_core::ParaId;
use frequency_runtime::{
	AuraId, CouncilConfig, SudoConfig, TechnicalCommitteeConfig, EXISTENTIAL_DEPOSIT,
};

use hex::FromHex;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_core::ByteArray;
use sp_runtime::traits::AccountIdConversion;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<frequency_runtime::GenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

use super::{get_properties, Extensions};

//TODO: Define FINAL keys for frequency mainnet
pub mod frequency_mainnet_keys {
	//TODO: final sudo key(s) for mainnet
	pub const MAINNET_FRQ_SUDO: &str =
		"0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"; // Alice

	//TODO: final collator key(s) for mainnet
	pub const COLLATOR_1_SR25519: &str =
		"0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"; // Alice
	pub const COLLATOR_2_SR25519: &str =
		"0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"; // Bob
	pub const COLLATOR_3_SR25519: &str =
		"0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22"; // Charlie
	pub const COLLATOR_4_SR25519: &str =
		"0x306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20"; // Dave
	pub const COLLATOR_5_SR25519: &str =
		"0xe659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e"; // Eve
}

// pub fn load_frequency_spec() -> Result<ChainSpec, String> {
// 	ChainSpec::from_json_bytes(&include_bytes!("../../specs/frequency.json")[..])
// }

pub fn frequency() -> ChainSpec {
	let properties = get_properties(FREQUENCY_TOKEN, 8, 42);
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
				vec![
					(
						frequency_mainnet_keys::COLLATOR_1_SR25519
							.parse::<AccountId>()
							.unwrap()
							.into(),
						AuraId::from_slice(
							&<[u8; 32]>::from_hex(
								frequency_mainnet_keys::COLLATOR_1_SR25519
									.strip_prefix("0x")
									.unwrap(),
							)
							.unwrap(),
						)
						.unwrap(),
					),
					(
						frequency_mainnet_keys::COLLATOR_2_SR25519
							.parse::<AccountId>()
							.unwrap()
							.into(),
						AuraId::from_slice(
							&<[u8; 32]>::from_hex(
								frequency_mainnet_keys::COLLATOR_2_SR25519
									.strip_prefix("0x")
									.unwrap(),
							)
							.unwrap(),
						)
						.unwrap(),
					),
					(
						frequency_mainnet_keys::COLLATOR_3_SR25519
							.parse::<AccountId>()
							.unwrap()
							.into(),
						AuraId::from_slice(
							&<[u8; 32]>::from_hex(
								frequency_mainnet_keys::COLLATOR_3_SR25519
									.strip_prefix("0x")
									.unwrap(),
							)
							.unwrap(),
						)
						.unwrap(),
					),
					(
						frequency_mainnet_keys::COLLATOR_4_SR25519
							.parse::<AccountId>()
							.unwrap()
							.into(),
						AuraId::from_slice(
							&<[u8; 32]>::from_hex(
								frequency_mainnet_keys::COLLATOR_4_SR25519
									.strip_prefix("0x")
									.unwrap(),
							)
							.unwrap(),
						)
						.unwrap(),
					),
					(
						frequency_mainnet_keys::COLLATOR_5_SR25519
							.parse::<AccountId>()
							.unwrap()
							.into(),
						AuraId::from_slice(
							&<[u8; 32]>::from_hex(
								frequency_mainnet_keys::COLLATOR_5_SR25519
									.strip_prefix("0x")
									.unwrap(),
							)
							.unwrap(),
						)
						.unwrap(),
					),
				],
				// Sudo Account
				Some(frequency_mainnet_keys::MAINNET_FRQ_SUDO.parse::<AccountId>().unwrap().into()),
				// TODO:: endowed accounts with initial balance.
				vec![
					(
						frequency_mainnet_keys::MAINNET_FRQ_SUDO
							.parse::<AccountId>()
							.unwrap()
							.into(),
						1 << 60,
					),
					(
						common_runtime::constants::TREASURY_PALLET_ID.into_account_truncating(),
						EXISTENTIAL_DEPOSIT,
					),
				],
				// TODO: initial council members
				Default::default(),
				// TODO: initial technical committee members
				Default::default(),
				//candidacy bond
				100_000 * UNIT,
				// TODO: include council/democracy/staking related inputs
				para_id,
			)
		},
		// Bootnodes
		vec![
			"/dns/0.boot.frequency.xyz/tcp/30333/p2p/12D3KooWBd4aEArNvXECtt2JHQACBdFmeafpyfre3q81iM1xCcpP".parse().unwrap(),
			"/dns/1.boot.frequency.xyz/tcp/30333/p2p/12D3KooWCW8d7Yz2d3Jcb49rWcNppRNEs1K2NZitCpPtrHSQb6dw".parse().unwrap(),
		],
		// Telemetry
		TelemetryEndpoints::new(vec![("wss://telemetry.polkadot.io/submit/".into(), 0)]).ok(),
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
			desired_candidates: 0,
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
		treasury: Default::default(),
		council: CouncilConfig { phantom: Default::default(), members: council_members },
		technical_committee: TechnicalCommitteeConfig {
			phantom: Default::default(),
			members: technical_committee_members,
		},
	}
}
