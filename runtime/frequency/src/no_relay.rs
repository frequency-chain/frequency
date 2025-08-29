#![cfg(feature = "frequency-no-relay")]

// No-relay (standalone) runtime variants centralize relay-disabled aliases.
// Keeps lib.rs cleaner by avoiding scattered #[cfg(feature = "frequency-no-relay")]

use super::*;

pub mod prelude {
	use super::*;

	pub type RuntimeMigrations = (MigratePalletsCurrentStorage<Runtime>,);
	pub type OnSetCodeHook = (); // No parachain set code hook
	pub type TimeReleaseBlockNumberProvider = System; // Local block number provider
	pub type OnTimestampSetHook = (); // No Aura timestamp hook coupling

	// TODO: Was this actually used in no-relay?
	// pub type ParachainDmpQueue = frame_support::traits::EnqueueWithOrigin<(), sp_core::ConstU8<0>>;
}
// No public re-export; lib.rs only pulls the specific needed aliases.
