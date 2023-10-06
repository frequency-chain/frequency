use crate::{
	tests::mock::*, BlockMessageIndex, Config, Error, Event as MessageEvent, Message, Messages,
};
use codec::Encode;
use common_primitives::{messages::MessageResponse, schema::*};
use frame_support::{assert_err, assert_noop, assert_ok, traits::OnInitialize, BoundedVec};
use frame_system::{EventRecord, Phase};
use multibase::Base;
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use rand::Rng;
use serde::Serialize;
use sp_core::{ConstU32, Get};
use sp_std::vec::Vec;

#[derive(Serialize)]
#[allow(non_snake_case)]
struct Payload {
	// Normally would be u64, but we keep it to u8 in order to keep payload size down in tests.
	fromId: u8,
	content: String,
}

pub const DUMMY_CID_SHA512: &str = "bafkrgqb76pscorjihsk77zpyst3p364zlti6aojlu4nga34vhp7t5orzwbwwytvp7ej44r5yhjzneanqwb5arcnvuvfwo2d4qgzyx5hymvto4";

/// Populate mocked Messages storage with message data.
///
/// # Arguments
/// * `schema_id` - Registered schema id to which stored messages should adhere
/// * `message_per_block` - A signed transaction origin from the provider
/// * `payload_location` - Determines how a message payload is encoded. PayloadLocation::IPFS
/// 		will encode (mock CID, IPFS_PAYLOAD_LENGTH) on the message payload.
fn populate_messages(
	schema_id: SchemaId,
	message_per_block: Vec<u32>,
	payload_location: PayloadLocation,
	cid_in: Option<&[u8]>,
) {
	let cid = match cid_in {
		Some(val) => val,
		None => &DUMMY_CID_BASE32[..],
	};

	let payload = match payload_location {
		// Just stick Itemized & Paginated here for coverage; we don't use them for Messages
		PayloadLocation::OnChain | PayloadLocation::Itemized | PayloadLocation::Paginated =>
			generate_payload(1, None),
		PayloadLocation::IPFS => (
			multibase::decode(sp_std::str::from_utf8(cid).unwrap()).unwrap().1,
			IPFS_PAYLOAD_LENGTH,
		)
			.encode(),
	};

	let mut counter = 0;
	for (idx, count) in message_per_block.iter().enumerate() {
		let mut list = BoundedVec::default();
		for _ in 0..*count {
			list.try_push(Message {
				msa_id: Some(10),
				payload: payload.clone().try_into().unwrap(),
				index: counter,
				provider_msa_id: 1,
			})
			.unwrap();
			counter += 1;
		}
		Messages::<Test>::insert(idx as u32, schema_id, list);
	}
}

/// Helper function to generate message payloads
///
/// # Arguments
/// * `num_items` - Number of message to include in payload
/// * `content_len` - Length of content string to generate
fn generate_payload(num_items: u8, content_len: Option<u8>) -> Vec<u8> {
	let mut result_str = String::new();
	let size = content_len.unwrap_or_else(|| 3);
	let mut rng = rand::thread_rng();

	for _ in 0..num_items {
		let payload = serde_json::to_string(&Payload {
			fromId: rng.gen(),
			content: (0..size).map(|_| "X").collect::<String>(),
		})
		.unwrap();

		result_str.push_str(payload.as_str());
	}

	result_str.as_bytes().to_vec()
}

#[test]
fn add_message_should_store_message_in_storage() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 5;
		let caller_2 = 2;
		let schema_id_1: SchemaId = 1;
		let schema_id_2: SchemaId = 2;
		let message_payload_1 = generate_payload(1, None);
		let message_payload_2 = generate_payload(1, None);
		let message_payload_3 = generate_payload(1, None);

		// act
		assert_ok!(MessagesPallet::add_onchain_message(
			RuntimeOrigin::signed(caller_1),
			None,
			schema_id_1,
			message_payload_1.clone()
		));

		assert_ok!(MessagesPallet::add_onchain_message(
			RuntimeOrigin::signed(caller_2),
			None,
			schema_id_2,
			message_payload_2.clone()
		));

		assert_ok!(MessagesPallet::add_onchain_message(
			RuntimeOrigin::signed(caller_2),
			None,
			schema_id_2,
			message_payload_3.clone()
		));

		// assert messages
		let list1 = Messages::<Test>::get(1, schema_id_1).into_inner();
		let list2 = Messages::<Test>::get(1, schema_id_2).into_inner();
		assert_eq!(list1.len(), 1);
		assert_eq!(list2.len(), 2);

		assert_eq!(
			list1[0],
			Message {
				msa_id: Some(get_msa_from_account(caller_1)),
				payload: message_payload_1.try_into().unwrap(),
				index: 0,
				provider_msa_id: get_msa_from_account(caller_1)
			}
		);

		assert_eq!(
			list2,
			vec![
				Message {
					msa_id: Some(get_msa_from_account(caller_2)),
					payload: message_payload_2.try_into().unwrap(),
					index: 1,
					provider_msa_id: get_msa_from_account(caller_2)
				},
				Message {
					msa_id: Some(get_msa_from_account(caller_2)),
					payload: message_payload_3.try_into().unwrap(),
					index: 2,
					provider_msa_id: get_msa_from_account(caller_2)
				},
			]
		);

		// assert events
		let events_occured = System::events();
		assert_eq!(
			events_occured,
			vec![
				EventRecord {
					phase: Phase::Initialization,
					event: RuntimeEvent::MessagesPallet(MessageEvent::MessagesStored {
						block_number: 1,
						schema_id: schema_id_1,
					}),
					topics: vec![]
				},
				EventRecord {
					phase: Phase::Initialization,
					event: RuntimeEvent::MessagesPallet(MessageEvent::MessagesStored {
						block_number: 1,
						schema_id: schema_id_2,
					}),
					topics: vec![]
				},
			]
		);
	});
}

#[test]
fn add_message_with_too_large_message_should_panic() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 5;
		let schema_id_1: SchemaId = 1;
		let message_payload_1 = generate_payload(3, None);

		// act
		assert_noop!(
			MessagesPallet::add_onchain_message(
				RuntimeOrigin::signed(caller_1),
				None,
				schema_id_1,
				message_payload_1
			),
			Error::<Test>::ExceedsMaxMessagePayloadSizeBytes
		);
	});
}

#[test]
fn add_message_with_invalid_msa_account_errors() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 1000;
		let schema_id_1: SchemaId = 1;
		let message_payload_1 = generate_payload(2, None);

		// act
		assert_noop!(
			MessagesPallet::add_onchain_message(
				RuntimeOrigin::signed(caller_1),
				None,
				schema_id_1,
				message_payload_1
			),
			Error::<Test>::InvalidMessageSourceAccount
		);
	});
}

#[test]
fn add_ipfs_message_with_invalid_msa_account_errors() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 1000;
		let schema_id_1: SchemaId = IPFS_SCHEMA_ID;

		// act
		assert_noop!(
			MessagesPallet::add_ipfs_message(
				RuntimeOrigin::signed(caller_1),
				schema_id_1,
				DUMMY_CID_BASE32.to_vec(),
				15
			),
			Error::<Test>::InvalidMessageSourceAccount
		);
	});
}

#[test]
fn add_message_with_maxed_out_storage_errors() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 5;
		let schema_id_1: SchemaId = 1;
		let message_payload_1 = generate_payload(1, None);

		// act
		for _ in 0..<Test as Config>::MaxMessagesPerBlock::get() {
			assert_ok!(MessagesPallet::add_onchain_message(
				RuntimeOrigin::signed(caller_1),
				None,
				schema_id_1,
				message_payload_1.clone()
			));
		}
		assert_noop!(
			MessagesPallet::add_onchain_message(
				RuntimeOrigin::signed(caller_1),
				None,
				schema_id_1,
				message_payload_1
			),
			Error::<Test>::TooManyMessagesInBlock
		);
	});
}

#[test]
fn add_ipfs_message_with_maxed_out_storage_errors() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 5;
		let schema_id_1: SchemaId = IPFS_SCHEMA_ID;

		// act
		for _ in 0..<Test as Config>::MaxMessagesPerBlock::get() {
			assert_ok!(MessagesPallet::add_ipfs_message(
				RuntimeOrigin::signed(caller_1),
				schema_id_1,
				DUMMY_CID_BASE32.to_vec(),
				15
			));
		}
		assert_noop!(
			MessagesPallet::add_ipfs_message(
				RuntimeOrigin::signed(caller_1),
				schema_id_1,
				DUMMY_CID_BASE32.to_vec(),
				15
			),
			Error::<Test>::TooManyMessagesInBlock
		);
	});
}

/// Assert that MessageResponse for IPFS messages returns the payload_length of the offchain message.
#[test]
fn get_messages_by_schema_with_ipfs_payload_location_should_return_offchain_payload_length() {
	new_test_ext().execute_with(|| {
		// Setup
		let schema_id: SchemaId = IPFS_SCHEMA_ID;
		let current_block = 1;

		// Populate
		populate_messages(schema_id, vec![1], PayloadLocation::IPFS, None);

		// Run to the block +
		run_to_block(current_block + 1);

		let list =
			MessagesPallet::get_messages_by_schema_and_block(schema_id, PayloadLocation::IPFS, 0);

		// IPFS messages should return the payload length that was encoded in a tuple along
		// with the CID: (cid, payload_length).
		assert_eq!(list.len(), 1);
		assert_eq!(
			list[0],
			MessageResponse {
				payload: None,
				index: 0,
				provider_msa_id: 1,
				block_number: 0,
				payload_length: Some(IPFS_PAYLOAD_LENGTH),
				msa_id: None,
				cid: Some(DUMMY_CID_BASE32.to_vec())
			}
		);
	});
}

#[test]
fn retrieved_ipfs_message_should_always_be_in_base32() {
	new_test_ext().execute_with(|| {
		let schema_id = IPFS_SCHEMA_ID;
		let current_block: u32 = 1;

		// Populate message storage using Base64-encoded CID
		populate_messages(schema_id, vec![1], PayloadLocation::IPFS, Some(DUMMY_CID_BASE64));

		// Run to the block
		run_to_block(current_block + 1);

		let list =
			MessagesPallet::get_messages_by_schema_and_block(schema_id, PayloadLocation::IPFS, 0);

		assert_eq!(list[0].cid.as_ref().unwrap(), &DUMMY_CID_BASE32.to_vec());
	})
}

#[test]
fn get_messages_by_schema_with_ipfs_payload_location_should_fail_bad_schema() {
	new_test_ext().execute_with(|| {
		let bad_message: Message<MessagesMaxPayloadSizeBytes> = Message {
			payload: BoundedVec::try_from(
				vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16].to_vec(),
			)
			.unwrap(),
			msa_id: Some(0),
			provider_msa_id: 1,
			index: 0,
		};
		let mapped_response = bad_message.map_to_response(0, PayloadLocation::IPFS);
		assert_eq!(
			mapped_response.cid,
			Some(multibase::encode(Base::Base32Lower, Vec::new()).as_bytes().to_vec())
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
		let message_payload_1 = generate_payload(1, None);
		// act
		assert_err!(
			MessagesPallet::add_onchain_message(
				RuntimeOrigin::signed(message_provider),
				Some(message_producer),
				schema_id_1,
				message_payload_1
			),
			Error::<Test>::UnAuthorizedDelegate
		);

		// assert
		let list = Messages::<Test>::get(1, schema_id_1).into_inner();
		assert_eq!(list.len(), 0);
	});
}

#[test]
fn add_message_with_invalid_schema_id_should_error() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 5;
		let schema_id_1: SchemaId = INVALID_SCHEMA_ID;
		let message_payload_1 = generate_payload(2, None);

		// act
		assert_err!(
			MessagesPallet::add_onchain_message(
				RuntimeOrigin::signed(caller_1),
				None,
				schema_id_1,
				message_payload_1
			),
			Error::<Test>::InvalidSchemaId
		);
	});
}

#[test]
fn add_ipfs_message_with_invalid_schema_id_should_error() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 5;
		let schema_id_1: SchemaId = INVALID_SCHEMA_ID;

		// act
		assert_err!(
			MessagesPallet::add_ipfs_message(
				RuntimeOrigin::signed(caller_1),
				schema_id_1,
				DUMMY_CID_BASE32.to_vec(),
				15
			),
			Error::<Test>::InvalidSchemaId
		);
	});
}

#[test]
fn valid_payload_location_ipfs() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5;
		let schema_id_1: SchemaId = IPFS_SCHEMA_ID;
		assert_ok!(MessagesPallet::add_ipfs_message(
			RuntimeOrigin::signed(caller_1),
			schema_id_1,
			DUMMY_CID_BASE32.to_vec(),
			1,
		));
	});
}

#[test]
fn invalid_payload_location_ipfs() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5;
		let schema_id_1: SchemaId = 1;

		assert_noop!(
			MessagesPallet::add_ipfs_message(
				RuntimeOrigin::signed(caller_1),
				schema_id_1,
				DUMMY_CID_BASE32.to_vec(),
				1
			),
			Error::<Test>::InvalidPayloadLocation
		);
	});
}

#[test]
fn invalid_payload_location_onchain() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5;
		let payload = generate_payload(1, None);

		assert_noop!(
			MessagesPallet::add_onchain_message(
				RuntimeOrigin::signed(caller_1),
				None,
				IPFS_SCHEMA_ID,
				payload,
			),
			Error::<Test>::InvalidPayloadLocation
		);
	});
}

#[test]
fn add_ipfs_message_with_large_payload_errors() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5u64;

		assert_noop!(
			MessagesPallet::add_ipfs_message(
				RuntimeOrigin::signed(caller_1),
				IPFS_SCHEMA_ID,
				// We've deliberately mocked MaxMessagePayloadSizeBytes to be too small to contain a  CIDv1 with a SHA2-512 hash.
				DUMMY_CID_SHA512.as_bytes().to_vec(),
				15
			),
			Error::<Test>::ExceedsMaxMessagePayloadSizeBytes
		);
	})
}

#[test]
fn add_ipfs_message_cid_v0_errors() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5u64;
		const CIDV0: &str = "QmYzm8KGxRHr7nGn5g5Z9Zv9r8nN5WNn7Ajya6x7RxmAB1";

		assert_noop!(
			MessagesPallet::add_ipfs_message(
				RuntimeOrigin::signed(caller_1),
				IPFS_SCHEMA_ID,
				CIDV0.as_bytes().to_vec(),
				15
			),
			Error::<Test>::UnsupportedCidVersion
		);
	})
}

#[test]
fn add_ipfs_message_bad_cid_errors() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5u64;

		assert_noop!(
			MessagesPallet::add_ipfs_message(
				RuntimeOrigin::signed(caller_1),
				IPFS_SCHEMA_ID,
				"foo".as_bytes().to_vec(),
				15
			),
			Error::<Test>::InvalidCid
		);
	})
}

#[test]
fn on_initialize_should_clean_up_temporary_storage() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 5;
		let caller_2 = 2;
		let schema_id_1: SchemaId = 1;
		let schema_id_2: SchemaId = 2;
		let message_payload_1 = generate_payload(1, None);
		let message_payload_2 = generate_payload(1, None);
		assert_ok!(MessagesPallet::add_onchain_message(
			RuntimeOrigin::signed(caller_1),
			None,
			schema_id_1,
			message_payload_1.clone()
		));
		assert_ok!(MessagesPallet::add_onchain_message(
			RuntimeOrigin::signed(caller_2),
			None,
			schema_id_2,
			message_payload_2.clone()
		));

		// act
		MessagesPallet::on_initialize(System::block_number() + 1);

		// assert
		assert_eq!(BlockMessageIndex::<Test>::get(), 0);
	});
}

#[test]
fn validate_cid_invalid_utf8_errors() {
	new_test_ext().execute_with(|| {
		let bad_cid = vec![0xfc, 0xa1, 0xa1, 0xa1, 0xa1, 0xa1];

		assert_noop!(MessagesPallet::validate_cid(&bad_cid), Error::<Test>::InvalidCid);
	})
}

#[test]
fn validate_cid_too_short_errors() {
	new_test_ext().execute_with(|| {
		let bad_cid = "a".as_bytes().to_vec();

		assert_noop!(MessagesPallet::validate_cid(&bad_cid), Error::<Test>::InvalidCid);
	})
}

#[test]
fn validate_cid_v0_errors() {
	new_test_ext().execute_with(|| {
		let bad_cid = "Qmxxx".as_bytes().to_vec();

		assert_noop!(MessagesPallet::validate_cid(&bad_cid), Error::<Test>::UnsupportedCidVersion);
	})
}

#[test]
fn validate_cid_invalid_multibase_errors() {
	new_test_ext().execute_with(|| {
		let bad_cid = "aaaa".as_bytes().to_vec();

		assert_noop!(MessagesPallet::validate_cid(&bad_cid), Error::<Test>::InvalidCid);
	})
}

#[test]
fn validate_cid_invalid_cid_errors() {
	new_test_ext().execute_with(|| {
		let bad_cid = multibase::encode(Base::Base32Lower, "foo").as_bytes().to_vec();

		assert_noop!(MessagesPallet::validate_cid(&bad_cid), Error::<Test>::InvalidCid);
	})
}

#[test]
fn validate_cid_valid_cid_succeeds() {
	new_test_ext().execute_with(|| {
		let bad_cid = DUMMY_CID_BASE32.to_vec();

		assert_ok!(MessagesPallet::validate_cid(&bad_cid));
	})
}

#[test]
fn validate_cid_not_utf8_aligned_errors() {
	new_test_ext().execute_with(|| {
		// This should not panic, but should return an error.
		let bad_cid = vec![55, 197, 136, 0, 0, 0, 0, 0, 0, 0, 0];

		assert_noop!(MessagesPallet::validate_cid(&bad_cid), Error::<Test>::InvalidCid);
	})
}

#[test]
fn validate_cid_not_correct_format_errors() {
	new_test_ext().execute_with(|| {
		// This should not panic, but should return an error.
		let bad_cid = vec![0, 1, 0, 1, 203, 155, 0, 0, 0, 5, 67];

		assert_noop!(MessagesPallet::validate_cid(&bad_cid), Error::<Test>::InvalidCid);
		// This should not panic, but should return an error.
		let another_bad_cid = vec![241, 0, 0, 0, 0, 0, 128, 132, 132, 132, 58];

		assert_noop!(MessagesPallet::validate_cid(&another_bad_cid), Error::<Test>::InvalidCid);
	})
}

#[test]
fn map_to_response_on_chain() {
	let payload_vec = b"123456789012345678901234567890".to_vec();
	let payload_bounded = BoundedVec::<u8, ConstU32<100>>::try_from(payload_vec.clone()).unwrap();
	let msg =
		Message { payload: payload_bounded, provider_msa_id: 10u64, msa_id: None, index: 1u16 };
	let expected = MessageResponse {
		provider_msa_id: 10u64,
		index: 1u16,
		block_number: 42,
		msa_id: None,
		payload: Some(payload_vec),
		cid: None,
		payload_length: None,
	};
	assert_eq!(msg.map_to_response(42, PayloadLocation::OnChain), expected);
}

#[test]
fn map_to_response_ipfs() {
	let cid = DUMMY_CID_SHA512;
	let payload_tuple: crate::OffchainPayloadType = (multibase::decode(cid).unwrap().1, 10);
	let payload = BoundedVec::<u8, ConstU32<500>>::try_from(payload_tuple.encode()).unwrap();
	let msg = Message { payload, provider_msa_id: 10u64, msa_id: None, index: 1u16 };
	let expected = MessageResponse {
		provider_msa_id: 10u64,
		index: 1u16,
		block_number: 42,
		msa_id: None,
		payload: None,
		cid: Some(cid.as_bytes().to_vec()),
		payload_length: Some(10),
	};
	assert_eq!(msg.map_to_response(42, PayloadLocation::IPFS), expected);
}
