use crate::{
	stateful_child_tree::StatefulChildTree,
	test_common::{constants::*, test_utility::*},
	tests::mock::*,
	types::*,
	Config, Error, Event as StatefulEvent,
};
use common_primitives::{
	schema::SchemaId,
	stateful_storage::{PageHash, PageId},
	utils::wrap_binary_data,
};
use frame_support::{assert_err, assert_ok};
use parity_scale_codec::Encode;
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use sp_core::{sr25519, Get, Pair};
use sp_runtime::{AccountId32, MultiSignature};
use sp_std::hash::Hasher;
use twox_hash::XxHash64;

pub fn hash_payload(data: &[u8]) -> PageHash {
	let mut hasher = XxHash64::with_seed(0);
	sp_std::hash::Hash::hash(&data, &mut hasher);
	let value_bytes: [u8; 4] =
		hasher.finish().to_be_bytes()[..4].try_into().expect("incorrect hash size");
	PageHash::from_be_bytes(value_bytes)
}

#[test]
fn upsert_page_id_out_of_bounds_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = 1;
		let page_id = <Test as Config>::MaxPaginatedPageId::get() + 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id.into(),
				schema_id,
				page_id.into(),
				hash_payload(&payload),
				payload
			),
			Error::<Test>::PageIdExceedsMaxAllowed
		)
	})
}

#[test]
fn upsert_page_with_invalid_msa_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = INVALID_MSA_ID;
		let caller_1 = test_public(msa_id); // hard-coded in mocks to return None for MSA
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id.into(),
				schema_id,
				page_id,
				hash_payload(&payload),
				payload
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	})
}

#[test]
fn upsert_page_with_invalid_schema_id_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = INVALID_SCHEMA_ID;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id.into(),
				schema_id,
				page_id,
				hash_payload(&payload),
				payload
			),
			Error::<Test>::InvalidSchemaId
		)
	})
}

#[test]
fn upsert_page_with_invalid_schema_payload_location_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = ITEMIZED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id.into(),
				schema_id,
				page_id,
				hash_payload(&payload),
				payload
			),
			Error::<Test>::SchemaPayloadLocationMismatch
		)
	})
}

#[test]
fn upsert_page_with_no_delegation_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = test_public(1);
		let msa_id = 2;
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
				PageHash::default(),
				payload
			),
			Error::<Test>::UnauthorizedDelegate
		)
	})
}

#[test]
fn upsert_new_page_with_bad_state_hash_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id.into(),
				schema_id,
				page_id,
				hash_payload(&payload),
				payload
			),
			Error::<Test>::StalePageState
		)
	})
}

#[test]
fn upsert_existing_page_with_bad_state_hash_errors() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let page = generate_page::<PaginatedPageSize>(None, Some(100));

		let key = (schema_id, page_id);
		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key,
			&page,
		);

		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
				0u32,
				page.data,
			),
			Error::<Test>::StalePageState
		)
	})
}

#[test]
fn upsert_new_page_succeeds() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let page: PaginatedPage<Test> = payload.clone().into();

		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id,
			0u32,
			payload.into()
		));

		let keys = (schema_id, page_id);
		let new_page: PaginatedPage<Test> = <StatefulChildTree>::try_read(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
		)
		.unwrap()
		.expect("new page is empty");
		assert_eq!(page.data, new_page.data, "new page contents incorrect");
		assert_eq!(new_page.nonce, 1, "new page nonce incorrect");
	})
}

#[test]
fn upsert_existing_page_modifies_page() {
	new_test_ext().execute_with(|| {
		// setup
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id: SchemaId = PAGINATED_SCHEMA;
		let page_id: PageId = 1;
		let old_nonce = 3;
		let old_content = generate_payload_bytes(Some(200));
		let new_content = generate_payload_bytes::<PaginatedPageSize>(Some(201));
		let mut old_page: PaginatedPage<Test> = old_content.clone().into();
		old_page.nonce = old_nonce;

		let keys = (schema_id, page_id);
		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
			old_page,
		);
		let old_page: PaginatedPage<Test> = <StatefulChildTree>::try_read(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
		)
		.unwrap()
		.unwrap();

		assert_eq!(old_content, old_page.data);
		assert_eq!(old_nonce, old_page.nonce);
		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id,
			old_page.get_hash(),
			new_content.clone().into()
		));

		let new_page: PaginatedPage<Test> = <StatefulChildTree>::try_read(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
		)
		.unwrap()
		.unwrap();
		assert_eq!(new_content, new_page.data);
		assert_eq!(old_nonce + 1, new_page.nonce);
	})
}

#[test]
#[allow(deprecated)]
fn upsert_page_with_signature_having_page_id_out_of_bounds_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = (<Test as Config>::MaxPaginatedPageId::get() + 1).into();
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::PageIdExceedsMaxAllowed
		)
	})
}

#[test]
#[allow(deprecated)]
fn upsert_page_with_signature_having_expired_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let block_number = 10;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: block_number,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		System::set_block_number(block_number);
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::ProofHasExpired
		)
	})
}

#[test]
#[allow(deprecated)]
fn upsert_page_with_signature_having_out_of_window_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let mortality_window: u32 = <Test as Config>::MortalityWindowSize::get();
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: (mortality_window + 1).into(),
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::ProofNotYetValid
		)
	})
}

#[test]
#[allow(deprecated)]
fn upsert_page_with_signature_having_wrong_signature_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let (signature_key, _) = sr25519::Pair::generate();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = signature_key.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidSignature
		)
	})
}

#[test]
#[allow(deprecated)]
fn upsert_page_with_signature_having_non_existing_msa_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let pair = get_invalid_msa_signature_account(); // hardcoded key that returns None Msa
		let delegator_key = pair.public();
		let msa_id = 1;
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	})
}

#[test]
#[allow(deprecated)]
fn upsert_page_with_signature_having_wrong_msa_in_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_correct_msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let wrong_msa_id = 3;
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id: wrong_msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	})
}

#[test]
#[allow(deprecated)]
fn upsert_page_with_signature_having_invalid_schema_id_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = INVALID_SCHEMA_ID;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidSchemaId
		)
	})
}

#[test]
#[allow(deprecated)]
fn upsert_page_with_signature_having_invalid_schema_location_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::SchemaPayloadLocationMismatch
		)
	})
}

#[test]
#[allow(deprecated)]
fn upsert_page_with_signature_having_invalid_hash_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default() + 1, // any non default hash value
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::StalePageState
		)
	})
}

#[test]
#[allow(deprecated)]
fn upsert_page_with_signature_having_valid_inputs_should_work() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let page: PaginatedPage<Test> = payload.clone().into();
		let payload = PaginatedUpsertSignaturePayload {
			payload,
			target_hash: PageHash::default(),
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_ok!(StatefulStoragePallet::upsert_page_with_signature(
			RuntimeOrigin::signed(caller_1),
			delegator_key.into(),
			owner_signature,
			payload
		));

		// assert
		let keys = (schema_id, page_id);
		let new_page: PaginatedPage<Test> = <StatefulChildTree>::try_read(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
		)
		.expect("new page could not be read")
		.expect("new page is empty");
		assert_eq!(page.data, new_page.data, "new page contents incorrect");
		assert_eq!(new_page.nonce, 1, "new page nonce incorrect");
		System::assert_last_event(
			StatefulEvent::PaginatedPageUpdated {
				msa_id,
				schema_id,
				page_id,
				prev_content_hash: PageHash::default(),
				curr_content_hash: new_page.get_hash(),
			}
			.into(),
		);
	})
}

#[test]
fn upsert_page_on_signature_schema_fails_for_non_owner() {
	new_test_ext().execute_with(|| {
		// setup
		// Note: normal use case for this test would be called by a delegate;
		// we don't bother setting up the delegation because the call should fail
		// before we check the delegation, as long as the owner_msa_id != caller_msa_id
		let (caller_msa_id, caller_keys) = get_signature_account();
		let caller_1: AccountId32 = caller_keys.public().into();
		let owner_msa_id = caller_msa_id.saturating_add(1);
		let schema_id = PAGINATED_SIGNED_SCHEMA;
		let page_id = 11;
		let payload = generate_payload_bytes::<PaginatedPageSize>(None);

		// assert
		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				owner_msa_id,
				schema_id,
				page_id,
				NONEXISTENT_PAGE_HASH,
				payload.into(),
			),
			Error::<Test>::UnsupportedOperationForSchema
		);
	});
}

#[test]
fn upsert_page_on_signature_schema_succeeds_for_owner() {
	new_test_ext().execute_with(|| {
		// setup
		let (msa_id, caller_keys) = get_signature_account();
		let caller_1: AccountId32 = caller_keys.public().into();
		let schema_id = PAGINATED_SIGNED_SCHEMA;
		let page_id = 11;
		let payload = generate_payload_bytes::<PaginatedPageSize>(None);

		// assert
		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			page_id,
			NONEXISTENT_PAGE_HASH,
			payload.into(),
		));
	});
}

#[test]
fn upsert_page_with_signature_v2_having_page_id_out_of_bounds_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = (<Test as Config>::MaxPaginatedPageId::get() + 1).into();
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayloadV2 {
			payload,
			target_hash: PageHash::default(),
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::PageIdExceedsMaxAllowed
		)
	})
}

#[test]
fn upsert_page_with_signature_v2_having_expired_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let block_number = 10;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayloadV2 {
			payload,
			target_hash: PageHash::default(),
			expiration: block_number,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		System::set_block_number(block_number);
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::ProofHasExpired
		)
	})
}

#[test]
fn upsert_page_with_signature_v2_having_out_of_window_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let mortality_window: u32 = <Test as Config>::MortalityWindowSize::get();
		let payload = PaginatedUpsertSignaturePayloadV2 {
			payload,
			target_hash: PageHash::default(),
			expiration: (mortality_window + 1).into(),
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::ProofNotYetValid
		)
	})
}

#[test]
fn upsert_page_with_signature_v2_having_wrong_signature_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_, pair) = get_signature_account();
		let delegator_key = pair.public();
		let (signature_key, _) = sr25519::Pair::generate();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayloadV2 {
			payload,
			target_hash: PageHash::default(),
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = signature_key.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidSignature
		)
	})
}

#[test]
fn upsert_page_with_signature_v2_having_non_existing_msa_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let pair = get_invalid_msa_signature_account(); // hardcoded key that returns None Msa
		let delegator_key = pair.public();
		let schema_id = UNDELEGATED_PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayloadV2 {
			payload,
			target_hash: PageHash::default(),
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	})
}

#[test]
fn upsert_page_with_signature_v2_having_invalid_schema_id_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = INVALID_SCHEMA_ID;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayloadV2 {
			payload,
			target_hash: PageHash::default(),
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidSchemaId
		)
	})
}

#[test]
fn upsert_page_with_signature_v2_having_invalid_schema_location_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayloadV2 {
			payload,
			target_hash: PageHash::default(),
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::SchemaPayloadLocationMismatch
		)
	})
}

#[test]
fn upsert_page_with_signature_v2_having_invalid_hash_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let payload = PaginatedUpsertSignaturePayloadV2 {
			payload,
			target_hash: PageHash::default() + 1, // any non default hash value
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::StalePageState
		)
	})
}

#[test]
fn upsert_page_with_signature_v2_having_valid_inputs_should_work() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let payload = generate_payload_bytes::<PaginatedPageSize>(Some(100));
		let page: PaginatedPage<Test> = payload.clone().into();
		let payload = PaginatedUpsertSignaturePayloadV2 {
			payload,
			target_hash: PageHash::default(),
			expiration: 10,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_ok!(StatefulStoragePallet::upsert_page_with_signature_v2(
			RuntimeOrigin::signed(caller_1),
			delegator_key.into(),
			owner_signature,
			payload
		));

		// assert
		let keys = (schema_id, page_id);
		let new_page: PaginatedPage<Test> = <StatefulChildTree>::try_read(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
		)
		.expect("new page could not be read")
		.expect("new page is empty");
		assert_eq!(page.data, new_page.data, "new page contents incorrect");
		assert_eq!(new_page.nonce, 1, "new page nonce incorrect");
		System::assert_last_event(
			StatefulEvent::PaginatedPageUpdated {
				msa_id,
				schema_id,
				page_id,
				prev_content_hash: PageHash::default(),
				curr_content_hash: new_page.get_hash(),
			}
			.into(),
		);
	})
}
