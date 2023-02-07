mod rpc_mock;

use super::*;
use rpc_mock::*;

use pallet_stateful_storage_runtime_api::StatefulStorageRuntimeApi;
use std::sync::Arc;
use substrate_test_runtime_client::runtime::Block;

const SUCCESSFUL_SCHEMA_ID: u16 = 1;
const SUCCESSFUL_MSA_ID: MessageSourceId = 1;
const SUCCESSFUL_PAYLOAD: &[u8; 33] = b"{'body':827, 'val':'another val'}";

sp_api::mock_impl_runtime_apis! {
	impl StatefulStorageRuntimeApi<Block> for TestRuntimeApi {
		fn get_pages(msa_id: MessageSourceId, schema_id: SchemaId) -> Vec<PageResponse> {
			match (msa_id, schema_id) {
				(SUCCESSFUL_MSA_ID, SUCCESSFUL_SCHEMA_ID) => vec![PageResponse::new(
					0,
					msa_id,
					schema_id,
					SUCCESSFUL_PAYLOAD.to_vec())],
				_ => vec![],
			}
		}
	}
}

type StatefulStateResult = Result<Vec<PageResponse>, jsonrpsee::core::Error>;

#[tokio::test]
async fn get_pages_with_non_existent_schema_id_should_return_empty_vec() {
	let client = Arc::new(TestApi {});
	let api = StatefulStorageHandler::new(client);

	let result: StatefulStateResult = api.get_pages(
		1, 1233, // Non-existing Schema Id
	);

	assert_eq!(true, result.is_ok());
	assert_eq!(Vec::<PageResponse>::new(), result.unwrap());
}

#[tokio::test]
async fn get_pages_with_non_existent_msa_id_should_return_empty_vec() {
	let client = Arc::new(TestApi {});
	let api = StatefulStorageHandler::new(client);

	let result: StatefulStateResult = api.get_pages(
		1029, // Non-existing Msa Id
		1,
	);

	assert_eq!(true, result.is_ok());
	assert_eq!(Vec::<PageResponse>::new(), result.unwrap());
}

#[tokio::test]
async fn get_pages_with_success() {
	let client = Arc::new(TestApi {});
	let api = StatefulStorageHandler::new(client);

	let result: StatefulStateResult = api.get_pages(
		SUCCESSFUL_MSA_ID,    // Msa Id
		SUCCESSFUL_SCHEMA_ID, // Schema Id
	);

	assert_eq!(true, result.is_ok());
	let response = result.unwrap();
	assert_eq!(1, response.len());
	let page = &response[0];
	assert_eq!(0, page.index_number);
	assert_eq!(SUCCESSFUL_MSA_ID, page.msa_id);
	assert_eq!(SUCCESSFUL_SCHEMA_ID, page.schema_id);
	assert_eq!(SUCCESSFUL_PAYLOAD.to_vec(), page.payload);
}
