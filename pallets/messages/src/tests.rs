use super::{mock::*, Event as MessageEvent};
use crate::{BlockMessages, Config, Error, Message};
use common_primitives::messages::SchemaId;
use frame_support::{assert_noop, assert_ok};
use sp_std::vec::Vec;

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
		assert_ok!(MessagesPallet::add(
			Origin::signed(caller_1),
			schema_id_1,
			message_payload_1.clone()
		));
		assert_ok!(MessagesPallet::add(
			Origin::signed(caller_2),
			schema_id_2,
			message_payload_2.clone()
		));

		// assert
		let list = BlockMessages::<Test>::get();
		assert_eq!(list.len(), 2);

		assert_eq!(
			list[0],
			(
				Message {
					msa_id: get_msa_from_account(caller_1),
					data: message_payload_1.clone(),
					index: 0,
					signer: caller_1
				},
				schema_id_1
			)
		);

		assert_eq!(
			list[1],
			(
				Message {
					msa_id: get_msa_from_account(caller_2),
					data: message_payload_2.clone(),
					index: 1,
					signer: caller_2
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
		assert_noop!(MessagesPallet::add(Origin::signed(caller_1), schema_id_1, message_payload_1.clone()), Error::<Test>::TooLargeMessage);
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
			MessagesPallet::add(Origin::signed(caller_1), schema_id_1, message_payload_1.clone()),
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
			assert_ok!(MessagesPallet::add(
				Origin::signed(caller_1),
				schema_id_1,
				message_payload_1.clone()
			));
		}
		assert_noop!(
			MessagesPallet::add(Origin::signed(caller_1), schema_id_1, message_payload_1.clone()),
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
		assert_ok!(MessagesPallet::add(
			Origin::signed(caller_1),
			schema_id_1,
			message_payload_1.clone()
		));
		assert_ok!(MessagesPallet::add(
			Origin::signed(caller_2),
			schema_id_1,
			message_payload_1.clone()
		));
		assert_ok!(MessagesPallet::add(
			Origin::signed(caller_2),
			schema_id_2,
			message_payload_2.clone()
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
			})
			.into(),
		);

		assert_eq!(
			events_occured[1].event,
			Event::MessagesPallet(MessageEvent::MessagesStored {
				block_number: current_block,
				schema_id: schema_id_2,
				count: 1
			})
			.into(),
		);
	});
}
