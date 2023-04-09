use crate::{tests::mock::*, Error, Event};
use codec::Decode;
use common_primitives::{handles::*, msa::MessageSourceId};
use frame_support::{assert_noop, assert_ok};
use numtoa::*;
use sp_core::{sr25519, Encode, Pair};

#[test]
fn test_handle_claim_replay_attack() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec());
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

		// Retire the handle
		assert_ok!(Handles::retire_handle(RuntimeOrigin::signed(alice.public().into()),));

		// Confirm that HandleRetired event was deposited
		System::assert_last_event(
			Event::HandleRetired { msa_id, handle: full_handle_vec.clone() }.into(),
		);

		// Try to claim the same handle again with the same payload which should fail
		assert_noop!(
			Handles::claim_handle(
				RuntimeOrigin::signed(alice.public().into()),
				alice.public().into(),
				proof,
				payload
			),
			Error::<Test>::SignatureAlreadySubmitted
		);
	});
}

#[test]
fn test_handle_claim_replay_attack_with_different_account() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec());
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

		// Try to claim the same handle again with a different account which should fail
		let bob = sr25519::Pair::from_seed(&[1; 32]);
		assert_noop!(
			Handles::claim_handle(
				RuntimeOrigin::signed(bob.public().into()),
				bob.public().into(),
				proof,
				payload
			),
			Error::<Test>::SignatureAlreadySubmitted
		);
	});
}

#[test]
fn test_handle_retire_replay_attack() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec());
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

		// Retire the handle
		assert_ok!(Handles::retire_handle(RuntimeOrigin::signed(alice.public().into()),));

		// Confirm that HandleRetired event was deposited
		System::assert_last_event(
			Event::HandleRetired { msa_id, handle: full_handle_vec.clone() }.into(),
		);

		// Try to retire the same handle again which should fail
		assert_noop!(
			Handles::retire_handle(RuntimeOrigin::signed(alice.public().into()),),
			Error::<Test>::HandleNotRegistered
		);
	});
}
