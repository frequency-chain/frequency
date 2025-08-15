pub mod genesis;
pub use paseo_runtime;

// Cumulus
use emulated_integration_tests_common::{
	impl_accounts_helpers_for_relay_chain, impl_assert_events_helpers_for_relay_chain,
	impl_hrmp_channels_helpers_for_relay_chain, impl_send_transact_helpers_for_relay_chain,
	xcm_emulator::decl_test_relay_chains,
};

// Paseo declaration
decl_test_relay_chains! {
	#[api_version(12)]
	pub struct Paseo {
		genesis = genesis::genesis(),
		on_init = (),
		runtime = paseo_runtime,
		core = {
			SovereignAccountOf: paseo_runtime::xcm_config::SovereignAccountOf,
		},
		pallets = {
			XcmPallet: paseo_runtime::XcmPallet,
			Balances: paseo_runtime::Balances,
			Treasury: paseo_runtime::Treasury,
			AssetRate: paseo_runtime::AssetRate,
			Hrmp: paseo_runtime::Hrmp,
		}
	},
}

// Paseo implementation
impl_accounts_helpers_for_relay_chain!(Paseo);
impl_assert_events_helpers_for_relay_chain!(Paseo);
impl_hrmp_channels_helpers_for_relay_chain!(Paseo);
impl_send_transact_helpers_for_relay_chain!(Paseo);