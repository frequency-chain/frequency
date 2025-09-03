#![cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]

// Relay-enabled runtime variants consolidate formerly scattered #[cfg] aliases & constants.
// Anything specific to parachain/XCM lives here so the main lib.rs can stay mostly cfg-free.

use super::*;

pub mod prelude {
	use super::*;

	// ---------------- Type Aliases ----------------
	pub type RuntimeMigrations =
		(MigratePalletsCurrentStorage<Runtime>, SetSafeXcmVersion<Runtime>);
	pub type OnSetCodeHook = cumulus_pallet_parachain_system::ParachainSetCode<Runtime>;
	pub type TimeReleaseBlockNumberProvider = RelaychainDataProvider<Runtime>;
	pub type OnTimestampSetHook = Aura;

	pub type ParachainDmpQueue =
		frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;

	// ---------------- Parachain authoring tuning ----------------
	pub const UNINCLUDED_SEGMENT_CAPACITY: u32 = 3; // Maximum unincluded blocks kept.
	pub const BLOCK_PROCESSING_VELOCITY: u32 = 1; // Parachain blocks processed per parent.
	pub const RELAY_CHAIN_SLOT_DURATION_MILLIS: u32 = 6_000; // Relay chain slot (ms).

	pub type ConsensusHook = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
		Runtime,
		RELAY_CHAIN_SLOT_DURATION_MILLIS,
		BLOCK_PROCESSING_VELOCITY,
		UNINCLUDED_SEGMENT_CAPACITY,
	>;
}

pub use prelude::*;

// Parachain system configuration (only compiled when relay enabled)
impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type DmpQueue = ParachainDmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberMonotonicallyIncreases;
	type WeightInfo = ();
	type ConsensusHook = ConsensusHook;
	type SelectCore = DefaultCoreSelector<Runtime>;
}

// Removed unused ConsensusHookType alias (lib uses ConsensusHook directly)

// Register validate block only for relay-enabled builds.
cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}
