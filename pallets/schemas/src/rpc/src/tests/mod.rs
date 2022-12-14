mod rpc_mock;

use super::*;
use rpc_mock::*;

use pallet_schemas_runtime_api::SchemasRuntimeApi;
use std::sync::Arc;
use substrate_test_runtime_client::runtime::Block;

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
				}),
				_ => None,
			}
		}
	}
}

type SchemaResult = Result<Option<SchemaResponse>, jsonrpsee::core::Error>;

#[tokio::test]
async fn get_schema_with_non_existent_schema_id_should_return_none() {
	let client = Arc::new(TestApi {});
	let api = SchemasHandler::new(client);

	let result: SchemaResult = api.get_by_schema_id(
		1233, // Non-existing Schema Id
	);

	assert_eq!(true, result.is_ok());
	assert_eq!(true, result.unwrap().is_none());
}

#[tokio::test]
async fn get_schema_with_success() {
	let client = Arc::new(TestApi {});
	let api = SchemasHandler::new(client);

	let result: SchemaResult = api.get_by_schema_id(
		SUCCESSFUL_SCHEMA_ID, // Schema Id
	);

	assert_eq!(true, result.is_ok());
	let response = result.unwrap().unwrap();
	assert_eq!(1, response.schema_id);
	assert_eq!(ModelType::AvroBinary, response.model_type);
	assert_eq!(PayloadLocation::OnChain, response.payload_location);
}

#[tokio::test]
async fn check_schema_validity_success() {
	let client = Arc::new(TestApi {});
	let api = SchemasHandler::new(client);

	let result = api.check_schema_validity(
		r#"
		{
			"type": "record",
			"name": "test",
			"fields": [
				{"name": "a", "type": "long", "default": 42},
				{"name": "b", "type": "string"}
			]
		}
		"#
		.as_bytes()
		.to_vec(),
		None,
	);

	assert_eq!(true, result.is_ok());
	assert_eq!(true, result.unwrap());
}

#[tokio::test]
async fn check_schema_validity_fail() {
	let client = Arc::new(TestApi {});
	let api = SchemasHandler::new(client);

	let result = api.check_schema_validity(
		r#"
		{
			"type": "NOT A REAL TYPE",
			"name": "test",
		}
		"#
		.as_bytes()
		.to_vec(),
		None,
	);

	assert_eq!(false, result.is_ok());
}
