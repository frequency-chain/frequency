pub use frequency_runtime::{self, xcm::RelayNetwork as FrequencyRelayNetworkId, TreasuryAccount};

mod genesis;
pub use genesis::{genesis, FrequencyAssetOwner, FrequencySudoAccount, ED, PARA_ID};

use frame_support::traits::OnInitialize;

use emulated_integration_tests_common::{
	impl_accounts_helpers_for_parachain, impl_assert_events_helpers_for_parachain,
	impl_xcm_helpers_for_parachain, impls::Parachain, xcm_emulator::decl_test_parachains,
};

decl_test_parachains! {
	pub struct FrequencyPaseo {
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
			System: frequency_runtime::System,
			Balances: frequency_runtime::Balances,
			Sudo: frequency_runtime::Sudo,
		}
	},
}

impl_accounts_helpers_for_parachain!(FrequencyPaseo);
impl_xcm_helpers_for_parachain!(FrequencyPaseo);
impl_assert_events_helpers_for_parachain!(FrequencyPaseo);
