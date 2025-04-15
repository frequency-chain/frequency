// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Custom APIs for [Schemas](../pallet_schemas/index.html)

use common_helpers::{avro, rpc::map_rpc_result};
use common_primitives::schema::*;
use jsonrpsee::{
	core::{async_trait, RpcResult},
	proc_macros::rpc,
	types::error::ErrorObject,
};
use pallet_schemas_runtime_api::SchemasRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
extern crate alloc;
use alloc::{sync::Arc, vec::Vec};

#[cfg(test)]
mod tests;

/// Errors that occur on the client RPC
pub enum SchemaRpcError {
	/// No schema was found for the given id.
	SchemaNotFound,
	/// Failed to fetch latest schema.
	SchemaSearchError,
	/// Schema model did not validate
	SchemaValidationError,
}

impl From<SchemaRpcError> for i32 {
	fn from(e: SchemaRpcError) -> i32 {
		match e {
			SchemaRpcError::SchemaNotFound => 1,
			SchemaRpcError::SchemaSearchError => 2,
			SchemaRpcError::SchemaValidationError => 3,
		}
	}
}

/// Frequency Schema Custom RPC API
#[rpc(client, server)]
pub trait SchemasApi<BlockHash> {
	/// retrieving schema by schema id
	#[method(name = "schemas_getBySchemaId")]
	fn get_by_schema_id(&self, schema_id: SchemaId) -> RpcResult<Option<SchemaResponse>>;

	/// validates a schema model and returns `true` if the model is correct.
	#[method(name = "schemas_checkSchemaValidity")]
	fn check_schema_validity(&self, model: Vec<u8>, at: Option<BlockHash>) -> RpcResult<bool>;

	/// returns an array of schema versions
	#[method(name = "schemas_getVersions")]
	fn get_versions(&self, schema_name: String) -> RpcResult<Option<Vec<SchemaVersionResponse>>>;
}

/// The client handler for the API used by Frequency Service RPC with `jsonrpsee`
pub struct SchemasHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> SchemasHandler<C, M> {
	/// Create new instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

#[async_trait]
impl<C, Block> SchemasApiServer<<Block as BlockT>::Hash> for SchemasHandler<C, Block>
where
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: SchemasRuntimeApi<Block>,
{
	fn check_schema_validity(
		&self,
		model: Vec<u8>,
		_at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<bool> {
		let validated_schema = avro::validate_raw_avro_schema(&model);
		match validated_schema {
			Ok(_) => Ok(true),
			Err(e) => Err(ErrorObject::owned(
				SchemaRpcError::SchemaValidationError.into(),
				"Unable to validate schema",
				Some(format!("{:?}", e)),
			)),
		}
	}

	fn get_by_schema_id(&self, schema_id: SchemaId) -> RpcResult<Option<SchemaResponse>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;
		let schema_api_result = api.get_by_schema_id(at, schema_id);
		map_rpc_result(schema_api_result)
	}

	fn get_versions(&self, schema_name: String) -> RpcResult<Option<Vec<SchemaVersionResponse>>> {
		let api = self.client.runtime_api();
		let at = self.client.info().best_hash;
		let schema_api_result = api.get_schema_versions_by_name(at, schema_name.into_bytes());
		map_rpc_result(schema_api_result)
	}
}
