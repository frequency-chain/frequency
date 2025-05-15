use crate::{
	AccountId, Balance, MessageQueue, ParachainSystem, Runtime, RuntimeBlockWeights, RuntimeEvent,
	XcmpQueue,
};

#[cfg(not(feature = "runtime-benchmarks"))]
use crate::RuntimeCall;

use frame_support::{
	parameter_types,
	traits::{ConstU32, TransformOrigin},
	weights::Weight,
};

use crate::xcm_commons::{AssetLocationId, XcmOriginToTransactDispatchOrigin};

use frame_system::EnsureRoot;

use cumulus_primitives_core::{AggregateMessageOrigin, ParaId};

use parachains_common::message_queue::{NarrowOriginToSibling, ParaIdToSibling};

// use xcm_config;
#[cfg(not(feature = "runtime-benchmarks"))]
use crate::xcm_config;

// use polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery;

pub use sp_runtime::{Perbill, Saturating};

#[cfg(not(feature = "runtime-benchmarks"))]
use xcm_executor;

parameter_types! {
	/// The asset ID for the asset that we use to pay for message delivery fees.
	pub FeeAssetId: AssetLocationId = AssetLocationId(xcm_config::RelayLocation::get());
	/// The base fee for the message delivery fees (3 CENTS).
	pub const BaseDeliveryFee: u128 = (1_000_000_000_000u128 / 100).saturating_mul(3);
}

pub const MICROUNIT: Balance = 1_000_000;

parameter_types! {
	/// Relay Chain `TransactionByteFee` / 10
	pub const TransactionByteFee: Balance = 10 * MICROUNIT;
}

pub type PriceForSiblingParachainDelivery = polkadot_runtime_common::xcm_sender::ExponentialPrice<
	FeeAssetId,
	BaseDeliveryFee,
	TransactionByteFee,
	XcmpQueue,
>;

parameter_types! {
	pub MessageQueueServiceWeight: Weight = Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;
}

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
		xcm_executor::XcmExecutor<xcm_config::XcmConfig>,
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
