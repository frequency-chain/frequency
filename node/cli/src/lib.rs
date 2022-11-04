//! Frequency CLI library.

#![allow(missing_docs)]
#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "cli")]
mod command;
#[cfg(feature = "cli")]
mod export_metadata_cmd;

#[cfg(feature = "cli")]
mod export_open_api_cmd;

#[cfg(feature = "cli")]
pub use cli::*;

#[cfg(feature = "cli")]
pub use command::*;

#[cfg(feature = "cli")]
pub use export_metadata_cmd::*;

#[cfg(feature = "cli")]
pub use export_open_api_cmd::*;

#[cfg(feature = "cli")]
pub use sc_cli::{Error, Result};
