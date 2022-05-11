use common_primitives::{rpc::*, schema::*};
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use pallet_schemas_runtime_api::SchemasRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait SchemasApi<BlockHash> {
	#[rpc(name = "schemas_getLatestSchemaId")]
	fn get_latest_schema_id(&self, at: Option<BlockHash>) -> Result<u16>;

	#[rpc(name = "schemas_getBySchemaId")]
	fn get_by_schema_id(&self, schema_id: SchemaId) -> Result<Option<SchemaResponse>>;
}

pub struct SchemasHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> SchemasHandler<C, M> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block> SchemasApi<<Block as BlockT>::Hash> for SchemasHandler<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: SchemasRuntimeApi<Block>,
{
	fn get_latest_schema_id(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u16> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		let schema_api_result = api.get_latest_schema_id(&at);
		map_rpc_result(schema_api_result)
	}

	fn get_by_schema_id(&self, schema_id: SchemaId) -> Result<Option<SchemaResponse>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);
		let schema_api_result = api.get_by_schema_id(&at, schema_id);
		map_rpc_result(schema_api_result)
	}
}
