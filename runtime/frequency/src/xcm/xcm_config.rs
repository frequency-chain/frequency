use crate::{
	AccountId, AllPalletsWithSystem, Balances, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent,
	RuntimeOrigin,
};

use staging_xcm_builder::{EnsureXcmOrigin, FrameTransactionalProcessor};

use crate::xcm::{
	AssetTransactors, Barrier, LocalOriginToLocation, LocationToAccountId, MaxAssetsIntoHolding,
	Trader, TrustedReserves, TrustedTeleporters, UniversalLocation, Weigher,
	XcmOriginToTransactDispatchOrigin, XcmRouter, FeeManager,
};

#[cfg(feature = "runtime-benchmarks")]
use staging_xcm::latest::prelude::{Location, Parachain};

#[cfg(feature = "runtime-benchmarks")]
use crate::xcm::ForeignAssetsAssetId;

use frame_support::{
	parameter_types,
	traits::{ConstU32, Disabled, Everything, Nothing},
};

pub use common_runtime::fee::WeightToFee;

use frame_system::EnsureRoot;

use xcm_executor::{XcmExecutor};

parameter_types! {
	pub CheckingAccount: AccountId = PolkadotXcm::check_account();
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	type XcmEventEmitter = PolkadotXcm;
	// How to withdraw and deposit an asset.
	type AssetTransactor = AssetTransactors;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = TrustedReserves;
	// in order to register our asset in asset hub
	// once the asset is registered we can teleport our native asset to asset hub
	type IsTeleporter = TrustedTeleporters;
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = Weigher;
	type Trader = Trader;
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type AssetLocker = ();
	type AssetExchanger = ();
	type FeeManager = FeeManager;
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type Aliasers = Nothing;
	type TransactionalProcessor = FrameTransactionalProcessor;
	type HrmpNewChannelOpenRequestHandler = ();
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type XcmRecorder = PolkadotXcm;
}

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	// update to Nothing and create extrinsic
	type XcmExecuteFilter = Everything;
	// ^ Disable dispatchable execute on the XCM pallet.
	// Needs to be `Everything` for local testing.
	type XcmExecutor = XcmExecutor<XcmConfig>;
	// update to only allow to teleport native
	type XcmTeleportFilter = Everything;
	// Lets only allow reserve transfers of DOT
	type XcmReserveTransferFilter = Everything;
	type Weigher = Weigher;
	type UniversalLocation = UniversalLocation;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;

	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	// ^ Override for AdvertisedXcmVersion default
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	// I do not thingk we need this
	type SovereignAccountOf = LocationToAccountId;
	/// Not sure what this is for?
	type MaxLockers = ConstU32<8>;
	type WeightInfo = pallet_xcm::TestWeightInfo;
	type AdminOrigin = EnsureRoot<AccountId>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	// Aliasing is disabled: xcm_executor::Config::Aliasers is set to `Nothing`.
	type AuthorizedAliasConsideration = Disabled;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

/// Simple conversion of `u32` into an `AssetId` for use in benchmarking.
#[cfg(feature = "runtime-benchmarks")]
pub struct XcmBenchmarkHelper;
#[cfg(feature = "runtime-benchmarks")]
impl pallet_assets::BenchmarkHelper<ForeignAssetsAssetId> for XcmBenchmarkHelper {
	fn create_asset_id_parameter(id: u32) -> ForeignAssetsAssetId {
		Location::new(1, [Parachain(id)])
	}
}
