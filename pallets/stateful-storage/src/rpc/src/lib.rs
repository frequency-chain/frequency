// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [Schemas](../pallet_schemas/index.html)

use common_helpers::rpc::map_rpc_result;
use common_primitives::{msa::MessageSourceId, schema::*, stateful_storage::PageResponse};
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
};
use pallet_stateful_storage_runtime_api::StatefulStorageRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use sp_std::vec::Vec;
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// Frequency Schema Custom RPC API
#[rpc(client, server)]
pub trait StatefulStorageApi<BlockHash> {
	/// retrieving schema by schema id
	#[method(name = "statefulStorage_getPages")]
	fn get_pages(
		&self,
		msa_id: MessageSourceId,
		schema_id: SchemaId,
	) -> RpcResult<Vec<PageResponse>>;
}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct StatefulStorageHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> StatefulStorageHandler<C, M> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

#[async_trait]
impl<C, Block> StatefulStorageApiServer<<Block as BlockT>::Hash>
	for StatefulStorageHandler<C, Block>
where
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: StatefulStorageRuntimeApi<Block>,
{
	fn get_pages(
		&self,
		msa_id: MessageSourceId,
		schema_id: SchemaId,
	) -> RpcResult<Vec<PageResponse>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);
		let schema_api_result = api.get_pages(&at, msa_id, schema_id);
		map_rpc_result(schema_api_result)
	}
}
