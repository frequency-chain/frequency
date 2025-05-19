// mod teleport;

mod reserve_transfer_dot_from_asset_hub_to_frequency;
mod reserve_transfer_dot_from_frequency_to_asset_hub;
mod reserve_transfer_dot_from_relay_to_frequency;
mod reserve_transfer_dot_from_frequency_to_relay;

// mod hybrid_transfer_native_from_frequency_to_asset_hub;

mod fee_estimation;
mod mock;

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

// fn foreign_balance_on_frequency_westend(id: v5::Location, who: &AccountId) -> u128 {
// 	FrequencyWestend::execute_with(|| {
// 		type ForeignAssets = <FrequencyWestend as FrequencyWestendPallet>::ForeignAssets;
// 		<ForeignAssets as FungiblesInspect<_>>::balance($id, $who)
// 	})
// }
