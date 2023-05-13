// use cumulus_primitives_utility::ParentAsUmp;

use crate::{
	AccountId, Balance, Balances, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime,
	RuntimeCall, RuntimeEvent, RuntimeOrigin,
};
use frame_support::{
	parameter_types,
	traits::{Everything, Nothing},
};

use pallet_xcm;
use xcm::{
	latest::{MultiLocation, NetworkId},
	prelude::Parachain,
};

use xcm_executor::XcmExecutor;

// Helpers to build XCM configuration
use xcm_builder::{EnsureXcmOrigin, FixedWeightBounds, LocationInverter, SignedToAccountId32};

/// Means for transacting the native currency on this chain
pub type LocalAssetTrasaction = CurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<SelfReserve>,
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chains account ID type
	AccountId,
	// We do not track any teleport of `Balances`
>;

pub struct XcmConfig;
// Most function of these traits are called iwthing the executor upon
// receving specific instruction opcodes.
impl xcm_executor::Config for XcmConfig {
	// How to withdraw and deposit assets
	type AssetTransactor = LocalAssetTransactor;
	// type AssetClaims = ();
	// type AssetExchanger = ();
	// type AssetLocker = ();
	// type AssetTrap = ();
	// type Barrier = Barrier;
	// type CallDispatcher = RuntimeCall;
	// type FeeManager = ();
	// type IsReserve = ();
	// type IsTeleporter = ();
	// type MaxAssetsIntoHolding = ConstU32<64>;
	// type MessageExporter = ();
	// type OriginConverter = SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>;
	// type PalletInstancesInfo = AllPalletsWithSystem;
	// type ResponseHandler = ();
	// type RuntimeCall = RuntimeCall;
	// type SafeCallFilter = DipTransactSafeCalls;
	// type SubscriptionService = ();
	// type UniversalAliases = Nothing;
	// type UniversalLocation = UniversalLocation;
	// type Trader = UsingComponents<IdentityFee<Balance>, HereLocation, AccountId, Balances, ()>;
	// type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, ConstU32<100>>;
	// type XcmSender = XcmRouter;
}

// impl cumulus_pallet_xcmp_queue::Config for Runtime {
// 	type ChannelInfo = ParachainSystem;
// 	type ControllerOrigin = EnsureRoot<AccountId>;
// 	type ControllerOriginConverter = ();
// 	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
// 	type PriceForSiblingDelivery = ();
// 	type RuntimeEvent = RuntimeEvent;
// 	type VersionWrapper = ();
// 	type WeightInfo = ();
// 	type XcmExecutor = XcmExecutor<XcmConfig>;
// }

// Converts a local signed origin into an XCM multilocation.
// Forms the basis for local origins sending/executing XCMS.

parameter_types! {
	pub const RelayNetwork: NetworkId = NetworkId::Polkadot;
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

parameter_types! {
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: u64 = 1_000_000_000;
	pub const MaxInstructions: u32 = 100;
}

// Configuration Questions:
// 1. Do we want to expand the origin to include calls from collective pallet?
// we cand do this by using a tupple instead:
// pub type LocalOriginToLocation = (SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>, ...BackingToPlurality<>);

// 2. Do we need XcmpQueue if we are only receiving messages from sibling parachains?
// 3. If pallet-xcm necessary when only receiving message from other chains.
// 4. Do we want to filter pallet-xcm extrinsics?
// .   yes for version checking?
// 5. Do want to change the XcmTeleportFilter to filter everything and not allow any Xcm teleport (seems irrelavant)?
// 6. Weigher configuration seems irrelavent since no execution of XCM messages or sending is allowed.
// 7. Whats the latest xcm version?

/// Converts a local signed origin into an XCM multilocation.
/// Forms the basis for local origins sending/executing XCMs.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for lcoal execution into the right message queues.
/// Does not play a central role in terms of security for parachains.
// Two routers - use UMP to communicate with the relay chain:
pub type XcmRouter = (
	// Sends the message for UMP by proxying the `ParentAsUmp` which checks that the message destination is the parent.
	// Additionally, it checks that the version of the XCM based on the MultiLocation is valid.
	// After checking it calls ParachainSystem::send_upward_message which
	// checks that the messages is not bigger than what the relay chain allows. If the check passes, it places it in PendingUpwardMessages storage queue to be processed
	// on_initialize.
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm>,
	// ..and XCMP to communicate with the sibling chains.
	// XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
	// construct_runtime
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;
	// Origins: Required origin for sending and executing XCM messages. A general way
	// to autorized specific origins and convert them to MultiLocations.
	// called in execute, do_reserve_transfer_asset and do_teleport_assets.
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	// Called in the send extrinsic.
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	// XCM pallet needs an XCM router to be able to send messages.
	type XcmRouter = XcmRouter;

	// The main component that contains the XCVM and that will be used by the pallet
	// to interpret instructions.
	type XcmExecutor = XcmExecutor<XcmConfig>;

	// Filters called by their respective extrinsics.
	// Which xcm messages to filter when when executing.
	type XcmExecuteFilter = Everything;
	// which messages to be filtered when teleporting assets.
	type XcmTeleportFilter = Nothing;
	// which messages to be reserve-transferred using the dedicated extrinsic must pass.
	type XcmReserveTransferFilter = Nothing;

	// Weighter are necessary to dynamically weight XCM messages that can be cmposed of many instructions
	// and nested XCM messages. Its important to correctly adjust the fee that needs to e paid for execution.
	// Means of measuring the weight consumed by an XCM message locally.
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;

	// The lcoation inverter is used when sending XCM messages to reverse the references to assets.
	// For instance, as all lcoations are relative in XCM, they must be modifed when changing context.
	type LocationInverter = LocationInverter<Ancestry>;

	// Part of the version negotation. The queue is popped by the on_initialize hook, called at each block, one element at a time.
	// The queue contains destinations whose latest XCM version we would like to know.
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;

	// the XCM version that will be advertised by the palled when being asked. The current version is V2 The current version is V2.
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

// impl cumulus_pallet_xcmp_queue::Config for Runtime {

// }
