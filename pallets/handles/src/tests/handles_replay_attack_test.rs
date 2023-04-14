use crate::{handles_signed_extension::HandlesSignedExtension, tests::mock::*, Error, Event};
use codec::Decode;
use common_primitives::{handles::*, msa::MessageSourceId};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchInfo};
use numtoa::*;
use sp_core::{sr25519, Encode, Pair};
use sp_runtime::traits::SignedExtension;

#[test]
fn test_handle_claim_replay_attack() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 20;
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec(), expiration);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof.clone(),
			payload.clone()
		));
		let msa_id = MessageSourceId::decode(&mut &alice.public().encode()[..]).unwrap();
		let handle = Handles::get_handle_for_msa(msa_id);
		assert!(handle.is_some());

		// Compose the full display handle from the base handle, "." delimeter and suffix
		let handle_result = handle.unwrap();
		let suffix: HandleSuffix = handle_result.suffix;
		let mut full_handle_vec: Vec<u8> = vec![];
		full_handle_vec.extend(base_handle_str.as_bytes());
		full_handle_vec.extend(b"."); // The delimeter
		let mut buff = [0u8; SUFFIX_MAX_DIGITS];
		full_handle_vec.extend(suffix.numtoa(10, &mut buff)); // Use base 10

		run_to_block(200);
		// Retire the handle
		assert_ok!(Handles::retire_handle(RuntimeOrigin::signed(alice.public().into()),));

		// Confirm that HandleRetired event was deposited
		System::assert_last_event(
			Event::HandleRetired { msa_id, handle: full_handle_vec.clone() }.into(),
		);
	});
}

#[test]
fn test_handle_claim_replay_attack_with_different_account() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 100;
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec(), expiration);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof.clone(),
			payload.clone()
		));
		let msa_id = MessageSourceId::decode(&mut &alice.public().encode()[..]).unwrap();
		let handle = Handles::get_handle_for_msa(msa_id);
		assert!(handle.is_some());

		// Compose the full display handle from the base handle, "." delimeter and suffix
		let handle_result = handle.unwrap();
		let suffix: HandleSuffix = handle_result.suffix;
		let mut full_handle_vec: Vec<u8> = vec![];
		full_handle_vec.extend(base_handle_str.as_bytes());
		full_handle_vec.extend(b"."); // The delimeter
		let mut buff = [0u8; SUFFIX_MAX_DIGITS];
		full_handle_vec.extend(suffix.numtoa(10, &mut buff)); // Use base 10

		// Try to claim the base handle again with a different account which should fail
		let bob = sr25519::Pair::from_seed(&[1; 32]);
		assert_noop!(
			Handles::claim_handle(
				RuntimeOrigin::signed(bob.public().into()),
				bob.public().into(),
				proof,
				payload
			),
			Error::<Test>::InvalidSignature
		);
	});
}

#[test]
fn test_handle_retire_replay_attack() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 50;
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec(), expiration);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof.clone(),
			payload.clone()
		));
		let msa_id = MessageSourceId::decode(&mut &alice.public().encode()[..]).unwrap();
		let handle = Handles::get_handle_for_msa(msa_id);
		assert!(handle.is_some());

		// Compose the full display handle from the base handle, "." delimeter and suffix
		let handle_result = handle.unwrap();
		let suffix: HandleSuffix = handle_result.suffix;
		let mut full_handle_vec: Vec<u8> = vec![];
		full_handle_vec.extend(base_handle_str.as_bytes());
		full_handle_vec.extend(b"."); // The delimeter
		let mut buff = [0u8; SUFFIX_MAX_DIGITS];
		full_handle_vec.extend(suffix.numtoa(10, &mut buff)); // Use base 10

		// Fail to retire due to mortality
		let call_retire_handle: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Handles(HandlesCall::retire_handle {});
		let info = DispatchInfo::default();
		let len = 0_usize;
		let early_retire_result = HandlesSignedExtension::<Test>::new().validate(
			&alice.public().into(),
			call_retire_handle,
			&info,
			len,
		);
		assert!(early_retire_result.is_err());

		// Retire the handle after the mortality period
		run_to_block(200);
		assert_ok!(Handles::retire_handle(RuntimeOrigin::signed(alice.public().into()),));

		// Confirm that HandleRetired event was deposited
		System::assert_last_event(
			Event::HandleRetired { msa_id, handle: full_handle_vec.clone() }.into(),
		);

		// Try to retire the same handle again which should fail
		assert_noop!(
			Handles::retire_handle(RuntimeOrigin::signed(alice.public().into()),),
			Error::<Test>::MSAHandleDoesNotExist
		);
	});
}

#[test]
fn test_handle_claim_replay_attack_with_different_payload() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 100;
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec(), expiration);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof.clone(),
			payload.clone()
		));
		let msa_id = MessageSourceId::decode(&mut &alice.public().encode()[..]).unwrap();
		let handle = Handles::get_handle_for_msa(msa_id);
		assert!(handle.is_some());

		// Compose the full display handle from the base handle, "." delimeter and suffix
		let handle_result = handle.unwrap();
		let suffix: HandleSuffix = handle_result.suffix;
		let mut full_handle_vec: Vec<u8> = vec![];
		full_handle_vec.extend(base_handle_str.as_bytes());
		full_handle_vec.extend(b"."); // The delimeter
		let mut buff = [0u8; SUFFIX_MAX_DIGITS];
		full_handle_vec.extend(suffix.numtoa(10, &mut buff)); // Use base 10

		// Try to claim the base handle again with a different payload which should fail
		let (payload2, proof2) =
			get_signed_claims_payload(&alice, b"another-handle".to_vec(), expiration);
		assert_noop!(
			Handles::claim_handle(
				RuntimeOrigin::signed(alice.public().into()),
				alice.public().into(),
				proof2,
				payload2
			),
			Error::<Test>::MSAHandleAlreadyExists
		);
	});
}

#[test]
fn test_handle_retire_replay_attack_with_different_account() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 100;
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec(), expiration);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof.clone(),
			payload.clone()
		));
		let msa_id = MessageSourceId::decode(&mut &alice.public().encode()[..]).unwrap();
		let handle = Handles::get_handle_for_msa(msa_id);
		assert!(handle.is_some());

		// Try to retire the handle with a different account which should fail
		let bob = sr25519::Pair::from_seed(&[1; 32]);
		assert_noop!(
			Handles::retire_handle(RuntimeOrigin::signed(bob.public().into()),),
			Error::<Test>::MSAHandleDoesNotExist
		);
	});
}

#[test]
fn test_base_handle_claim_and_retire_with_multiple_accounts() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 100;
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec(), expiration);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof.clone(),
			payload.clone()
		));

		// Get suffix
		let msa_id = MessageSourceId::decode(&mut &alice.public().encode()[..]).unwrap();
		let handle = Handles::get_handle_for_msa(msa_id);
		assert!(handle.is_some());
		let handle_result = handle.unwrap();
		let prev_suffix: HandleSuffix = handle_result.suffix;

		// Try to claim the base andle with a different account which should not fail
		let bob = sr25519::Pair::from_seed(&[1; 32]);
		let (payload2, proof2) =
			get_signed_claims_payload(&bob, base_handle_str.as_bytes().to_vec(), expiration);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(bob.public().into()),
			bob.public().into(),
			proof2,
			payload2
		));

		// Suffix for the new account should be different than the previous one
		let msa_id = MessageSourceId::decode(&mut &bob.public().encode()[..]).unwrap();
		let handle = Handles::get_handle_for_msa(msa_id);
		assert!(handle.is_some());
		let handle_result = handle.unwrap();
		let suffix: HandleSuffix = handle_result.suffix;

		assert_ne!(prev_suffix, suffix);
	});
}

#[test]
fn test_handle_claim_for_other_msa_with_no_existing_handle() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 100;
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec(), expiration);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof.clone(),
			payload.clone()
		));

		// Try to claim the base handle for another MSA with a different account which should fail
		let bob = sr25519::Pair::from_seed(&[1; 32]);
		let (payload2, proof2) =
			get_signed_claims_payload(&bob, base_handle_str.as_bytes().to_vec(), expiration);
		assert_noop!(
			Handles::claim_handle(
				RuntimeOrigin::signed(bob.public().into()),
				alice.public().into(),
				proof2,
				payload2
			),
			Error::<Test>::InvalidSignature
		);
	});
}

#[test]
fn test_handle_claim_proof_expired() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 10;
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec(), expiration);
		// run to block 10
		run_to_block(11);
		assert_noop!(
			Handles::claim_handle(
				RuntimeOrigin::signed(alice.public().into()),
				alice.public().into(),
				proof,
				payload
			),
			Error::<Test>::ProofHasExpired
		);
	});
}

#[test]
fn test_handle_claim_proof_not_yet_valid() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 200;
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec(), expiration);
		// run to block 9
		run_to_block(9);
		assert_noop!(
			Handles::claim_handle(
				RuntimeOrigin::signed(alice.public().into()),
				alice.public().into(),
				proof,
				payload
			),
			Error::<Test>::ProofNotYetValid
		);
	});
}
