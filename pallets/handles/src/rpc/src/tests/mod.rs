mod rpc_mock;

use super::*;
use rpc_mock::*;

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
					canonical_handle: b"canonical_handle".to_vec(),
					suffix: 1,
				}),
				_ => None,
			}
		}

		fn get_next_suffixes(_handle: PresumtiveSuffixesRequest) -> PresumtiveSuffixesResponse {
			let count = _handle.count;
			let mut suffixes = Vec::new();
			for i in 0..count {
				suffixes.push(i);
			}
			PresumtiveSuffixesResponse{ base_handle: _handle.base_handle, suffixes }
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
	assert_eq!(b"canonical_handle".to_vec(), response.canonical_handle);
	assert_eq!(1, response.suffix);
}

#[tokio::test]
async fn get_next_suffixes_with_success() {
	let client = Arc::new(TestApi {});
	let api = HandlesHandler::new(client);
	let getSuffixInput =
		PresumtiveSuffixesRequest { base_handle: b"base_handle".to_vec(), count: 3 };
	let result = api.get_next_suffixes(getSuffixInput);

	assert_eq!(true, result.is_ok());
	let response = result.unwrap();
	assert_eq!(b"base_handle".to_vec(), response.base_handle);
	assert_eq!(vec![0, 1, 2], response.suffixes);
}
