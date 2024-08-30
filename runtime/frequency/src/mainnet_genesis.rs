use common_primitives::node::Verify;
use cumulus_primitives_core::ParaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::IdentifyAccount;
#[cfg(not(feature = "std"))]
use sp_std::alloc::format;

use crate::*;

type AccountPublic = <Signature as Verify>::Signer;

#[allow(clippy::unwrap_used)]
fn frequency_genesis(
	initial_authorities: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<(AccountId, Balance)>,
	council_members: Vec<AccountId>,
	technical_committee_members: Vec<AccountId>,
	candidacy_bond: Balance,
	id: ParaId,
) -> serde_json::Value {
	let genesis = RuntimeGenesisConfig {
		system: SystemConfig { ..Default::default() },
		balances: BalancesConfig { balances: endowed_accounts },
		parachain_info: ParachainInfoConfig { parachain_id: id, ..Default::default() },
		collator_selection: CollatorSelectionConfig {
			invulnerables: initial_authorities.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond,
			desired_candidates: 0,
		},
		session: SessionConfig {
			keys: initial_authorities
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),          // account id
						acc,                  // validator id
						SessionKeys { aura }, // session keys
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

pub fn frequency_initial_authorities() -> Vec<(AccountId, AuraId)> {
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
	]
}

pub fn frequency_endowed_accounts() -> Vec<AccountId> {
	// Total initial tokens: 1_000_000_000 FRQCY
	vec![
		// Project Liberty (5% Total)
		(
			project_liberty_keys::MULTISIG_THRESHOLD_2.parse::<AccountId>().unwrap().into(),
			49_999_700 * DOLLARS,
		),
		(
			project_liberty_keys::SIGNATORY_1_OF_3.parse::<AccountId>().unwrap().into(),
			100 * DOLLARS,
		),
		(
			project_liberty_keys::SIGNATORY_2_OF_3.parse::<AccountId>().unwrap().into(),
			100 * DOLLARS,
		),
		(
			project_liberty_keys::SIGNATORY_3_OF_3.parse::<AccountId>().unwrap().into(),
			100 * DOLLARS,
		),
		// Unfinished Labs (40% Total)
		(
			unfinished_labs_keys::MULTISIG_THRESHOLD_2.parse::<AccountId>().unwrap().into(),
			399_999_700 * DOLLARS,
		),
		(
			unfinished_labs_keys::SIGNATORY_1_OF_3.parse::<AccountId>().unwrap().into(),
			100 * DOLLARS,
		),
		(
			unfinished_labs_keys::SIGNATORY_2_OF_3.parse::<AccountId>().unwrap().into(),
			100 * DOLLARS,
		),
		(
			unfinished_labs_keys::SIGNATORY_3_OF_3.parse::<AccountId>().unwrap().into(),
			100 * DOLLARS,
		),
		// Foundation Operational (2% Total)
		(
			foundation_keys::MULTISIG_THRESHOLD_2.parse::<AccountId>().unwrap().into(),
			19_999_700 * DOLLARS,
		),
		(foundation_keys::SIGNATORY_1_OF_3.parse::<AccountId>().unwrap().into(), 100 * DOLLARS),
		(foundation_keys::SIGNATORY_2_OF_3.parse::<AccountId>().unwrap().into(), 100 * DOLLARS),
		(foundation_keys::SIGNATORY_3_OF_3.parse::<AccountId>().unwrap().into(), 100 * DOLLARS),
		// Frequency (53% Total)
		(
			// Treasury Pallet
			common_runtime::constants::TREASURY_PALLET_ID.into_account_truncating(),
			529_988_600 * DOLLARS,
		),
		(
			// Council Member
			frequency_mainnet_keys::FREQUENCY_COUNCIL_1.parse::<AccountId>().unwrap().into(),
			200 * DOLLARS,
		),
		(
			// Council Member
			frequency_mainnet_keys::FREQUENCY_COUNCIL_2.parse::<AccountId>().unwrap().into(),
			200 * DOLLARS,
		),
		(
			// Council Member
			frequency_mainnet_keys::FREQUENCY_COUNCIL_3.parse::<AccountId>().unwrap().into(),
			200 * DOLLARS,
		),
		(
			// Council Member
			frequency_mainnet_keys::FREQUENCY_COUNCIL_4.parse::<AccountId>().unwrap().into(),
			200 * DOLLARS,
		),
		(
			// Council Member and TECHNICAL_COUNCIL_2
			frequency_mainnet_keys::FREQUENCY_COUNCIL_5.parse::<AccountId>().unwrap().into(),
			200 * DOLLARS,
		),
		(
			// Council Member
			frequency_mainnet_keys::TECHNICAL_COUNCIL_1.parse::<AccountId>().unwrap().into(),
			200 * DOLLARS,
		),
		(
			// Council Member
			frequency_mainnet_keys::TECHNICAL_COUNCIL_3.parse::<AccountId>().unwrap().into(),
			200 * DOLLARS,
		),
	]
}

fn frequency_council_members() -> Vec<AccountId> {
	vec![
		frequency_mainnet_keys::FREQUENCY_COUNCIL_1.parse::<AccountId>().unwrap().into(),
		frequency_mainnet_keys::FREQUENCY_COUNCIL_2.parse::<AccountId>().unwrap().into(),
		frequency_mainnet_keys::FREQUENCY_COUNCIL_3.parse::<AccountId>().unwrap().into(),
		frequency_mainnet_keys::FREQUENCY_COUNCIL_4.parse::<AccountId>().unwrap().into(),
		frequency_mainnet_keys::FREQUENCY_COUNCIL_5.parse::<AccountId>().unwrap().into(),
	]
}

fn frequency_technical_committee_members() -> Vec<AccountId> {
	vec![
		frequency_mainnet_keys::TECHNICAL_COUNCIL_1.parse::<AccountId>().unwrap().into(),
		frequency_mainnet_keys::TECHNICAL_COUNCIL_2.parse::<AccountId>().unwrap().into(),
		frequency_mainnet_keys::TECHNICAL_COUNCIL_3.parse::<AccountId>().unwrap().into(),
	]
}
