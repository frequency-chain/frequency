use frame_support::{assert_noop, assert_ok};
use sp_core::{sr25519, Pair};

use crate::{tests::mock::*, Error, Event, MsaIdToRecoveryCommitment};

#[test]
fn add_recovery_commitment_with_valid_data_should_succeed() {
	new_test_ext().execute_with(|| {
		// Create an MSA account
		let (msa_id, msa_owner_key_pair) = create_account();
		// Create recovery commitment payload
		let recovery_commitment = [1u8; 32];

		let (payload, msa_owner_signature) = generate_and_sign_recovery_commitment_payload(
			msa_id,
			&msa_owner_key_pair,
			recovery_commitment,
			100u32,
		);

		// Execute the extrinsic
		assert_ok!(Msa::add_recovery_commitment(
			test_origin_signed(2),              // provider_key origin
			msa_owner_key_pair.public().into(), // msa_owner_key
			msa_owner_signature,
			payload.clone()
		));

		assert_eq!(MsaIdToRecoveryCommitment::<Test>::get(msa_id), Some(recovery_commitment));
		// Verify event was emitted
		System::assert_last_event(
			Event::RecoveryCommitmentAdded { who: msa_owner_key_pair.public().into(), msa_id }
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
		let recovery_commitment = [1u8; 32];

		// Sign the payload with wrong key
		let (payload, fake_signature) = generate_and_sign_recovery_commitment_payload(
			msa_id,
			&fake_key_pair,
			recovery_commitment,
			100u32,
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
		let fake_msa_id = 99999u64;
		// Create recovery commitment payload with non-existent MSA ID
		let recovery_commitment = [1u8; 32];

		let (payload, signature) = generate_and_sign_recovery_commitment_payload(
			fake_msa_id,
			&fake_key_pair,
			recovery_commitment,
			100u32,
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
fn add_recovery_commitment_with_expired_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// Create an MSA account
		let (msa_id, msa_owner_key_pair) = create_account();

		// Set current block to 50
		run_to_block(50);

		// Create recovery commitment payload with past expiration
		let recovery_commitment = [1u8; 32];

		let (payload, signature) = generate_and_sign_recovery_commitment_payload(
			msa_id,
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
		// Create an MSA account
		let (msa_id, msa_owner_key_pair) = create_account();
		// Add first recovery commitment
		let first_commitment = [1u8; 32];

		let (first_payload, first_signature) = generate_and_sign_recovery_commitment_payload(
			msa_id,
			&msa_owner_key_pair,
			first_commitment,
			100u32,
		);

		assert_ok!(Msa::add_recovery_commitment(
			test_origin_signed(2),
			msa_owner_key_pair.public().into(),
			first_signature,
			first_payload
		));

		// Verify first commitment was stored
		assert_eq!(MsaIdToRecoveryCommitment::<Test>::get(msa_id), Some(first_commitment));

		// Add second recovery commitment (should update the first one)
		let second_commitment = [2u8; 32];
		let (second_payload, second_signature) = generate_and_sign_recovery_commitment_payload(
			msa_id,
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
			Event::RecoveryCommitmentAdded { who: msa_owner_key_pair.public().into(), msa_id }
				.into(),
		);
	});
}

#[test]
fn add_recovery_commitment_duplicate_signature_should_fail() {
	new_test_ext().execute_with(|| {
		// Create an MSA account
		let (msa_id, msa_owner_key_pair) = create_account();
		// Create recovery commitment payload
		let recovery_commitment = [1u8; 32];

		let (payload, signature) = generate_and_sign_recovery_commitment_payload(
			msa_id,
			&msa_owner_key_pair,
			recovery_commitment,
			100u32,
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
fn add_recovery_commitment_with_wrong_msa_in_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// TODO: Possibly remove this test if we decide the payload should not contain MSA ID
		// Create two MSA accounts
		let (_msa_id1, msa_owner_key_pair1) = create_account();
		let (msa_id2, _msa_owner_key_pair2) = create_account();
		// Create recovery commitment payload with different MSA ID than the signer's
		let recovery_commitment = [1u8; 32];

		let (payload, signature) = generate_and_sign_recovery_commitment_payload(
			msa_id2,              // Using msa_id2 here
			&msa_owner_key_pair1, // But signing with msa_owner_key_pair1 (which is for msa_id1)
			recovery_commitment,
			100u32,
		);

		// Execute the extrinsic - should fail because the key doesn't match the MSA
		assert_noop!(
			Msa::add_recovery_commitment(
				test_origin_signed(3),
				msa_owner_key_pair1.public().into(), // This key is for msa_id1
				signature,
				payload // But payload is for msa_id2
			),
			Error::<Test>::NotMsaOwner
		);
	});
}

#[test]
fn add_recovery_commitment_unsigned_origin_should_fail() {
	new_test_ext().execute_with(|| {
		// Create an MSA account
		let (msa_id, msa_owner_key_pair) = create_account();
		// Create recovery commitment payload
		let recovery_commitment = [1u8; 32];

		let (payload, signature) = generate_and_sign_recovery_commitment_payload(
			msa_id,
			&msa_owner_key_pair,
			recovery_commitment,
			100u32,
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
