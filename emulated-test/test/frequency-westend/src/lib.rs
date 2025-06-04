#[cfg(test)]
mod imports {
	pub use frame_support::{
		assert_ok,
		sp_runtime::DispatchResult,
		traits::{
			fungible::Inspect,
			fungibles::{Inspect as FungiblesInspect, Mutate as FungiblesMutate},
		},
		BoundedVec,
	};
	pub use staging_xcm_builder::{DescribeTerminus, HashedDescription};

	pub use staging_xcm::{
		latest::AssetTransferFilter,
		prelude::{AccountId32 as AccountId32Junction, *},
	};

	pub use emulated_integration_tests_common::{
		xcm_emulator::{
			assert_expected_events, bx, AccountIdOf, Chain, Parachain as Para, RelayChain as Relay,
			Test, TestArgs, TestContext, TestExt,
		},
		xcm_helpers::{fee_asset, non_fee_asset},
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
				xcm::CheckingAccount as FrequencyCheckingAccount,
				ExistentialDeposit as FrequencyExistentialDeposit,
			},
			FrequencyAssetOwner, FrequencyWestendParaPallet as FrequencyWestendPallet, TreasuryAccount as FrequencyTreasuryAccount,
		},
		westend_emulated_chain::{genesis::ED as WESTEND_ED, WestendRelayPallet as WestendPallet},
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
	pub type FrequencyToRelayTest = Test<FrequencyWestend, Westend>;

	pub fn frequency_balance_of(
		who: &AccountIdOf<<FrequencyWestend as Chain>::Runtime>,
	) -> Balance {
		FrequencyWestend::execute_with(|| {
			type Balances = <FrequencyWestend as FrequencyWestendPallet>::Balances;
			<Balances as Inspect<_>>::balance(who)
		})
	}

	pub fn assethub_balance_of(who: &AccountIdOf<<AssetHubWestend as Chain>::Runtime>) -> Balance {
		AssetHubWestend::execute_with(|| {
			type Balances = <AssetHubWestend as AssetHubWestendPallet>::Balances;
			<Balances as Inspect<_>>::balance(who)
		})
	}
}

#[macro_export]
macro_rules! foreign_balance_on {
	( $chain:ident, $id:expr, $who:expr ) => {
		emulated_integration_tests_common::impls::paste::paste! {
			<$chain>::execute_with(|| {
				type ForeignAssets = <$chain as [<$chain Pallet>]>::ForeignAssets;
				<ForeignAssets as FungiblesInspect<_>>::balance($id, $who)
			})
		}
	};
}

#[cfg(test)]
mod tests;
