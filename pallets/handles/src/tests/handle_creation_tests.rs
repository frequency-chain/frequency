use crate::{tests::mock::*, Error, Event};
use codec::Decode;
use common_primitives::{handles::HandleSuffix, msa::MessageSourceId};
use frame_support::{assert_noop, assert_ok};
use sp_core::{sr25519, Encode, Pair};
use sp_std::collections::btree_set::BTreeSet;

#[test]
fn test_full_handle_creation() {
	new_test_ext().execute_with(|| {
		// Min is 10, Max is 99 inclusive
		for sequence_index in 0..89 {
			let full_handle = Handles::create_full_handle_for_index("test", sequence_index);
			let full_handle_str = core::str::from_utf8(&full_handle).ok().unwrap();
			println!("full_handle_str={}", full_handle_str);
		}
	})
}

#[test]
fn claim_handle_happy_path() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let (payload, proof) = get_signed_claims_payload(&alice, "test1".as_bytes().to_vec());
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload
		));

		// Confirm that HandleClaimed event was deposited
		let msa_id = MessageSourceId::decode(&mut &alice.public().encode()[..]).unwrap();
		let handle = Handles::create_full_handle_for_index("test1", 0);
		System::assert_last_event(Event::HandleClaimed { msa_id, handle }.into());
	});
}

#[test]
fn claim_handle_already_claimed() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let (payload, proof) = get_signed_claims_payload(&alice, "test1".as_bytes().to_vec());
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload.clone()
		));

		let (payload, proof) = get_signed_claims_payload(&alice, "test1".as_bytes().to_vec());
		assert_noop!(
			Handles::claim_handle(
				RuntimeOrigin::signed(alice.public().into()),
				alice.public().into(),
				proof,
				payload
			),
			Error::<Test>::MSAHandleAlreadyExists
		);
	});
}

#[test]
fn claim_handle_already_claimed_with_different_case() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let (payload, proof) = get_signed_claims_payload(&alice, "test1".as_bytes().to_vec());
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload.clone()
		));

		let (payload, proof) = get_signed_claims_payload(&alice, "TEST1".as_bytes().to_vec());
		assert_noop!(
			Handles::claim_handle(
				RuntimeOrigin::signed(alice.public().into()),
				alice.public().into(),
				proof,
				payload
			),
			Error::<Test>::MSAHandleAlreadyExists
		);
	});
}

#[test]
fn claim_handle_already_claimed_with_homoglyph() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let (payload, proof) = get_signed_claims_payload(&alice, "test1".as_bytes().to_vec());
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload.clone()
		));

		let (payload, proof) = get_signed_claims_payload(&alice, "tést1".as_bytes().to_vec());
		assert_noop!(
			Handles::claim_handle(
				RuntimeOrigin::signed(alice.public().into()),
				alice.public().into(),
				proof,
				payload
			),
			Error::<Test>::MSAHandleAlreadyExists
		);
	});
}

#[test]
fn claim_handle_get_msa_handle() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let (payload, proof) = get_signed_claims_payload(&alice, "test1".as_bytes().to_vec());
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload
		));
		let msa_id = MessageSourceId::decode(&mut &alice.public().encode()[..]).unwrap();
		let handle = Handles::get_handle_for_msa(msa_id);
		assert!(handle.is_some());
		let handle_result = handle.unwrap();
		let base_handle = handle_result.base_handle;
		assert_eq!(base_handle, "test1".as_bytes().to_vec());
		let suffix = handle_result.suffix;
		assert!(suffix > 0);
	});
}

#[test]
fn claim_handle_invalid_length_too_long() {
	// Try to claim a 36 byte handle which is over the byte and character limit
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let (payload, proof) = get_signed_claims_payload(
			&alice,
			"abcdefghijklmnopqrstuvwxyz0123456789".as_bytes().to_vec(),
		);
		assert_noop!(
			Handles::claim_handle(
				RuntimeOrigin::signed(alice.public().into()),
				alice.public().into(),
				proof,
				payload
			),
			Error::<Test>::InvalidHandleByteLength
		);
	});
}

#[test]
fn claim_handle_invalid_length_too_short() {
	// Try to claim a 1 character handle which is under the character limit
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let (payload, proof) = get_signed_claims_payload(&alice, "a".as_bytes().to_vec());
		assert_noop!(
			Handles::claim_handle(
				RuntimeOrigin::signed(alice.public().into()),
				alice.public().into(),
				proof,
				payload
			),
			Error::<Test>::InvalidHandleCharacterLength
		);
	});
}

#[test]
fn claim_handle_invalid_byte_length() {
	// Try to claim a character handle which is over the byte limit but under the character limit
	// ℂн𝔸RℒℰᏕ𝔇𝔸𐒴𑣯1𝒩𝓐𑣯𝔸R𝔻Ꮥ is 19 characters but 61 bytes
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let (payload, proof) =
			get_signed_claims_payload(&alice, "ℂн𝔸RℒℰᏕ𝔇𝔸𐒴𑣯1𝒩𝓐𑣯𝔸R𝔻Ꮥ".as_bytes().to_vec());
		assert_noop!(
			Handles::claim_handle(
				RuntimeOrigin::signed(alice.public().into()),
				alice.public().into(),
				proof,
				payload
			),
			Error::<Test>::InvalidHandleByteLength
		);
	});
}

#[test]
fn test_get_next_suffixes() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let (payload, proof) = get_signed_claims_payload(&alice, "test1".as_bytes().to_vec());
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload
		));
		let msa_id = MessageSourceId::decode(&mut &alice.public().encode()[..]).unwrap();
		let handle = Handles::get_handle_for_msa(msa_id);
		assert!(handle.is_some());
		let handle_result = handle.unwrap();
		let base_handle = handle_result.base_handle;
		assert_eq!(base_handle, "test1".as_bytes().to_vec());
		let suffix = handle_result.suffix;
		assert!(suffix > 0);
		let next_suffixes: Vec<HandleSuffix> = Handles::get_next_suffixes(base_handle.clone(), 5);
		assert_eq!(next_suffixes.len(), 5);
		let mut presumptive_suffixes = BTreeSet::new();
		for suffix in next_suffixes {
			assert!(suffix > 0 && suffix != handle_result.suffix);
			presumptive_suffixes.insert(suffix);
		}
		assert_eq!(presumptive_suffixes.len(), 5);
	});
}
