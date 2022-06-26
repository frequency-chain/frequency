use codec::Codec;
use common_primitives::{messages::*, rpc::*, schema::*};
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorCode, ErrorObject},
};
use pallet_messages_runtime_api::MessagesApi as MessagesRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc(client, server)]
pub trait MessagesApi<BlockHash, AccountId, BlockNumber> {
	#[method(name = "messages_getBySchema")]
	fn get_messages_by_schema(
		&self,
		schema_id: SchemaId,
		pagination: BlockPaginationRequest<BlockNumber>,
	) -> RpcResult<BlockPaginationResponse<BlockNumber, MessageResponse<AccountId, BlockNumber>>>;
}

/// A struct that implements the `MessagesApi`.
pub struct MessagesHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> MessagesHandler<C, M> {
	/// Create new `MessagesApi` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

#[async_trait]
impl<C, Block, AccountId, BlockNumber>
	MessagesApiServer<<Block as BlockT>::Hash, AccountId, BlockNumber> for MessagesHandler<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: MessagesRuntimeApi<Block, AccountId, BlockNumber>,
	AccountId: Codec,
	BlockNumber: Codec,
{
	fn get_messages_by_schema(
		&self,
		schema_id: SchemaId,
		pagination: BlockPaginationRequest<BlockNumber>,
	) -> RpcResult<BlockPaginationResponse<BlockNumber, MessageResponse<AccountId, BlockNumber>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);
		let runtime_api_result = api.get_messages_by_schema(&at, schema_id, pagination);
		map_rpc_result(runtime_api_result)
	}
}
