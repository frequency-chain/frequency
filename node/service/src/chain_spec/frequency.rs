#![allow(missing_docs)]
use common_primitives::node::{AccountId, Balance};
use common_runtime::constants::{
	currency::{DOLLARS, UNITS},
	FREQUENCY_TOKEN, TOKEN_DECIMALS,
};
use cumulus_primitives_core::ParaId;
use frequency_runtime::{AuraId, CouncilConfig, Ss58Prefix, TechnicalCommitteeConfig};

use hex::FromHex;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_core::ByteArray;
use sp_runtime::traits::AccountIdConversion;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<Extensions>;

use super::{get_properties, Extensions};

pub mod frequency_mainnet_keys {

	// Unfinished Collator 1 public key (sr25519) and session key
	pub const UNFINISHED_COLLATOR_1_SR25519: &str =
		"0x40c0cf083ba1d081fce7c1f38f5344a4eba4ee5c13e1dddcb9cc6acf74559710";
	pub const UNFINISHED_COLLATOR_1_SESSION_KEY: &str =
		"0x7c63b545dc15066a4867f5d127b554d2bad49eab73303f550e99ea11d9ab123c";

	// Unfinished Collator 2 public key (sr25519) and session key
	pub const UNFINISHED_COLLATOR_2_SR25519: &str =
		"0x1e5dc06d8418a0dd695bf3a23ecdc219103a149a863c1977396bf0697a474015";
	pub const UNFINISHED_COLLATOR_2_SESSION_KEY: &str =
		"0xa0e9a263e02d8b504808a67abbab811cb14fb376c0f79f5b7d6f7c2e7dcb585d";

	// Unfinished Collator 3 public key (sr25519) and session key
	pub const UNFINISHED_COLLATOR_3_SR25519: &str =
		"0x7eb4de08f44e6077ed072e2fc357aac007150e3b73373434c47ec56323b52a75";

	pub const UNFINISHED_COLLATOR_3_SESSION_KEY: &str =
		"0x8062ffd04e84549b541668e4974f37f79f490ea846321e8a613c3406fceff565";

	// OnFinality Collator public key (sr25519) and session key
	pub const ON_FINALITY_COLLATOR_1_SR25519: &str =
		"0x3c9ea854901c1b36bd203400c90c31a69367316f7deaeeb14037f2be44bcb118";
	pub const ON_FINALITY_COLLATOR_1_SESSION_KEY: &str =
		"0xb615416b0c34c5f3d1451a5d44390325832154ea20196b152ea5fa49346f5a30";

	// External Collator 1  public key (sr25519) and session key
	pub const EXTERNAL_COLLATOR_1_SR25519: &str =
		"0x3c33f8c4c1155958d60d4d52a5181677da96052493d876b747272311b52cfe61";
	pub const EXTERNAL_COLLATOR_1_SESSION_KEY: &str =
		"0xbe16ebba3525a83e5f5e49cea331d5bc15c723d1dff7319b42837524b40c5970";

	// External Collator 2 public key (sr25519) and session key
	pub const EXTERNAL_COLLATOR_2_SR25519: &str =
		"0x36523fba06e0f31f35dd904386a0ebe25d505ef09af4d39041afa1d994a23f5d";
	pub const EXTERNAL_COLLATOR_2_SESSION_KEY: &str =
		"0x8415832dcab4bcc60023222bbef2e222082ba69d53c9a870129e1c858a9ff87a";

	pub const TECHNICAL_COUNCIL_1: &str =
		"0x92e8dba6b1cbd25e32a786a128406a24ec3e7f8c0c17c65e7d146f14f0fd3e69";
	pub const TECHNICAL_COUNCIL_2: &str =
		"0xfcac9ce9c9807732f092a84efde7cfbf77b4c3abedffadfb12cc63c6f6836605";
	pub const TECHNICAL_COUNCIL_3: &str =
		"0xcee0137a698a9a616193c80550185ea91387f16208544a8b47d477703b055b4b";

	pub const FREQUENCY_COUNCIL_1: &str =
		"0x9286ca034a2523b7a53c30e433bc148aa43a4eccbceffe2569df30c915bf666f";
	pub const FREQUENCY_COUNCIL_2: &str =
		"0x3692687b80fcc7293aeae47847e234d9744a81963881d262d044d0dc7080f059";
	pub const FREQUENCY_COUNCIL_3: &str =
		"0x96a666eb5d96e39f555094e37f32885f6c8fad30e17bee5fddb138843285c237";
	pub const FREQUENCY_COUNCIL_4: &str =
		"0xaab55f57a977dfed1fc64e88c242c9a6c7f02abe2b581fc58f2bbfc0bdaad224";
	pub const FREQUENCY_COUNCIL_5: &str =
		"0xfcac9ce9c9807732f092a84efde7cfbf77b4c3abedffadfb12cc63c6f6836605";
}

pub mod project_liberty_keys {
	pub const MULTISIG_THRESHOLD_2: &str =
		"0xcd0760f6b5ebbcf0181368f7d7258e7e1f3154c43df4a1112b6d7e366937ca42";
	pub const SIGNATORY_1_OF_3: &str =
		"0xb6fdaa1aea71c2cabf6fed4c2943d6fcd7c7a594e0bc36c8293220a1971dbc74";
	pub const SIGNATORY_2_OF_3: &str =
		"0x8e49e6382a3e6fa0e0cecefca2af3c7f9d3eebde95bd9e75893ed179d03cf61c";
	pub const SIGNATORY_3_OF_3: &str =
		"0x5854d238bff3f7a924eadcb1efa6fb2cce5dbdd340afdbdaaa4586567083883d";
}

pub mod unfinished_labs_keys {
	pub const MULTISIG_THRESHOLD_2: &str =
		"0xf06e128a8f9b813d07c1212475183c174de545f1705bf9b1ae8ce468a7f0ec0c";
	pub const SIGNATORY_1_OF_3: &str =
		"0x46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a";
	pub const SIGNATORY_2_OF_3: &str =
		"0xb26873df000ddae71bbc035cd907b1d23b06e662d7db11cd5802f845fbae547e";
	pub const SIGNATORY_3_OF_3: &str =
		"0x2041926697db16c0a27bd1bfe2883b69cd12186f89bc44c2e1cdd2eb69508268";
}

pub mod foundation_keys {
	pub const MULTISIG_THRESHOLD_2: &str =
		"0xb2bacd25b122a125e0196107e0a13e0505d17399e46b93f339fae9387d9921cf";
	pub const SIGNATORY_1_OF_3: &str =
		"0x6a793e27cb1bafc5c241872cafc75e08333b6b95765bf868e47056909cbc5b07";
	pub const SIGNATORY_2_OF_3: &str =
		"0x202f792d3edc84813c8e89b6a2cbc6dff7d077522f0b8be78e4b0877f2c0b70a";
	pub const SIGNATORY_3_OF_3: &str =
		"0xf811cbc8611b38f68f4e486284e71eb45523fc5cf685c070bc76331d595cf81c";
}

#[allow(clippy::unwrap_used)]
pub fn load_frequency_spec() -> ChainSpec {
	ChainSpec::from_json_bytes(&include_bytes!("../../../../resources/frequency.raw.json")[..])
		.unwrap()
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
	]).with_genesis_config(
			frequency_genesis(
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
				// Total initial tokens: 1_000_000_000 FRQCY
				vec![
					// Project Liberty (5% Total)
					(
						project_liberty_keys::MULTISIG_THRESHOLD_2
							.parse::<AccountId>()
							.unwrap()
							.into(),
						49_999_700 * DOLLARS,
					),
					(
						project_liberty_keys::SIGNATORY_1_OF_3
							.parse::<AccountId>()
							.unwrap()
							.into(),
						100 * DOLLARS,
					),
					(
						project_liberty_keys::SIGNATORY_2_OF_3
							.parse::<AccountId>()
							.unwrap()
							.into(),
						100 * DOLLARS,
					),
					(
						project_liberty_keys::SIGNATORY_3_OF_3
							.parse::<AccountId>()
							.unwrap()
							.into(),
						100 * DOLLARS,
					),
					// Unfinished Labs (40% Total)
					(
						unfinished_labs_keys::MULTISIG_THRESHOLD_2
							.parse::<AccountId>()
							.unwrap()
							.into(),
						399_999_700 * DOLLARS,
					),
					(
						unfinished_labs_keys::SIGNATORY_1_OF_3
							.parse::<AccountId>()
							.unwrap()
							.into(),
						100 * DOLLARS,
					),
					(
						unfinished_labs_keys::SIGNATORY_2_OF_3
							.parse::<AccountId>()
							.unwrap()
							.into(),
						100 * DOLLARS,
					),
					(
						unfinished_labs_keys::SIGNATORY_3_OF_3
							.parse::<AccountId>()
							.unwrap()
							.into(),
						100 * DOLLARS,
					),
					// Foundation Operational (2% Total)
					(
						foundation_keys::MULTISIG_THRESHOLD_2
							.parse::<AccountId>()
							.unwrap()
							.into(),
						19_999_700 * DOLLARS,
					),
					(
						foundation_keys::SIGNATORY_1_OF_3
							.parse::<AccountId>()
							.unwrap()
							.into(),
						100 * DOLLARS,
					),
					(
						foundation_keys::SIGNATORY_2_OF_3
							.parse::<AccountId>()
							.unwrap()
							.into(),
						100 * DOLLARS,
					),
					(
						foundation_keys::SIGNATORY_3_OF_3
							.parse::<AccountId>()
							.unwrap()
							.into(),
						100 * DOLLARS,
					),
					// Frequency (53% Total)
					(
						// Treasury Pallet
						common_runtime::constants::TREASURY_PALLET_ID.into_account_truncating(),
						529_988_600 * DOLLARS,
					),
					(
						// Council Member
						frequency_mainnet_keys::FREQUENCY_COUNCIL_1
							.parse::<AccountId>()
							.unwrap()
							.into(),
						200 * DOLLARS,
					),
					(
						// Council Member
						frequency_mainnet_keys::FREQUENCY_COUNCIL_2
							.parse::<AccountId>()
							.unwrap()
							.into(),
						200 * DOLLARS,
					),
					(
						// Council Member
						frequency_mainnet_keys::FREQUENCY_COUNCIL_3
							.parse::<AccountId>()
							.unwrap()
							.into(),
						200 * DOLLARS,
					),
					(
						// Council Member
						frequency_mainnet_keys::FREQUENCY_COUNCIL_4
							.parse::<AccountId>()
							.unwrap()
							.into(),
						200 * DOLLARS,
					),
					(
						// Council Member and TECHNICAL_COUNCIL_2
						frequency_mainnet_keys::FREQUENCY_COUNCIL_5
							.parse::<AccountId>()
							.unwrap()
							.into(),
						200 * DOLLARS,
					),
					(
						// Council Member
						frequency_mainnet_keys::TECHNICAL_COUNCIL_1
							.parse::<AccountId>()
							.unwrap()
							.into(),
						200 * DOLLARS,
					),
					(
						// Council Member
						frequency_mainnet_keys::TECHNICAL_COUNCIL_3
							.parse::<AccountId>()
							.unwrap()
							.into(),
						200 * DOLLARS,
					),
				],
				vec!(
					frequency_mainnet_keys::FREQUENCY_COUNCIL_1.parse::<AccountId>().unwrap().into(),
					frequency_mainnet_keys::FREQUENCY_COUNCIL_2.parse::<AccountId>().unwrap().into(),
					frequency_mainnet_keys::FREQUENCY_COUNCIL_3.parse::<AccountId>().unwrap().into(),
					frequency_mainnet_keys::FREQUENCY_COUNCIL_4.parse::<AccountId>().unwrap().into(),
					frequency_mainnet_keys::FREQUENCY_COUNCIL_5.parse::<AccountId>().unwrap().into(),
				),
				vec!(
					frequency_mainnet_keys::TECHNICAL_COUNCIL_1.parse::<AccountId>().unwrap().into(),
					frequency_mainnet_keys::TECHNICAL_COUNCIL_2.parse::<AccountId>().unwrap().into(),
					frequency_mainnet_keys::TECHNICAL_COUNCIL_3.parse::<AccountId>().unwrap().into(),
				),
				//candidacy bond
				100_000 * UNITS,
				para_id,
			)

	).build()
}

#[allow(clippy::unwrap_used)]
fn frequency_genesis(
	initial_authorities: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<(AccountId, Balance)>,
	council_members: Vec<AccountId>,
	technical_committee_members: Vec<AccountId>,
	candidacy_bond: Balance,
	id: ParaId,
) -> serde_json::Value {
	let genesis = frequency_runtime::RuntimeGenesisConfig {
		system: frequency_runtime::SystemConfig { ..Default::default() },
		balances: frequency_runtime::BalancesConfig { balances: endowed_accounts },
		parachain_info: frequency_runtime::ParachainInfoConfig {
			parachain_id: id,
			..Default::default()
		},
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
		// SUDO removed Jan 2023, but needed when testing and checking with frequency-lint-check
		#[cfg(any(not(feature = "frequency"), feature = "frequency-lint-check"))]
		sudo: Default::default(),
		schemas: Default::default(),
		time_release: Default::default(),
		democracy: Default::default(),
		treasury: Default::default(),
		council: CouncilConfig { phantom: Default::default(), members: council_members },
		technical_committee: TechnicalCommitteeConfig {
			phantom: Default::default(),
			members: technical_committee_members,
		},
	};

	serde_json::to_value(&genesis).unwrap()
}
