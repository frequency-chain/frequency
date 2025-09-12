///State migration module for the MSA pallet.
#[allow(missing_docs)]
pub mod v1;
/// Migration module for the MSA pallet.
pub mod v2;
pub use v2::MigrateProviderToRegistryEntryV2;
