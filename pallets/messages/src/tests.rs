use super::{mock::*, Event as MessageEvent};
use crate::{
	types::{IPFSPayload, CID},
	BlockMessages, Config, Error, Message, Messages,
};
use common_primitives::{
	messages::{BlockPaginationRequest, MessageResponse},
	schema::*,
};
use frame_support::{
	assert_err, assert_noop, assert_ok,
	weights::{Pays, PostDispatchInfo},
	BoundedVec,
};
use sp_std::vec::Vec;

fn populate_messages(schema_id: SchemaId, message_per_block: Vec<u32>) {
	let payload = Vec::from("{'fromId': 123, 'content': '232323114432'}".as_bytes());
	let mut counter = 0;
	for (idx, count) in message_per_block.iter().enumerate() {
		let mut list = BoundedVec::default();
		for _ in 0..*count {
			list.try_push(Message {
				msa_id: 10,
				payload: payload.clone().try_into().unwrap(),
				index: counter,
				provider_msa_id: 1,
			})
			.unwrap();
			counter += 1;
		}
		Messages::<Test>::insert(idx as u64, schema_id, list);
	}
}

#[test]
fn add_message_should_store_message_on_temp_storage() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 5;
		let caller_2 = 2;
		let schema_id_1: SchemaId = 1;
		let schema_id_2: SchemaId = 2;
		let message_payload_1 = Vec::from("{'fromId': 123, 'content': '232323114432'}".as_bytes());
		let message_payload_2 = Vec::from("{'fromId': 343, 'content': '34333'}".as_bytes());

		// act
		assert_ok!(MessagesPallet::add_onchain_message(
			Origin::signed(caller_1),
			None,
			schema_id_1,
			message_payload_1.clone()
		));
		assert_ok!(MessagesPallet::add_onchain_message(
			Origin::signed(caller_2),
			None,
			schema_id_2,
			message_payload_2.clone()
		));

		// assert
		let list = BlockMessages::<Test>::get().into_inner();
		assert_eq!(list.len(), 2);

		assert_eq!(
			list[0],
			(
				Message {
					msa_id: get_msa_from_account(caller_1),
					payload: message_payload_1.try_into().unwrap(),
					index: 0,
					provider_msa_id: get_msa_from_account(caller_1)
				},
				schema_id_1
			)
		);

		assert_eq!(
			list[1],
			(
				Message {
					msa_id: get_msa_from_account(caller_2),
					payload: message_payload_2.try_into().unwrap(),
					index: 1,
					provider_msa_id: get_msa_from_account(caller_2)
				},
				schema_id_2
			)
		);
	});
}

#[test]
fn add_message_with_too_large_message_should_panic() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 5;
		let schema_id_1: SchemaId = 1;
		let message_payload_1 = Vec::from("{'fromId': 123, 'content': '232323114432'}{'fromId': 123, 'content': '232323114432'}{'fromId': 123, 'content': '232323114432'}".as_bytes());

		// act
		assert_noop!(MessagesPallet::add_onchain_message(Origin::signed(caller_1), None, schema_id_1, message_payload_1), Error::<Test>::ExceedsMaxMessagePayloadSizeBytes);
	});
}

#[test]
fn add_message_with_invalid_msa_account_should_panic() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 1000;
		let schema_id_1: SchemaId = 1;
		let message_payload_1 = Vec::from(
			"{'fromId': 123, 'content': '232323114432'}{'fromId': 123, 'content': '232323114432'}"
				.as_bytes(),
		);

		// act
		assert_noop!(
			MessagesPallet::add_onchain_message(
				Origin::signed(caller_1),
				None,
				schema_id_1,
				message_payload_1
			),
			Error::<Test>::InvalidMessageSourceAccount
		);
	});
}

#[test]
fn add_message_with_maxed_out_storage_should_panic() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 5;
		let schema_id_1: SchemaId = 1;
		let message_payload_1 = Vec::from("{'fromId': 123, 'content': '232323114432'}".as_bytes());

		// act
		for _ in 0..<Test as Config>::MaxMessagesPerBlock::get() {
			assert_ok!(MessagesPallet::add_onchain_message(
				Origin::signed(caller_1),
				None,
				schema_id_1,
				message_payload_1.clone()
			));
		}
		assert_noop!(
			MessagesPallet::add_onchain_message(
				Origin::signed(caller_1),
				None,
				schema_id_1,
				message_payload_1
			),
			Error::<Test>::TooManyMessagesInBlock
		);
	});
}

#[test]
fn on_initialize_should_add_messages_into_storage_and_clean_temp() {
	new_test_ext().execute_with(|| {
		// arrange
		let current_block = 1;
		let caller_1 = 5;
		let caller_2 = 2;
		let schema_id_1: SchemaId = 1;
		let schema_id_2: SchemaId = 2;
		let message_payload_1 = Vec::from("{'fromId': 123, 'content': '232323114432'}".as_bytes());
		let message_payload_2 = Vec::from("{'fromId': 343, 'content': '34333'}".as_bytes());
		assert_ok!(MessagesPallet::add_onchain_message(
			Origin::signed(caller_1),
			None,
			schema_id_1,
			message_payload_1.clone()
		));
		assert_ok!(MessagesPallet::add_onchain_message(
			Origin::signed(caller_2),
			None,
			schema_id_1,
			message_payload_1
		));
		assert_ok!(MessagesPallet::add_onchain_message(
			Origin::signed(caller_2),
			None,
			schema_id_2,
			message_payload_2
		));

		// act
		run_to_block(current_block + 1);

		// assert
		assert_eq!(BlockMessages::<Test>::get().len(), 0);

		let list_1 = MessagesPallet::get_messages(current_block, schema_id_1);
		assert_eq!(list_1.len(), 2);
		let list_2 = MessagesPallet::get_messages(current_block, schema_id_2);
		assert_eq!(list_2.len(), 1);

		let events_occured = System::events();
		assert_eq!(events_occured.len(), 2);

		assert_eq!(
			events_occured[0].event,
			Event::MessagesPallet(MessageEvent::MessagesStored {
				block_number: current_block,
				schema_id: schema_id_1,
				count: 2
			}),
		);

		assert_eq!(
			events_occured[1].event,
			Event::MessagesPallet(MessageEvent::MessagesStored {
				block_number: current_block,
				schema_id: schema_id_2,
				count: 1
			}),
		);
	});
}

#[test]
fn get_messages_by_schema_with_valid_request_should_return_paginated() {
	new_test_ext().execute_with(|| {
		// arrange
		let schema_id: SchemaId = 1;
		let page_size = 3;
		let from_index = 2;
		let messages_per_block = vec![10, 0, 5, 2, 0, 3, 9];
		populate_messages(schema_id, messages_per_block.clone());
		let request = BlockPaginationRequest {
			page_size,
			from_block: 0,
			to_block: messages_per_block.len() as u64,
			from_index,
		};

		// act
		let res = MessagesPallet::get_messages_by_schema(schema_id, request);

		// assert
		assert_ok!(&res);

		let pagination_response = res.ok().unwrap();
		assert_eq!(pagination_response.content.len() as u32, page_size);
		assert_eq!(pagination_response.has_next, true);
		assert_eq!(pagination_response.next_block, Some(0));
		assert_eq!(pagination_response.next_index, Some(from_index + page_size));
		assert_eq!(
			pagination_response.content[0],
			MessageResponse {
				msa_id: 10,
				payload: Vec::from("{'fromId': 123, 'content': '232323114432'}".as_bytes()),
				index: from_index as u16,
				provider_msa_id: 1,
				block_number: 0
			}
		);
	});
}

#[test]
fn get_messages_by_schema_with_valid_request_should_return_next_block_and_index() {
	new_test_ext().execute_with(|| {
		// arrange
		let schema_id: SchemaId = 1;
		let page_size = 7;
		let from_index = 8;
		let messages_per_block = vec![10, 0, 5, 2, 0, 3, 9];
		populate_messages(schema_id, messages_per_block.clone());
		let request = BlockPaginationRequest {
			page_size,
			from_block: 0,
			to_block: messages_per_block.len() as u64,
			from_index,
		};

		// act
		let res = MessagesPallet::get_messages_by_schema(schema_id, request);

		// assert
		assert_ok!(&res);

		let pagination_response = res.ok().unwrap();
		assert_eq!(pagination_response.content.len() as u32, page_size);
		assert_eq!(pagination_response.has_next, true);
		assert_eq!(pagination_response.next_block, Some(3));
		assert_eq!(pagination_response.next_index, Some(0));
	});
}

#[test]
fn get_messages_by_schema_with_less_messages_than_page_size_should_not_has_next() {
	new_test_ext().execute_with(|| {
		// arrange
		let schema_id: SchemaId = 1;
		let page_size = 30;
		let from_index = 0;
		let messages_per_block = vec![10, 0, 5, 2, 0, 3, 9];
		populate_messages(schema_id, messages_per_block.clone());
		let request = BlockPaginationRequest {
			page_size,
			from_block: 0,
			to_block: messages_per_block.len() as u64,
			from_index,
		};

		// act
		let res = MessagesPallet::get_messages_by_schema(schema_id, request);

		// assert
		assert_ok!(&res);

		let pagination_response = res.ok().unwrap();
		assert_eq!(pagination_response.content.len() as u32, 29);
		assert_eq!(pagination_response.has_next, false);
		assert_eq!(pagination_response.next_block, None);
		assert_eq!(pagination_response.next_index, None);
	});
}

#[test]
fn get_messages_by_schema_with_invalid_request_should_panic() {
	new_test_ext().execute_with(|| {
		// arrange
		let schema_id: SchemaId = 1;
		let page_size = 30;
		let from_index = 0;
		let messages_per_block = vec![10, 0, 5, 2, 0, 3, 9];
		populate_messages(schema_id, messages_per_block);
		let request =
			BlockPaginationRequest { page_size, from_block: 22, to_block: 15, from_index };

		// act
		let res = MessagesPallet::get_messages_by_schema(schema_id, request);

		// assert
		assert_err!(res, Error::<Test>::InvalidPaginationRequest);
	});
}

#[test]
fn get_messages_by_schema_with_overflowing_input_should_panic() {
	new_test_ext().execute_with(|| {
		// arrange
		let schema_id: SchemaId = 1;
		let page_size = 30;
		let from_index = 0;
		let messages_per_block = vec![10, 0, 5, 2, 0, 3, 9];
		populate_messages(schema_id, messages_per_block);
		let request = BlockPaginationRequest {
			page_size,
			from_block: 22_343_223_111,
			to_block: 22_343_223_999,
			from_index,
		};

		// act
		let res = MessagesPallet::get_messages_by_schema(schema_id, request);

		// assert
		assert_err!(res, Error::<Test>::TypeConversionOverflow);
	});
}

#[test]
fn add_message_via_valid_delegate_should_pass() {
	new_test_ext().execute_with(|| {
		// arrange
		let message_producer = 1;
		let caller_1 = 5;
		let caller_2 = 2;
		let schema_id_1: SchemaId = 1;
		let schema_id_2: SchemaId = 2;
		let message_payload_1 = Vec::from("{'fromId': 123, 'content': '232323114432'}".as_bytes());
		let message_payload_2 = Vec::from("{'fromId': 343, 'content': '34333'}".as_bytes());

		// act
		assert_ok!(MessagesPallet::add_onchain_message(
			Origin::signed(caller_1),
			Some(message_producer),
			schema_id_1,
			message_payload_1.clone()
		));
		assert_ok!(MessagesPallet::add_onchain_message(
			Origin::signed(caller_2),
			Some(message_producer),
			schema_id_2,
			message_payload_2.clone()
		));

		// assert
		let list = BlockMessages::<Test>::get().into_inner();
		assert_eq!(list.len(), 2);

		assert_eq!(
			list[0],
			(
				Message {
					msa_id: message_producer,
					payload: message_payload_1.try_into().unwrap(),
					index: 0,
					provider_msa_id: get_msa_from_account(caller_1)
				},
				schema_id_1
			)
		);

		assert_eq!(
			list[1],
			(
				Message {
					msa_id: message_producer,
					payload: message_payload_2.try_into().unwrap(),
					index: 1,
					provider_msa_id: get_msa_from_account(caller_2)
				},
				schema_id_2
			)
		);
	});
}

#[test]
fn add_message_via_non_delegate_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let message_producer = 1;
		let message_provider = 2000;
		let schema_id_1: SchemaId = 1;
		let message_payload_1 = Vec::from("{'fromId': 123, 'content': '232323114432'}".as_bytes());
		// act
		assert_err!(
			MessagesPallet::add_onchain_message(
				Origin::signed(message_provider),
				Some(message_producer),
				schema_id_1,
				message_payload_1
			),
			Error::<Test>::UnAuthorizedDelegate
		);

		// assert
		let list = BlockMessages::<Test>::get().into_inner();
		assert_eq!(list.len(), 0);
	});
}

#[test]
fn add_message_with_invalid_schema_id_should_error() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 5;
		let schema_id_1: SchemaId = INVALID_SCHEMA_ID;
		let message_payload_1 = Vec::from(
			"{'fromId': 123, 'content': '232323114432'}{'fromId': 123, 'content': '232323114432'}"
				.as_bytes(),
		);

		// act
		assert_err!(
			MessagesPallet::add_onchain_message(
				Origin::signed(caller_1),
				None,
				schema_id_1,
				message_payload_1
			),
			Error::<Test>::InvalidSchemaId
		);
	});
}

#[test]
fn valid_payload_location() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5;
		let schema_id_1: SchemaId = IPFS_SCHEMA_ID;
		let payload: IPFSPayload = IPFSPayload::new(CID::new(Vec::from("foo")), 1);
		let info_result =
			MessagesPallet::add_ipfs_message(Origin::signed(caller_1), None, schema_id_1, payload);

		assert_eq!(info_result.is_ok(), true);
		let info: PostDispatchInfo = info_result.unwrap();

		assert_eq!(info.actual_weight.is_some(), true);
		assert_eq!(info.pays_fee, Pays::Yes);
	});
}

#[test]
fn invalid_payload_location_ipfs() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5;
		let schema_id_1: SchemaId = 1;
		let payload: IPFSPayload = IPFSPayload::new(CID::new(Vec::from("foo")), 1);

		assert_noop!(
			MessagesPallet::add_ipfs_message(Origin::signed(caller_1), None, schema_id_1, payload,),
			Error::<Test>::InvalidPayloadLocation
		);
	});
}

#[test]
fn invalid_payload_location_onchain() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5;
		let payload: Vec<u8> = Vec::from("foo");

		assert_noop!(
			MessagesPallet::add_onchain_message(
				Origin::signed(caller_1),
				None,
				IPFS_SCHEMA_ID,
				payload,
			),
			Error::<Test>::InvalidPayloadLocation
		);
	});
}

#[test]
fn ipfs_payload() {
	new_test_ext().execute_with(|| {
		let cid = CID::new(Vec::from("foo"));
		let payload = IPFSPayload::new(cid, 1);
		assert_eq!(payload.cid.get().len(), 3);
	});
}

#[test]
fn offchain_payload_size() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 5;
		let schema_id_1: SchemaId = 1;
		let cid = CID::new(Vec::from("{'fromId': 123, 'content': '232323114432'}{'fromId': 123, 'content': '232323114432'}{'fromId': 123, 'content': '232323114432'}".as_bytes()));
		let payload = IPFSPayload::new(cid, 1);

		// act
		assert_noop!(MessagesPallet::add_ipfs_message(Origin::signed(caller_1), None, schema_id_1, payload), Error::<Test>::ExceedsMaxMessagePayloadSizeBytes);
	});
}
