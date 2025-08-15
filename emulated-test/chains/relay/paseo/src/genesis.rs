// Substrate
use authority_discovery_primitives::AuthorityId as AuthorityDiscoveryId;
use babe_primitives::AuthorityId as BabeId;
use beefy_primitives::ecdsa_crypto::AuthorityId as BeefyId;
use emulated_integration_tests_common::validators;
use grandpa::AuthorityId as GrandpaId;
use sp_runtime::Perbill;

// Paseo
use polkadot_primitives::{AssignmentId, ValidatorId};

// Cumulus
use emulated_integration_tests_common::{accounts, build_genesis_storage, get_host_config};
use parachains_common::Balance;
use paseo_runtime_constants::currency::UNITS as PAS;

pub const ED: Balance = paseo_runtime::ExistentialDeposit::get();
const ENDOWMENT: u128 = 1_000_000 * PAS;
const STASH: u128 = 100 * PAS;

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	para_validator: ValidatorId,
	para_assignment: AssignmentId,
	authority_discovery: AuthorityDiscoveryId,
	beefy: BeefyId,
) -> paseo_runtime::SessionKeys {
	paseo_runtime::SessionKeys {
		babe,
		grandpa,
		para_validator,
		para_assignment,
		authority_discovery,
		beefy,
	}
}

pub fn genesis() -> sp_core::storage::Storage {
	let genesis_config = paseo_runtime::RuntimeGenesisConfig {
		system: paseo_runtime::SystemConfig::default(),
		balances: paseo_runtime::BalancesConfig {
			balances: accounts::init_balances().iter().cloned().map(|k| (k, ENDOWMENT)).collect(),
		},
		session: paseo_runtime::SessionConfig {
			keys: validators::initial_authorities()
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone(),
							x.6.clone(),
							x.7.clone(),
						),
					)
				})
				.collect::<Vec<_>>(),
			..Default::default()
		},
		staking: paseo_runtime::StakingConfig {
			validator_count: validators::initial_authorities().len() as u32,
			minimum_validator_count: 1,
			stakers: validators::initial_authorities()
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), STASH, pallet_staking::StakerStatus::Validator))
				.collect(),
			invulnerables: validators::initial_authorities().iter().map(|x| x.0.clone()).collect(),
			force_era: pallet_staking::Forcing::ForceNone,
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		},
		babe: paseo_runtime::BabeConfig {
			authorities: Default::default(),
			epoch_config: paseo_runtime::BABE_GENESIS_EPOCH_CONFIG,
			..Default::default()
		},
		configuration: paseo_runtime::ConfigurationConfig { config: get_host_config() },
		..Default::default()
	};

	build_genesis_storage(&genesis_config, paseo_runtime::WASM_BINARY.unwrap())
}