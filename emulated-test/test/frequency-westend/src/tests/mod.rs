// mod teleport;

mod reserve_transfer_dot_from_asset_hub;
mod reserve_transfer_dot_from_relay;
mod reserve_transfer_dot_to_assethub;
mod reserve_transfer_dot_to_relay;

mod teleport_xfrqcy_to_assethub_with_dot_fee;

mod teleport_xfrqcy_with_dot_fee_from_assethub;

mod utils;

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
