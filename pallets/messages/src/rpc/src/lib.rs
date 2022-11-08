// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [Messages](../pallet_messages/index.html)

use codec::Codec;
#[cfg(feature = "std")]
use common_helpers::rpc::map_rpc_result;
use common_primitives::{messages::*, schema::*};
use frame_support::{ensure, fail};
use jsonrpsee::{
	core::{async_trait, error::Error as RpcError, RpcResult},
	proc_macros::rpc,
};
use pallet_messages_runtime_api::MessagesRuntimeApi;
use poem_openapi::OpenApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
	generic::BlockId,
	traits::{AtLeast32BitUnsigned, Block as BlockT},
};
use std::sync::Arc;
#[cfg(test)]
mod tests;

/// Frequency Messages Custom RPC API
#[rpc(client, server)]
pub trait MessagesApi<BlockNumber> {
	/// Retrieve paginated messages by schema id
	#[method(name = "messages_getBySchemaId")]
	async fn get_messages_by_schema_id(
		&self,
		schema_id: SchemaId,
		pagination: BlockPaginationRequest,
	) -> RpcResult<BlockPaginationResponse<MessageResponse>>;
}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct MessagesHandler<C, M, B> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
	block_number: std::marker::PhantomData<B>,
}

impl<C, M, B> MessagesHandler<C, M, B> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		// get current block_number

		Self { client, _marker: Default::default(), block_number: Default::default() }
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

#[OpenApi]
#[async_trait]
impl<C, Block, BlockNumber> MessagesApiServer<BlockNumber>
	for MessagesHandler<C, Block, BlockNumber>
where
	Block: BlockT,
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + 'static,
	C::Api: MessagesRuntimeApi<Block, BlockNumber>,
	BlockNumber:
		Codec + Copy + AtLeast32BitUnsigned + std::marker::Send + std::marker::Sync + 'static,
{
	#[oai(path = "/messages/getBySchemaId", method = "get")]
	async fn get_messages_by_schema_id(
		&self,
		schema_id: SchemaId,
		pagination: BlockPaginationRequest,
	) -> RpcResult<BlockPaginationResponse<MessageResponse>> {
		// Request Validation
		ensure!(pagination.validate(), MessageRpcError::InvalidPaginationRequest);

		// Connect to on-chain data
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);

		// Schema Fetch and Check
		let schema: SchemaResponse = match api.get_schema_by_id(&at, schema_id) {
			Ok(Some(s)) => s,
			_ => fail!(MessageRpcError::InvalidSchemaId),
		};

		let mut response = BlockPaginationResponse::new();
		let from: u32 = pagination
			.from_block
			.try_into()
			.map_err(|_| MessageRpcError::TypeConversionOverflow)?;
		let to: u32 = pagination
			.to_block
			.try_into()
			.map_err(|_| MessageRpcError::TypeConversionOverflow)?;
		let mut from_index = pagination.from_index;

		'loops: for bid in from..to {
			let block_number: BlockNumber = bid.into();
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
				let given_block: u32 =
					block_number.try_into().map_err(|_| MessageRpcError::TypeConversionOverflow)?;
				if response.check_end_condition_and_set_next_pagination(
					given_block,
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
