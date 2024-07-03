#![allow(missing_docs)]
use common_primitives::{
	node::AccountId,
	schema::{ModelType, PayloadLocation, SchemaSetting},
};
use common_runtime::constants::{
	currency::EXISTENTIAL_DEPOSIT, FREQUENCY_LOCAL_TOKEN, TOKEN_DECIMALS,
};
use cumulus_primitives_core::ParaId;
use frequency_runtime::{
	pallet_schemas::GenesisSchema, AuraId, CouncilConfig, Ss58Prefix, SudoConfig,
	TechnicalCommitteeConfig,
};
use sc_service::ChainType;
use sp_core::sr25519;
use sp_runtime::traits::AccountIdConversion;
/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec =
	sc_service::GenericChainSpec<frequency_runtime::RuntimeGenesisConfig, Extensions>;

use super::{get_account_id_from_seed, get_collator_keys_from_seed, get_properties, Extensions};

/// Generates the chain spec for a development (no relay)
pub fn development_config() -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let properties =
		get_properties(FREQUENCY_LOCAL_TOKEN, TOKEN_DECIMALS as u32, Ss58Prefix::get().into());

	ChainSpec::builder(
		frequency_runtime::wasm_binary_unwrap(),
		Extensions {
			relay_chain: "dev".into(), // You MUST set this to the correct network!
			para_id: 1000,
		},
	)
	.with_name("Frequency Development (No Relay)")
	.with_id("dev")
	.with_properties(properties)
	.with_chain_type(ChainType::Development)
	.with_protocol_id("dev")
	.with_genesis_config(development_genesis(
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
		1000.into(),
	))
	.build()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
fn template_session_keys(keys: AuraId) -> frequency_runtime::SessionKeys {
	frequency_runtime::SessionKeys { aura: keys }
}

#[allow(clippy::unwrap_used)]
fn development_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root_key: Option<AccountId>,
	endowed_accounts: Vec<AccountId>,
	council_members: Vec<AccountId>,
	technical_committee_members: Vec<AccountId>,
	id: ParaId,
) -> serde_json::Value {
	let genesis = frequency_runtime::RuntimeGenesisConfig {
		system: Default::default(),
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
		schemas: frequency_runtime::pallet_schemas::GenesisConfig {
			initial_schemas: vec![
				GenesisSchema {
					model_type: ModelType::Parquet,
					payload_location: PayloadLocation::IPFS,
					model: b"{}".to_vec(),
					name: b"dsnp.broadcast".to_vec(),
					settings: Default::default(),
				},
				GenesisSchema {
					model_type: ModelType::AvroBinary,
					payload_location: PayloadLocation::Paginated,
					model: b"{}".to_vec(),
					name: b"dsnp.broadcast".to_vec(),
					settings: vec![SchemaSetting::SignatureRequired],
				},
			],
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

	serde_json::to_value(&genesis).unwrap()
}
