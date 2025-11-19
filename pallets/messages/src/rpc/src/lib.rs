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
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::{ErrorObject, ErrorObjectOwned},
};
use pallet_messages_runtime_api::MessagesRuntimeApi;
use pallet_schemas_runtime_api::SchemasRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// Frequency Messages Custom RPC API
#[rpc(client, server)]
pub trait MessagesApi {
    /// Retrieve paginated messages by schema id
    #[method(name = "messages_getByIntentId")]
    fn get_messages_by_intent_id(
        &self,
        intent_id: IntentId,
        pagination: BlockPaginationRequest,
    ) -> RpcResult<BlockPaginationResponse<MessageResponseV2>>;
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
    /// Intent Id doesn't exist or errored when retrieving from state
    InvalidIntentId,
}

impl From<MessageRpcError> for ErrorObjectOwned {
    fn from(e: MessageRpcError) -> Self {
        let msg = format!("{e:?}");
        match e {
            MessageRpcError::InvalidPaginationRequest => ErrorObject::owned(1, msg, None::<()>),
            MessageRpcError::TypeConversionOverflow => ErrorObject::owned(2, msg, None::<()>),
            MessageRpcError::InvalidSchemaId => ErrorObject::owned(3, msg, None::<()>),
            MessageRpcError::InvalidIntentId => ErrorObject::owned(4, msg, None::<()>),
        }
    }
}

#[async_trait]
impl<C, Block> MessagesApiServer for MessagesHandler<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + 'static,
    C::Api: MessagesRuntimeApi<Block> + SchemasRuntimeApi<Block>,
{
    fn get_messages_by_intent_id(
        &self,
        intent_id: IntentId,
        pagination: BlockPaginationRequest,
    ) -> RpcResult<BlockPaginationResponse<MessageResponseV2>> {
        // Request Validation
        ensure!(pagination.validate(), MessageRpcError::InvalidPaginationRequest);

        // Connect to on-chain data
        let api = self.client.runtime_api();
        let at = self.client.info().best_hash;

        // Schema Fetch and Check
        match api.get_intent_by_id(at, intent_id, false) {
            Ok(Some(s)) => s,
            _ => fail!(MessageRpcError::InvalidIntentId),
        };

        let response = api.get_messages_by_intent_id(
            at,
            intent_id,
            pagination,
        )
            .unwrap_or_default();
        map_rpc_result(Ok(response))
    }
}
