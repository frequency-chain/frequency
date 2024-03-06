#![allow(missing_docs)]
use common_primitives::node::AccountId;
use common_runtime::constants::{
	currency::EXISTENTIAL_DEPOSIT, FREQUENCY_LOCAL_TOKEN, FREQUENCY_ROCOCO_TOKEN, TOKEN_DECIMALS,
};
use cumulus_primitives_core::ParaId;
use frequency_runtime::{AuraId, CouncilConfig, Ss58Prefix, SudoConfig, TechnicalCommitteeConfig};
use hex::FromHex;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_core::ByteArray;
use sp_runtime::traits::AccountIdConversion;
/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec =
	sc_service::GenericChainSpec<frequency_runtime::RuntimeGenesisConfig, Extensions>;
use sp_core::sr25519;

use super::{get_account_id_from_seed, get_collator_keys_from_seed, get_properties, Extensions};

#[allow(clippy::unwrap_used)]
/// Generates the Frequency Paseo chain spec from the raw json
pub fn load_frequency_paseo_spec() -> ChainSpec {
	ChainSpec::from_json_bytes(
		&include_bytes!("../../../../resources/frequency-paseo.raw.json")[..],
	)
	.unwrap()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn template_session_keys(keys: AuraId) -> frequency_runtime::SessionKeys {
	frequency_runtime::SessionKeys { aura: keys }
}

pub mod public_testnet_keys {
	pub const COLLATOR_1_SR25519: &str =
		"0x5c0f55ba602f76d69b5cc075d81f6d27db9157c90dc4be492f4edbe7d7c96d18";
	pub const COLLATOR_2_SR25519: &str =
		"0x202be6542ed50679271280b8c37140681bbd5acfd0e508668297029f871b9f0c";
	pub const ROCOCO_FRQ_SUDO: &str =
		"0xccca4a5b784105460c5466cbb8d11b34f29ffcf6c725d07b65940e697763763c";
	pub const TECH_COUNCIL1: &str =
		"0x847c1ac02474b90cf1e9d8e722318b75fd56d370e6f35e9c983fe671e788d23a";
	pub const TECH_COUNCIL2: &str =
		"0x52b580c22c5ff6f586a0966fbd2373de279d1aa1b2d05dff47616b5a338fce27";
	pub const TECH_COUNCIL3: &str =
		"0x6a13f08b279cb33b249954190bcee832747b9aa9dc14cc290f82d73d496cfc0a";
	pub const FRQ_COUNCIL1: &str =
		"0xa608f3e0030c157b6e2a540c5f0c7dbd6004793813cad2c9fbda0c84c093c301";
	pub const FRQ_COUNCIL2: &str =
		"0x52d76db441043a5d47d9bf83e6fd2d5acb86b8547062571ee7b68255b6bada10";
	pub const FRQ_COUNCIL3: &str =
		"0x809d0a4e6683ebff9d74c7f6ba9fe504a64a7227d74eb45ee85556cc01013a63";
	pub const FRQ_COUNCIL4: &str =
		"0x8e47c13fd0f028f56378e202523fa44508fd64df89fddb482fc0b128989e9f0b";
	pub const FRQ_COUNCIL5: &str =
		"0xf23d555b95ca8c752b531e48848bfb4d3aa2b4eea407484ccee947501e77d04f";
	pub const FRQ_COUNCIL6: &str =
		"0xe87a126794cb727b5a7760922f81fbf0f80fd64b7e86e6ae4fee0be4289c7512";
	pub const FRQ_COUNCIL7: &str =
		"0x14a6bff08e9637457a165779765417feca01a2119dec98ec134f8ae470111318";
	pub const FRQ_COUNCIL8: &str =
		"0x140c17ced6e4fba8b62a6935052cfb7c5a8ad8ecc43dee1f4fc7c30c1ca3cb14";
	pub const FRQ_COUNCIL9: &str =
		"0xfc61655783e14b361d2b9601c657c3c5361a2cf32aa1a448fc83b1a356808a1a";
}

#[allow(clippy::unwrap_used)]
pub fn development_config() -> ChainSpec {
	let properties =
		get_properties(FREQUENCY_ROCOCO_TOKEN, TOKEN_DECIMALS as u32, Ss58Prefix::get().into());
	let para_id: ParaId = 4000.into();
	ChainSpec::from_genesis(
		// Name
		"Frequency Paseo",
		// ID
		"frequency-paseo",
		ChainType::Live,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					(
						// 5FLZcmvR22FvM2u4GDbeSAQ8uJpePuaEQccquD89UcVBwEgB
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
						// 5CkeZ1PLWpdS4aSYKBLxVwzZGfvGa2SitSKhX72KT1KAih4Z
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
					// 5GhDid9V5rpReBSAK6sSEmCzeiUXTgAmAoMJQUuK5gFzPXoq
					public_testnet_keys::ROCOCO_FRQ_SUDO.parse::<AccountId>().unwrap().into(),
					// 5E46myWYmywo8cF9Wk77NZM7TqLqVG7uMYjGuyyfrve9waa9
					public_testnet_keys::TECH_COUNCIL1.parse::<AccountId>().unwrap().into(),
					// 5Dw9hdb8Wy3cMFf1ZpQh5HuK2GvMNMcxfka4JsM2GRpjqHSA
					public_testnet_keys::TECH_COUNCIL2.parse::<AccountId>().unwrap().into(),
					// 5ETnrcfDwCTKZt5pLoKcLNbbwUcddkDAxEjtcgibAtkDhBBe
					public_testnet_keys::TECH_COUNCIL3.parse::<AccountId>().unwrap().into(),
					// 5FpQTxEFHAdF2hpkA89qKenSjGQBAHi8uCo8F6fJ5RxnR1sn
					public_testnet_keys::FRQ_COUNCIL1.parse::<AccountId>().unwrap().into(),
					// 5DwKn9yg7Kqa71qTaZVfoigwtzyPgBp23wzJKmGiUr8S3j7A
					public_testnet_keys::FRQ_COUNCIL2.parse::<AccountId>().unwrap().into(),
					// 5EyLduUEpNfUox5HuogrhbKugAdHnsLHrysKSWL6u4TQFAra
					public_testnet_keys::FRQ_COUNCIL3.parse::<AccountId>().unwrap().into(),
					// 5FHFyMYQH39mq3FA2a5ZcDe275K7RZs3zi76gsQb2TE1PF7i
					public_testnet_keys::FRQ_COUNCIL4.parse::<AccountId>().unwrap().into(),
					// 5HYKfXdGi2GUupHksmQLtHoWtwH3wPkJRSr3RpbKivTovpqX
					public_testnet_keys::FRQ_COUNCIL5.parse::<AccountId>().unwrap().into(),
					// 5HKXEFkNLeutZ2yZ9mbG2v5AbTDMXhza9VEfeDCiJGGBUyi3
					public_testnet_keys::FRQ_COUNCIL6.parse::<AccountId>().unwrap().into(),
					// 5CXnMFFgruHxAFtvCogwm8TFygWg1MWd9KhMnfEPbRdCpf74
					public_testnet_keys::FRQ_COUNCIL7.parse::<AccountId>().unwrap().into(),
					// 5CWzQaAJFqYoF1bZWsvEnzMQDGokk2csTFBwfPpo1oNfBGkn
					public_testnet_keys::FRQ_COUNCIL8.parse::<AccountId>().unwrap().into(),
					// 5HmcreGLq25iA7fiyb6An4YVWC3k1Cq8SQgYn2Qpeepq24nV
					public_testnet_keys::FRQ_COUNCIL9.parse::<AccountId>().unwrap().into(),
					// Treasury $$
					common_runtime::constants::TREASURY_PALLET_ID.into_account_truncating(),
				],
				// Council Members
				vec![
					public_testnet_keys::FRQ_COUNCIL1.parse::<AccountId>().unwrap().into(),
					public_testnet_keys::FRQ_COUNCIL2.parse::<AccountId>().unwrap().into(),
					public_testnet_keys::FRQ_COUNCIL3.parse::<AccountId>().unwrap().into(),
					public_testnet_keys::FRQ_COUNCIL4.parse::<AccountId>().unwrap().into(),
					public_testnet_keys::FRQ_COUNCIL5.parse::<AccountId>().unwrap().into(),
					public_testnet_keys::FRQ_COUNCIL6.parse::<AccountId>().unwrap().into(),
					public_testnet_keys::FRQ_COUNCIL7.parse::<AccountId>().unwrap().into(),
					public_testnet_keys::FRQ_COUNCIL8.parse::<AccountId>().unwrap().into(),
					public_testnet_keys::FRQ_COUNCIL9.parse::<AccountId>().unwrap().into(),
				],
				// Technical Committee Members
				vec![
					public_testnet_keys::TECH_COUNCIL1.parse::<AccountId>().unwrap().into(),
					public_testnet_keys::TECH_COUNCIL2.parse::<AccountId>().unwrap().into(),
					public_testnet_keys::TECH_COUNCIL3.parse::<AccountId>().unwrap().into(),
				],
				para_id,
			)
		},
		// Bootnodes
		vec![
			"/dns4/0.boot.testnet.amplica.io/tcp/30333/ws/p2p/12D3KooWArmKDbY8Y6XXHGodosWAjRWWxSw5YxWEjSZTBNjJXVSC".parse().unwrap(),
		],
		// Telemetry
		TelemetryEndpoints::new(vec![("wss://telemetry.frequency.xyz/submit/".into(), 0)]).ok(),
		// Protocol ID
		Some("frequency-paseo"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "paseo".into(), // You MUST set this to the correct network!
			para_id: para_id.into(),
		},
	)
}

/// Generates the chain spec for a local testnet
pub fn local_paseo_testnet_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let properties =
		get_properties(FREQUENCY_LOCAL_TOKEN, TOKEN_DECIMALS as u32, Ss58Prefix::get().into());

	ChainSpec::from_genesis(
		// Name
		"Frequency Local Testnet",
		// ID
		"frequency-paseo-local",
		ChainType::Local,
		move || {
			testnet_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
					),
				],
				// Sudo
				Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
				// Endowed Accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
					common_runtime::constants::TREASURY_PALLET_ID.into_account_truncating(),
				],
				// Council members
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
				],
				// Technical Committee members
				vec![
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
				],
				// ParaId
				2000.into(),
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		Some("frequency-paseo-local"),
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "paseo-local".into(), // You MUST set this to the correct network!
			para_id: 2000,
		},
	)
}

#[allow(clippy::expect_used)]
fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root_key: Option<AccountId>,
	endowed_accounts: Vec<AccountId>,
	council_members: Vec<AccountId>,
	technical_committee_members: Vec<AccountId>,
	id: ParaId,
) -> frequency_runtime::RuntimeGenesisConfig {
	frequency_runtime::RuntimeGenesisConfig {
		system: frequency_runtime::SystemConfig {
			code: frequency_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			..Default::default()
		},
		balances: frequency_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		parachain_info: frequency_runtime::ParachainInfoConfig {
			parachain_id: id,
			..Default::default()
		},
		collator_selection: frequency_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: frequency_runtime::SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						template_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
		parachain_system: Default::default(),
		sudo: SudoConfig {
			// Assign network admin rights.
			key: root_key,
		},
		schemas: Default::default(),
		time_release: Default::default(),
		democracy: Default::default(),
		treasury: Default::default(),
		council: CouncilConfig { phantom: Default::default(), members: council_members },
		technical_committee: TechnicalCommitteeConfig {
			phantom: Default::default(),
			members: technical_committee_members,
		},
	}
}
