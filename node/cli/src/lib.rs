//! Frequency CLI library.

#![allow(missing_docs)]
#[cfg(feature = "cli")]
mod benchmarking;

#[cfg(feature = "cli")]
mod cli;

#[cfg(feature = "cli")]
mod command;

#[cfg(feature = "cli")]
mod export_metadata_cmd;

#[cfg(feature = "cli")]
mod runtime_version_cmd;

#[cfg(feature = "cli")]
pub use benchmarking::*;

#[cfg(feature = "cli")]
pub use cli::*;

#[cfg(feature = "cli")]
pub use command::*;

#[cfg(feature = "cli")]
pub use export_metadata_cmd::*;

#[cfg(feature = "cli")]
pub use runtime_version_cmd::*;

#[cfg(feature = "cli")]
pub use sc_cli::{Error, Result};

#[cfg(feature = "frequency-rococo-local")]
pub mod run_localchain;
#[cfg(any(not(feature = "frequency-rococo-local"), feature = "all-frequency-features"))]
pub mod run_parachain;
