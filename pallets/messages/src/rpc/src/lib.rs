//! # Message API Interface
//!
//! - [`MessageApi`]: Message API trait.
//! - [`MessagesOAIHandler`]: OpenAPI handler for the Message API.
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]
/// Structs and traits for Messages OpenAPI.
pub mod messages_oai;
/// OpenAPI for messages pallet
pub mod open_api;
/// RPC for messages pallet
pub mod rpc;
#[cfg(test)]
mod tests;
