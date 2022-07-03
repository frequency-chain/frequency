use common_helpers::{avro, rpc::*};
use common_primitives::schema::*;
use jsonrpsee::{
	core::{async_trait, Error as RpcError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use pallet_schemas_runtime_api::SchemasRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use sp_std::vec::Vec;
use std::sync::Arc;

/// Error type of this RPC api.
pub enum Error {
	/// No schema was found for the given id.
	SchemaNotFound,
	/// Failed to fetch latest schema.
	SchemaSearchError,
	// Schema validation error.
	SchemaValidationError,
}

impl From<Error> for i32 {
	fn from(e: Error) -> i32 {
		match e {
			Error::SchemaNotFound => 1,
			Error::SchemaSearchError => 2,
			Error::SchemaValidationError => 3,
		}
	}
}

/// MRC Schema API
#[rpc(client, server)]
pub trait SchemasApi<BlockHash> {
	/// returns the latest registered schema id
	///
	/// `at`: block number to query. If it's `None` will use the latest block number.
	///
	/// Returns schema id.
	#[method(name = "schemas_getLatestSchemaId")]
	fn get_latest_schema_id(&self, at: Option<BlockHash>) -> RpcResult<u16>;

	/// retrieving schema by schema id
	#[method(name = "schemas_getBySchemaId")]
	fn get_by_schema_id(&self, schema_id: SchemaId) -> RpcResult<Option<SchemaResponse>>;

	/// validates a schema model and returns `true` if the model is correct.
	#[method(name = "schemas_checkSchemaValidity")]
	fn check_schema_validity(&self, at: Option<BlockHash>, model: Vec<u8>) -> RpcResult<bool>;
}

pub struct SchemasHandler<C, M> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> SchemasHandler<C, M> {
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
	fn get_latest_schema_id(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<u16> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		let schema_api_result = api.get_latest_schema_id(&at);
		match schema_api_result {
			Ok(schema_id) => match schema_id {
				Some(id) => Ok(id),
				None => Err(RpcError::Call(CallError::Custom(ErrorObject::owned(
					Error::SchemaNotFound.into(),
					"No schema found for given id",
					None::<()>,
				)))),
			},
			Err(e) => Err(RpcError::Call(CallError::Custom(ErrorObject::owned(
				Error::SchemaSearchError.into(),
				"Unable to get latest schema",
				Some(format!("{:?}", e)),
			)))),
		}
	}

	fn check_schema_validity(
		&self,
		_at: Option<<Block as BlockT>::Hash>,
		model: Vec<u8>,
	) -> RpcResult<bool> {
		let validated_schema = avro::validate_raw_avro_schema(&model);
		match validated_schema {
			Ok(_) => Ok(true),
			Err(e) => Err(RpcError::Call(CallError::Custom(ErrorObject::owned(
				Error::SchemaValidationError.into(),
				"Unable to validate schema",
				Some(format!("{:?}", e)),
			)))),
		}
	}

	fn get_by_schema_id(&self, schema_id: SchemaId) -> RpcResult<Option<SchemaResponse>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);
		let schema_api_result = api.get_by_schema_id(&at, schema_id);
		map_rpc_result(schema_api_result)
	}
}
