// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [Schemas](../pallet_schemas/index.html)

use common_helpers::rpc::map_rpc_result;
use common_primitives::schema::*;
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
};
use pallet_schemas_runtime_api::SchemasRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
extern crate alloc;
use alloc::sync::Arc;

#[cfg(test)]
mod tests;

/// Frequency Schema Custom RPC API
#[rpc(client, server)]
pub trait SchemasApi<BlockHash> {
	/// retrieving schema by schema id
	#[method(name = "schemas_getBySchemaId")]
	fn get_by_schema_id(&self, schema_id: SchemaId) -> RpcResult<Option<SchemaResponse>>;
}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct SchemasHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> SchemasHandler<C, M> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

#[async_trait]
impl<C, Block> SchemasApiServer<<Block as BlockT>::Hash> for SchemasHandler<C, Block>
where
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: SchemasRuntimeApi<Block>,
{
	fn get_by_schema_id(&self, schema_id: SchemaId) -> RpcResult<Option<SchemaResponse>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;
		#[allow(deprecated)]
		let schema_api_result = api.get_by_schema_id(at, schema_id);
		map_rpc_result(schema_api_result)
	}
}
