#[cfg(test)]
mod imports {
	pub use frame_support::{
		assert_ok,
		sp_runtime::DispatchResult,
		traits::fungible::Inspect,
		traits::fungibles::{
			Create as FungiblesCreate, Inspect as FungiblesInspect, Mutate as FungiblesMutate,
		},
	};

	// Polkadot
	pub use staging_xcm::{latest::WESTEND_GENESIS_HASH, prelude::*};

	// Cumulus
	pub use asset_test_utils::xcm_helpers;
	pub use emulated_integration_tests_common::{xcm_emulator::{
		assert_expected_events, bx, Chain, Parachain as Para, RelayChain as Relay, Test, TestArgs,
		TestContext, TestExt,}, RESERVABLE_ASSET_ID,
	};
	pub use parachains_common::Balance;
	pub use westend_system_emulated_network::{
		self,
		asset_hub_westend_emulated_chain::{
			asset_hub_westend_runtime::{
				xcm_config::WestendLocation, ExistentialDeposit as AssetHubExistentialDeposit,
			},
			AssetHubWestendParaPallet as AssetHubWestendPallet,
		},
		frequency_emulated_chain::{
			frequency_runtime::{
				self, xcm_config::XcmConfig as FrequencyWestendXcmConfig,
				ExistentialDeposit as FrequencyExistentialDeposit,
			},
			FrequencyWestendParaPallet as FrequencyWestendPallet,
		},
		westend_emulated_chain::{
			genesis::ED as WESTEND_ED, westend_runtime::xcm_config::XcmConfig as WestendXcmConfig,
			WestendRelayPallet as WestendPallet,
		},
		AssetHubWestendPara as AssetHubWestend,
		AssetHubWestendParaReceiver as AssetHubWestendReceiver,
		AssetHubWestendParaSender as AssetHubWestendSender,
		FrequencyWestendPara as FrequencyWestend,
		FrequencyWestendParaReceiver as FrequencyWestendReceiver,
		FrequencyWestendParaSender as FrequencyWestendSender, WestendRelay as Westend,
		WestendRelayReceiver as WestendReceiver, WestendRelaySender as WestendSender,
	};

	pub type AssetHubToFrequencyTest = Test<AssetHubWestend, FrequencyWestend>;
	pub type FrequencyToAssetHubTest = Test<FrequencyWestend, AssetHubWestend>;
	pub type RelayToFrequencyTest = Test<Westend, FrequencyWestend>;
}

#[cfg(test)]
mod tests;
