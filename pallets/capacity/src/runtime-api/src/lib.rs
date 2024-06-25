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

use sp_std::vec::Vec;
use parity_scale_codec::{Codec, Decode, Encode, EncodeLike, MaxEncodedLen};
use common_primitives::capacity::{UnclaimedRewardInfoRPC};

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime files (the `runtime` folder)
sp_api::decl_runtime_apis! {
	/// Runtime Version for Capacity
	/// - MUST be incremented if anything changes
	/// - Also update in js/api-augment
	/// - See: https://paritytech.github.io/polkadot/doc/polkadot_primitives/runtime_api/index.html
	#[api_version(1)]

	/// Runtime APIs for [Capacity](../pallet_capacity/index.html)
    pub trait CapacityRuntimeApi<AccountId> where
		AccountId: Codec,
	{
		// TODO: this could just return the list_unclaimed_reward result
		/// Get the list of unclaimed rewards information for each eligible Reward Era.
        fn list_unclaimed_rewards(who: AccountId) -> Vec<UnclaimedRewardInfoRPC>;
    }
}