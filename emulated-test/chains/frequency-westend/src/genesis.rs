use frame_support::parameter_types;
use sp_core::storage::Storage;
use sp_keyring::Sr25519Keyring as Keyring;

// Cumulus
use emulated_integration_tests_common::{accounts, build_genesis_storage, collators};
use parachains_common::{AccountId, Balance};
// use frequency_runtime::xcm_config::{LocalReservableFromAssetHub, RelayLocation, UsdtFromAssetHub};

pub const PARA_ID: u32 = 2000;
pub const ED: Balance = frequency_runtime::EXISTENTIAL_DEPOSIT;

parameter_types! {
	pub FrequencySudoAccount: AccountId = Keyring::Alice.to_account_id();
	pub FrequencyAssetOwner: AccountId = FrequencySudoAccount::get();
}

pub fn genesis(para_id: u32) -> Storage {
	let genesis_config = frequency_runtime::RuntimeGenesisConfig {
		system: frequency_runtime::SystemConfig::default(),
		balances: frequency_runtime::BalancesConfig {
			balances: accounts::init_balances().iter().cloned().map(|k| (k, ED * 4096)).collect(),
			..Default::default()
		},
		parachain_info: frequency_runtime::ParachainInfoConfig {
			parachain_id: para_id.into(),
			..Default::default()
		},
		collator_selection: frequency_runtime::CollatorSelectionConfig {
			invulnerables: collators::invulnerables().iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: ED * 16,
			..Default::default()
		},
		session: frequency_runtime::SessionConfig {
			keys: collators::invulnerables()
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                             // account id
						acc,                                     // validator id
						frequency_runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
			..Default::default()
		},
		// polkadot_xcm: frequency_runtime::PolkadotXcmConfig {
		// 	safe_xcm_version: Some(SAFE_XCM_VERSION),
		// 	..Default::default()
		// },
		sudo: frequency_runtime::SudoConfig { key: Some(FrequencySudoAccount::get()) },
		// foreign_assets: frequency_runtime::ForeignAssetsConfig {
		// 	assets: vec![
		// 		// Relay Native asset representation
		// 		(RelayLocation::get(), FrequencyAssetOwner::get(), true, ED),
		// 	],
		// 	..Default::default()
		// },
		..Default::default()
	};

	build_genesis_storage(
		&genesis_config,
		frequency_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
	)
}
