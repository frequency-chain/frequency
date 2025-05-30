use crate::{
	AccountId, MessageQueue, ParachainSystem, Runtime, RuntimeBlockWeights, RuntimeEvent, XcmpQueue,
};

#[cfg(not(feature = "runtime-benchmarks"))]
use crate::RuntimeCall;

use frame_support::{
	parameter_types,
	traits::{ConstU32, TransformOrigin},
	weights::Weight,
};

use crate::xcm::{
	location_converter::XcmOriginToTransactDispatchOrigin,
	parameters::{BaseDeliveryFee, FeeAssetId, TransactionByteFee},
};

use frame_system::EnsureRoot;
use staging_xcm_builder::WithUniqueTopic;

use cumulus_primitives_core::{AggregateMessageOrigin, ParaId};

use parachains_common::message_queue::{NarrowOriginToSibling, ParaIdToSibling};

pub use sp_runtime::{Perbill, Saturating};

#[cfg(not(feature = "runtime-benchmarks"))]
use crate::xcm::XcmConfig;

#[cfg(not(feature = "runtime-benchmarks"))]
use xcm_executor;

/// Pricing logic for sibling parachain message delivery
pub type PriceForSiblingParachainDelivery = polkadot_runtime_common::xcm_sender::ExponentialPrice<
	FeeAssetId,
	BaseDeliveryFee,
	TransactionByteFee,
	XcmpQueue,
>;

/// Pricing logic for relay message delivery
pub type PriceForParentDelivery = polkadot_runtime_common::xcm_sender::ExponentialPrice<
	FeeAssetId,
	BaseDeliveryFee,
	TransactionByteFee,
	ParachainSystem,
>;

parameter_types! {
	/// How much weight to allocate to the background message queue service
	pub MessageQueueServiceWeight: Weight = Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;
}

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = WithUniqueTopic<(
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, (), PriceForParentDelivery>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
)>;

/// Configures the queue that handles incoming XCMP messages
impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	// Enqueue XCMP messages from siblings for later processing.
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
	type MaxInboundSuspended = sp_core::ConstU32<1_000>;
	type MaxActiveOutboundChannels = ConstU32<128>;
	type MaxPageSize = ConstU32<{ 1 << 16 }>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = ();
	type PriceForSiblingDelivery = PriceForSiblingParachainDelivery;
}

/// Configures the local message queue service
impl pallet_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type MessageProcessor = pallet_message_queue::mock_helpers::NoopMessageProcessor<
		cumulus_primitives_core::AggregateMessageOrigin,
	>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MessageProcessor = staging_xcm_builder::ProcessXcmMessage<
		AggregateMessageOrigin,
		xcm_executor::XcmExecutor<XcmConfig>,
		RuntimeCall,
	>;
	type Size = u32;
	// The XCMP queue pallet is only ever able to handle the `Sibling(ParaId)` origin:
	type QueueChangeHandler = NarrowOriginToSibling<XcmpQueue>;
	type QueuePausedQuery = NarrowOriginToSibling<XcmpQueue>;
	type HeapSize = sp_core::ConstU32<{ 103 * 1024 }>;
	type MaxStale = sp_core::ConstU32<8>;
	type ServiceWeight = MessageQueueServiceWeight;
	type IdleMaxServiceWeight = ();
}
