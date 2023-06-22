use super::{
	AccountId, Balances, ParachainInfo, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin,
};

use cumulus_primitives_core::InteriorMultiLocation;
use frame_support::{
	parameter_types,
	traits::{ConstU32, Nothing},
};

use frame_system::EnsureRoot;

use xcm::laster::prelude::*;

use xcm_builder::{
	AccountId32Aliases, EnsureXcmOrigin, FixedWeightBounds, ParentIsPreset,
	SiblingParachainConvertsVia,
};

use polkadot_parachain::primitives::Sibling;

parameter_types! {
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
	pub const MaxInstructions: u32 = 100;
}

parameter_types! {
	pub const RelayNetwork: Option<NetworkId> = None;
	pub UniversalLocation: InteriorMultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();0
}

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
    // When would we ask the Relay chain to send xcm?
	// The parent (Relay-chain) origin converts to the parent `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
    // since we are not allowing local xcm calls we can remove this.
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// Version discovery
// Configurations are to not allow to send or receive XCM messages.
// Used for versioning verification.
impl pallet_xcm::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;

	type UniversalLocation = UniversalLocation;

	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmExecuteFilter = None;

	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Nothing;

	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;

	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;

	type Currency = Balance;
	type TrustedLockers = ();
	type CurrencyMatcher = ();
	type MaxLockers = ConstU32<8>;

	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	type AdminOrigin = EnsureRoot<AccountId>;

	type SovereignAccountOf = LocationToAccountId;

	// No need to calculate these since extrinsics will not be allowed.
	type WeightInfo = pallet_xcm::TestWeightInfo;
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
}

// TODO: Add filter for teleporting extrinsics
