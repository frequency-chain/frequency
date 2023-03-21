// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [Stateful-Storage](../pallet_stateful_storage/index.html)

use common_primitives::{
	msa::MessageSourceId,
	schema::*,
	stateful_storage::{ItemizedStoragePageResponse, PaginatedStorageResponse},
};
use jsonrpsee::{
	core::{async_trait, Error as RpcError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorCode, ErrorObject},
};
use pallet_stateful_storage_runtime_api::StatefulStorageRuntimeApi;
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT, DispatchError};
use sp_std::vec::Vec;
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// Frequency Handles Custom RPC API
#[rpc(client, server)]
pub trait HandlesApi<BlockHash> {
	
}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct HandlesHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> HandlesHandler<C, M> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}
