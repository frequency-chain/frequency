// pub use asset_hub_westend_emulated_chain;
pub use frequency_emulated_chain_paseo;
use frequency_emulated_chain_paseo::FrequencyPaseo;

pub use paseo_emulated_chain;
use paseo_emulated_chain::Paseo;

// Cumulus
use emulated_integration_tests_common::{
	accounts::{ALICE, BOB},
	xcm_emulator::{decl_test_networks, decl_test_sender_receiver_accounts_parameter_types},
};

decl_test_networks! {
	pub struct PaseoMockNet {
		relay_chain = Paseo,
		parachains = vec![
			FrequencyPaseo,
		],
		bridge = ()
	},
}

decl_test_sender_receiver_accounts_parameter_types! {
	PaseoRelay { sender: ALICE, receiver: BOB },
	FrequencyPaseoPara { sender: ALICE, receiver: BOB }
}

// decl_test_sender_receiver_accounts_parameter_types! {
// 	WestendRelay { sender: ALICE, receiver: BOB },
// 	AssetHubWestendPara { sender: ALICE, receiver: BOB },
// 	FrequencyWestendPara { sender: ALICE, receiver: BOB }
// }
