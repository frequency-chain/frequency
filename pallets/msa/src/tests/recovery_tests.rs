use common_primitives::{
	msa::{MsaLookup, ProviderId},
	utils::wrap_binary_data,
};
use frame_support::{assert_noop, assert_ok};
use parity_scale_codec::Encode;
use sp_core::{sr25519, Pair};
use sp_runtime::MultiSignature;

use crate::{
	tests::mock::*,
	types::{AddKeyData, RecoveryHash},
	Error, Event, MsaIdToRecoveryCommitment,
};

// Common test constants
const TEST_AUTHENTICATION_CONTACT: &str = "user@example.com";
const TEST_EXPIRATION_BLOCK: u32 = 100;

#[test]
fn add_recovery_commitment_with_valid_data_should_succeed() {
	new_test_ext().execute_with(|| {
		let (msa_id, msa_owner_key_pair, recovery_commitment) = setup_recovery_with_commitment(
			&generate_test_recovery_secret(),
			TEST_AUTHENTICATION_CONTACT,
		);

		assert_eq!(MsaIdToRecoveryCommitment::<Test>::get(msa_id), Some(recovery_commitment));
		// Verify event was emitted
		System::assert_last_event(
			Event::RecoveryCommitmentAdded {
				who: msa_owner_key_pair.public().into(),
				msa_id,
				recovery_commitment,
			}
			.into(),
		);
	});
}

#[test]
fn add_recovery_commitment_with_invalid_signature_should_fail() {
	new_test_ext().execute_with(|| {
		// Create an MSA account
		let (msa_id, msa_owner_key_pair) = create_account();
		let (fake_key_pair, _) = sr25519::Pair::generate();
		// Create recovery commitment payload
		let recovery_commitment: RecoveryHash = [1u8; 32];

		// Sign the payload with wrong key
		let (payload, fake_signature) = generate_and_sign_recovery_commitment_payload(
			&fake_key_pair,
			recovery_commitment,
			TEST_EXPIRATION_BLOCK,
		);

		// Execute the extrinsic and expect failure
		assert_noop!(
			Msa::add_recovery_commitment(
				test_origin_signed(2),              // provider_key origin
				msa_owner_key_pair.public().into(), // msa_owner_key
				fake_signature,
				payload
			),
			Error::<Test>::InvalidSignature
		);

		// Verify storage was not updated
		assert_eq!(MsaIdToRecoveryCommitment::<Test>::get(msa_id), None);
	});
}

#[test]
fn add_recovery_commitment_with_nonexistent_msa_should_fail() {
	new_test_ext().execute_with(|| {
		let (fake_key_pair, _) = sr25519::Pair::generate();
		// Create recovery commitment payload with non-existent MSA ID
		let recovery_commitment: RecoveryHash = [1u8; 32];

		let (payload, signature) = generate_and_sign_recovery_commitment_payload(
			&fake_key_pair,
			recovery_commitment,
			TEST_EXPIRATION_BLOCK,
		);

		// Execute the extrinsic and expect failure
		assert_noop!(
			Msa::add_recovery_commitment(
				test_origin_signed(2),         // provider_key origin
				fake_key_pair.public().into(), // non-existent msa_owner_key
				signature,
				payload
			),
			Error::<Test>::NoKeyExists
		);
	});
}

#[test]
fn add_recovery_commitment_with_not_yet_valid_signature_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(11_122);
		let mortality_block = 11_323;

		// Create an MSA account
		let (msa_id, msa_owner_key_pair) = create_account();

		// Create recovery commitment payload with not yet valid signature
		let recovery_commitment: RecoveryHash = [1u8; 32];

		let (payload, signature) = generate_and_sign_recovery_commitment_payload(
			&msa_owner_key_pair,
			recovery_commitment,
			mortality_block,
		);

		// Execute the extrinsic and expect failure
		assert_noop!(
			Msa::add_recovery_commitment(
				test_origin_signed(2),              // provider_key origin
				msa_owner_key_pair.public().into(), // msa_owner_key
				signature,
				payload
			),
			Error::<Test>::ProofNotYetValid
		);

		// Verify storage was not updated
		assert_eq!(MsaIdToRecoveryCommitment::<Test>::get(msa_id), None);
	});
}

#[test]
fn add_recovery_commitment_with_expired_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// Create an MSA account
		let (msa_id, msa_owner_key_pair) = create_account();

		// Set current block to 50
		run_to_block(50);

		// Create recovery commitment payload with past expiration
		let recovery_commitment: RecoveryHash = [1u8; 32];

		let (payload, signature) = generate_and_sign_recovery_commitment_payload(
			&msa_owner_key_pair,
			recovery_commitment,
			10u32, // Already expired
		);

		// Execute the extrinsic and expect failure
		assert_noop!(
			Msa::add_recovery_commitment(
				test_origin_signed(2),              // provider_key origin
				msa_owner_key_pair.public().into(), // msa_owner_key
				signature,
				payload
			),
			Error::<Test>::ProofHasExpired
		);

		// Verify storage was not updated
		assert_eq!(MsaIdToRecoveryCommitment::<Test>::get(msa_id), None);
	});
}

#[test]
fn add_recovery_commitment_updates_existing_commitment() {
	new_test_ext().execute_with(|| {
		let (msa_id, msa_owner_key_pair, first_commitment) = setup_recovery_with_commitment(
			&generate_test_recovery_secret(),
			TEST_AUTHENTICATION_CONTACT,
		);

		// Verify first commitment was stored
		assert_eq!(MsaIdToRecoveryCommitment::<Test>::get(msa_id), Some(first_commitment));

		// Add second recovery commitment (should update the first one)
		let second_commitment: RecoveryHash = [2u8; 32];
		let (second_payload, second_signature) = generate_and_sign_recovery_commitment_payload(
			&msa_owner_key_pair,
			second_commitment,
			50u32, // Use a smaller expiration that's within mortality window
		);

		assert_ok!(Msa::add_recovery_commitment(
			test_origin_signed(2),
			msa_owner_key_pair.public().into(),
			second_signature,
			second_payload
		));

		// Verify second commitment replaced the first one
		assert_eq!(MsaIdToRecoveryCommitment::<Test>::get(msa_id), Some(second_commitment));

		// Verify event was emitted for the second commitment
		System::assert_last_event(
			Event::RecoveryCommitmentAdded {
				who: msa_owner_key_pair.public().into(),
				msa_id,
				recovery_commitment: second_commitment,
			}
			.into(),
		);
	});
}

#[test]
fn add_recovery_commitment_duplicate_signature_should_fail() {
	new_test_ext().execute_with(|| {
		// Create an MSA account
		let (_msa_id, msa_owner_key_pair) = create_account();
		// Create recovery commitment payload
		let recovery_commitment: RecoveryHash = [1u8; 32];

		let (payload, signature) = generate_and_sign_recovery_commitment_payload(
			&msa_owner_key_pair,
			recovery_commitment,
			TEST_EXPIRATION_BLOCK,
		);

		// Execute the extrinsic first time - should succeed
		assert_ok!(Msa::add_recovery_commitment(
			test_origin_signed(2),
			msa_owner_key_pair.public().into(),
			signature.clone(),
			payload.clone()
		));

		// Execute the extrinsic second time with same signature - should fail
		assert_noop!(
			Msa::add_recovery_commitment(
				test_origin_signed(2),
				msa_owner_key_pair.public().into(),
				signature,
				payload
			),
			Error::<Test>::SignatureAlreadySubmitted
		);
	});
}

#[test]
fn add_recovery_commitment_unsigned_origin_should_fail() {
	new_test_ext().execute_with(|| {
		// Create an MSA account
		let (_msa_id, msa_owner_key_pair) = create_account();
		// Create recovery commitment payload
		let recovery_commitment: RecoveryHash = [1u8; 32];

		let (payload, signature) = generate_and_sign_recovery_commitment_payload(
			&msa_owner_key_pair,
			recovery_commitment,
			TEST_EXPIRATION_BLOCK,
		);

		// Execute the extrinsic with unsigned origin - should fail
		assert_noop!(
			Msa::add_recovery_commitment(
				RuntimeOrigin::none(), // Unsigned origin
				msa_owner_key_pair.public().into(),
				signature,
				payload
			),
			frame_support::error::BadOrigin
		);
	});
}

#[test]
fn recover_account_with_valid_data_should_succeed() {
	new_test_ext().execute_with(|| {
		let test_recovery_secret = generate_test_recovery_secret();
		// Use helper function to setup complete recovery scenario
		let (msa_id, _msa_owner_key_pair, recovery_commitment) =
			setup_recovery_with_commitment(&test_recovery_secret, TEST_AUTHENTICATION_CONTACT);

		let (provider_msa_id, provider_key_pair) = create_and_approve_recovery_provider();

		// Generate a new control key for recovery
		let (new_control_key_pair, _) = sr25519::Pair::generate();

		// Generate AddKeyData payload and signature from new control key
		let (add_key_payload, new_key_proof) =
			generate_and_sign_add_key_payload(&new_control_key_pair, msa_id, TEST_EXPIRATION_BLOCK);

		// Compute intermediary hashes for recovery
		let (intermediary_hash_a, intermediary_hash_b) = compute_recovery_intermediary_hashes(
			&test_recovery_secret,
			TEST_AUTHENTICATION_CONTACT,
		);

		// Clear previous events
		System::reset_events();

		// Execute the recovery
		assert_ok!(Msa::recover_account(
			RuntimeOrigin::signed(provider_key_pair.public().into()),
			intermediary_hash_a,
			intermediary_hash_b,
			new_key_proof,
			add_key_payload.clone()
		));

		// Verify the new control key is added to the MSA
		assert_eq!(Msa::get_msa_id(&add_key_payload.new_public_key), Some(msa_id));

		// Verify the recovery commitment is invalidated
		assert_eq!(MsaIdToRecoveryCommitment::<Test>::get(msa_id), None);

		// Verify all events were emitted
		let events = System::events();
		assert_eq!(events.len(), 3); // PublicKeyAdded, AccountRecovered, RecoveryCommitmentInvalidated

		// First event should be PublicKeyAdded
		assert!(matches!(
			&events[0].event,
			RuntimeEvent::Msa(Event::PublicKeyAdded {
				msa_id: added_msa_id,
				key: added_key,
			}) if *added_msa_id == msa_id &&
				 *added_key == add_key_payload.new_public_key
		));

		// Second event should be AccountRecovered
		assert!(matches!(
			&events[1].event,
			RuntimeEvent::Msa(Event::AccountRecovered {
				msa_id: recovered_msa_id,
				recovery_provider: recovered_provider,
				new_control_key: recovered_key,
			}) if *recovered_msa_id == msa_id &&
				 *recovered_provider == ProviderId(provider_msa_id) &&
				 *recovered_key == add_key_payload.new_public_key
		));

		// Third event should be RecoveryCommitmentInvalidated
		assert!(matches!(
			events[2].event,
			RuntimeEvent::Msa(Event::RecoveryCommitmentInvalidated {
				msa_id: invalidated_msa_id,
				recovery_commitment: invalidated_commitment,
			}) if invalidated_msa_id == msa_id &&
				 invalidated_commitment == recovery_commitment
		));
	});
}

#[test]
fn recover_account_with_non_approved_provider_should_fail() {
	new_test_ext().execute_with(|| {
		// Use helper function to setup complete recovery scenario
		let (msa_id, _msa_owner_key_pair, recovery_commitment) = setup_recovery_with_commitment(
			&generate_test_recovery_secret(),
			TEST_AUTHENTICATION_CONTACT,
		);

		// Create a provider but don't approve it for recovery
		let (provider_msa_id, provider_key_pair) = create_account();
		assert_ok!(Msa::create_provider_for(provider_msa_id.into(), Vec::from("NotApproved")));

		// Generate a new control key for recovery
		let (new_control_key_pair, _) = sr25519::Pair::generate();

		// Generate AddKeyData payload and signature from new control key
		let (add_key_payload, new_key_proof) =
			generate_and_sign_add_key_payload(&new_control_key_pair, msa_id, 100u32);

		// Compute intermediary hashes for recovery
		let (intermediary_hash_a, intermediary_hash_b) = compute_recovery_intermediary_hashes(
			&generate_test_recovery_secret(),
			TEST_AUTHENTICATION_CONTACT,
		);

		// Execute the recovery and expect failure
		assert_noop!(
			Msa::recover_account(
				RuntimeOrigin::signed(provider_key_pair.public().into()),
				intermediary_hash_a,
				intermediary_hash_b,
				new_key_proof,
				add_key_payload
			),
			Error::<Test>::NotAuthorizedRecoveryProvider
		);

		// Verify the recovery commitment is still there
		assert_eq!(MsaIdToRecoveryCommitment::<Test>::get(msa_id), Some(recovery_commitment));
	});
}

#[test]
fn recover_account_with_invalid_recovery_commitment_should_fail() {
	new_test_ext().execute_with(|| {
		// Use helper function to setup complete recovery scenario
		let (msa_id, _msa_owner_key_pair, recovery_commitment) = setup_recovery_with_commitment(
			&generate_test_recovery_secret(),
			TEST_AUTHENTICATION_CONTACT,
		);

		// Create and approve a recovery provider using helper function
		let (_provider_msa_id, provider_key_pair) = create_and_approve_recovery_provider();

		// Generate a new control key for recovery
		let (new_control_key_pair, _) = sr25519::Pair::generate();

		// Generate AddKeyData payload and signature from new control key
		let (add_key_payload, new_key_proof) =
			generate_and_sign_add_key_payload(&new_control_key_pair, msa_id, TEST_EXPIRATION_BLOCK);

		// Try to recover with wrong intermediary hashes
		let wrong_intermediary_hash_a = [2u8; 32];
		let wrong_intermediary_hash_b = [3u8; 32];
		assert_noop!(
			Msa::recover_account(
				RuntimeOrigin::signed(provider_key_pair.public().into()),
				wrong_intermediary_hash_a,
				wrong_intermediary_hash_b,
				new_key_proof,
				add_key_payload
			),
			Error::<Test>::InvalidRecoveryCommitment
		);

		// Verify the recovery commitment is still there
		assert_eq!(MsaIdToRecoveryCommitment::<Test>::get(msa_id), Some(recovery_commitment));
	});
}

#[test]
fn recover_account_with_no_recovery_commitment_should_fail() {
	new_test_ext().execute_with(|| {
		// Create an MSA account without recovery commitment
		let (msa_id, _msa_owner_key_pair) = create_account();

		// Create and approve a recovery provider using helper function
		let (_provider_msa_id, provider_key_pair) = create_and_approve_recovery_provider();

		// Generate a new control key for recovery
		let (new_control_key_pair, _) = sr25519::Pair::generate();

		// Generate AddKeyData payload and signature from new control key
		let (add_key_payload, new_key_proof) =
			generate_and_sign_add_key_payload(&new_control_key_pair, msa_id, 100u32);

		// Try to recover without any recovery commitment
		let some_intermediary_hash_a = [1u8; 32];
		let some_intermediary_hash_b = [2u8; 32];
		assert_noop!(
			Msa::recover_account(
				RuntimeOrigin::signed(provider_key_pair.public().into()),
				some_intermediary_hash_a,
				some_intermediary_hash_b,
				new_key_proof,
				add_key_payload
			),
			Error::<Test>::NoRecoveryCommitment
		);
	});
}

#[test]
fn recover_account_with_existing_control_key_should_fail() {
	new_test_ext().execute_with(|| {
		let test_recovery_secret = generate_test_recovery_secret();
		// Use helper function to setup complete recovery scenario
		let (msa_id, msa_owner_key_pair, recovery_commitment) =
			setup_recovery_with_commitment(&test_recovery_secret, TEST_AUTHENTICATION_CONTACT);

		// Create and approve a recovery provider using helper function
		let (_provider_msa_id, provider_key_pair) = create_and_approve_recovery_provider();

		// Use the existing MSA owner key as the "new" control key
		// Generate AddKeyData payload and signature from existing key
		let (add_key_payload, new_key_proof) =
			generate_and_sign_add_key_payload(&msa_owner_key_pair, msa_id, TEST_EXPIRATION_BLOCK);

		// Compute intermediary hashes for recovery
		let (intermediary_hash_a, intermediary_hash_b) = compute_recovery_intermediary_hashes(
			&test_recovery_secret,
			TEST_AUTHENTICATION_CONTACT,
		);

		// Execute the recovery - should fail with KeyAlreadyRegistered
		assert_noop!(
			Msa::recover_account(
				RuntimeOrigin::signed(provider_key_pair.public().into()),
				intermediary_hash_a,
				intermediary_hash_b,
				new_key_proof,
				add_key_payload.clone()
			),
			Error::<Test>::KeyAlreadyRegistered
		);

		// Verify the recovery commitment is still there since the operation failed
		assert_eq!(MsaIdToRecoveryCommitment::<Test>::get(msa_id), Some(recovery_commitment));
	});
}

#[test]
fn recover_account_unsigned_origin_should_fail() {
	new_test_ext().execute_with(|| {
		// Use helper function to setup complete recovery scenario
		let (msa_id, _msa_owner_key_pair, _recovery_commitment) = setup_recovery_with_commitment(
			&generate_test_recovery_secret(),
			TEST_AUTHENTICATION_CONTACT,
		);

		// Generate a new control key for recovery
		let (new_control_key_pair, _) = sr25519::Pair::generate();

		// Generate AddKeyData payload and signature from new control key
		let (add_key_payload, new_key_proof) =
			generate_and_sign_add_key_payload(&new_control_key_pair, msa_id, TEST_EXPIRATION_BLOCK);

		// Compute intermediary hashes for recovery
		let (intermediary_hash_a, intermediary_hash_b) = compute_recovery_intermediary_hashes(
			&generate_test_recovery_secret(),
			TEST_AUTHENTICATION_CONTACT,
		);

		// Try to recover with unsigned origin
		assert_noop!(
			Msa::recover_account(
				RuntimeOrigin::none(),
				intermediary_hash_a,
				intermediary_hash_b,
				new_key_proof,
				add_key_payload
			),
			frame_support::error::BadOrigin
		);
	});
}

#[test]
fn recover_account_double_recovery_attempt_should_fail() {
	new_test_ext().execute_with(|| {
		let test_recovery_secret = generate_test_recovery_secret();
		// Use helper function to setup complete recovery scenario
		let (msa_id, _msa_owner_key_pair, _recovery_commitment) =
			setup_recovery_with_commitment(&test_recovery_secret, TEST_AUTHENTICATION_CONTACT);

		// Create and approve a recovery provider using helper function
		let (_provider_msa_id, provider_key_pair) = create_and_approve_recovery_provider();

		// Generate a new control key for recovery
		let (new_control_key_pair, _) = sr25519::Pair::generate();

		// Generate AddKeyData payload and signature from new control key
		let (add_key_payload, new_key_proof) =
			generate_and_sign_add_key_payload(&new_control_key_pair, msa_id, TEST_EXPIRATION_BLOCK);

		// Compute intermediary hashes for recovery
		let (intermediary_hash_a, intermediary_hash_b) = compute_recovery_intermediary_hashes(
			&test_recovery_secret,
			TEST_AUTHENTICATION_CONTACT,
		);

		// Execute the first recovery - should succeed
		assert_ok!(Msa::recover_account(
			RuntimeOrigin::signed(provider_key_pair.public().into()),
			intermediary_hash_a,
			intermediary_hash_b,
			new_key_proof,
			add_key_payload.clone()
		));

		// Verify the recovery commitment is invalidated
		assert_eq!(MsaIdToRecoveryCommitment::<Test>::get(msa_id), None);

		// Try to use the same recovery commitment again - should fail
		let (another_new_control_key_pair, _) = sr25519::Pair::generate();
		let (another_add_key_payload, another_new_key_proof) = generate_and_sign_add_key_payload(
			&another_new_control_key_pair,
			msa_id,
			TEST_EXPIRATION_BLOCK,
		);

		assert_noop!(
			Msa::recover_account(
				RuntimeOrigin::signed(provider_key_pair.public().into()),
				intermediary_hash_a,
				intermediary_hash_b,
				another_new_key_proof,
				another_add_key_payload
			),
			Error::<Test>::NoRecoveryCommitment
		);
	});
}

#[test]
fn recover_account_with_non_provider_should_fail() {
	new_test_ext().execute_with(|| {
		let test_recovery_secret = generate_test_recovery_secret();
		// Use helper function to setup complete recovery scenario
		let (msa_id, _msa_owner_key_pair, _recovery_commitment) =
			setup_recovery_with_commitment(&test_recovery_secret, TEST_AUTHENTICATION_CONTACT);

		// Create a regular account (not a provider)
		let (_regular_account_msa_id, regular_account_key_pair) = create_account();

		// Generate a new control key for recovery
		let (new_control_key_pair, _) = sr25519::Pair::generate();

		// Generate AddKeyData payload and signature from new control key
		let (add_key_payload, new_key_proof) =
			generate_and_sign_add_key_payload(&new_control_key_pair, msa_id, TEST_EXPIRATION_BLOCK);

		// Compute intermediary hashes for recovery
		let (intermediary_hash_a, intermediary_hash_b) = compute_recovery_intermediary_hashes(
			&test_recovery_secret,
			TEST_AUTHENTICATION_CONTACT,
		);

		// Try to recover with a non-provider account
		assert_noop!(
			Msa::recover_account(
				RuntimeOrigin::signed(regular_account_key_pair.public().into()),
				intermediary_hash_a,
				intermediary_hash_b,
				new_key_proof,
				add_key_payload
			),
			Error::<Test>::NotAuthorizedRecoveryProvider
		);
	});
}

#[test]
fn recover_account_with_revoked_recovery_provider_should_fail() {
	new_test_ext().execute_with(|| {
		let test_recovery_secret = generate_test_recovery_secret();
		// Use helper function to setup complete recovery scenario
		let (msa_id, _msa_owner_key_pair, _recovery_commitment) =
			setup_recovery_with_commitment(&test_recovery_secret, TEST_AUTHENTICATION_CONTACT);

		// Create, approve, then revoke a recovery provider
		let (provider_msa_id, provider_key_pair) = create_and_approve_recovery_provider();

		// Revoke the recovery provider
		assert_ok!(Msa::remove_recovery_provider(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
			ProviderId(provider_msa_id)
		));

		// Generate a new control key for recovery
		let (new_control_key_pair, _) = sr25519::Pair::generate();

		// Generate AddKeyData payload and signature from new control key
		let (add_key_payload, new_key_proof) =
			generate_and_sign_add_key_payload(&new_control_key_pair, msa_id, TEST_EXPIRATION_BLOCK);

		// Compute intermediary hashes for recovery
		let (intermediary_hash_a, intermediary_hash_b) = compute_recovery_intermediary_hashes(
			&test_recovery_secret,
			TEST_AUTHENTICATION_CONTACT,
		);

		// Try to recover with a revoked provider
		assert_noop!(
			Msa::recover_account(
				RuntimeOrigin::signed(provider_key_pair.public().into()),
				intermediary_hash_a,
				intermediary_hash_b,
				new_key_proof,
				add_key_payload
			),
			Error::<Test>::NotAuthorizedRecoveryProvider
		);
	});
}

#[test]
fn recover_account_with_invalid_new_key_signature_should_fail() {
	new_test_ext().execute_with(|| {
		let test_recovery_secret = generate_test_recovery_secret();
		// Use helper function to setup complete recovery scenario
		let (msa_id, _msa_owner_key_pair, recovery_commitment) =
			setup_recovery_with_commitment(&test_recovery_secret, TEST_AUTHENTICATION_CONTACT);

		// Create and approve a recovery provider using helper function
		let (_provider_msa_id, provider_key_pair) = create_and_approve_recovery_provider();

		// Generate a new control key for recovery
		let (new_control_key_pair, _) = sr25519::Pair::generate();
		let (fake_key_pair, _) = sr25519::Pair::generate();

		// Generate AddKeyData payload but sign with wrong key
		let add_key_payload = AddKeyData::<Test> {
			msa_id,
			expiration: TEST_EXPIRATION_BLOCK,
			new_public_key: new_control_key_pair.public().into(),
		};

		let encoded_payload = wrap_binary_data(add_key_payload.encode());
		let invalid_signature: MultiSignature = fake_key_pair.sign(&encoded_payload).into();

		// Compute intermediary hashes for recovery
		let (intermediary_hash_a, intermediary_hash_b) = compute_recovery_intermediary_hashes(
			&test_recovery_secret,
			TEST_AUTHENTICATION_CONTACT,
		);

		// Execute the recovery with invalid signature - should fail
		assert_noop!(
			Msa::recover_account(
				RuntimeOrigin::signed(provider_key_pair.public().into()),
				intermediary_hash_a,
				intermediary_hash_b,
				invalid_signature,
				add_key_payload
			),
			Error::<Test>::NewKeyOwnershipInvalidSignature
		);

		// Verify the recovery commitment is still there
		assert_eq!(MsaIdToRecoveryCommitment::<Test>::get(msa_id), Some(recovery_commitment));
	});
}
