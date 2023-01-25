#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Runtime API definition for [Stateful Message Storage](../pallet_stateful_message_storage/index.html)
//!
//! This api must be implemented by the node runtime.
//! Runtime APIs Provide:
//! - An interface between the runtime and Custom RPCs.
//! - Runtime interfaces for end users beyond just State Queries

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime files (the `runtime` folder)
sp_api::decl_runtime_apis! {

	/// Runtime Version for Stateful Message Storage
	/// - MUST be incremented if anything changes
	/// - Also update in js/api-augment
	/// - See: https://paritytech.github.io/polkadot/doc/polkadot_primitives/runtime_api/index.html
	#[api_version(1)]

	/// Runtime APIs for [Stateful Message Storage](../pallet_stateful_message_storage/index.html)
	pub trait StatefulMessgeStorageRuntimeApi
	{
		/// dummy
		fn dummy();
	}
}
