use crate::{
	tests::mock::*, types::EMPTY_FUNCTION, AuthorizedKeyData, CheckFreeExtrinsicUse, Config,
	ValidityError,
};
use common_primitives::{
	msa::H160,
	signatures::{AccountAddressMapper, EthereumAddressMapper},
};
use common_runtime::extensions::check_nonce::CheckNonce;
use frame_support::{
	assert_err, assert_ok,
	dispatch::{DispatchInfo, GetDispatchInfo},
	pallet_prelude::InvalidTransaction,
	traits::Currency,
};
use sp_core::{crypto::AccountId32, sr25519, sr25519::Public, Pair};
use sp_runtime::MultiSignature;
#[allow(deprecated)]
use sp_runtime::{traits::SignedExtension, transaction_validity::TransactionValidity};

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

	let expected_err: TransactionValidity =
		InvalidTransaction::Custom(expected_err_enum as u8).into();

	assert_eq!(
		CheckFreeExtrinsicUse::<Test>::new().validate(
			caller_key,
			call_delete_msa_public_key,
			&DispatchInfo::default(),
			0_usize,
		),
		expected_err
	);
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
	let result = CheckFreeExtrinsicUse::<Test>::new().validate(
		&provider_account.into(),
		call_revoke_delegation,
		&info,
		len,
	);
	assert_err!(result, expected_err);
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
	let result = CheckFreeExtrinsicUse::<Test>::new().validate(
		&origin_key.into(),
		call_withdraw_msa_token,
		&info,
		len,
	);
	assert_err!(result, expected_err);
}

/// Assert that revoking an MSA delegation passes the signed extension CheckFreeExtrinsicUse
/// validation when a valid delegation exists.
#[test]
#[allow(deprecated)]
fn signed_extension_revoke_delegation_by_delegator_success() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, delegator_account) = create_provider_msa_and_delegator();
		let call_revoke_delegation: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::revoke_delegation_by_delegator { provider_msa_id });
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate(
			&delegator_account.into(),
			call_revoke_delegation,
			&info,
			len,
		);
		assert_ok!(result);
	});
}

/// Assert that revoking an MSA delegation fails the signed extension CheckFreeExtrinsicUse
/// validation when no valid delegation exists.
#[test]
#[allow(deprecated)]
fn signed_extension_fails_when_revoke_delegation_by_delegator_called_twice() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, delegator_account) = create_provider_msa_and_delegator();
		let call_revoke_delegation: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::revoke_delegation_by_delegator { provider_msa_id });
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate(
			&delegator_account.into(),
			call_revoke_delegation,
			&info,
			len,
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
		let result_revoked = CheckFreeExtrinsicUse::<Test>::new().validate(
			&delegator_account.into(),
			call_revoke_delegation,
			&info,
			len,
		);
		assert!(result_revoked.is_err());
	});
}

#[test]
#[allow(deprecated)]
fn signed_extension_revoke_delegation_by_provider_success() {
	new_test_ext().execute_with(|| {
		let (delegator_msa_id, provider_account) = create_delegator_msa_and_provider();
		let call_revoke_delegation: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::revoke_delegation_by_provider {
				delegator: delegator_msa_id,
			});
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate(
			&provider_account.into(),
			call_revoke_delegation,
			&info,
			len,
		);
		assert_ok!(result);
	})
}

#[test]
#[allow(deprecated)]
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
#[allow(deprecated)]
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
#[allow(deprecated)]
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
#[allow(deprecated)]
fn signed_extension_validation_valid_for_other_extrinsics() {
	let random_call_should_pass: &<Test as frame_system::Config>::RuntimeCall =
		&RuntimeCall::Msa(MsaCall::create {});
	let info = DispatchInfo::default();
	let len = 0_usize;
	let result = CheckFreeExtrinsicUse::<Test>::new().validate(
		&test_public(1),
		random_call_should_pass,
		&info,
		len,
	);
	assert_ok!(result);
}

// Assert that check nonce validation does not create a token account for delete_msa_public_key call.
#[test]
#[allow(deprecated)]
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
		assert_ok!(CheckNonce::<Test>(0).pre_dispatch(
			&who,
			call_delete_msa_public_key,
			&info,
			len
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
#[allow(deprecated)]
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
		assert_ok!(CheckNonce::<Test>(0).pre_dispatch(
			&who,
			call_revoke_delegation_by_delegator,
			&info,
			len
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
#[allow(deprecated)]
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
		assert_ok!(CheckNonce::<Test>(0).pre_dispatch(
			&who,
			pays_call_should_pass,
			&pays_call_should_pass_info,
			len
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
#[allow(deprecated)]
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
		assert_ok!(CheckNonce::<Test>(0).pre_dispatch(
			&who.clone(),
			free_call,
			&free_call_info,
			len
		));

		// assert
		let account_after = frame_system::Account::<Test>::try_get(who).expect("should resolve");
		assert_eq!(account_after.nonce, 1);
	})
}

#[test]
#[allow(deprecated)]
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
		assert_ok!(CheckFreeExtrinsicUse::<Test>::new().validate(
			&new_key,
			call_delete_msa_public_key,
			&info,
			len,
		));

		// validate other direction
		let call_delete_msa_public_key2: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::delete_msa_public_key { public_key_to_delete: new_key });
		assert_ok!(CheckFreeExtrinsicUse::<Test>::new().validate(
			&original_key,
			call_delete_msa_public_key2,
			&info,
			len,
		));
	});
}

#[test]
#[allow(deprecated)]
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
		assert_ok!(CheckFreeExtrinsicUse::<Test>::new().validate(
			&new_key,
			call_delete_msa_public_key,
			&DispatchInfo::default(),
			0_usize,
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
#[allow(deprecated)]
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
#[allow(deprecated)]
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
fn signed_ext_validate_fails_when_withdraw_tokens_payload_payload_is_has_wrong_type_discriminant() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();

		const BAD_DISCRIMIMINANT: &[u8; 17] = b"NotAuthorizedKeyD";

		let (payload, msa_signature) = generate_and_sign_authorized_key_payload(
			msa_id,
			&owner_key_pair,
			&origin_key_pair,
			None,
			Some(BAD_DISCRIMIMINANT),
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
		let result = CheckFreeExtrinsicUse::<Test>::new().validate(
			&origin_key_pair.public().into(),
			call_withdraw_msa_token,
			&info,
			len,
		);
		assert_ok!(result);
	});
}
