extern crate alloc;
use crate::{
	pallet::{Config, MessagesV3},
	tests::mock::*,
	BlockMessageIndex, Error, Event as MessageEvent, MapToResponse, Message,
};
use alloc::vec::Vec;
use common_primitives::{messages::MessageResponseV2, schema::*};
use frame_support::{assert_err, assert_noop, assert_ok, traits::OnInitialize, BoundedVec};
use frame_system::{EventRecord, Phase};
use multibase::Base;
use parity_scale_codec::Encode;
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use sp_core::ConstU32;

pub const DUMMY_CID_SHA512: &str = "bafkrgqb76pscorjihsk77zpyst3p364zlti6aojlu4nga34vhp7t5orzwbwwytvp7ej44r5yhjzneanqwb5arcnvuvfwo2d4qgzyx5hymvto4";
pub const DUMMY_CID_SHA256: &str = "bagaaierasords4njcts6vs7qvdjfcvgnume4hqohf65zsfguprqphs3icwea";
pub const DUMMY_CID_BLAKE3: &str = "bafkr4ihn4xalcdzoyslzy2nvf5q6il7vwqjvdhhatpqpctijrxh6l5xzru";

/// Populate mocked Messages storage with message data.
///
/// # Arguments
/// * `intent_id` - Registered intent id with which to associate messages
/// * `message_per_block` - A signed transaction origin from the provider
/// * `payload_location` - Determines how a message payload is encoded. PayloadLocation::IPFS
/// 		will encode (mock CID, IPFS_PAYLOAD_LENGTH) on the message payload.
fn populate_messages_v3(
	intent_id: IntentId,
	message_per_block: Vec<u32>,
	payload_location: PayloadLocation,
	cid_in: Option<&[u8]>,
) {
	let cid = cid_in.unwrap_or_else(|| &DUMMY_CID_BASE32[..]);

	let payload = match payload_location {
		// Just stick Itemized & Paginated here for coverage; we don't use them for Messages
		PayloadLocation::OnChain | PayloadLocation::Itemized | PayloadLocation::Paginated =>
			generate_payload(1),
		PayloadLocation::IPFS =>
			(multibase::decode(core::str::from_utf8(cid).unwrap()).unwrap().1, IPFS_PAYLOAD_LENGTH)
				.encode(),
	};

	let mut counter = 0;
	for (idx, count) in message_per_block.iter().enumerate() {
		for _ in 0..*count {
			MessagesV3::<Test>::set(
				(idx as u32, intent_id, counter),
				Some(Message {
					schema_id: intent_id as SchemaId,
					msa_id: Some(DUMMY_MSA_ID),
					payload: payload.clone().try_into().unwrap(),
					provider_msa_id: 1,
				}),
			);
			counter += 1;
		}
	}
}

/// Helper function to generate message payloads
///
/// # Arguments
/// * `content_len` - Length of payload
fn generate_payload(content_len: u32) -> Vec<u8> {
	vec![1u8; content_len as usize].to_vec()
}

#[test]
fn add_message_should_store_message_in_storage() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 5;
		let caller_2 = 2;
		let schema_id_1: SchemaId = 1;
		let schema_id_2: SchemaId = 2;
		let message_payload_1 = generate_payload(1);
		let message_payload_2 = generate_payload(1);
		let message_payload_3 = generate_payload(1);

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
		let msg1 = MessagesV3::<Test>::get((1, schema_id_1, 0u16));
		let msg2 = MessagesV3::<Test>::get((1, schema_id_2, 1u16));
		let msg3 = MessagesV3::<Test>::get((1, schema_id_2, 2u16));

		assert_eq!(
			msg1,
			Some(Message {
				schema_id: schema_id_1,
				msa_id: Some(get_msa_from_account(caller_1)),
				payload: message_payload_1.try_into().unwrap(),
				provider_msa_id: get_msa_from_account(caller_1)
			})
		);

		assert_eq!(
			msg2,
			Some(Message {
				schema_id: schema_id_2,
				msa_id: Some(get_msa_from_account(caller_2)),
				payload: message_payload_2.try_into().unwrap(),
				provider_msa_id: get_msa_from_account(caller_2)
			})
		);

		assert_eq!(
			msg3,
			Some(Message {
				schema_id: schema_id_2,
				msa_id: Some(get_msa_from_account(caller_2)),
				payload: message_payload_3.try_into().unwrap(),
				provider_msa_id: get_msa_from_account(caller_2)
			})
		);

		// assert events
		let events_occurred = System::events();
		assert_eq!(
			events_occurred,
			vec![EventRecord {
				phase: Phase::Initialization,
				event: RuntimeEvent::MessagesPallet(MessageEvent::MessagesInBlock),
				topics: vec![]
			},]
		);
	});
}

#[test]
fn add_message_with_too_large_message_should_panic() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 5;
		let message_payload_1 =
			generate_payload(<Test as Config>::MessagesMaxPayloadSizeBytes::get() + 1);

		// act
		assert_noop!(
			MessagesPallet::add_onchain_message(
				RuntimeOrigin::signed(caller_1),
				None,
				ON_CHAIN_SCHEMA_ID,
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
		let message_payload_1 = generate_payload(2);

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

/// Assert that MessageResponse for IPFS messages returns the payload_length of the offchain message.
#[test]
fn get_messages_by_intent_with_ipfs_payload_location_should_return_offchain_payload_length() {
	new_test_ext().execute_with(|| {
		// Setup
		let intent_id = 1 as IntentId;
		let current_block = 0;

		// Populate
		populate_messages_v3(intent_id, vec![1], PayloadLocation::IPFS, None);

		// Run to the block +
		run_to_block(current_block + 1);

		let list =
			MessagesPallet::get_messages_by_intent_and_block(intent_id, PayloadLocation::IPFS, 0);

		// IPFS messages should return the payload length that was encoded in a tuple along
		// with the CID: (cid, payload_length).
		assert_eq!(list.len(), 1);
		assert_eq!(
			list[0],
			MessageResponseV2 {
				schema_id: intent_id,
				payload: None,
				index: 0,
				provider_msa_id: 1,
				block_number: 0,
				payload_length: Some(IPFS_PAYLOAD_LENGTH),
				msa_id: Some(DUMMY_MSA_ID),
				cid: Some(DUMMY_CID_BASE32.to_vec()),
				..Default::default()
			}
		);
	});
}

#[test]
fn retrieved_ipfs_message_cid_should_always_be_in_base32() {
	new_test_ext().execute_with(|| {
		let intent_id = 1 as IntentId;
		let current_block: u32 = 1;

		// Populate message storage using Base64-encoded CID
		populate_messages_v3(intent_id, vec![1], PayloadLocation::IPFS, Some(DUMMY_CID_BASE64));

		// Run to the block
		run_to_block(current_block + 1);

		let list =
			MessagesPallet::get_messages_by_intent_and_block(intent_id, PayloadLocation::IPFS, 0);

		assert_eq!(list[0].cid.as_ref().unwrap(), &DUMMY_CID_BASE32.to_vec());
	})
}

#[test]
fn decode_message_with_invalid_payload_should_yield_empty_cid() {
	new_test_ext().execute_with(|| {
		let bad_message: Message<MessagesMaxPayloadSizeBytes> = Message {
			payload: BoundedVec::try_from(
				[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16].to_vec(),
			)
			.unwrap(),
			msa_id: Some(0),
			provider_msa_id: 1,
			schema_id: 1,
		};
		let mapped_response: MessageResponseV2 =
			bad_message.map_to_response((0, PayloadLocation::IPFS, 0)).unwrap();
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
		let message_payload_1 = generate_payload(1);
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
		let msg = MessagesV3::<Test>::get((1, schema_id_1, 0));
		assert_eq!(msg, None);
	});
}

#[test]
fn add_onchain_message_with_invalid_schema_id_should_error() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = 5;
		let schema_id_1: SchemaId = INVALID_SCHEMA_ID;
		let message_payload_1 = generate_payload(2);

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
fn add_ipfs_message_with_invalid_payload_location_should_fail() {
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
fn add_onchain_message_with_invalid_payload_location_should_fail() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5;
		let payload = generate_payload(1);

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
fn add_ipfs_message_with_unsupported_cid_hash_should_fail() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5u64;

		assert_noop!(
			MessagesPallet::add_ipfs_message(
				RuntimeOrigin::signed(caller_1),
				IPFS_SCHEMA_ID,
				DUMMY_CID_SHA512.as_bytes().to_vec(),
				15
			),
			Error::<Test>::InvalidCid
		);
	})
}

#[test]
fn add_ipfs_message_with_sha2_256_cid_should_succeed() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5u64;

		assert_ok!(MessagesPallet::add_ipfs_message(
			RuntimeOrigin::signed(caller_1),
			IPFS_SCHEMA_ID,
			DUMMY_CID_SHA256.as_bytes().to_vec(),
			15
		));
	})
}

#[test]
fn add_ipfs_message_with_blake3_cid_should_succeed() {
	new_test_ext().execute_with(|| {
		let caller_1 = 5u64;

		assert_ok!(MessagesPallet::add_ipfs_message(
			RuntimeOrigin::signed(caller_1),
			IPFS_SCHEMA_ID,
			DUMMY_CID_BLAKE3.as_bytes().to_vec(),
			15
		));
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
		let message_payload_1 = generate_payload(1);
		let message_payload_2 = generate_payload(1);
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
fn validate_cid_valid_cid_sha2_256_succeeds() {
	new_test_ext().execute_with(|| {
		let cid = DUMMY_CID_BASE32.to_vec();

		assert_ok!(MessagesPallet::validate_cid(&cid));
	})
}

#[test]
fn validate_cid_valid_cid_blake3_succeeds() {
	new_test_ext().execute_with(|| {
		let cid = DUMMY_CID_BLAKE3.as_bytes().to_vec();

		assert_ok!(MessagesPallet::validate_cid(&cid));
	})
}

#[test]
fn validate_cid_invalid_hash_function_errors() {
	new_test_ext().execute_with(|| {
		let bad_cid = DUMMY_CID_SHA512.as_bytes().to_vec();

		assert_noop!(MessagesPallet::validate_cid(&bad_cid), Error::<Test>::InvalidCid);
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
fn validate_cid_unwrap_errors() {
	new_test_ext().execute_with(|| {
		// This should not panic, but should return an error.
		let bad_cid = vec![102, 70, 70, 70, 70, 70, 70, 70, 70, 48, 48, 48, 54, 53, 53, 48, 48];

		assert_noop!(MessagesPallet::validate_cid(&bad_cid), Error::<Test>::InvalidCid);
	})
}

#[test]
fn map_to_response_on_chain() {
	let payload_vec = b"123456789012345678901234567890".to_vec();
	let payload_bounded = BoundedVec::<u8, ConstU32<100>>::try_from(payload_vec.clone()).unwrap();
	let msg = Message {
		payload: payload_bounded,
		provider_msa_id: DUMMY_MSA_ID,
		msa_id: None,
		schema_id: 1,
	};
	let expected = MessageResponseV2 {
		provider_msa_id: DUMMY_MSA_ID,
		index: 1u16,
		block_number: 42,
		msa_id: None,
		payload: Some(payload_vec),
		cid: None,
		payload_length: None,
		schema_id: 1,
	};
	assert_eq!(msg.map_to_response((42, 1, PayloadLocation::OnChain, 1)).unwrap(), expected);
}

#[test]
fn map_to_response_ipfs() {
	let cid = DUMMY_CID_SHA512;
	let payload_tuple: crate::OffchainPayloadType = (multibase::decode(cid).unwrap().1, 10);
	let payload = BoundedVec::<u8, ConstU32<500>>::try_from(payload_tuple.encode()).unwrap();
	let msg = Message { payload, provider_msa_id: 10u64, msa_id: None, schema_id: 1 };
	let expected = MessageResponseV2 {
		provider_msa_id: DUMMY_MSA_ID,
		index: 1u16,
		block_number: 42,
		msa_id: None,
		payload: None,
		cid: Some(cid.as_bytes().to_vec()),
		payload_length: Some(10),
		schema_id: 1,
	};
	assert_eq!(msg.map_to_response((42, 1, PayloadLocation::IPFS, 1)).unwrap(), expected);
}
