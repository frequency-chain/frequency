use crate::{AccountId, Balances, ForeignAssets, PolkadotXcm};
use staging_xcm::latest::prelude::*;
use staging_xcm_builder::{
	FungibleAdapter, FungiblesAdapter, IsConcrete, IsParentsOnly, MatchedConvertedConcreteId,
	NoChecking,
};

use crate::xcm::location_converter::LocationToAccountId;

use frame_support::{parameter_types, traits::ConstU8};

parameter_types! {
	/// A placeholder account used to hold reserved balances temporarily during checking.
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
	pub HereLocation: Location = Location::here();
}

/// Adapter for transacting native assets (like balances)
/// Means for transacting assets on this chain.
/// How xcm deals with assets on this chain
pub type LocalAssetTransactor = FungibleAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<HereLocation>,
	// Do a simple punn to convert an AccountId32 Location into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;

/// Adapter for transacting foreign assets (like DOT)
/// Type for specifying how a `Location` can be converted into an `AccountId`.
///// Transactors ///////
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
	// We do not allow teleportation of foreign assets. We only allow the reserve-based
	// transfer of USDT, USDC and DOT.
	NoChecking,
	CheckingAccount,
>;

/// The complete set of transactors
pub type AssetTransactors = (LocalAssetTransactor, ForeignAssetsAdapter);
