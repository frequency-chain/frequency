mod rpc_mock;

use super::*;
use rpc_mock::*;

use common_primitives::stateful_storage::*;
use pallet_stateful_storage_runtime_api::StatefulStorageRuntimeApi;
use sp_runtime::DispatchError;
use std::sync::Arc;
use substrate_test_runtime_client::runtime::Block;

const SUCCESSFUL_SCHEMA_ID: u16 = 1;
const SUCCESSFUL_MSA_ID: MessageSourceId = 1;
const SUCCESSFUL_PAYLOAD: &[u8; 33] = b"{'body':827, 'val':'another val'}";

sp_api::mock_impl_runtime_apis! {
	impl StatefulStorageRuntimeApi<Block> for TestRuntimeApi {
		/// Retrieve the itemized storages for a particular msa and schema
		fn get_paginated_storages(msa_id: MessageSourceId, schema_id: SchemaId) -> Result<Vec<PaginatedStorageResponse>, DispatchError> {
			match (msa_id, schema_id) {
				(SUCCESSFUL_MSA_ID, SUCCESSFUL_SCHEMA_ID) => Ok(vec![PaginatedStorageResponse::new(
					0,
					msa_id,
					schema_id,
					SUCCESSFUL_PAYLOAD.to_vec())]),
				_ => Err(DispatchError::Other("some error")),
			}
		}

		fn get_itemized_storages(msa_id: MessageSourceId, schema_id: SchemaId) -> Result<ItemizedStoragePageResponse, DispatchError> {
			match (msa_id, schema_id) {
				(SUCCESSFUL_MSA_ID, SUCCESSFUL_SCHEMA_ID) => Ok(ItemizedStoragePageResponse::new(
					msa_id,
					schema_id,
					vec![ItemizedStorageResponse::new(0,SUCCESSFUL_PAYLOAD.to_vec())])),
				_ => Err(DispatchError::Other("some error")),
			}
		}
	}
}

type PaginatedStateResult = Result<Vec<PaginatedStorageResponse>, jsonrpsee::core::Error>;
type ItemizedStateResult = Result<ItemizedStoragePageResponse, jsonrpsee::core::Error>;

#[tokio::test]
async fn get_paginated_storages_with_non_existent_schema_id_should_return_error() {
	let client = Arc::new(TestApi {});
	let api = StatefulStorageHandler::new(client);

	let result: PaginatedStateResult = api.get_paginated_storages(
		1, 1233, // Non-existing Schema Id
	);

	assert_eq!(true, result.is_err());
}

#[tokio::test]
async fn get_paginated_storages_with_non_existent_msa_id_should_return_error() {
	let client = Arc::new(TestApi {});
	let api = StatefulStorageHandler::new(client);

	let result: PaginatedStateResult = api.get_paginated_storages(
		1029, // Non-existing Msa Id
		1,
	);

	assert_eq!(true, result.is_err());
}

#[tokio::test]
async fn get_paginated_storages_with_success() {
	let client = Arc::new(TestApi {});
	let api = StatefulStorageHandler::new(client);

	let result: PaginatedStateResult = api.get_paginated_storages(
		SUCCESSFUL_MSA_ID,    // Msa Id
		SUCCESSFUL_SCHEMA_ID, // Schema Id
	);

	assert_eq!(true, result.is_ok());
	let response = result.unwrap();
	assert_eq!(1, response.len());
	let page = &response[0];
	assert_eq!(0, page.page_id);
	assert_eq!(SUCCESSFUL_MSA_ID, page.msa_id);
	assert_eq!(SUCCESSFUL_SCHEMA_ID, page.schema_id);
	assert_eq!(SUCCESSFUL_PAYLOAD.to_vec(), page.payload);
}

#[tokio::test]
async fn get_itemized_storages_with_non_existent_schema_id_should_return_error() {
	let client = Arc::new(TestApi {});
	let api = StatefulStorageHandler::new(client);

	let result: ItemizedStateResult = api.get_itemized_storages(
		1, 1233, // Non-existing Schema Id
	);

	assert_eq!(true, result.is_err());
}

#[tokio::test]
async fn get_itemized_storages_with_non_existent_msa_id_should_return_error() {
	let client = Arc::new(TestApi {});
	let api = StatefulStorageHandler::new(client);

	let result: ItemizedStateResult = api.get_itemized_storages(
		1029, // Non-existing Msa Id
		1,
	);

	assert_eq!(true, result.is_err());
}

#[tokio::test]
async fn get_itemized_storages_with_success() {
	let client = Arc::new(TestApi {});
	let api = StatefulStorageHandler::new(client);

	let result: ItemizedStateResult = api.get_itemized_storages(
		SUCCESSFUL_MSA_ID,    // Msa Id
		SUCCESSFUL_SCHEMA_ID, // Schema Id
	);

	assert_eq!(true, result.is_ok());
	let page = result.unwrap();
	let items = page.items;
	assert_eq!(1, items.len());
	assert_eq!(SUCCESSFUL_MSA_ID, page.msa_id);
	assert_eq!(SUCCESSFUL_SCHEMA_ID, page.schema_id);
	assert_eq!(ItemizedStorageResponse::new(0, SUCCESSFUL_PAYLOAD.to_vec()), items[0]);
}
