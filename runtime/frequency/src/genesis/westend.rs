extern crate alloc;
use alloc::{vec, vec::Vec};
use hex::FromHex;
use common_primitives::node::AccountId;
use crate::{AuraId, Balance, SessionKeys};
use sp_core::ByteArray;
use sp_runtime::traits::AccountIdConversion;

// Helper function to convert hex string to AccountId
#[allow(clippy::unwrap_used)]
fn account_id_from_hex(hex_str: &str) -> AccountId {
	AccountId::from(<[u8; 32]>::from_hex(hex_str.strip_prefix("0x").unwrap()).unwrap())
}

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

fn default_invulnerables() -> Vec<(AccountId, AuraId)> {
	vec![
		(
			// 5Fy4Ng81A32LqbZ5JNKLZwW6AA8Nme3kqSUiVSJzTd7yu4a2
			account_id_from_hex(public_testnet_keys::COLLATOR_1_SR25519),
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
			account_id_from_hex(public_testnet_keys::COLLATOR_2_SR25519),
			AuraId::from_slice(
				&<[u8; 32]>::from_hex(
					public_testnet_keys::COLLATOR_2_SR25519.strip_prefix("0x").unwrap(),
				)
				.unwrap(),
			)
			.unwrap(),
		),
	]
}

fn endowed_accounts() -> Vec<(AccountId, Balance)> {
	vec![
		// 5GhDid9V5rpReBSAK6sSEmCzeiUXTgAmAoMJQUuK5gFzPXoq
		account_id_from_hex(public_testnet_keys::SUDO),
		// 5E46myWYmywo8cF9Wk77NZM7TqLqVG7uMYjGuyyfrve9waa9
		account_id_from_hex(public_testnet_keys::TECH_COUNCIL1),
		// 5Dw9hdb8Wy3cMFf1ZpQh5HuK2GvMNMcxfka4JsM2GRpjqHSA
		account_id_from_hex(public_testnet_keys::TECH_COUNCIL2),
		// 5ETnrcfDwCTKZt5pLoKcLNbbwUcddkDAxEjtcgibAtkDhBBe
		account_id_from_hex(public_testnet_keys::TECH_COUNCIL3),
		// 5FpQTxEFHAdF2hpkA89qKenSjGQBAHi8uCo8F6fJ5RxnR1sn
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL1),
		// 5DwKn9yg7Kqa71qTaZVfoigwtzyPgBp23wzJKmGiUr8S3j7A
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL2),
		// 5EyLduUEpNfUox5HuogrhbKugAdHnsLHrysKSWL6u4TQFAra
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL3),
		// 5FHFyMYQH39mq3FA2a5ZcDe275K7RZs3zi76gsQb2TE1PF7i
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL4),
		// 5HYKfXdGi2GUupHksmQLtHoWtwH3wPkJRSr3RpbKivTovpqX
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL5),
		// 5HKXEFkNLeutZ2yZ9mbG2v5AbTDMXhza9VEfeDCiJGGBUyi3
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL6),
		// 5CXnMFFgruHxAFtvCogwm8TFygWg1MWd9KhMnfEPbRdCpf74
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL7),
		// 5CWzQaAJFqYoF1bZWsvEnzMQDGokk2csTFBwfPpo1oNfBGkn
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL8),
		// 5HmcreGLq25iA7fiyb6An4YVWC3k1Cq8SQgYn2Qpeepq24nV
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL9),
		// Treasury $$
		common_runtime::constants::TREASURY_PALLET_ID.into_account_truncating(),
	]
	.iter()
	.cloned()
	.map(|k| (k, 1 << 60))
	.collect()
}

fn frequency_council() -> Vec<AccountId> {
	vec![
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL1),
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL2),
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL3),
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL4),
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL5),
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL6),
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL7),
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL8),
		account_id_from_hex(public_testnet_keys::FRQ_COUNCIL9),
	]
}

fn technical_committee() -> Vec<AccountId> {
	vec![
		account_id_from_hex(public_testnet_keys::TECH_COUNCIL1),
		account_id_from_hex(public_testnet_keys::TECH_COUNCIL2),
		account_id_from_hex(public_testnet_keys::TECH_COUNCIL3),
	]
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
// pub fn template_session_keys(keys: AuraId) -> SessionKeys {
// 	SessionKeys { aura: keys }
// }

fn session_keys() -> Vec<(AccountId, AccountId, SessionKeys)> {
	default_invulnerables()
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
		default_invulnerables(),
		crate::EXISTENTIAL_DEPOSIT * 16,
		Some(account_id_from_hex(public_testnet_keys::SUDO)),
		endowed_accounts(),
		session_keys(),
		frequency_council(),
		technical_committee(),
		super::helpers::load_genesis_schemas(),
		2313.into(),
	)
}
