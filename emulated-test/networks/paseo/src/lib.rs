// pub use asset_hub_paseo_emulated_chain;
pub use paseo_emulated_chain;
// pub use frequency_paseo_emulated_chain;

// use asset_hub_paseo_emulated_chain::AssetHubPaseo;
// use bridge_hub_paseo_emulated_chain::BridgeHubPaseo;
use paseo_emulated_chain::Westend as Paseo;
// use frequency_paseo_emulated_chain::FrequencyPaseo;

// Cumulus
use emulated_integration_tests_common::{
	xcm_emulator::decl_test_networks,
};

decl_test_networks! {
	pub struct PaseoMockNet {
		relay_chain = Paseo,
		parachains = vec![],
		bridge = ()
	},
}

// decl_test_networks! {
// 	pub struct PaseoMockNet {
// 		relay_chain = Paseo,
// 		parachains = vec![
// 			// AssetHubPaseo,
// 			// BridgeHubPaseo,
// 			FrequencyPaseo,
// 		],
// 		bridge = ()
// 	},
// }

// // decl_test_sender_receiver_accounts_parameter_types! {
// // 	PaseoRelay { sender: ALICE, receiver: BOB }
// 	// AssetHubPaseoPara { sender: ALICE, receiver: BOB },
// 	// BridgeHubPaseoPara { sender: ALICE, receiver: BOB },
// 	// FrequencyPaseoPara { sender: ALICE, receiver: BOB }
// // }