pub use frequency_runtime::{self, xcm::RelayNetwork as FrequencyRelayNetworkId};

mod genesis;
pub use genesis::{genesis, FrequencyAssetOwner, FrequencySudoAccount, ED, PARA_ID};

use frame_support::traits::OnInitialize;

use emulated_integration_tests_common::{
	impl_accounts_helpers_for_parachain, impl_assert_events_helpers_for_parachain,
	impl_xcm_helpers_for_parachain, impls::Parachain, xcm_emulator::decl_test_parachains,
};

decl_test_parachains! {
	pub struct FrequencyWestend {
		genesis = genesis(PARA_ID),
		on_init = {
			frequency_runtime::AuraExt::on_initialize(1);
		},
		runtime = frequency_runtime,
		core = {
			XcmpMessageHandler: frequency_runtime::XcmpQueue,
			LocationToAccountId: frequency_runtime::xcm::LocationToAccountId,
			ParachainInfo: frequency_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: frequency_runtime::PolkadotXcm,
			ForeignAssets: frequency_runtime::ForeignAssets,
			Balances: frequency_runtime::Balances,
		}
	},
}

impl_accounts_helpers_for_parachain!(FrequencyWestend);
impl_xcm_helpers_for_parachain!(FrequencyWestend);
impl_assert_events_helpers_for_parachain!(FrequencyWestend);
