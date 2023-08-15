#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]
#![allow(rustdoc::bare_urls)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]
//! Runtime API definition for [Capacity](../pallet_capacity/index.html)
//!
//! This api must be implemented by the node runtime.
//! Runtime APIs Provide:
//! - An interface between the runtime and Custom RPCs.
//! - Runtime interfaces for end users beyond just State Queries

use codec::Codec;
use frame_support::sp_runtime;
use sp_runtime::traits::MaybeDisplay;

pub use pallet_transaction_payment::{FeeDetails, InclusionFee};

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime files (the `runtime` folder)
sp_api::decl_runtime_apis! {
	/// Runtime Version for Frequency Payment
	#[api_version(1)]
	pub trait CapacityTransactionPaymentApi<Balance> where
		Balance: Codec + MaybeDisplay,
	{
		/// Query the capacity fee details for a given extrinsic.
		fn query_capacity_fee_details(uxt: Block::Extrinsic, len: u32) -> FeeDetails<Balance>;
	}
}
