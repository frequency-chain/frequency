use common_helpers::rpc::*;
use common_primitives::msa::MessageSourceId;
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
};
use pallet_graph_runtime_api::GraphRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use sp_std::vec::Vec;
use std::sync::Arc;

/// Frequency Schema API
#[rpc(client, server)]
pub trait GraphApi<BlockHash> {
	/// get connections
	#[method(name = "graph_publicGetFollowingList")]
	fn get_following_list_public(
		&self,
		at: Option<BlockHash>,
		static_id: MessageSourceId,
	) -> RpcResult<Vec<MessageSourceId>>;
}

/// A struct that implements the `GraphApi`.
pub struct GraphHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> GraphHandler<C, M> {
	/// Create new `GraphApi` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

#[async_trait]
impl<C, Block> GraphApiServer<<Block as BlockT>::Hash> for GraphHandler<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: GraphRuntimeApi<Block>,
{
	fn get_following_list_public(
		&self,
		at: Option<<Block as BlockT>::Hash>,
		static_id: MessageSourceId,
	) -> RpcResult<Vec<MessageSourceId>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api.get_following_list_public(&at, static_id);
		map_rpc_result(result)
	}
}
