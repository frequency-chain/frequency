pub use frequency_runtime::{self, xcm_config::RelayNetwork as FrequencyRelayNetworkId};

mod genesis;
pub use genesis::{genesis, FrequencySudoAccount, ED, PARA_ID};

// Substrate
use frame_support::traits::OnInitialize;
use sp_core::Encode;

// Cumulus
use emulated_integration_tests_common::{
	impl_accounts_helpers_for_parachain, impl_assert_events_helpers_for_parachain,
	impl_assets_helpers_for_parachain, impl_foreign_assets_helpers_for_parachain,
	impl_xcm_helpers_for_parachain,
	impls::{NetworkId, Parachain},
	xcm_emulator::decl_test_parachains,
};

// Polkadot
use staging_xcm::latest::WESTEND_GENESIS_HASH;

// Penpal Parachain declaration
decl_test_parachains! {
	pub struct FrequencyWestend {
		genesis = genesis(PARA_ID),
		on_init = {
			frequency_runtime::AuraExt::on_initialize(1);
			// frame_support::assert_ok!(frequency_runtime::System::set_storage(
			// 	frequency_runtime::RuntimeOrigin::root(),
			// 	vec![(FrequencyRelayNetworkId::get().unwrap(), NetworkId::ByGenesis(WESTEND_GENESIS_HASH).encode())],
			// ));
		},
		runtime = frequency_runtime,
		core = {
			XcmpMessageHandler: frequency_runtime::XcmpQueue,
			LocationToAccountId: frequency_runtime::xcm_config::LocationToAccountId,
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
impl_assert_events_helpers_for_parachain!(FrequencyWestend);
// impl_assets_helpers_for_parachain!(Frequency);
// impl_foreign_assets_helpers_for_parachain!(Frequency, staging_xcm::latest::Location);
impl_xcm_helpers_for_parachain!(FrequencyWestend);
