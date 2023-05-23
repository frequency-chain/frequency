// use cumulus_primitives_utility::ParentAsUmp;

use super::{
	AccountId, Balances, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime, RuntimeCall,
	RuntimeEvent, RuntimeOrigin, XcmpQueue,
};
use frame_support::{
	parameter_types,
	traits::{ConstU32, Everything, Nothing},
};

use frame_system::EnsureRoot;

pub use common_runtime::fee::WeightToFee;

use polkadot_parachain::primitives::Sibling;

use cumulus_pallet_xcm;

use pallet_xcm;
use xcm::{
	latest::{MultiLocation, NetworkId},
	prelude::{Here, Parachain},
};

use xcm_executor::XcmExecutor;

// Helpers to build XCM configuration
use xcm_builder::{
	AllowTopLevelPaidExecutionFrom, CurrencyAdapter, EnsureXcmOrigin, FixedWeightBounds,
	IsConcrete, LocationInverter, SiblingParachainAsNative, SiblingParachainConvertsVia,
	SignedToAccountId32, SovereignSignedViaLocation, UsingComponents, TakeWeightCredit, AccountId32Aliases,
};

// use common_primitives::node::AccountId;

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// Sibling parachain origin convert to AcountId via the `ParaId::into`.
	// SiblingParachainConvertsVia<Sibling, AccountId>,
	//
	AccountId32Aliases<RelayNetwork, AccountId>,
);

/// Means for transacting the native currency on this chain
pub type LocalAssetTrasaction = CurrencyAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<FrequencyLocation>,
	// Convert an XCM MultiLocation into a local account id:
	LocationToAccountId,
	// Our chains account ID type
	AccountId,
	// We do not track any teleport of `Balances`
	(),
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// if kind is Native and origin is a parachain, convert to a
	// ParachainOrigin origin.
	// SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// SignedToAccountId32
);

/// Means for transacting assets on this chain.
pub type AssetTransactors = (LocalAssetTrasaction,);

// The barriers one of which must be passed for an XCM message to be executed.
pub type Barrier = (
	// TakeWeightCredit,
	// If the message is one that immediately attemps to pay for execution, then allow it.
	// WithComputedOrigin<AllowTopLevelPaidExecutionFrom<Everything>>,
	AllowTopLevelPaidExecutionFrom<Everything>,
);

pub struct XcmConfig;
// Most function of these traits are called iwthing the executor upon
// receving specific instruction opcodes.
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	// How to withdraw and deposit assets
	type AssetTransactor = AssetTransactors;
	// Defines a single function to implement for converting the origin on the behalf of which,
	// the extrinsic call of a Transact instruction will be performed.
	// Converts a Multilocation to an OriginTrait type.
	type OriginConverter = XcmOriginToTransactDispatchOrigin;

	// 	It must return whether it accepts the origin as a reliable source for respectively a reserve or
	// teleported assets. The reserve check is performed when receiving a ReceiveAssetDeposited
	// instruction and the teleport when receiving a ReceiveTeleportedAsset.
	type IsReserve = ();
	type IsTeleporter = ();

	// 	The framework provides a single
	// implementation parameterized by the Ancestry which is Here for a relay chain and is Parachain
	// for a parachain.
	type LocationInverter = LocationInverter<Ancestry>;

	// Answer the question “should this message be executed?”
	type Barrier = Barrier;

	/// The means of determining an XCM message's weight.
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, ConstU32<100>>;

	// The way to purchase the weight necessary to execute the message.
	type Trader = UsingComponents<WeightToFee, FrequencyLocation, AccountId, Balances, ()>;

	// Enables implementing defined actions when receiving QueryResponse
	// DO we need this for version negotiation?
	type ResponseHandler = ();

	// The AssetTrap defines the action to perform on assets left in the holding of the XCVM after
	// executing a message.
	type AssetClaims = ();
	type AssetTrap = ();

	// Do we need this?
	// since we are not sending messages I do not think so.
	// Defines the action to perform when reeving XCM version changes notifications.
	type SubscriptionService = ();

	/// How to send an onward XCM message.
	/// Do we need this?
	type XcmSender = XcmRouter;
}

// Converts a local signed origin into an XCM multilocation.
// Forms the basis for local origins sending/executing XCMS.

parameter_types! {
	pub const RelayNetwork: NetworkId = NetworkId::Kusama;
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
	pub FrequencyLocation: MultiLocation = MultiLocation { parents: 0, interior: Here };
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
	XcmpQueue,
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

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

// A pallet which uses the XCMP transport layer to handle both incoming and outgoing XCM message sending and dispatch, queuing, signalling and backpressure.
// To do so, it implements:
//  - XcmpMessageHandler
//  - XcmpMessageSource
// Also provides an implementation of SendXcm which can be placed in a router tuple for relaying XCM over XCMP if the destination is Parent/Parachain. It requires an implementation of XcmExecutor for dispatching incoming XCM messages.
impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type ChannelInfo = ParachainSystem;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = ();
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type VersionWrapper = ();
	type WeightInfo = ();
	type XcmExecutor = XcmExecutor<XcmConfig>;
}
