use frame_support::parameter_types;
use staging_xcm::latest::prelude::*;
use staging_xcm_builder::Case;

/// The location of AssetHub on the relay network.
pub const ASSET_HUB_ID: u32 = 1000;

parameter_types! {
	/// The location we use to represent native assets (e.g. XRQCY).
	pub NativeTokenLocation: Location = Location::here();
	/// A filter matching the native token only.
	pub NativeTokenFilter: AssetFilter = Wild(AllOf {
		fun: WildFungible,
		id: AssetId(NativeTokenLocation::get()),
	});

	pub AssetHubLocation: Location = Location::new(1, [Parachain(ASSET_HUB_ID)]);

	/// The specific (asset, destination) combination allowed for teleport.
	pub NativeForAssetHub: (AssetFilter, Location) = (
		NativeTokenFilter::get(),
		AssetHubLocation::get(),
	);
}

/// Assets allowed to teleport (e.g. native token to AssetHub)
pub type TrustedTeleporters = Case<NativeForAssetHub>;
