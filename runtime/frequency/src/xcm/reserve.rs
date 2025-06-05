use crate::xcm::parameters::{AssetHubLocation, RelayLocation};

use frame_support::traits::ContainsPair;
use parachains_common::xcm_config::ConcreteAssetFromSystem;
use staging_xcm::latest::prelude::*;
use staging_xcm_builder::NativeAsset;

/// Checks whether the asset originates from a given prefix location
pub struct AssetFrom<T>(core::marker::PhantomData<T>);

impl<T: frame_support::pallet_prelude::Get<Location>> ContainsPair<Asset, Location>
	for AssetFrom<T>
{
	fn contains(_asset: &Asset, location: &Location) -> bool {
		let prefix = T::get();
		location == &prefix
	}
}

/// Assets considered as reserve-based (e.g. DOT, native, system-registered)
pub type TrustedReserves =
	(NativeAsset, ConcreteAssetFromSystem<RelayLocation>, AssetFrom<AssetHubLocation>);
