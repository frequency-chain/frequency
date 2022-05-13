use codec::Codec;
use common_primitives::{messages::*, rpc::*, schema::*};
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use pallet_messages_runtime_api::MessagesApi as MessagesRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait MessagesApi<BlockHash, AccountId, BlockNumber> {
	#[rpc(name = "messages_getBySchema")]
	fn get_messages_by_schema(
		&self,
		schema_id: SchemaId,
		pagination: BlockPaginationRequest<BlockNumber>,
	) -> Result<BlockPaginationResponse<BlockNumber, MessageResponse<AccountId, BlockNumber>>>;
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

impl<C, Block, AccountId, BlockNumber> MessagesApi<<Block as BlockT>::Hash, AccountId, BlockNumber>
	for MessagesHandler<C, Block>
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
	) -> Result<BlockPaginationResponse<BlockNumber, MessageResponse<AccountId, BlockNumber>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);
		let runtime_api_result = api.get_messages_by_schema(&at, schema_id, pagination);
		map_rpc_result(runtime_api_result)
	}
}
