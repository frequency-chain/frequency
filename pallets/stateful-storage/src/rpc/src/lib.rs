// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [Stateful-Storage](../pallet_stateful_storage/index.html)

use common_primitives::{
	msa::MessageSourceId,
	schema::*,
	stateful_storage::{ItemizedStoragePageResponse, PaginatedStorageResponse},
};
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	types::error::{ErrorCode, ErrorObject},
};
use pallet_stateful_storage_runtime_api::StatefulStorageRuntimeApi;
use sp_api::{ApiError, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_runtime::{traits::Block as BlockT, DispatchError};
extern crate alloc;
use alloc::{sync::Arc, vec::Vec};

#[cfg(test)]
mod tests;

/// Frequency Stateful Storage Custom RPC API
#[rpc(client, server)]
pub trait StatefulStorageApi<BlockHash> {
	/// retrieving pages of stateful storage
	#[method(name = "statefulStorage_getPaginatedStorage")]
	fn get_paginated_storage(
		&self,
		msa_id: MessageSourceId,
		schema_id: SchemaId,
	) -> RpcResult<Vec<PaginatedStorageResponse>>;

	/// retrieving itemized storage of stateful storage
	#[method(name = "statefulStorage_getItemizedStorage")]
	fn get_itemized_storage(
		&self,
		msa_id: MessageSourceId,
		schema_id: SchemaId,
	) -> RpcResult<ItemizedStoragePageResponse>;
}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct StatefulStorageHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> StatefulStorageHandler<C, M> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

#[async_trait]
impl<C, Block> StatefulStorageApiServer<<Block as BlockT>::Hash>
	for StatefulStorageHandler<C, Block>
where
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: StatefulStorageRuntimeApi<Block>,
{
	fn get_paginated_storage(
		&self,
		msa_id: MessageSourceId,
		schema_id: SchemaId,
	) -> RpcResult<Vec<PaginatedStorageResponse>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;
		let api_result = api.get_paginated_storage(at, msa_id, schema_id);
		map_result(api_result)
	}

	fn get_itemized_storage(
		&self,
		msa_id: MessageSourceId,
		schema_id: SchemaId,
	) -> RpcResult<ItemizedStoragePageResponse> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;
		let api_result = api.get_itemized_storage(at, msa_id, schema_id);
		map_result(api_result)
	}
}

fn map_result<T>(api_result: Result<Result<T, DispatchError>, ApiError>) -> RpcResult<T> {
	match api_result {
		Ok(Ok(result)) => Ok(result),
		Ok(Err(e)) => Err(ErrorObject::owned(
			ErrorCode::ServerError(300).code(), // No real reason for this value
			"Runtime Error",
			Some(format!("{:?}", e)),
		)),
		Err(e) => Err(ErrorObject::owned(
			ErrorCode::ServerError(301).code(), // No real reason for this value
			"Api Error",
			Some(format!("{:?}", e)),
		)),
	}
}
