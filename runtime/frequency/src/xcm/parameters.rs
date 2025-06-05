use crate::{
	polkadot_xcm_fee::default_fee_per_second, Balance, ParachainInfo, RuntimeOrigin,
	MAXIMUM_BLOCK_WEIGHT,
};
use cumulus_primitives_core::AggregateMessageOrigin;
use frame_support::parameter_types;
use staging_xcm::latest::{prelude::*, WESTEND_GENESIS_HASH};

pub const MICROUNIT: Balance = 1_000_000;
pub const RELAY_GENESIS: [u8; 32] = WESTEND_GENESIS_HASH;
const ASSET_HUB_ID: u32 = 1000;

pub type ForeignAssetsAssetId = Location;

parameter_types! {
	pub const RelayLocation: Location = Location::parent();
	pub FeeAssetId: AssetId = AssetId(RelayLocation::get());
	pub const BaseDeliveryFee: u128 = (1_000_000_000_000u128 / 100).saturating_mul(3);
	pub const TransactionByteFee: Balance = 10 * MICROUNIT;
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
	pub const RelayNetwork: NetworkId = NetworkId::ByGenesis(RELAY_GENESIS);
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorLocation = [GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into())].into();

	pub RelayPerSecondAndByte: (AssetId, u128,u128) = (Location::new(1,Here).into(), default_fee_per_second(), 1024);
	pub HereLocation: Location = Location::here();

	pub NativeToken: AssetId = AssetId(Location::here());
	pub NativeTokenFilter: AssetFilter = Wild(AllOf { fun: WildFungible, id: NativeToken::get() });
	pub AssetHubLocation: Location = Location::new(1, [Parachain(ASSET_HUB_ID)]);
	pub NativeForAssetHub: (AssetFilter, Location) = (NativeTokenFilter::get(), AssetHubLocation::get());

	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	// XCM instruction weight cost
	// pub UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
	// pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
}
