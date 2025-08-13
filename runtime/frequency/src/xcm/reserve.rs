use crate::xcm::parameters::{AssetHubLocation, NativeToken, RelayLocation};

use frame_support::{
	pallet_prelude::{Get, PhantomData},
	parameter_types,
	traits::ContainsPair,
};
use parachains_common::xcm_config::ConcreteAssetFromSystem;
use staging_xcm::latest::prelude::*;
use staging_xcm_builder::NativeAsset;

parameter_types! {
	pub ExceptAsset: (Location, AssetId) = (AssetHubLocation::get(), NativeToken::get());
}
/// Checks whether the asset originates from a given prefix location
pub struct AssetFromExcept<T>(core::marker::PhantomData<T>);

impl<T: frame_support::pallet_prelude::Get<(Location, AssetId)>> ContainsPair<Asset, Location>
	for AssetFromExcept<T>
{
	fn contains(asset: &Asset, location: &Location) -> bool {
		let (prefix, exclude_asset) = T::get();
		location == &prefix && asset.id != exclude_asset
	}
}

pub struct AssetPrefixFrom<Prefix, Origin>(PhantomData<(Prefix, Origin)>);
impl<Prefix, Origin> ContainsPair<Asset, Location> for AssetPrefixFrom<Prefix, Origin>
where
	Prefix: Get<Location>,
	Origin: Get<Location>,
{
	fn contains(asset: &Asset, origin: &Location) -> bool {
		let loc = Origin::get();
		&loc == origin &&
			matches!(asset, Asset { id: AssetId(asset_loc), fun: Fungible(_a) }
			if asset_loc.starts_with(&Prefix::get()))
	}
}

type AssetsFrom<T> = AssetPrefixFrom<T, T>;

/// Assets considered as reserve-based (e.g. DOT, native, system-registered)
pub type TrustedReserves = (
	NativeAsset,
	ConcreteAssetFromSystem<RelayLocation>,
	AssetsFrom<AssetHubLocation>,
	AssetFromExcept<ExceptAsset>,
);
