mod rpc_mock;

use super::*;
use common_primitives::node::Block;
use rpc_mock::*;

use pallet_schemas_runtime_api::SchemasRuntimeApi;
use std::sync::Arc;

const SUCCESSFUL_SCHEMA_ID: u16 = 1;

sp_api::mock_impl_runtime_apis! {
	impl SchemasRuntimeApi<Block> for TestRuntimeApi {
		fn get_by_schema_id(schema_id: SchemaId) -> Option<SchemaResponse> {
			match schema_id {
				SUCCESSFUL_SCHEMA_ID => Some(SchemaResponse {
					schema_id,
					model: b"schema".to_vec(),
					model_type: ModelType::AvroBinary,
					payload_location: PayloadLocation::OnChain,
					settings: Vec::new(),
				}),
				_ => None,
			}
		}
	}
}

type SchemaResult = Result<Option<SchemaResponse>, jsonrpsee::types::ErrorObjectOwned>;

#[tokio::test]
async fn get_schema_with_non_existent_schema_id_should_return_none() {
	let client = Arc::new(TestApi {});
	let api = SchemasHandler::new(client);

	let result: SchemaResult = api.get_by_schema_id(
		1233, // Non-existing Schema Id
	);

	assert!(result.is_ok());
	assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn get_schema_with_success() {
	let client = Arc::new(TestApi {});
	let api = SchemasHandler::new(client);

	let result: SchemaResult = api.get_by_schema_id(
		SUCCESSFUL_SCHEMA_ID, // Schema Id
	);

	assert!(result.is_ok());
	let response = result.unwrap().unwrap();
	assert_eq!(1, response.schema_id);
	assert_eq!(ModelType::AvroBinary, response.model_type);
	assert_eq!(PayloadLocation::OnChain, response.payload_location);
}
