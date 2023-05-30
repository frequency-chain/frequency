// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

use common_helpers::rpc::map_rpc_result;
use common_primitives::rpc::RpcEvent;
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
};
use sp_api::ProvideRuntimeApi;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;
use system_runtime_api::AdditionalRuntimeApi;

/// Frequency MSA Custom RPC API
#[rpc(client, server)]
pub trait FrequencyRpcApi<B: BlockT> {
	/// gets the events for a block hash
	#[method(name = "frequency_getEvents")]
	fn get_events(&self, at: B::Hash) -> RpcResult<Vec<RpcEvent>>;
}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct FrequencyRpcHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> FrequencyRpcHandler<C, M> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

#[async_trait]
impl<C, Block> FrequencyRpcApiServer<Block> for FrequencyRpcHandler<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C::Api: AdditionalRuntimeApi<Block>,
{
	fn get_events(&self, at: <Block as BlockT>::Hash) -> RpcResult<Vec<RpcEvent>> {
		let api = self.client.runtime_api();
		return map_rpc_result(api.get_events(at))
	}
}
