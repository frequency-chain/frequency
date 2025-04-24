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

//! Runtime API definition for [MSA](../pallet_msa/index.html)
//!
//! This api must be implemented by the node runtime.
//! Runtime APIs Provide:
//! - An interface between the runtime and Custom RPCs.
//! - Runtime interfaces for end users beyond just State Queries

use common_primitives::{msa::*, node::BlockNumber};
use parity_scale_codec::Codec;
extern crate alloc;
use alloc::vec::Vec;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime files (the `runtime` folder)
sp_api::decl_runtime_apis! {

	/// Runtime Version for MSAs
	/// - MUST be incremented if anything changes
	/// - See: https://paritytech.github.io/polkadot/doc/polkadot_primitives/runtime_api/index.html
	#[api_version(2)]

	/// Runtime API definition for [MSA](../pallet_msa/index.html)
	pub trait MsaRuntimeApi<AccountId> where
		AccountId: Codec,
	{
		/// Check to see if a delegation existed between the given delegator and provider at a given block
		fn has_delegation(delegator: DelegatorId, provider: ProviderId, block_number: BlockNumber, schema_id: Option<SchemaId>) -> bool;

		/// Get the list of schema permission grants (if any) that exist in any delegation between the delegator and provider
		/// The returned list contains both schema id and the block number at which permission was revoked (0 if currently not revoked)
		fn get_granted_schemas_by_msa_id(delegator: DelegatorId, provider: ProviderId) -> Option<Vec<SchemaGrant<SchemaId, BlockNumber>>>;

		/// Get the list of all delegated providers with schema permission grants (if any) that exist in any delegation between the delegator and provider
		/// The returned list contains both schema id and the block number at which permission was revoked (0 if currently not revoked)
		fn get_all_granted_delegations_by_msa_id(delegator: DelegatorId) -> Vec<DelegationResponse<SchemaId, BlockNumber>>;

		/// Get the Ethereum address of the given MSA as a 42-byte hex string (including 0x prefix)
		fn get_ethereum_address_for_msa_id(msa_id: MessageSourceId) -> [u8; 42];
	}
}
