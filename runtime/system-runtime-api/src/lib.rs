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

//! Runtime API definition for [Schemas](../pallet_schemas/index.html)
//!
//! This api must be implemented by the node runtime.
//! Runtime APIs Provide:
//! - An interface between the runtime and Custom RPCs.
//! - Runtime interfaces for end users beyond just State Queries

use frame_system::EventRecord;
use codec::{Codec, EncodeLike};
use frame_support::pallet_prelude::TypeInfo;
use frame_support::dispatch::fmt::Debug;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {

	/// Runtime Version for Schemas
	/// - MUST be incremented if anything changes
	/// - Also update in js/api-augment
	/// - See: https://paritytech.github.io/polkadot/doc/polkadot_primitives/runtime_api/index.html
	#[api_version(1)]

	/// Runtime API definition for [Schemas](../pallet_schemas/index.html)
	pub trait AdditionalRuntimeApi<RuntimeEvent, Hash> where
		Hash: Codec + Sync + Send + TypeInfo,
		RuntimeEvent: Codec + Sync + Send + TypeInfo + Debug + Eq + Clone + EncodeLike + 'static
	{
		/// Fetch the schema by id
		fn get_events(encoded: Vec<u8>) -> Vec<EventRecord<RuntimeEvent, Hash>>;
	}
}
