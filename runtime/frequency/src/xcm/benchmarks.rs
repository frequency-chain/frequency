pub use crate::xcm::{
	asset_transactor::CheckingAccount,
	parameters::{HereLocation, RelayLocation},
};
use crate::{
	xcm::{queue::PriceForSiblingParachainDelivery, xcm_config::XcmConfig},
	AccountId, Balance, ExistentialDeposit, ForeignAssets, ParachainSystem, RuntimeOrigin,
};
use cumulus_primitives_core::ParaId;
use frame_support::parameter_types;
use polkadot_runtime_common::xcm_sender::ToParachainDeliveryHelper;
use sp_runtime::traits::AccountIdConversion;
pub use staging_xcm::latest::prelude::{
	Asset, AssetId, Assets, Fungibility, InteriorLocation, Junction, Location, NetworkId,
	Parachain, Parent, ParentThen, Response,
};
use staging_xcm_builder::MintLocation;

parameter_types! {
	pub Dollars: Balance = 1_000_000_000_000;
	pub Cents: Balance = Dollars::get() / 100;
	pub RelayAssetId: AssetId = AssetId(RelayLocation::get());
	pub RelayAsset: Asset = (RelayAssetId::get(), Fungibility::Fungible((3 * Cents::get()) * (10 ^ 9))).into();
	pub ExistentialDepositAsset: Option<Asset> = Some((
		HereLocation::get(),
		ExistentialDeposit::get()
	).into());

	pub const AssetHubParaId: ParaId = ParaId::new(1000);

	pub AssetHubSovereignAccount: AccountId = AssetHubParaId::get().into_account_truncating();
	pub AssetHubParachainLocation: Location = ParentThen(Parachain(AssetHubParaId::get().into()).into()).into();

	pub NativeAsset: Asset = (HereLocation::get(), Fungibility::Fungible(ExistentialDeposit::get())).into();
	pub CheckAccount: Option<(AccountId, MintLocation)> = Some((CheckingAccount::get(), MintLocation::Local));

	pub TrustedReserve: Option<(Location, Asset)> = Some((AssetHubParachainLocation::get(), RelayAsset::get()));
	pub TrustedTeleporter: Option<(Location, Asset)> = Some((AssetHubParachainLocation::get(), NativeAsset::get()));

}

pub fn create_foreign_asset_DOT_on_frequency() {
	let owner: AccountId = frame_benchmarking::whitelisted_caller();

	let _ = ForeignAssets::force_create(
		RuntimeOrigin::root(),
		staging_xcm::latest::prelude::Parent.into(),
		owner.clone().into(),
		true,
		1u128,
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
}

pub type ParachainDeliveryHelper = ToParachainDeliveryHelper<
	XcmConfig,
	ExistentialDepositAsset,
	PriceForSiblingParachainDelivery,
	AssetHubParaId,
	ParachainSystem,
>;
