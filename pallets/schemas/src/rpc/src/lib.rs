use common_helpers::avro;
use common_primitives::{rpc::*, schema::*};
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use pallet_schemas_runtime_api::SchemasRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use sp_std::vec::Vec;
use std::sync::Arc;

/// Error type of this RPC api.
pub enum Error {
	/// The transaction was not decodable.
	DecodeError,
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

#[rpc]
pub trait SchemasApi<BlockHash> {
	#[rpc(name = "schemas_getLatestSchemaId")]
	fn get_latest_schema_id(&self, at: Option<BlockHash>) -> Result<u16>;

	#[rpc(name = "schemas_getBySchemaId")]
	fn get_by_schema_id(&self, schema_id: SchemaId) -> Result<Option<SchemaResponse>>;

	#[rpc(name = "schemas_checkSchemaValidity")]
	fn check_schema_validity(&self, at: Option<BlockHash>, schema: Vec<u8>) -> Result<bool>;
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

impl<C, Block> SchemasApi<<Block as BlockT>::Hash> for SchemasHandler<C, Block>
where
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: SchemasRuntimeApi<Block>,
{
	fn get_latest_schema_id(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u16> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
		let schema_api_result = api.get_latest_schema_id(&at);
		match schema_api_result {
			Ok(schema_id) => match schema_id {
				Some(id) => Ok(id),
				None => Err(RpcError {
					code: ErrorCode::ServerError(1),
					message: "No schema found".into(),
					data: None,
				}
				.into()),
			},
			Err(e) => Err(RpcError {
				code: ErrorCode::ServerError(1),
				message: "Unable to get latest schema id".into(),
				data: Some(format!("{:?}", e).into()),
			}),
		}
	}

	fn check_schema_validity(
		&self,
		at: Option<<Block as BlockT>::Hash>,
		schema: Vec<u8>,
	) -> Result<bool> {
		let validated_schema = avro::validate_raw_avro_schema(&schema);
		if validated_schema.is_err() {
			return Err(RpcError {
				code: ErrorCode::ServerError(1),
				message: "Invalid schema".into(),
				data: Some(format!("{:?}", validated_schema).into()),
			}
			.into())
		}
		Ok(true)
	}

	fn get_by_schema_id(&self, schema_id: SchemaId) -> Result<Option<SchemaResponse>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(self.client.info().best_hash);
		let schema_api_result = api.get_by_schema_id(&at, schema_id);
		map_rpc_result(schema_api_result)
	}
}
