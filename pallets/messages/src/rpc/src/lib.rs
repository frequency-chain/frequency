// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [Messages](../pallet_messages/index.html)

use codec::Decode;
#[cfg(feature = "std")]
use common_helpers::rpc::map_rpc_result;
use common_primitives::{messages::*, schema::*};
use frame_support::{ensure, fail, log};
use jsonrpsee::{
	core::{async_trait, error::Error as RpcError, RpcResult},
	proc_macros::rpc,
};
use pallet_messages_runtime_api::MessagesRuntimeApi;
use sc_rpc_api::author::error::Error;
use sc_transaction_pool_api::{
	error::IntoPoolError, BlockHash, TransactionPool, TransactionSource, TxHash,
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_runtime::{generic, traits::Block as BlockT};
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// Frequency Messages Custom RPC API
#[rpc(client, server)]
pub trait MessagesApi<Hash, BlockHash> {
	/// Retrieve paginated messages by schema id
	#[method(name = "messages_getBySchemaId")]
	fn get_messages_by_schema_id(
		&self,
		schema_id: SchemaId,
		pagination: BlockPaginationRequest,
	) -> RpcResult<BlockPaginationResponse<MessageResponse>>;

	/// Submit hex-encoded extrinsic for inclusion in block.
	#[method(name = "messages_submitExtrinsic")]
	async fn submit_extrinsic(&self, extrinsic: Bytes) -> RpcResult<Hash>;
}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct MessagesHandler<P, Client> {
	client: Arc<Client>,
	/// Transactions pool
	pool: Arc<P>,
}

impl<P, Client> MessagesHandler<P, Client> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<Client>, pool: Arc<P>) -> Self {
		Self { client, pool }
	}
}

/// Errors that occur on the client RPC
#[derive(Debug)]
pub enum MessageRpcError {
	/// Pagination request is bad
	InvalidPaginationRequest,
	/// Likely passed u32 block count
	TypeConversionOverflow,
	/// Schema Id doesn't exist or errored when retrieving from state
	InvalidSchemaId,
}

impl From<MessageRpcError> for RpcError {
	fn from(e: MessageRpcError) -> Self {
		RpcError::Custom(format!("{:?}", e))
	}
}

const TX_SOURCE: TransactionSource = TransactionSource::External;

#[async_trait]
impl<P, Client> MessagesApiServer<TxHash<P>, BlockHash<P>> for MessagesHandler<P, Client>
where
	P: TransactionPool + Sync + Send + 'static,
	Client: HeaderBackend<P::Block> + ProvideRuntimeApi<P::Block> + Send + Sync + 'static,
	Client::Api: MessagesRuntimeApi<P::Block>,
	P::Hash: Unpin,
	<P::Block as BlockT>::Hash: Unpin,
{
	fn get_messages_by_schema_id(
		&self,
		schema_id: SchemaId,
		pagination: BlockPaginationRequest,
	) -> RpcResult<BlockPaginationResponse<MessageResponse>> {
		// Request Validation
		ensure!(pagination.validate(), MessageRpcError::InvalidPaginationRequest);

		// Connect to on-chain data
		let api = self.client.runtime_api();
		let at = sp_runtime::generic::BlockId::hash(self.client.info().best_hash);

		// Schema Fetch and Check
		let schema: SchemaResponse = match api.get_schema_by_id(&at, schema_id) {
			Ok(Some(s)) => s,
			_ => fail!(MessageRpcError::InvalidSchemaId),
		};

		let mut response = BlockPaginationResponse::new();
		let from: u32 = pagination.from_block;
		let to: u32 = pagination.to_block;
		let mut from_index = pagination.from_index;

		'loops: for block_number in from..to {
			let list: Vec<MessageResponse> = api
				.get_messages_by_schema_and_block(
					&at,
					schema.schema_id,
					schema.payload_location,
					block_number,
				)
				.unwrap_or_default();

			let list_size: u32 =
				list.len().try_into().map_err(|_| MessageRpcError::TypeConversionOverflow)?;
			for i in from_index..list_size {
				response.content.push(list[i as usize].clone());

				if response.check_end_condition_and_set_next_pagination(
					block_number,
					i,
					list_size,
					&pagination,
				) {
					break 'loops
				}
			}

			// next block starts from 0
			from_index = 0;
		}

		map_rpc_result(Ok(response))
	}

	async fn submit_extrinsic(&self, ext: Bytes) -> RpcResult<TxHash<P>> {
		let xt = match Decode::decode(&mut &ext[..]) {
			Ok(xt) => xt,
			Err(err) => return Err(Error::Client(Box::new(err)).into()),
		};
		log::info!("incoming extrinsic {:?}", xt);

		let best_block_hash = self.client.info().best_hash;
		let at = sp_runtime::generic::BlockId::hash(best_block_hash);
		let api = self.client.runtime_api();
		let valid = api.validate_extrinsic(&at, ext.0).unwrap_or_default();
		log::info!("valid =  {:?}", valid);
		if valid == false {
			return Err(Error::BadFormat(codec::Error::from("Custom Error")).into())
		}

		self.pool
			.submit_one(&generic::BlockId::hash(best_block_hash), TX_SOURCE, xt)
			.await
			.map_err(|e| {
				e.into_pool_error()
					.map(|e| Error::Pool(e))
					.unwrap_or_else(|e| Error::Verification(Box::new(e)))
					.into()
			})
	}
}
