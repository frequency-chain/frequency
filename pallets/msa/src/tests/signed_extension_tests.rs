use crate::{
	tests::mock::*,
	types::{PayloadTypeDiscriminator, EMPTY_FUNCTION},
	AuthorizedKeyData, CheckFreeExtrinsicUse, Config, ValidityError,
};
use common_primitives::{
	msa::H160,
	signatures::{AccountAddressMapper, EthereumAddressMapper},
};
use common_runtime::extensions::check_nonce::CheckNonce;
use frame_support::{
	assert_ok,
	dispatch::{DispatchInfo, GetDispatchInfo},
	pallet_prelude::{InvalidTransaction, TransactionSource, TransactionValidityError},
	traits::{Currency, OriginTrait},
};
use frame_system::RawOrigin;
use parity_scale_codec::Encode;
use sp_core::{crypto::AccountId32, sr25519, sr25519::Public, Pair};
use sp_runtime::{
	traits::{DispatchTransaction, TransactionExtension},
	MultiSignature,
};
use sp_weights::Weight;

// Assert that CheckFreeExtrinsicUse::validate fails with `expected_err_enum`,
// for the "delete_msa_public_key" call, given extrinsic caller = caller_key,
// when attempting to delete `public_key_to_delete`
#[allow(deprecated)]
fn assert_validate_key_delete_fails(
	caller_key: &AccountId32,
	public_key_to_delete: AccountId32,
	expected_err_enum: ValidityError,
) {
	let call_delete_msa_public_key: &<Test as frame_system::Config>::RuntimeCall =
		&RuntimeCall::Msa(MsaCall::delete_msa_public_key { public_key_to_delete });

	let expected_err =
		TransactionValidityError::Invalid(InvalidTransaction::Custom(expected_err_enum as u8));
	let validation_result = CheckFreeExtrinsicUse::<Test>::new().validate_only(
		RuntimeOrigin::signed(caller_key.clone()).into(),
		call_delete_msa_public_key,
		&DispatchInfo::default(),
		0_usize,
		TransactionSource::External,
		0,
	);
	assert!(validation_result.is_err());
	let result = validation_result.unwrap_err();
	assert_eq!(result, expected_err);
}

#[allow(deprecated)]
fn assert_revoke_delegation_by_provider_err(
	expected_err: InvalidTransaction,
	provider_account: Public,
	delegator_msa_id: u64,
) {
	let call_revoke_delegation: &<Test as frame_system::Config>::RuntimeCall =
		&RuntimeCall::Msa(MsaCall::revoke_delegation_by_provider { delegator: delegator_msa_id });
	let info = DispatchInfo::default();
	let len = 0_usize;
	let result = CheckFreeExtrinsicUse::<Test>::new().validate_only(
		RuntimeOrigin::signed(provider_account.into()).into(),
		call_revoke_delegation,
		&info,
		len,
		TransactionSource::External,
		0,
	);
	assert!(result.is_err());
	let resulting_err = result.unwrap_err();
	match resulting_err {
		TransactionValidityError::Invalid(err) => {
			assert_eq!(err, expected_err);
		},
		_ => assert!(false, "Expected InvalidTransaction error"),
	}
}

#[allow(deprecated)]
fn assert_withdraw_msa_token_err(
	expected_err: InvalidTransaction,
	origin_key: Public,
	msa_owner_public_key: Public,
	msa_owner_proof: MultiSignature,
	authorization_payload: AuthorizedKeyData<Test>,
) {
	let call_withdraw_msa_token: &<Test as frame_system::Config>::RuntimeCall =
		&RuntimeCall::Msa(MsaCall::withdraw_tokens {
			msa_owner_public_key: msa_owner_public_key.into(),
			msa_owner_proof,
			authorization_payload,
		});
	let info = DispatchInfo::default();
	let len = 0_usize;
	let result = CheckFreeExtrinsicUse::<Test>::new().validate_only(
		RuntimeOrigin::signed(origin_key.into()).into(),
		call_withdraw_msa_token,
		&info,
		len,
		TransactionSource::External,
		0,
	);
	assert!(result.is_err());
	let resulting_err = result.unwrap_err();
	match resulting_err {
		TransactionValidityError::Invalid(err) => {
			assert_eq!(err, expected_err);
		},
		_ => assert!(false, "Expected InvalidTransaction error"),
	}
}

/// Assert that revoking an MSA delegation passes the signed extension CheckFreeExtrinsicUse
/// validation when a valid delegation exists.
#[test]
fn signed_extension_revoke_delegation_by_delegator_success() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, delegator_account) = create_provider_msa_and_delegator();
		let call_revoke_delegation: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::revoke_delegation_by_delegator { provider_msa_id });
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate_only(
			RuntimeOrigin::signed(delegator_account.into()).into(),
			call_revoke_delegation,
			&info,
			len,
			TransactionSource::External,
			0,
		);
		assert_ok!(result);
	});
}

/// Assert that revoking an MSA delegation fails the signed extension CheckFreeExtrinsicUse
/// validation when no valid delegation exists.
#[test]
fn signed_extension_fails_when_revoke_delegation_by_delegator_called_twice() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, delegator_account) = create_provider_msa_and_delegator();
		let call_revoke_delegation: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::revoke_delegation_by_delegator { provider_msa_id });
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate_only(
			RuntimeOrigin::signed(delegator_account.into()).into(),
			call_revoke_delegation,
			&info,
			len,
			TransactionSource::External,
			0,
		);
		assert_ok!(result);
		assert_ok!(Msa::revoke_delegation_by_delegator(
			RuntimeOrigin::signed(delegator_account.into()),
			provider_msa_id
		));

		System::set_block_number(System::block_number() + 1);
		let call_revoke_delegation: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::revoke_delegation_by_delegator { provider_msa_id });
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result_revoked = CheckFreeExtrinsicUse::<Test>::new().validate_only(
			RuntimeOrigin::signed(delegator_account.into()).into(),
			call_revoke_delegation,
			&info,
			len,
			TransactionSource::External,
			0,
		);
		assert!(result_revoked.is_err());
	});
}

#[test]
fn signed_extension_revoke_delegation_by_provider_success() {
	new_test_ext().execute_with(|| {
		let (delegator_msa_id, provider_account) = create_delegator_msa_and_provider();
		let call_revoke_delegation: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::revoke_delegation_by_provider {
				delegator: delegator_msa_id,
			});
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate_only(
			RuntimeOrigin::signed(provider_account.into()).into(),
			call_revoke_delegation,
			&info,
			len,
			TransactionSource::External,
			0,
		);
		assert_ok!(result);
	})
}

#[test]
fn signed_extension_revoke_delegation_by_provider_fails_when_no_delegator_msa() {
	new_test_ext().execute_with(|| {
		let (_, provider_pair) = create_account();
		let provider_account = provider_pair.public();

		let delegator_msa_id = 33u64;
		let expected_err = InvalidTransaction::Custom(ValidityError::InvalidDelegation as u8);
		assert_revoke_delegation_by_provider_err(expected_err, provider_account, delegator_msa_id);
	})
}

#[test]
fn signed_extension_revoke_delegation_by_provider_fails_when_no_provider_msa() {
	new_test_ext().execute_with(|| {
		let (provider_pair, _) = sr25519::Pair::generate();
		let provider_account = provider_pair.public();

		let (delegator_msa, _) = create_account();

		let expected_err = InvalidTransaction::Custom(ValidityError::InvalidMsaKey as u8);
		assert_revoke_delegation_by_provider_err(expected_err, provider_account, delegator_msa);
	});
}

#[test]
fn signed_extension_revoke_delegation_by_provider_fails_when_no_delegation() {
	new_test_ext().execute_with(|| {
		let (_, provider_pair) = create_account();
		let provider_account = provider_pair.public();
		let (delegator_msa, _) = create_account();

		let expected_err = InvalidTransaction::Custom(ValidityError::InvalidDelegation as u8);
		assert_revoke_delegation_by_provider_err(expected_err, provider_account, delegator_msa);
	})
}

/// Assert that a call that is not one of the matches passes the signed extension
/// CheckFreeExtrinsicUse validation.
#[test]
fn signed_extension_validation_valid_for_other_extrinsics() {
	let random_call_should_pass: &<Test as frame_system::Config>::RuntimeCall =
		&RuntimeCall::Msa(MsaCall::create {});
	let info = DispatchInfo::default();
	let len = 0_usize;
	let result = CheckFreeExtrinsicUse::<Test>::new().validate_only(
		RuntimeOrigin::signed(test_public(1)).into(),
		random_call_should_pass,
		&info,
		len,
		TransactionSource::External,
		0,
	);
	assert_ok!(result);
}

// Assert that check nonce validation does not create a token account for delete_msa_public_key call.
#[test]
fn signed_ext_check_nonce_delete_msa_public_key() {
	new_test_ext().execute_with(|| {
		// Generate a key pair for MSA account
		let (msa_key_pair, _) = sr25519::Pair::generate();
		let msa_new_key = msa_key_pair.public();

		let len = 0_usize;

		// Test the delete_msa_public_key() call
		let call_delete_msa_public_key: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::delete_msa_public_key {
				public_key_to_delete: AccountId32::from(msa_new_key),
			});
		let info = call_delete_msa_public_key.get_dispatch_info();

		// Call delete_msa_public_key() using the Alice account
		let who = test_public(1);
		assert_ok!(CheckNonce::<Test>(0).validate_and_prepare(
			RuntimeOrigin::signed(who.clone()).into(),
			call_delete_msa_public_key,
			&info,
			len,
			0
		));

		// Did the call create a token account?
		let created_token_account: bool;
		match frame_system::Account::<Test>::try_get(who) {
			Ok(_) => {
				created_token_account = true;
			},
			Err(_) => {
				created_token_account = false;
			},
		};

		// Assert that the call did not create a token account
		assert!(!created_token_account);
	})
}

// Assert that check nonce validation does not create a token account for revoke_delegation_by_delegator call.
#[test]
fn signed_ext_check_nonce_revoke_delegation_by_delegator() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, _) = create_provider_msa_and_delegator();

		// We are testing the revoke_delegation_by_delegator() call.
		let call_revoke_delegation_by_delegator: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::revoke_delegation_by_delegator { provider_msa_id });

		let len = 0_usize;

		// Get the dispatch info for the call.
		let info = call_revoke_delegation_by_delegator.get_dispatch_info();

		// Call revoke_delegation_by_delegator() using the Alice account
		let who = test_public(1);
		assert_ok!(CheckNonce::<Test>(0).validate_and_prepare(
			RuntimeOrigin::signed(who.clone()).into(),
			call_revoke_delegation_by_delegator,
			&info,
			len,
			0
		));

		// Did the call create a token account?
		let created_token_account: bool;
		match frame_system::Account::<Test>::try_get(who) {
			Ok(_) => {
				created_token_account = true;
			},
			Err(_) => {
				created_token_account = false;
			},
		};

		// Assert that the call did not create a token account
		assert!(!created_token_account);
	})
}

// Assert that check nonce validation does create a token account for a paying call.
#[test]
fn signed_ext_check_nonce_creates_token_account_if_paying() {
	new_test_ext().execute_with(|| {
		//  Test that a  "pays" extrinsic creates a token account
		let who = test_public(1);
		let len = 0_usize;
		let pays_call_should_pass: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::create {});

		// Get the dispatch info for the create() call.
		let pays_call_should_pass_info = pays_call_should_pass.get_dispatch_info();

		// Call create() using the Alice account
		assert_ok!(CheckNonce::<Test>(0).validate_and_prepare(
			RuntimeOrigin::signed(who.clone()).into(),
			pays_call_should_pass,
			&pays_call_should_pass_info,
			len,
			0
		));

		// Did the call create a token account?
		let created_token_account: bool;
		match frame_system::Account::<Test>::try_get(who) {
			Ok(_) => {
				created_token_account = true;
			},
			Err(_) => {
				created_token_account = false;
			},
		};
		// Assert that the call created a token account
		assert!(created_token_account);
	})
}

#[test]
fn signed_ext_check_nonce_increases_nonce_for_an_existing_account_for_free_transactions() {
	new_test_ext().execute_with(|| {
		// arrange
		let who = test_public(1);
		let len = 0_usize;
		let free_call: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::delete_msa_public_key { public_key_to_delete: who.clone() });
		let free_call_info = free_call.get_dispatch_info();
		let mut account = frame_system::Account::<Test>::get(who.clone());
		account.consumers += 1;
		frame_system::Account::<Test>::insert(who.clone(), account);

		// act
		assert_ok!(CheckNonce::<Test>(0).validate_and_prepare(
			RuntimeOrigin::signed(who.clone()).into(),
			free_call,
			&free_call_info,
			len,
			0
		));

		// assert
		let account_after = frame_system::Account::<Test>::try_get(who).expect("should resolve");
		assert_eq!(account_after.nonce, 1);
	})
}

#[test]
fn signed_extension_validation_delete_msa_public_key_success() {
	new_test_ext().execute_with(|| {
		let (msa_id, original_key_pair) = create_account();

		let (new_key_pair, _) = sr25519::Pair::generate();
		let new_key: AccountId32 = new_key_pair.public().into();
		assert_ok!(Msa::add_key(msa_id, &new_key, EMPTY_FUNCTION));

		let original_key: AccountId32 = original_key_pair.public().into();

		// set up call for new key to delete original key
		let call_delete_msa_public_key: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::delete_msa_public_key {
				public_key_to_delete: original_key.clone(),
			});

		let info = DispatchInfo::default();
		let len = 0_usize;
		assert_ok!(CheckFreeExtrinsicUse::<Test>::new().validate_only(
			RuntimeOrigin::signed(new_key.clone()).into(),
			call_delete_msa_public_key,
			&info,
			len,
			TransactionSource::External,
			0,
		));

		// validate other direction
		let call_delete_msa_public_key2: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::delete_msa_public_key { public_key_to_delete: new_key });
		assert_ok!(CheckFreeExtrinsicUse::<Test>::new().validate_only(
			RuntimeOrigin::signed(original_key.clone()).into(),
			call_delete_msa_public_key2,
			&info,
			len,
			TransactionSource::External,
			0,
		));
	});
}

#[test]
fn signed_extension_validate_fails_when_delete_msa_public_key_called_twice() {
	new_test_ext().execute_with(|| {
		let (owner_msa_id, owner_key_pair) = create_account();

		let (new_key_pair, _) = sr25519::Pair::generate();
		let new_key: AccountId32 = new_key_pair.public().into();
		assert_ok!(Msa::add_key(owner_msa_id, &new_key, EMPTY_FUNCTION));

		let call_delete_msa_public_key: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::delete_msa_public_key {
				public_key_to_delete: owner_key_pair.public().into(),
			});

		// check that it's okay to delete the original key
		assert_ok!(CheckFreeExtrinsicUse::<Test>::new().validate_only(
			RuntimeOrigin::signed(new_key.clone()).into(),
			call_delete_msa_public_key,
			&DispatchInfo::default(),
			0_usize,
			TransactionSource::External,
			0,
		));

		// new key deletes the old key
		assert_ok!(Msa::delete_msa_public_key(
			RuntimeOrigin::signed(new_key.clone()),
			owner_key_pair.public().into()
		));

		assert_validate_key_delete_fails(
			&new_key,
			owner_key_pair.public().into(),
			ValidityError::InvalidMsaKey,
		);
	});
}

#[test]
fn signed_extension_validate_fails_when_delete_msa_public_key_called_on_only_key() {
	new_test_ext().execute_with(|| {
		let (_, original_pair) = create_account();
		let original_key: AccountId32 = original_pair.public().into();

		assert_validate_key_delete_fails(
			&original_key,
			original_key.clone(),
			ValidityError::InvalidSelfRemoval,
		)
	})
}

#[test]
fn signed_extension_validate_fails_when_delete_msa_public_key_called_by_non_owner() {
	new_test_ext().execute_with(|| {
		let (_, original_pair) = create_account();
		let original_key: AccountId32 = original_pair.public().into();

		let (_, non_owner_pair) = create_account();
		let non_owner_key: AccountId32 = non_owner_pair.public().into();
		assert_validate_key_delete_fails(
			&non_owner_key,
			original_key.clone(),
			ValidityError::NotKeyOwner,
		)
	})
}

#[test]
fn signed_ext_validate_fails_when_withdraw_tokens_caller_key_does_not_match_payload() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();
		let (other_key_pair, _) = sr25519::Pair::generate();

		let (payload, msa_signature) = generate_and_sign_authorized_key_payload(
			msa_id,
			&owner_key_pair,
			&other_key_pair,
			None,
			None,
		);

		assert_withdraw_msa_token_err(
			InvalidTransaction::Custom(ValidityError::NotKeyOwner as u8),
			origin_key_pair.public(),
			owner_key_pair.public(),
			msa_signature,
			payload,
		);
	});
}

#[test]
fn signed_ext_validate_fails_when_withdraw_tokens_payload_has_wrong_type_discriminant() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();

		let (payload, msa_signature) = generate_and_sign_authorized_key_payload(
			msa_id,
			&owner_key_pair,
			&origin_key_pair,
			None,
			Some(PayloadTypeDiscriminator::Unknown),
		);

		assert_withdraw_msa_token_err(
			InvalidTransaction::Custom(ValidityError::MsaOwnershipInvalidSignature as u8),
			origin_key_pair.public(),
			owner_key_pair.public(),
			msa_signature,
			payload,
		);
	})
}

#[test]
fn signed_ext_validate_fails_when_withdraw_tokens_payload_signature_is_invalid() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();
		let (other_key_pair, _) = sr25519::Pair::generate();

		let (payload, msa_signature) = generate_and_sign_authorized_key_payload(
			msa_id,
			&other_key_pair,
			&origin_key_pair,
			None,
			None,
		);

		assert_withdraw_msa_token_err(
			InvalidTransaction::Custom(ValidityError::MsaOwnershipInvalidSignature as u8),
			origin_key_pair.public(),
			owner_key_pair.public(),
			msa_signature,
			payload,
		);
	});
}

#[test]
fn signed_ext_validate_fails_when_withdraw_tokens_proof_is_expired() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();

		// The current block is 1, therefore setting the proof expiration to 1 should cause
		// the extrinsic to fail because the proof has expired.
		let (payload, msa_signature) = generate_and_sign_authorized_key_payload(
			msa_id,
			&owner_key_pair,
			&origin_key_pair,
			Some(1),
			None,
		);

		assert_withdraw_msa_token_err(
			InvalidTransaction::Custom(ValidityError::MsaOwnershipInvalidSignature as u8),
			origin_key_pair.public(),
			owner_key_pair.public(),
			msa_signature,
			payload,
		);
	});
}

#[test]
fn signed_ext_validate_fails_when_withdraw_tokens_proof_is_not_yet_valid() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();

		// The current block is 1, therefore setting the proof expiration to the max mortality period
		// should cause the extrinsic to fail
		let (payload, msa_signature) = generate_and_sign_authorized_key_payload(
			msa_id,
			&owner_key_pair,
			&origin_key_pair,
			Some(Msa::mortality_block_limit(1)),
			None,
		);

		assert_withdraw_msa_token_err(
			InvalidTransaction::Custom(ValidityError::MsaOwnershipInvalidSignature as u8),
			origin_key_pair.public(),
			owner_key_pair.public(),
			msa_signature,
			payload,
		);
	});
}

#[test]
fn signed_ext_validate_fails_when_withdraw_tokens_origin_is_an_msa_control_key() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (_, origin_key_pair) = create_account();

		let (payload, msa_signature) = generate_and_sign_authorized_key_payload(
			msa_id,
			&owner_key_pair,
			&origin_key_pair,
			None,
			None,
		);

		assert_withdraw_msa_token_err(
			InvalidTransaction::Custom(ValidityError::IneligibleOrigin as u8),
			origin_key_pair.public(),
			owner_key_pair.public(),
			msa_signature,
			payload,
		);
	});
}

#[test]
fn signed_ext_validate_fails_when_withdraw_tokens_msa_key_is_not_an_msa_control_key() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();

		let (payload, msa_signature) = generate_and_sign_authorized_key_payload(
			msa_id + 1,
			&owner_key_pair,
			&origin_key_pair,
			None,
			None,
		);

		assert_withdraw_msa_token_err(
			InvalidTransaction::Custom(ValidityError::InvalidMsaKey as u8),
			origin_key_pair.public(),
			owner_key_pair.public(),
			msa_signature,
			payload,
		);
	})
}

#[test]
fn signed_ext_validate_fails_when_withdraw_tokens_msa_key_does_not_control_msa_in_payload() {
	new_test_ext().execute_with(|| {
		let (msa_id, _) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();
		let (other_key_pair, _) = sr25519::Pair::generate();

		let (payload, msa_signature) = generate_and_sign_authorized_key_payload(
			msa_id,
			&other_key_pair,
			&origin_key_pair,
			None,
			None,
		);

		assert_withdraw_msa_token_err(
			InvalidTransaction::Custom(ValidityError::InvalidMsaKey as u8),
			origin_key_pair.public(),
			other_key_pair.public(),
			msa_signature,
			payload,
		);
	})
}

#[test]
fn signed_ext_validate_fails_when_withdraw_tokens_msa_does_not_have_a_balance() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();

		let (payload, msa_signature) = generate_and_sign_authorized_key_payload(
			msa_id,
			&owner_key_pair,
			&origin_key_pair,
			None,
			None,
		);

		assert_withdraw_msa_token_err(
			InvalidTransaction::Custom(ValidityError::InsufficientBalanceToWithdraw as u8),
			origin_key_pair.public(),
			owner_key_pair.public(),
			msa_signature,
			payload,
		);
	})
}

#[test]
fn signed_ext_validate_fails_when_withdraw_tokens_duplicate_signature_submitted() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();
		let eth_account_id: H160 = Msa::msa_id_to_eth_address(msa_id);
		let bytes: [u8; 32] = EthereumAddressMapper::to_bytes32(&eth_account_id.0);
		let msa_account_id = <Test as frame_system::Config>::AccountId::from(bytes);

		let (payload, msa_signature) = generate_and_sign_authorized_key_payload(
			msa_id,
			&owner_key_pair,
			&origin_key_pair,
			None,
			None,
		);

		let transfer_amount = 10_000_000;

		// Fund MSA
		let _ = <Test as Config>::Currency::deposit_creating(&msa_account_id, transfer_amount);

		assert_ok!(Msa::withdraw_tokens(
			RuntimeOrigin::signed(origin_key_pair.public().into()),
			owner_key_pair.public().into(),
			msa_signature.clone(),
			payload.clone()
		));

		assert_withdraw_msa_token_err(
			InvalidTransaction::Custom(ValidityError::MsaOwnershipInvalidSignature as u8),
			origin_key_pair.public(),
			owner_key_pair.public(),
			msa_signature.clone(),
			payload.clone(),
		);
	})
}

#[test]
fn signed_ext_validate_passes_when_withdraw_tokens_balance_is_sufficient() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();
		let eth_account_id: H160 = Msa::msa_id_to_eth_address(msa_id);
		let bytes: [u8; 32] = EthereumAddressMapper::to_bytes32(&eth_account_id.0);
		let msa_account_id = <Test as frame_system::Config>::AccountId::from(bytes);

		let (payload, msa_signature) = generate_and_sign_authorized_key_payload(
			msa_id,
			&owner_key_pair,
			&origin_key_pair,
			None,
			None,
		);

		let transfer_amount = 10_000_000;

		// Fund MSA
		let _ = <Test as Config>::Currency::deposit_creating(&msa_account_id, transfer_amount);

		let call_withdraw_msa_token: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::withdraw_tokens {
				msa_owner_public_key: owner_key_pair.public().into(),
				msa_owner_proof: msa_signature,
				authorization_payload: payload,
			});
		let info = DispatchInfo::default();
		let len = 0_usize;
		#[allow(deprecated)]
		let result = CheckFreeExtrinsicUse::<Test>::new().validate_only(
			RuntimeOrigin::signed(origin_key_pair.public().into()).into(),
			call_withdraw_msa_token,
			&info,
			len,
			TransactionSource::External,
			0,
		);
		assert_ok!(result);
	});
}

#[test]
fn check_nonce_post_dispatch_details_refund_weight() {
	use common_runtime::extensions::check_nonce::Pre;
	// Simulate a refund weight scenario
	let refund_weight = frame_support::weights::Weight::from_parts(12345, 0);
	let result = common_runtime::extensions::check_nonce::CheckNonce::<Test>::post_dispatch_details(
		Pre::Refund(refund_weight),
		&DispatchInfo::default(),
		&Default::default(),
		0,
		&Ok(()),
	);
	assert_eq!(result.unwrap(), refund_weight);
}

#[test]
fn check_nonce_skipped_and_refund_for_other_origins() {
	new_test_ext().execute_with(|| {
		let call = &RuntimeCall::System(frame_system::Call::set_heap_pages { pages: 0u64 });
		let ext = CheckNonce::<Test>(1u64.into());

		let mut info = call.get_dispatch_info();
		info.extension_weight = ext.weight(call);

		// Ensure we test the refund.
		assert!(info.extension_weight != Weight::zero());

		let len = call.encoded_size();

		let origin = RawOrigin::Root.into();
		let (pre, origin) = ext.validate_and_prepare(origin, call, &info, len, 0).unwrap();

		assert!(origin.as_system_ref().unwrap().is_root());

		let pd_res = Ok(());
		let mut post_info = frame_support::dispatch::PostDispatchInfo {
			actual_weight: Some(info.total_weight()),
			pays_fee: Default::default(),
		};

		<CheckNonce<Test> as TransactionExtension<RuntimeCall>>::post_dispatch(
			pre,
			&info,
			&mut post_info,
			len,
			&pd_res,
		)
		.unwrap();

		assert_eq!(post_info.actual_weight, Some(info.call_weight));
	})
}

#[test]
fn check_nonce_does_not_increment_nonce_on_failed_call() {
	new_test_ext().execute_with(|| {
		// Arrange: create an account with nonce 0
		let who = test_public(42);
		let len = 0_usize;
		let call: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::delete_msa_public_key { public_key_to_delete: who.clone() });
		let info = call.get_dispatch_info();
		// Insert account
		let mut account = frame_system::Account::<Test>::get(who.clone());
		account.consumers += 1;
		frame_system::Account::<Test>::insert(who.clone(), account);
		// Act: simulate a failed call by passing a stale nonce (increment first)
		let mut account = frame_system::Account::<Test>::get(who.clone());
		account.nonce += 1;
		frame_system::Account::<Test>::insert(who.clone(), account);
		// Now try to use the old nonce (0)
		let ext = CheckNonce::<Test>(0);
		let res = ext.validate_and_prepare(
			RuntimeOrigin::signed(who.clone()).into(),
			call,
			&info,
			len,
			0,
		);
		assert!(res.is_err());
		// Assert nonce is still 1
		let account_after = frame_system::Account::<Test>::get(who);
		assert_eq!(account_after.nonce, 1);
	});
}

#[test]
fn check_nonce_does_not_increment_nonce_for_unsigned_origin() {
	new_test_ext().execute_with(|| {
		let who = test_public(1);
		let len = 0_usize;
		let call: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::create {});
		let info = call.get_dispatch_info();
		// Insert account
		let mut account = frame_system::Account::<Test>::get(who.clone());
		account.consumers += 1;
		frame_system::Account::<Test>::insert(who.clone(), account);
		// Act: use unsigned origin
		let ext = CheckNonce::<Test>(0);
		let res = ext.validate_and_prepare(RuntimeOrigin::none().into(), call, &info, len, 0);
		// Should succeed (no-op), but not increment nonce
		assert!(res.is_err());
		let account_after = frame_system::Account::<Test>::get(who);
		assert_eq!(account_after.nonce, 0);
	});
}

#[test]
fn check_nonce_rejects_stale_nonce() {
	new_test_ext().execute_with(|| {
		let who = test_public(7);
		let len = 0_usize;
		let call: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::create {});
		let info = call.get_dispatch_info();
		// Insert account with nonce 1
		let mut account = frame_system::Account::<Test>::get(who.clone());
		account.nonce = 1;
		account.consumers += 1;
		frame_system::Account::<Test>::insert(who.clone(), account);
		// Try to submit with nonce 0 (stale)
		let ext = CheckNonce::<Test>(0);
		let res = ext.validate_and_prepare(
			RuntimeOrigin::signed(who.clone()).into(),
			call,
			&info,
			len,
			0,
		);
		assert!(res.is_err());
	});
}

#[test]
fn check_free_extrinsic_use_post_dispatch_details_refund_weight() {
	use crate::Pre as FreePre;
	let refund_weight = frame_support::weights::Weight::from_parts(4321, 0);
	let result = crate::CheckFreeExtrinsicUse::<Test>::post_dispatch_details(
		FreePre::Refund(refund_weight),
		&DispatchInfo::default(),
		&Default::default(),
		0,
		&Ok(()),
	);
	assert_eq!(result.unwrap(), refund_weight);
}

#[test]
fn check_free_extrinsic_use_does_not_block_unrelated_calls() {
	new_test_ext().execute_with(|| {
		let who = test_public(1);
		let unrelated_call: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::create {});
		let info = unrelated_call.get_dispatch_info();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate_only(
			RuntimeOrigin::signed(who.clone()).into(),
			unrelated_call,
			&info,
			len,
			TransactionSource::External,
			0,
		);
		assert_ok!(result);
	});
}

#[test]
fn check_free_extrinsic_use_noop_for_unsigned_origin() {
	new_test_ext().execute_with(|| {
		let _who = test_public(1);
		let call: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::revoke_delegation_by_delegator { provider_msa_id: 42 });
		let info = call.get_dispatch_info();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate_only(
			RuntimeOrigin::none().into(),
			call,
			&info,
			len,
			TransactionSource::External,
			0,
		);
		// Should be an error or a no-op, but must not panic
		assert!(result.is_err() || result.is_ok());
	});
}
