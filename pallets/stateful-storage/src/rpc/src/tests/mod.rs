mod rpc_mock;

use super::*;
use rpc_mock::*;

use common_primitives::{node::Block, stateful_storage::*};
use pallet_stateful_storage_runtime_api::StatefulStorageRuntimeApi;
use sp_runtime::DispatchError;
use std::sync::Arc;

const SUCCESSFUL_SCHEMA_ID: SchemaId = 1;
const SUCCESSFUL_INTENT_ID: IntentId = 1;
const SUCCESSFUL_MSA_ID: MessageSourceId = 1;
const NONCE: PageNonce = 1;
const SUCCESSFUL_PAYLOAD: &[u8; 33] = b"{'body':827, 'val':'another val'}";
const DUMMY_STATE_HASH: u32 = 32767;

sp_api::mock_impl_runtime_apis! {
	impl StatefulStorageRuntimeApi<Block> for TestRuntimeApi {
		/// Retrieve the itemized storage for a particular msa and schema
		fn get_paginated_storage_v2(msa_id: MessageSourceId, intent_id: IntentId) -> Result<Vec<PaginatedStorageResponseV2>, DispatchError> {
			match (msa_id, intent_id) {
				(SUCCESSFUL_MSA_ID, SUCCESSFUL_INTENT_ID) => Ok(vec![PaginatedStorageResponseV2::new(
					0,
					msa_id,
					SUCCESSFUL_INTENT_ID,
					SUCCESSFUL_SCHEMA_ID,
					DUMMY_STATE_HASH,
					NONCE,
					SUCCESSFUL_PAYLOAD.to_vec(),
					).into()]),
				_ => Err(DispatchError::Other("some error")),
			}
		}

		fn get_itemized_storage_v2(msa_id: MessageSourceId, intent_id: IntentId) -> Result<ItemizedStoragePageResponseV2, DispatchError> {
			match (msa_id, intent_id) {
				(SUCCESSFUL_MSA_ID, SUCCESSFUL_INTENT_ID) => Ok(ItemizedStoragePageResponseV2::new(
					msa_id,
					SUCCESSFUL_INTENT_ID,
					DUMMY_STATE_HASH,
					NONCE,
					vec![ItemizedStorageResponseV2::new(0, intent_id, SUCCESSFUL_PAYLOAD.to_vec()).into()]).into()),
				_ => Err(DispatchError::Other("some error")),
			}
		}
	}
}

type PaginatedStateResult =
	Result<Vec<PaginatedStorageResponseV2>, jsonrpsee::types::ErrorObjectOwned>;
type ItemizedStateResult =
	Result<ItemizedStoragePageResponseV2, jsonrpsee::types::ErrorObjectOwned>;

#[tokio::test]
async fn get_paginated_storage_with_non_existent_schema_id_should_return_error() {
	let client = Arc::new(TestApi {});
	let api = StatefulStorageHandler::new(client);

	let result: PaginatedStateResult = api.get_paginated_storage(
		1, 1233, // Non-existing Schema Id
	);

	assert!(result.is_err());
}

#[tokio::test]
async fn get_paginated_storage_with_non_existent_msa_id_should_return_error() {
	let client = Arc::new(TestApi {});
	let api = StatefulStorageHandler::new(client);

	let result: PaginatedStateResult = api.get_paginated_storage(
		1029, // Non-existing Msa Id
		1,
	);

	assert!(result.is_err());
}

#[tokio::test]
async fn get_paginated_storage_with_success() {
	let client = Arc::new(TestApi {});
	let api = StatefulStorageHandler::new(client);

	let result: PaginatedStateResult = api.get_paginated_storage(
		SUCCESSFUL_MSA_ID,    // Msa Id
		SUCCESSFUL_SCHEMA_ID, // Schema Id
	);

	assert!(result.is_ok());
	let response = result.unwrap();
	assert_eq!(1, response.len());
	let page = &response[0];
	assert_eq!(0, page.page_id);
	assert_eq!(SUCCESSFUL_MSA_ID, page.msa_id);
	assert_eq!(SUCCESSFUL_SCHEMA_ID, page.schema_id);
	assert_eq!(NONCE, page.nonce);
	assert_eq!(SUCCESSFUL_PAYLOAD.to_vec(), page.payload);
}

#[tokio::test]
async fn get_itemized_storage_with_non_existent_schema_id_should_return_error() {
	let client = Arc::new(TestApi {});
	let api = StatefulStorageHandler::new(client);

	let result: ItemizedStateResult = api.get_itemized_storage(
		1, 1233, // Non-existing Schema Id
	);

	assert!(result.is_err());
}

#[tokio::test]
async fn get_itemized_storage_with_non_existent_msa_id_should_return_error() {
	let client = Arc::new(TestApi {});
	let api = StatefulStorageHandler::new(client);

	let result: ItemizedStateResult = api.get_itemized_storage(
		1029, // Non-existing Msa Id
		1,
	);

	assert!(result.is_err());
}

#[tokio::test]
async fn get_itemized_storage_with_success() {
	let client = Arc::new(TestApi {});
	let api = StatefulStorageHandler::new(client);

	let result: ItemizedStateResult = api.get_itemized_storage(
		SUCCESSFUL_MSA_ID,    // Msa Id
		SUCCESSFUL_SCHEMA_ID, // Schema Id
	);

	assert!(result.is_ok());
	let page = result.unwrap();
	let items = page.items;
	assert_eq!(1, items.len());
	assert_eq!(SUCCESSFUL_MSA_ID, page.msa_id);
	assert_eq!(SUCCESSFUL_INTENT_ID, page.intent_id);
	assert_eq!(NONCE, page.nonce);
	assert_eq!(DUMMY_STATE_HASH, page.content_hash);
	assert_eq!(
		items[0],
		ItemizedStorageResponseV2::new(0, SUCCESSFUL_SCHEMA_ID, SUCCESSFUL_PAYLOAD.to_vec()).into()
	);
}
