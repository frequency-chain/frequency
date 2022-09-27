use codec::Codec;
use common_helpers::rpc::*;
use common_primitives::{graph::*, msa::MessageSourceId, node::BlockNumber};
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
};
use pallet_graph_runtime_api::GraphApi as GraphRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc(client, server)]
pub trait GraphApi<BlockHash> {
	#[method(name = "graph_getPrivate")]
	fn get_private_graph(
		&self,
		at: Option<BlockHash>,
		msa_id: MessageSourceId,
	) -> RpcResult<Vec<(GraphKey, PrivatePage)>>;
}

/// A struct that implements the `MessagesApi`.
pub struct GraphHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> GraphHandler<C, M> {
	/// Create new `MessagesApi` instance with the given reference to the client.
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
	fn get_private_graph(
		&self,
		at: Option<<Block as BlockT>::Hash>,
		msa_id: MessageSourceId,
	) -> RpcResult<Vec<(GraphKey, PrivatePage)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		let runtime_api_result = api.get_private_graph(&at, msa_id);
		map_rpc_result(runtime_api_result)
	}
}
