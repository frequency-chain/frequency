use codec::Codec;
use common_helpers::rpc::*;
use common_primitives::{
	msa::{Delegator, KeyInfoResponse, MessageSourceId, Provider},
	schema::SchemaId,
};

use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
};
use pallet_msa_runtime_api::MsaApi as MsaRuntimeApi;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc(client, server)]
pub trait MsaApi<BlockHash, AccountId> {
	#[method(name = "msa_getMsaKeys")]
	fn get_msa_keys(&self, msa_id: MessageSourceId) -> RpcResult<Vec<KeyInfoResponse<AccountId>>>;

	#[method(name = "msa_getMsaId")]
	fn get_msa_id(&self, key: AccountId) -> RpcResult<Option<MessageSourceId>>;

	#[method(name = "msa_checkDelegations")]
	fn check_delegations(
		&self,
		delegator_msa_ids: Vec<MessageSourceId>,
		provider_msa_id: MessageSourceId,
	) -> RpcResult<Vec<(MessageSourceId, bool)>>;

	#[method(name = "msa_grantedSchemaIds")]
	fn get_granted_schemas(
		&self,
		delegator_msa_id: MessageSourceId,
		provider_msa_id: MessageSourceId,
	) -> RpcResult<Option<Vec<SchemaId>>>;
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

#[async_trait]
impl<C, Block, AccountId> MsaApiServer<<Block as BlockT>::Hash, AccountId> for MsaHandler<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: MsaRuntimeApi<Block, AccountId>,
	AccountId: Codec,
{
	fn get_msa_keys(&self, msa_id: MessageSourceId) -> RpcResult<Vec<KeyInfoResponse<AccountId>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);
		let runtime_api_result = api.get_msa_keys(&at, msa_id);
		map_rpc_result(runtime_api_result)
	}

	fn get_msa_id(&self, key: AccountId) -> RpcResult<Option<MessageSourceId>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);
		let runtime_api_result = api.get_msa_id(&at, key);
		map_rpc_result(runtime_api_result)
	}

	fn check_delegations(
		&self,
		delegator_msa_ids: Vec<MessageSourceId>,
		provider_msa_id: MessageSourceId,
	) -> RpcResult<Vec<(MessageSourceId, bool)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);

		let provider = Provider(provider_msa_id);

		Ok(delegator_msa_ids
			.par_iter()
			.map(|&id| {
				let delegator = Delegator(id);
				(id, map_rpc_result(api.has_delegation(&at, delegator, provider)).unwrap())
			})
			.collect())
	}

	fn get_granted_schemas(
		&self,
		delegator_msa_id: MessageSourceId,
		provider_msa_id: MessageSourceId,
	) -> RpcResult<Option<Vec<SchemaId>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);
		let delegator = Delegator(delegator_msa_id);
		let provider = Provider(provider_msa_id);
		let runtime_api_result = api.get_granted_schemas(&at, delegator, provider);
		map_rpc_result(runtime_api_result)
	}
}
