mod rpc_mock;

use super::*;
use rpc_mock::*;

use common_primitives::node::Block;
use pallet_messages_runtime_api::MessagesRuntimeApi;
use std::sync::Arc;

const INTENT_ID_EMPTY: u16 = 1;
const INTENT_ID_HAS_MESSAGES: u16 = 2;
const SCHEMA_ID_EMPTY: u16 = 1;
const SCHEMA_ID_HAS_MESSAGES: u16 = 2;
const DUMMY_CID: &str = "bafkreidgvpkjawlxz6sffxzwgooowe5yt7i6wsyg236mfoks77nywkptdq";

// Simulate finding 2 messages per block until the pagination limit is reached
fn test_messages(
	block_pagination_request: BlockPaginationRequest,
) -> BlockPaginationResponse<MessageResponseV2> {
	let mut response = BlockPaginationResponse::<MessageResponseV2> {
		content: vec![],
		has_next: false,
		next_block: None,
		next_index: None,
	};

	'outer: for block_number in
		block_pagination_request.from_block..=block_pagination_request.to_block
	{
		for index in 0..2u16 {
			response.content.push(MessageResponseV2 {
				schema_id: 1,
				payload: None,
				msa_id: None,
				provider_msa_id: 1,
				index,
				block_number,
				cid: Some(DUMMY_CID.as_bytes().to_vec()),
				payload_length: Some(42),
			});

			if response.content.len() >= block_pagination_request.page_size as usize {
				if index == 0 {
					response.has_next = true;
					response.next_block = Some(block_number);
					response.next_index = Some(index as u32 + 1);
				} else if block_number < block_pagination_request.to_block {
					response.has_next = true;
					response.next_block = Some(block_number + 1);
					response.next_index = Some(0);
				}
				break 'outer;
			}
		}
	}

	response
}

sp_api::mock_impl_runtime_apis! {
	impl MessagesRuntimeApi<Block> for TestRuntimeApi {
		fn get_messages_by_intent_id(intent_id: IntentId, pagination: BlockPaginationRequest) ->
			BlockPaginationResponse<MessageResponseV2> {
				match intent_id {
					INTENT_ID_HAS_MESSAGES => test_messages(pagination),
					_ => BlockPaginationResponse::<MessageResponseV2>::default(),
				}
			}
	}

	impl SchemasRuntimeApi<Block> for TestRuntimeApi {
		fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponseV2> {
			match schema_id {
				SCHEMA_ID_EMPTY | SCHEMA_ID_HAS_MESSAGES => Some(SchemaResponseV2 {
					intent_id: if schema_id == SCHEMA_ID_EMPTY { INTENT_ID_EMPTY } else { INTENT_ID_HAS_MESSAGES },
					schema_id,
					model: b"schema".to_vec(),
					model_type: ModelType::AvroBinary,
					status: SchemaStatus::Active,
					payload_location: PayloadLocation::OnChain,
					settings: Vec::new(),
				}),
				_ =>  None,
			}
		}

		fn get_schema_versions_by_name(_schema_name: Vec<u8>) -> Option<Vec<SchemaVersionResponse>> {
			todo!()
		}

		fn get_registered_entities_by_name(_name: Vec<u8>) -> Option<Vec<NameLookupResponse>> {
			todo!()
		}

		fn get_intent_by_id(intent_id: IntentId, _include_schemas: bool) -> Option<IntentResponse> {
			match intent_id {
				INTENT_ID_EMPTY => Some(IntentResponse {
					intent_id,
					payload_location: PayloadLocation::OnChain,
					settings: vec![],
					schema_ids: None,
				}),
				INTENT_ID_HAS_MESSAGES => Some(IntentResponse {
					intent_id,
					payload_location: PayloadLocation::OnChain,
					settings: vec![],
					schema_ids: Some(vec![SCHEMA_ID_HAS_MESSAGES]),
				}),
				_ => None,

			}
		}

		fn get_intent_group_by_id(_intent_group_id: IntentGroupId) -> Option<IntentGroupResponse> {
			todo!()
		}
	}
}

type GetMessagesByIntentResult = Result<
	common_primitives::messages::BlockPaginationResponse<
		common_primitives::messages::MessageResponseV2,
	>,
	jsonrpsee::types::ErrorObjectOwned,
>;

#[tokio::test]
async fn get_messages_by_intent_with_invalid_request_should_panic() {
	let client = Arc::new(TestApi {});
	let api = MessagesHandler::new(client);

	let result: GetMessagesByIntentResult = api.get_messages_by_intent_id(
		INTENT_ID_EMPTY, // Intent Id
		BlockPaginationRequest { from_block: 1, to_block: 2, from_index: 0, page_size: 0 },
	);

	assert!(result.is_err());
	assert_eq!("InvalidPaginationRequest", result.unwrap_err().message());
}

#[tokio::test]
async fn get_messages_by_intent_with_bad_intent_id_should_err() {
	let client = Arc::new(TestApi {});
	let api = MessagesHandler::new(client);

	let result: GetMessagesByIntentResult = api.get_messages_by_intent_id(
		0, // Intent Id
		BlockPaginationRequest { from_block: 1, to_block: 5, from_index: 0, page_size: 10 },
	);

	assert!(result.clone().is_err());
	assert_eq!("InvalidIntentId", result.unwrap_err().message());
}

#[tokio::test]
async fn get_messages_by_intent_with_success() {
	let client = Arc::new(TestApi {});
	let api = MessagesHandler::new(client);
	let pagination =
		BlockPaginationRequest { from_block: 1, to_block: 5, from_index: 0, page_size: 2 };

	let result: GetMessagesByIntentResult = api.get_messages_by_intent_id(
		INTENT_ID_HAS_MESSAGES, // Intent Id
		pagination.clone(),
	);

	assert!(result.is_ok());
	let response = result.unwrap();
	// Because the page size is set to 2, we only get one set of messages
	assert_eq!(test_messages(pagination), response);
	// There is more because we haven't done all the blocks yet
	assert!(response.has_next);
	// Next block is 2 and index 0 as block 1 just has those two messages
	assert_eq!(Some(2), response.next_block);
	assert_eq!(Some(0), response.next_index);
}
