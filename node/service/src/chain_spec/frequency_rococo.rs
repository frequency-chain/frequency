#![allow(missing_docs)]
use common_runtime::constants::FREQUENCY_ROCOCO_TOKEN;
use cumulus_primitives_core::ParaId;
use frequency_rococo_runtime::{
	AccountId, AuraId, CouncilConfig, SudoConfig, TechnicalCommitteeConfig, EXISTENTIAL_DEPOSIT,
};
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
	pub const FRQ_COUNCIL10: &str =
		"0x1caccabbe8095a35292782cffc29145030264f9706753923a5c237e8f5aceb1a";
}

// pub fn load_frequency_rococo_spec() -> Result<ChainSpec, String> {
// 	ChainSpec::from_json_bytes(&include_bytes!("../../specs/frequency_rococo.json")[..])
// }

pub fn frequency_rococo_testnet() -> ChainSpec {
	let properties = get_properties(FREQUENCY_ROCOCO_TOKEN, 12, 42);
	let para_id: ParaId = 4044.into();
	ChainSpec::from_genesis(
		// Name
		"Frequency Rococo",
		// ID
		"frequency-rococo",
		ChainType::Live,
		move || {
			frequency_rococo_genesis(
				// initial collators.
				vec![
					(
						// 5DaTHXTHUckbmLK2mUABQDrbwYKEaCLjKpTRB4VHYEbvF9fp
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
						// 5E46myWYmywo8cF9Wk77NZM7TqLqVG7uMYjGuyyfrve9waa9
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
					// 5FnjAszaYTVfEFDooTN37DCBinQyw4dvsZDr7PbYovmAhEqn
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
					// 5CiJXZxsko8YZGhTZAwrhREUzjCVEGwNE7Fgrf1twbt8WsmY
					public_testnet_keys::FRQ_COUNCIL10.parse::<AccountId>().unwrap().into(),
				],
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
					public_testnet_keys::FRQ_COUNCIL10.parse::<AccountId>().unwrap().into(),
				],
				vec![
					public_testnet_keys::TECH_COUNCIL1.parse::<AccountId>().unwrap().into(),
					public_testnet_keys::TECH_COUNCIL2.parse::<AccountId>().unwrap().into(),
					public_testnet_keys::TECH_COUNCIL3.parse::<AccountId>().unwrap().into(),
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
	council_members: Vec<AccountId>,
	technical_committee_members: Vec<AccountId>,
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
		democracy: Default::default(),
		council: CouncilConfig { phantom: Default::default(), members: council_members },
		technical_committee: TechnicalCommitteeConfig {
			phantom: Default::default(),
			members: technical_committee_members,
		},
	}
}
