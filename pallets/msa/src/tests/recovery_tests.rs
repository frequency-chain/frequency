use frame_support::{assert_noop, assert_ok};
use sp_core::{sr25519, Pair};

use crate::{
	tests::mock::*, Error, Event, MsaIdToRecoveryCommitment,
};

#[test]
fn add_recovery_commitment_with_valid_data_should_succeed() {
	new_test_ext().execute_with(|| {
		// Create an MSA account
		let (msa_id, msa_owner_key_pair) = create_account();
		// Create recovery commitment payload
		let recovery_commitment = [1u8; 32];

		let (payload, msa_owner_signature) = generate_and_sign_recovery_commitment_payload(
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
		let recovery_commitment = [1u8; 32];

		// Sign the payload with wrong key
		let (payload, fake_signature) = generate_and_sign_recovery_commitment_payload(
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
		// Create recovery commitment payload with non-existent MSA ID
		let recovery_commitment = [1u8; 32];

		let (payload, signature) = generate_and_sign_recovery_commitment_payload(
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
fn add_recovery_commitment_with_not_yet_valid_signature_should_fail() {
	new_test_ext().execute_with(|| {
		System::set_block_number(11_122);
		let mortality_block = 11_323;

		// Create an MSA account
		let (msa_id, msa_owner_key_pair) = create_account();

		// Create recovery commitment payload with not yet valid signature
		let recovery_commitment = [1u8; 32];

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
		let recovery_commitment = [1u8; 32];

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
		// Create an MSA account
		let (msa_id, msa_owner_key_pair) = create_account();
		// Add first recovery commitment
		let first_commitment = [1u8; 32];

		let (first_payload, first_signature) = generate_and_sign_recovery_commitment_payload(
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
		let recovery_commitment = [1u8; 32];

		let (payload, signature) = generate_and_sign_recovery_commitment_payload(
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
fn add_recovery_commitment_unsigned_origin_should_fail() {
	new_test_ext().execute_with(|| {
		// Create an MSA account
		let (_msa_id, msa_owner_key_pair) = create_account();
		// Create recovery commitment payload
		let recovery_commitment = [1u8; 32];

		let (payload, signature) = generate_and_sign_recovery_commitment_payload(
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
