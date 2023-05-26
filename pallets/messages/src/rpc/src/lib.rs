// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [Messages](../pallet_messages/index.html)

#[cfg(feature = "std")]
use common_helpers::rpc::map_rpc_result;
use common_primitives::{messages::*, schema::*};
use frame_support::{ensure, fail};
use jsonrpsee::{
	core::{async_trait, Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
};
use pallet_messages_runtime_api::MessagesRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{traits::Block as BlockT};
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// Frequency Messages Custom RPC API
#[rpc(client, server)]
pub trait MessagesApi {
	/// Retrieve paginated messages by schema id
	#[method(name = "messages_getBySchemaId")]
	fn get_messages_by_schema_id(
		&self,
		schema_id: SchemaId,
		pagination: BlockPaginationRequest,
	) -> RpcResult<BlockPaginationResponse<MessageResponse>>;
}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct MessagesHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> MessagesHandler<C, M> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
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

impl From<MessageRpcError> for JsonRpseeError {
	fn from(e: MessageRpcError) -> Self {
		JsonRpseeError::Custom(format!("{:?}", e))
	}
}

#[async_trait]
impl<C, Block> MessagesApiServer for MessagesHandler<C, Block>
where
	Block: BlockT,
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + 'static,
	C::Api: MessagesRuntimeApi<Block>,
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

		// Schema Fetch and Check
		let schema: SchemaResponse = match api.get_schema_by_id(self.client.info().best_hash, schema_id) {
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
					self.client.info().best_hash,
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
}
