use codec::Codec;
use common_primitives::{messages::*, schema::*};
use core::result::Result as CoreResult;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_messages_runtime_api::MessagesApi as MessagesRuntimeApi;
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT, DispatchError};
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
		map_result(runtime_api_result)
	}
}

fn map_result<T>(response: CoreResult<CoreResult<T, DispatchError>, ApiError>) -> Result<T> {
	match response {
		Ok(Ok(res)) => Ok(res),
		Ok(Err(DispatchError::Module(e))) => Err(RpcError {
			code: ErrorCode::ServerError(100), // No real reason for this value
			message: format!("Dispatch Error Module:{} error:{}", e.index, e.error).into(),
			data: Some(e.message.unwrap_or_default().into()),
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
