pub use crate::xcm::parameters::{HereLocation, RelayLocation};
use crate::xcm::{queue::PriceForSiblingParachainDelivery, xcm_config::XcmConfig};
use crate::{
	AccountId, Balance, ExistentialDeposit, ForeignAssets, ParachainSystem, Runtime, RuntimeOrigin,
};
use cumulus_primitives_core::ParaId;
use frame_support::parameter_types;
use polkadot_runtime_common::xcm_sender::ToParachainDeliveryHelper;
use sp_runtime::traits::AccountIdConversion;
use staging_xcm::latest::prelude::{
	Asset, Assets, AssetId, Fungibility, Location, Parachain, Parent, ParentThen,
};
use xcm_executor::traits::TransactAsset;
use xcm_executor::traits::XcmAssetTransfers;

parameter_types! {
	pub Dollars: Balance = 1_000_000_000_000;
	pub Cents: Balance = Dollars::get() / 100;
	pub RelayAssetId: AssetId = AssetId(RelayLocation::get());
	pub RelayAsset: Asset = (RelayAssetId::get(), Fungibility::Fungible(Cents::get() / 10)).into();
	pub ExistentialDepositAsset: Option<Asset> = Some((
		HereLocation::get(),
		ExistentialDeposit::get()
	).into());

	pub const AssetHubParaId: ParaId = ParaId::new(1000);

	pub AssetHubSovereignAccount: AccountId = ParaId::from(AssetHubParaId::get()).into_account_truncating();
	pub AssetHubParachainLocation: Location = ParentThen(Parachain(AssetHubParaId::get().into()).into()).into();
}

pub type ParachainDeliveryHelper = ToParachainDeliveryHelper<
	XcmConfig,
	ExistentialDepositAsset,
	PriceForSiblingParachainDelivery,
	AssetHubParaId,
	ParachainSystem,
>;

pub fn benchamark_asset_transfer_patch() {
	let owner: AccountId = frame_benchmarking::whitelisted_caller();

	let _ = ForeignAssets::force_create(
		RuntimeOrigin::root(),
		staging_xcm::latest::prelude::Parent.into(),
		owner.clone().into(),
		true,
		1u128.into(),
	);

	let amount = match RelayAsset::get().fun {
		Fungibility::Fungible(amount) => amount,
		_ => panic!("Asset is not fungible"),
	};

	let _ = ForeignAssets::mint(
		RuntimeOrigin::signed(owner),
		Parent.into(),
		AssetHubSovereignAccount::get().into(),
		amount * 2,
	);

	let _ = <<Runtime as pallet_xcm::Config>::XcmExecutor as XcmAssetTransfers>::AssetTransactor::deposit_asset(
		&Asset { fun: Fungibility::Fungible(amount), id: RelayAssetId::get().into() },
		&AssetHubParachainLocation::get(),
		None,
	);
}

pub fn benchamark_set_up_complex_transfer(
) -> Option<(Assets, u32, Location, Box<dyn FnOnce()>)> {

}
