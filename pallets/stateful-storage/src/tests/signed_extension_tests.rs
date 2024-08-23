use crate::{
	signed_extension::StatefulStorageSignedExtension,
	test_common::{constants::*, test_utility::*},
	tests::mock::*,
	types::*,
	Call,
};
use common_primitives::stateful_storage::PageHash;
use frame_support::{assert_ok, dispatch::DispatchInfo, BoundedVec};
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use sp_core::Pair;
use sp_runtime::{
	traits::SignedExtension,
	transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError},
	MultiSignature,
};

#[test]
fn signed_extension_validation_with_stale_hash_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let itemized_schema_id = ITEMIZED_SCHEMA;
		let paginated_schema_id = PAGINATED_SCHEMA;
		let payload = vec![1; 5];
		let prev_content_hash: PageHash = 0;
		let actions = vec![ItemAction::Add { data: payload.clone().try_into().unwrap() }];
		let info = DispatchInfo::default();
		let len = 0_usize;
		let sig = [1; 64];
		let signature = MultiSignature::Sr25519(sp_core::sr25519::Signature::from_raw(sig));
		let (msa_id, delegator_1) = get_signature_account();
		let delegator_key = delegator_1.public();
		let wrong_hash = 10;
		assert_ok!(StatefulStoragePallet::apply_item_actions(
			RuntimeOrigin::signed(caller_1.clone()),
			msa_id,
			itemized_schema_id,
			prev_content_hash,
			BoundedVec::try_from(actions.clone()).unwrap(),
		));
		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1.clone()),
			msa_id,
			paginated_schema_id,
			0,
			prev_content_hash,
			BoundedVec::try_from(payload.clone()).unwrap(),
		));
		let calls: Vec<<Test as frame_system::Config>::RuntimeCall> = vec![
			RuntimeCall::StatefulStoragePallet(Call::apply_item_actions {
				actions: BoundedVec::try_from(actions.clone()).unwrap(),
				target_hash: wrong_hash,
				state_owner_msa_id: msa_id,
				schema_id: itemized_schema_id,
			}),
			RuntimeCall::StatefulStoragePallet(Call::upsert_page {
				payload: BoundedVec::try_from(payload.clone()).unwrap(),
				target_hash: wrong_hash,
				state_owner_msa_id: msa_id,
				schema_id: paginated_schema_id,
				page_id: 0,
			}),
			RuntimeCall::StatefulStoragePallet(Call::delete_page {
				target_hash: wrong_hash,
				state_owner_msa_id: msa_id,
				schema_id: paginated_schema_id,
				page_id: 0,
			}),
			RuntimeCall::StatefulStoragePallet(Call::apply_item_actions_with_signature {
				payload: ItemizedSignaturePayload {
					actions: BoundedVec::try_from(actions.clone()).unwrap(),
					target_hash: wrong_hash,
					msa_id,
					schema_id: itemized_schema_id,
					expiration: 0,
				},
				delegator_key: delegator_key.clone().into(),
				proof: signature.clone(),
			}),
			RuntimeCall::StatefulStoragePallet(Call::apply_item_actions_with_signature_v2 {
				payload: ItemizedSignaturePayloadV2 {
					actions: BoundedVec::try_from(actions.clone()).unwrap(),
					target_hash: wrong_hash,
					schema_id: itemized_schema_id,
					expiration: 0,
				},
				delegator_key: delegator_key.clone().into(),
				proof: signature.clone(),
			}),
			RuntimeCall::StatefulStoragePallet(Call::upsert_page_with_signature {
				payload: PaginatedUpsertSignaturePayload {
					payload: BoundedVec::try_from(payload.clone()).unwrap(),
					target_hash: wrong_hash,
					schema_id: paginated_schema_id,
					page_id: 0,
					expiration: 0,
					msa_id,
				},
				delegator_key: delegator_key.clone().into(),
				proof: signature.clone(),
			}),
			RuntimeCall::StatefulStoragePallet(Call::delete_page_with_signature {
				payload: PaginatedDeleteSignaturePayload {
					target_hash: wrong_hash,
					schema_id: paginated_schema_id,
					page_id: 0,
					expiration: 0,
					msa_id,
				},
				delegator_key: delegator_key.clone().into(),
				proof: signature.clone(),
			}),
			RuntimeCall::StatefulStoragePallet(Call::upsert_page_with_signature_v2 {
				payload: PaginatedUpsertSignaturePayloadV2 {
					payload: BoundedVec::try_from(payload.clone()).unwrap(),
					target_hash: wrong_hash,
					schema_id: paginated_schema_id,
					page_id: 0,
					expiration: 0,
				},
				delegator_key: delegator_key.clone().into(),
				proof: signature.clone(),
			}),
			RuntimeCall::StatefulStoragePallet(Call::delete_page_with_signature_v2 {
				payload: PaginatedDeleteSignaturePayloadV2 {
					target_hash: wrong_hash,
					schema_id: paginated_schema_id,
					page_id: 0,
					expiration: 0,
				},
				delegator_key: delegator_key.clone().into(),
				proof: signature.clone(),
			}),
		];

		for call in calls {
			// act
			let result = StatefulStorageSignedExtension::<Test>::new()
				.validate(&caller_1, &call, &info, len);

			// assert
			assert_eq!(
				result,
				TransactionValidity::Err(TransactionValidityError::Invalid(
					InvalidTransaction::Custom(9)
				))
			);
		}
	});
}

#[test]
fn signed_extension_validation_for_signature_payloads_without_msa_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = INVALID_MSA_ID;
		let caller_1 = test_public(msa_id);
		let schema_id = PAGINATED_SCHEMA;
		let payload = vec![1; 5];
		let sig = [1; 64];
		let signature = MultiSignature::Sr25519(sp_core::sr25519::Signature::from_raw(sig));

		let info = DispatchInfo::default();
		let len = 0_usize;
		let call: <Test as frame_system::Config>::RuntimeCall =
			RuntimeCall::StatefulStoragePallet(Call::upsert_page_with_signature_v2 {
				payload: PaginatedUpsertSignaturePayloadV2 {
					payload: BoundedVec::try_from(payload.clone()).unwrap(),
					target_hash: NONEXISTENT_PAGE_HASH,
					schema_id,
					page_id: 0,
					expiration: 0,
				},
				delegator_key: caller_1.clone().into(),
				proof: signature.clone(),
			});

		// act
		let result =
			StatefulStorageSignedExtension::<Test>::new().validate(&caller_1, &call, &info, len);

		// assert
		assert_eq!(
			result,
			TransactionValidity::Err(TransactionValidityError::Invalid(
				InvalidTransaction::Custom(255)
			))
		);
	});
}
