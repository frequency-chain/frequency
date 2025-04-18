use cumulus_primitives_core::ParaId;
use sp_core::{Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
extern crate alloc;
use alloc::{format, vec};

use crate::{
	AccountId, AccountIdConversion, AuraId, Balance, BalancesConfig, CollatorSelectionConfig,
	CouncilConfig, ParachainInfoConfig, RuntimeGenesisConfig, SessionConfig, SessionKeys,
	Signature, SystemConfig, TechnicalCommitteeConfig, Vec,
};

use sp_core::sr25519;

/// Helper function to generate a crypto pair from seed
// The panic from expect will not occur here because these input values are hardcoded.
#[allow(clippy::expect_used)]
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_public_from_seed::<AuraId>(seed)
}

/// Helper function to generate a crypto pair from seed
#[allow(clippy::expect_used)]
// The panic from expect will not occur here because these input values are hardcoded.
pub fn get_public_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

#[allow(clippy::unwrap_used)]
pub fn load_genesis_schemas() -> Vec<crate::pallet_schemas::GenesisSchema> {
	serde_json::from_slice(include_bytes!("../../../../resources/genesis-schemas.json")).unwrap()
}

type AccountPublic = <Signature as Verify>::Signer;

pub fn build_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	candidacy_bond: Balance,
	endowed_accounts: Vec<(AccountId, Balance)>,
	session_keys: Vec<(AccountId, AccountId, SessionKeys)>,
	council_members: Vec<AccountId>,
	technical_committee_members: Vec<AccountId>,
	schemas: Vec<crate::pallet_schemas::GenesisSchema>,
	id: ParaId,
) -> serde_json::Value {
	let genesis = RuntimeGenesisConfig {
		system: SystemConfig { ..Default::default() },
		balances: BalancesConfig { balances: endowed_accounts, ..Default::default() },
		parachain_info: ParachainInfoConfig { parachain_id: id, ..Default::default() },
		collator_selection: CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond,
			..Default::default()
		},
		session: SessionConfig { keys: session_keys, non_authority_keys: Default::default() },
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
		parachain_system: Default::default(),
		sudo: crate::SudoConfig {
			// Assign network admin rights.
			key: Some(get_account_id_from_seed::<sp_core::sr25519::Public>("Alice")),
		},
		schemas: crate::pallet_schemas::GenesisConfig {
			initial_schemas: schemas,
			..Default::default()
		},
		time_release: Default::default(),
		democracy: Default::default(),
		treasury: Default::default(),
		council: CouncilConfig { phantom: Default::default(), members: council_members },
		technical_committee: TechnicalCommitteeConfig {
			phantom: Default::default(),
			members: technical_committee_members,
		},
	};

	serde_json::json!(&genesis)
}

pub fn default_endowed_accounts() -> Vec<(AccountId, Balance)> {
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
	]
	.iter()
	.cloned()
	.map(|k| (k, 1 << 60))
	.collect()
}

pub fn default_session_keys() -> Vec<(AccountId, AccountId, SessionKeys)> {
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

pub fn default_invulnerables() -> Vec<(AccountId, AuraId)> {
	vec![
		(
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_collator_keys_from_seed("Alice"),
		),
		(get_account_id_from_seed::<sr25519::Public>("Bob"), get_collator_keys_from_seed("Bob")),
	]
}

pub fn default_council_members() -> Vec<AccountId> {
	vec![
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		get_account_id_from_seed::<sr25519::Public>("Charlie"),
		get_account_id_from_seed::<sr25519::Public>("Eve"),
	]
}

pub fn default_technical_committee_members() -> Vec<AccountId> {
	vec![
		get_account_id_from_seed::<sr25519::Public>("Bob"),
		get_account_id_from_seed::<sr25519::Public>("Dave"),
	]
}
