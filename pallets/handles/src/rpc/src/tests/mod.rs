mod rpc_mock;

use super::*;
use rpc_mock::*;

use common_primitives::handles::{BaseHandle, DisplayHandle, PresumptiveSuffixesResponse};
use pallet_handles_runtime_api::HandlesRuntimeApi;
use std::sync::Arc;
use substrate_test_runtime_client::runtime::Block;

const VALID_MSA_ID: u64 = 1;

sp_api::mock_impl_runtime_apis! {
	impl HandlesRuntimeApi<Block> for TestRuntimeApi {
		fn get_handle_for_msa(&self, msa_id: MessageSourceId) -> Option<HandleResponse> {
			match msa_id {
				VALID_MSA_ID => Some(HandleResponse {
					base_handle: b"base_handle".to_vec(),
					canonical_base: b"canonical_base".to_vec(),
					suffix: 1,
				}),
				_ => None,
			}
		}

		fn get_next_suffixes(base_handle: BaseHandle, count: u16) -> PresumptiveSuffixesResponse {
			let mut suffixes = Vec::new();
			for i in 0..count {
				suffixes.push(i);
			}
			PresumptiveSuffixesResponse { base_handle: base_handle.into(),  suffixes }
		}

		fn get_msa_for_handle(_display_handle: DisplayHandle) -> Option<MessageSourceId>{
			Some(VALID_MSA_ID)
		}

		fn validate_handle(_base_handle: BaseHandle) -> bool {
			true
		}
	}
}

type HandleResult = Result<Option<HandleResponse>, jsonrpsee::core::Error>;

#[tokio::test]
async fn get_handle_with_non_existent_msa_id_should_return_none() {
	let client = Arc::new(TestApi {});
	let api = HandlesHandler::new(client);

	let result: HandleResult = api.get_handle_for_msa(
		1233, // Non-existing MSA Id
	);

	assert_eq!(true, result.is_ok());
	assert_eq!(true, result.unwrap().is_none());
}

#[tokio::test]
async fn get_handle_with_success() {
	let client = Arc::new(TestApi {});
	let api = HandlesHandler::new(client);

	let result: HandleResult = api.get_handle_for_msa(
		VALID_MSA_ID, // MSA Id
	);

	assert_eq!(true, result.is_ok());
	let response = result.unwrap().unwrap();
	assert_eq!(b"base_handle".to_vec(), response.base_handle);
	assert_eq!(b"canonical_base".to_vec(), response.canonical_base);
	assert_eq!(1, response.suffix);
}

#[tokio::test]
async fn get_next_suffixes_with_success() {
	let client = Arc::new(TestApi {});
	let api = HandlesHandler::new(client);
	let result = api.get_next_suffixes("base_handle".to_string(), Some(3));

	assert_eq!(true, result.is_ok());
	let response = result.unwrap();
	assert_eq!(3, response.suffixes.len());
}

#[tokio::test]
async fn get_msa_for_handle_with_success() {
	let client = Arc::new(TestApi {});
	let api = HandlesHandler::new(client);
	let result = api.get_msa_for_handle("base_handle".to_string());

	assert_eq!(true, result.is_ok());
	assert_eq!(Some(VALID_MSA_ID), result.unwrap());
}
