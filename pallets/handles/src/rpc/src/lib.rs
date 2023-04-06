// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [Stateful-Storage](../pallet_stateful_storage/index.html)

use common_helpers::rpc::map_rpc_result;
use common_primitives::{
	handles::{HandleResponse, HandleSuffix},
	msa::MessageSourceId,
};
use jsonrpsee::{
	core::{async_trait, Error as RpcError, RpcResult},
	proc_macros::rpc,
};

use pallet_handles_runtime_api::HandlesRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// Frequency Handles Custom RPC API
#[rpc(client, server)]
pub trait HandlesApi<BlockHash> {
	/// retrieve handle for a given msa
	#[method(name = "handles_getHandleForMsa")]
	fn get_handle_for_msa(&self, msa_id: MessageSourceId) -> RpcResult<Option<HandleResponse>>;

	/// retrieve next `n` suffixes for a given handle
	#[method(name = "handles_getNextSuffixes")]
	fn get_next_suffixes(&self, handle: String, count: u16) -> RpcResult<Vec<HandleSuffix>>;
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

/// Errors that occur on the client RPC
#[derive(Debug)]
pub enum HandlesRpcError {}

impl From<HandlesRpcError> for RpcError {
	fn from(e: HandlesRpcError) -> Self {
		RpcError::Custom(format!("{:?}", e))
	}
}

#[async_trait]
impl<C, Block> HandlesApiServer<<Block as BlockT>::Hash> for HandlesHandler<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: HandlesRuntimeApi<Block>,
{
	fn get_handle_for_msa(&self, msa_id: MessageSourceId) -> RpcResult<Option<HandleResponse>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);
		let result = api.get_handle_for_msa(&at, msa_id);
		map_rpc_result(result)
	}

	fn get_next_suffixes(&self, handle: String, count: u16) -> RpcResult<Vec<HandleSuffix>> {
		let api = self.client.runtime_api();
		let handle_bytes = handle.as_bytes().to_vec();
		let at = BlockId::hash(self.client.info().best_hash);
		let result = api.get_next_suffixes(&at, handle_bytes, count);
		map_rpc_result(result)
	}
}
