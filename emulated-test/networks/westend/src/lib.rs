pub use asset_hub_westend_emulated_chain;
pub use frequency_emulated_chain;
pub use westend_emulated_chain;

pub use asset_hub_westend_emulated_chain::AssetHubWestend;
use frequency_emulated_chain::FrequencyWestend;
use westend_emulated_chain::Westend;

// Cumulus
use emulated_integration_tests_common::{
	accounts::{ALICE, BOB},
	xcm_emulator::{decl_test_networks, decl_test_sender_receiver_accounts_parameter_types},
};

decl_test_networks! {
	pub struct WestendMockNet {
		relay_chain = Westend,
		parachains = vec![
			AssetHubWestend,
			FrequencyWestend,
		],
		bridge = ()
	},
}

decl_test_sender_receiver_accounts_parameter_types! {
	WestendRelay { sender: ALICE, receiver: BOB },
	AssetHubWestendPara { sender: ALICE, receiver: BOB },
	FrequencyWestendPara { sender: ALICE, receiver: BOB }
}
