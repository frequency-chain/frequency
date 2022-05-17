use codec::Codec;
use common_primitives::{
	msa::{KeyInfoResponse, MessageSenderId},
	rpc::*,
};
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use pallet_msa_runtime_api::MsaApi as MsaRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait MsaApi<BlockHash, AccountId, BlockNumber> {
	#[rpc(name = "msa_getMsaKeys")]
	fn get_msa_keys(
		&self,
		msa_id: MessageSenderId,
	) -> Result<Vec<KeyInfoResponse<AccountId, BlockNumber>>>;
}

/// A struct that implements the `MessagesApi`.
pub struct MsaHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> MsaHandler<C, M> {
	/// Create new `MessagesApi` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block, AccountId, BlockNumber> MsaApi<<Block as BlockT>::Hash, AccountId, BlockNumber>
	for MsaHandler<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: MsaRuntimeApi<Block, AccountId, BlockNumber>,
	AccountId: Codec,
	BlockNumber: Codec,
{
	fn get_msa_keys(
		&self,
		msa_id: MessageSenderId,
	) -> Result<Vec<KeyInfoResponse<AccountId, BlockNumber>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);
		let runtime_api_result = api.get_msa_keys(&at, msa_id);
		map_rpc_result(runtime_api_result)
	}
}
