pub use crate::xcm::{
	asset_transactor::CheckingAccount,
	parameters::{HereLocation, RelayLocation},
};
use crate::{
	xcm::{queue::PriceForSiblingParachainDelivery, xcm_config::XcmConfig},
	AccountId, Balance, ExistentialDeposit, ForeignAssets, ParachainSystem, Runtime, RuntimeOrigin,
};
use cumulus_primitives_core::ParaId;
use frame_support::parameter_types;
use polkadot_runtime_common::xcm_sender::ToParachainDeliveryHelper;
use sp_runtime::traits::AccountIdConversion;
pub use staging_xcm::latest::prelude::{
	Asset, AssetId, Assets, Fungibility, Location, Parachain, Parent, ParentThen,
};
use staging_xcm_builder::MintLocation;
use xcm_executor::traits::{TransactAsset, XcmAssetTransfers};

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

	pub NativeAsset: Asset = (HereLocation::get(), Fungibility::Fungible(u128::MAX)).into();
	pub CheckAccount: Option<(AccountId, MintLocation)> = Some((AssetHubSovereignAccount::get(), MintLocation::Local));

	pub TrustedReserve: Option<(Location, Asset)> = Some((AssetHubParachainLocation::get(), RelayAsset::get()));
	pub TrustedTeleporter: Option<(Location, Asset)> = Some((AssetHubParachainLocation::get(), NativeAsset::get()));

}

pub type ParachainDeliveryHelper = ToParachainDeliveryHelper<
	XcmConfig,
	ExistentialDepositAsset,
	PriceForSiblingParachainDelivery,
	AssetHubParaId,
	ParachainSystem,
>;
