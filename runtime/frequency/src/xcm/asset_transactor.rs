use crate::{AccountId, Balances, ForeignAssets, PolkadotXcm};
use staging_xcm::latest::prelude::*;
use staging_xcm_builder::{
	FungibleAdapter, FungiblesAdapter, IsConcrete, IsParentsOnly, MatchedConvertedConcreteId,
	MintLocation, NoChecking,
};

use crate::xcm::location_converter::LocationToAccountId;

use frame_support::{parameter_types, traits::ConstU8};

parameter_types! {
	pub HereLocation: Location = Location::here();
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
	pub Checking: (AccountId, MintLocation) = (CheckingAccount::get(), MintLocation::Local);
}

/// Adapter for transacting native assets (like balances)
pub type LocalAssetTransactor = FungibleAdapter<
	// Use native currency:
	Balances,
	// Use this currency when it is a fungible asset matching the native location
	IsConcrete<HereLocation>,
	// Do a simple punn to convert an AccountId32 Location into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We track teleports.
	Checking,
>;

/// Adapter for transacting foreign assets (like DOT)
/// Type for specifying how a `Location` can be converted into an `AccountId`.
pub type ForeignAssetsAdapter = FungiblesAdapter<
	ForeignAssets,
	// Use this currency when it is a fungible asset matching the given location or name:
	MatchedConvertedConcreteId<
		Location,
		u128,
		IsParentsOnly<ConstU8<1>>,
		xcm_executor::traits::JustTry,
		xcm_executor::traits::JustTry,
	>,
	// Convert an XCM Location into a local account id:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We do not allow teleportation of foreign assets
	NoChecking,
	CheckingAccount,
>;

/// The complete set of transactors
pub type AssetTransactors = (LocalAssetTransactor, ForeignAssetsAdapter);
