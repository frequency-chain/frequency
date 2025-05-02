extern crate alloc;
use alloc::{vec, vec::Vec};
use hex::FromHex;
use common_primitives::node::AccountId;
// use frequency_runtime::{AuraId, SessionKeys};
use crate::{AuraId, Balance, SessionKeys};

mod public_testnet_keys {
	pub const COLLATOR_1_SR25519: &str =
		"0xaca28769c5bf9c1417002b87598b2287f487aad6b4a5fa01ea332a16ee4ba24c";
	pub const COLLATOR_2_SR25519: &str =
		"0x6463476668f638099e8a0df6e66a2e8e28c99bd5379b1c4f392b5705e2f78b7b";
	pub const SUDO: &str = "0xccca4a5b784105460c5466cbb8d11b34f29ffcf6c725d07b65940e697763763c";
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

const INVULNERABLES: Vec<(AccountId, AuraId)> = vec![
	(
		// 5Fy4Ng81A32LqbZ5JNKLZwW6AA8Nme3kqSUiVSJzTd7yu4a2
		public_testnet_keys::COLLATOR_1_SR25519.parse::<AccountId>().unwrap().into(),
		AuraId::from_slice(
			&<[u8; 32]>::from_hex(
				public_testnet_keys::COLLATOR_1_SR25519.strip_prefix("0x").unwrap(),
			)
			.unwrap(),
		)
		.unwrap(),
	),
	(
		// 5ELL8wEC9K3QFxeK6beVhwRLQScRw7A3wquhPnuJdpeCyNxX
		public_testnet_keys::COLLATOR_2_SR25519.parse::<AccountId>().unwrap().into(),
		AuraId::from_slice(
			&<[u8; 32]>::from_hex(
				public_testnet_keys::COLLATOR_2_SR25519.strip_prefix("0x").unwrap(),
			)
			.unwrap(),
		)
		.unwrap(),
	),
];

const ENDOWED_ACCOUNTS: Vec<(AccountId, Balance)> = vec![
	// 5GhDid9V5rpReBSAK6sSEmCzeiUXTgAmAoMJQUuK5gFzPXoq
	public_testnet_keys::SUDO.parse::<AccountId>().unwrap().into(),
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
];

const FREQUENCY_COUNCIL: Vec<AccountId> = vec![
	public_testnet_keys::FRQ_COUNCIL1.parse::<AccountId>().unwrap().into(),
	public_testnet_keys::FRQ_COUNCIL2.parse::<AccountId>().unwrap().into(),
	public_testnet_keys::FRQ_COUNCIL3.parse::<AccountId>().unwrap().into(),
	public_testnet_keys::FRQ_COUNCIL4.parse::<AccountId>().unwrap().into(),
	public_testnet_keys::FRQ_COUNCIL5.parse::<AccountId>().unwrap().into(),
	public_testnet_keys::FRQ_COUNCIL6.parse::<AccountId>().unwrap().into(),
	public_testnet_keys::FRQ_COUNCIL7.parse::<AccountId>().unwrap().into(),
	public_testnet_keys::FRQ_COUNCIL8.parse::<AccountId>().unwrap().into(),
	public_testnet_keys::FRQ_COUNCIL9.parse::<AccountId>().unwrap().into(),
];

const TECHNICAL_COMMITTEE: Vec<AccountId> = vec![
	public_testnet_keys::TECH_COUNCIL1.parse::<AccountId>().unwrap().into(),
	public_testnet_keys::TECH_COUNCIL2.parse::<AccountId>().unwrap().into(),
	public_testnet_keys::TECH_COUNCIL3.parse::<AccountId>().unwrap().into(),
];

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn template_session_keys(keys: AuraId) -> frequency_runtime::SessionKeys {
	frequency_runtime::SessionKeys { aura: keys }
}

fn session_keys() -> Vec<(AccountId, AccountId, SessionKeys)> {
	INVULNERABLES
		.into_iter()
		.map(|(acc, aura)| {
			(
				acc.clone(),          // account id
				acc,                  // validator id
				SessionKeys { aura }, // session keys
			)
		})
		.collect()
}

#[allow(clippy::unwrap_used)]
pub fn frequency_westend_genesis_config() -> serde_json::Value {
	super::helpers::build_genesis(
		INVULNERABLES,
		crate::EXISTENTIAL_DEPOSIT * 16,
		Some(public_testnet_keys::SUDO.parse::<AccountId>().unwrap().into()),
		ENDOWED_ACCOUNTS,
		session_keys(),
		FREQUENCY_COUNCIL,
		TECHNICAL_COMMITTEE,
		super::helpers::load_genesis_schemas(),
		2313.into(),
	)
}
