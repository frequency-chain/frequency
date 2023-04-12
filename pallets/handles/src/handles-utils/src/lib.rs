//! This package provides utilities for handling handles.

#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod confusables;
pub mod converter;
mod types;
pub mod validator;

pub use confusables::*;
pub use converter::*;
pub use validator::*;
#[cfg(test)]
mod tests;
