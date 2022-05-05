use codec::Codec;
use core::result::Result as CoreResult;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use schema_runtime_api::SchemaRuntimeApi;
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT, DispatchError};
use std::sync::Arc;

#[rpc]
pub trait SchemaApi<BlockHash, AccountId> {
	#[rpc(name = "schema_get_latest_schema_id")]
	fn get_latest_schema_id(&self, at: Option<BlockHash>) -> Result<u16>;
}

pub struct SchemaHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> SchemaHandler<C, M> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block, AccountId> SchemaApi<<Block as BlockT>::Hash, AccountId> for SchemaHandler<C, Block>
where
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: SchemaRuntimeApi<Block, AccountId>,
	AccountId: Codec,
{
	fn get_latest_schema_id(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u16> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		let schema_api_result = api.get_latest_schema_id(&at);
		map_result(schema_api_result)
	}
}

fn map_result<T>(response: CoreResult<CoreResult<T, DispatchError>, ApiError>) -> Result<T> {
	match response {
		Ok(Ok(res)) => Ok(res),
		Ok(Err(DispatchError::Module(error))) => Err(RpcError {
			code: ErrorCode::ServerError(100), // No real reason for this value
			message: format!("Dispatch Error Module:{} error:{}", error.index, error.error).into(),
			data: Some(error.message.unwrap_or_default().into()),
		}),
		Ok(Err(e)) => Err(RpcError {
			code: ErrorCode::ServerError(200), // No real reason for this value
			message: "Dispatch Error".into(),
			data: Some(format!("{:?}", e).into()),
		}),
		Err(e) => Err(RpcError {
			code: ErrorCode::ServerError(300), // No real reason for this value
			message: "Api Error".into(),
			data: Some(format!("{:?}", e).into()),
		}),
	}
}
