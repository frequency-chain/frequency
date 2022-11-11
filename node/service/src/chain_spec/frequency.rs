#![allow(missing_docs)]
use common_primitives::node::{AccountId, Balance};
use common_runtime::constants::{
	currency::{EXISTENTIAL_DEPOSIT, UNITS},
	FREQUENCY_TOKEN, TOKEN_DECIMALS,
};
use cumulus_primitives_core::ParaId;
use frequency_runtime::{AuraId, CouncilConfig, SS58Prefix, SudoConfig, TechnicalCommitteeConfig};

use hex::FromHex;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_core::ByteArray;
use sp_runtime::traits::AccountIdConversion;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<frequency_runtime::GenesisConfig, Extensions>;

use super::{get_properties, Extensions};

//TODO: Define FINAL keys for frequency mainnet
pub mod frequency_mainnet_keys {
	//TODO: final sudo key(s) for mainnet
	pub const MAINNET_FRQ_SUDO: &str =
		"0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"; // Alice

	//TODO: update collator key naming convention
	pub const UNFINISHED_COLLATOR_1_SR25519: &str =
		"0x40c0cf083ba1d081fce7c1f38f5344a4eba4ee5c13e1dddcb9cc6acf74559710"; // Unfinished Labs Collator 1
	pub const UNFINISHED_COLLATOR_2_SR25519: &str =
		"0x1e5dc06d8418a0dd695bf3a23ecdc219103a149a863c1977396bf0697a474015"; // Unfinished Labs Collator 2
	pub const UNFINISHED_COLLATOR_3_SR25519: &str =
		"0x7eb4de08f44e6077ed072e2fc357aac007150e3b73373434c47ec56323b52a75"; // Unfinished Labs Collator 3
	pub const ON_FINALITY_COLLATOR_1_SR25519: &str =
		"0x3c9ea854901c1b36bd203400c90c31a69367316f7deaeeb14037f2be44bcb118"; // OnFinality Collator 1
	pub const EXTERNAL_COLLATOR_1_SR25519: &str =
		"0x3c33f8c4c1155958d60d4d52a5181677da96052493d876b747272311b52cfe61"; // External Collator 1
	pub const EXTERNAL_COLLATOR_2_SR25519: &str =
		"0x36523fba06e0f31f35dd904386a0ebe25d505ef09af4d39041afa1d994a23f5d"; // External Collator 2

	pub const UNFINISHED_COLLATOR_1_SESSION_KEY: &str =
		"0x7c63b545dc15066a4867f5d127b554d2bad49eab73303f550e99ea11d9ab123c"; // Unfinished Labs Collator 1
	pub const UNFINISHED_COLLATOR_2_SESSION_KEY: &str =
		"0xa0e9a263e02d8b504808a67abbab811cb14fb376c0f79f5b7d6f7c2e7dcb585d"; // Unfinished Labs Collator 2
	pub const UNFINISHED_COLLATOR_3_SESSION_KEY: &str =
		"0x8062ffd04e84549b541668e4974f37f79f490ea846321e8a613c3406fceff565"; // Unfinished Labs Collator 3
	pub const ON_FINALITY_COLLATOR_1_SESSION_KEY: &str =
		"0xb615416b0c34c5f3d1451a5d44390325832154ea20196b152ea5fa49346f5a30"; // OnFinality Collator 1
	pub const EXTERNAL_COLLATOR_1_SESSION_KEY: &str =
		"0xbe16ebba3525a83e5f5e49cea331d5bc15c723d1dff7319b42837524b40c5970"; // External Collator 1
	pub const EXTERNAL_COLLATOR_2_SESSION_KEY: &str =
		"0x8415832dcab4bcc60023222bbef2e222082ba69d53c9a870129e1c858a9ff87a"; // External Collator 2
}

// pub fn load_frequency_spec() -> Result<ChainSpec, String> {
// 	ChainSpec::from_json_bytes(&include_bytes!("../../specs/frequency.json")[..])
// }

pub fn frequency() -> ChainSpec {
	let properties =
		get_properties(FREQUENCY_TOKEN, TOKEN_DECIMALS as u32, SS58Prefix::get().into());
	let para_id: ParaId = 2091.into();
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
						frequency_mainnet_keys::UNFINISHED_COLLATOR_1_SR25519
							.parse::<AccountId>()
							.unwrap()
							.into(),
						AuraId::from_slice(
							&<[u8; 32]>::from_hex(
								frequency_mainnet_keys::UNFINISHED_COLLATOR_1_SESSION_KEY
									.strip_prefix("0x")
									.unwrap(),
							)
							.unwrap(),
						)
						.unwrap(),
					),
					(
						frequency_mainnet_keys::UNFINISHED_COLLATOR_2_SR25519
							.parse::<AccountId>()
							.unwrap()
							.into(),
						AuraId::from_slice(
							&<[u8; 32]>::from_hex(
								frequency_mainnet_keys::UNFINISHED_COLLATOR_2_SESSION_KEY
									.strip_prefix("0x")
									.unwrap(),
							)
							.unwrap(),
						)
						.unwrap(),
					),
					(
						frequency_mainnet_keys::UNFINISHED_COLLATOR_3_SR25519
							.parse::<AccountId>()
							.unwrap()
							.into(),
						AuraId::from_slice(
							&<[u8; 32]>::from_hex(
								frequency_mainnet_keys::UNFINISHED_COLLATOR_3_SESSION_KEY
									.strip_prefix("0x")
									.unwrap(),
							)
							.unwrap(),
						)
						.unwrap(),
					),
					(
						frequency_mainnet_keys::ON_FINALITY_COLLATOR_1_SR25519
							.parse::<AccountId>()
							.unwrap()
							.into(),
						AuraId::from_slice(
							&<[u8; 32]>::from_hex(
								frequency_mainnet_keys::ON_FINALITY_COLLATOR_1_SESSION_KEY
									.strip_prefix("0x")
									.unwrap(),
							)
							.unwrap(),
						)
						.unwrap(),
					),
					(
						frequency_mainnet_keys::EXTERNAL_COLLATOR_1_SR25519
							.parse::<AccountId>()
							.unwrap()
							.into(),
						AuraId::from_slice(
							&<[u8; 32]>::from_hex(
								frequency_mainnet_keys::EXTERNAL_COLLATOR_1_SESSION_KEY
									.strip_prefix("0x")
									.unwrap(),
							)
							.unwrap(),
						)
						.unwrap(),
					),
					(
						frequency_mainnet_keys::EXTERNAL_COLLATOR_2_SR25519
							.parse::<AccountId>()
							.unwrap()
							.into(),
						AuraId::from_slice(
							&<[u8; 32]>::from_hex(
								frequency_mainnet_keys::EXTERNAL_COLLATOR_2_SESSION_KEY
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
				100_000 * UNITS,
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
		TelemetryEndpoints::new(vec![("wss://telemetry.polkadot.io/submit/".into(), 0), ("wss://telemetry.frequency.xyz/submit/".into(), 0)]).ok(),
		// Protocol ID
		Some("frequency"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions { relay_chain: "polkadot".into(), para_id: para_id.into() },
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
