use asset_hub_westend_emulated_chain::AssetHubWestend;
use emulated_integration_tests_common::{
	accounts::{ALICE, BOB},
	xcm_emulator::{decl_test_networks, decl_test_sender_receiver_accounts_parameter_types},
};
use westend_emulated_chain::Westend;

decl_test_networks! {
	pub struct WestendMockNet {
		relay_chain = Westend,
		parachains = vec![
			AssetHubWestend,
		],
		bridge = ()
	},
}

// use xcm_emulator::{decl_test_relay_chains};
// use sp_core::storage::Storage;
// use sp_runtime::{traits::AccountIdConversion, BuildStorage};
// use emulated_integration_tests_common::{accounts, build_genesis_storage, get_host_config, validators};

// pub use westend_runtime;

// pub mod genesis;

// decl_test_relay_chains! {
//     #[api_version(11)]
//     pub struct Westend {
//         on_init = (),
//         runtime = westend_runtime,
//         core = {
//             SovereignAccountOf: westend_runtime::xcm_config::LocationConverter,
//         },
//         pallets = {
//             XcmPallet: westend_runtime::XcmPallet,
//             Sudo: westend_runtime::Sudo,
//             Balances: westend_runtime::Balances,
//             Treasury: westend_runtime::Treasury,
//             AssetRate: westend_runtime::AssetRate,
//             Hrmp: westend_runtime::Hrmp,
//             Identity: westend_runtime::Identity,
//             IdentityMigrator: westend_runtime::IdentityMigrator,
//         }
//     },
// }
