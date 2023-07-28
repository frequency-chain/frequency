use crate::{tests::mock::*, Error, Event};
use codec::Decode;
use common_primitives::msa::MessageSourceId;
use frame_support::{assert_noop, assert_ok, dispatch::DispatchResult};
use sp_core::{sr25519, Encode, Pair};
use sp_std::collections::btree_set::BTreeSet;

struct TestCase<T> {
	handle: &'static str,
	expected: T,
}

#[test]
fn test_full_handle_creation() {
	new_test_ext().execute_with(|| {
		// Min is 10, Max is 99 inclusive
		for sequence_index in 0..89 {
			let display_handle = create_full_handle_for_index("test", sequence_index);
			assert_ok!(core::str::from_utf8(&display_handle));
		}
	})
}

#[test]
fn claim_handle_happy_path() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiry = 100;
		let (payload, proof) =
			get_signed_claims_payload(&alice, "test1".as_bytes().to_vec(), expiry);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload
		));

		// Confirm that HandleClaimed event was deposited
		let msa_id = MessageSourceId::decode(&mut &alice.public().encode()[..]).unwrap();
		let handle = create_full_handle_for_index("test1", 0);
		System::assert_last_event(Event::HandleClaimed { msa_id, handle }.into());
	});
}

#[test]
fn claim_handle_already_claimed() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 100;

		let test_cases: [TestCase<DispatchResult>; 2] = [
			TestCase { handle: "test1", expected: Ok(()) },
			TestCase {
				handle: "test1",
				expected: Err(Error::<Test>::MSAHandleAlreadyExists.into()),
			},
		];

		for test_case in test_cases {
			let (payload, proof) =
				get_signed_claims_payload(&alice, test_case.handle.as_bytes().to_vec(), expiration);

			assert_eq!(
				Handles::claim_handle(
					RuntimeOrigin::signed(alice.public().into()),
					alice.public().into(),
					proof,
					payload
				),
				test_case.expected
			);
		}
	});
}

#[test]
fn claim_handle_already_claimed_with_different_case() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 100;

		let test_cases: [TestCase<DispatchResult>; 2] = [
			TestCase { handle: "test1", expected: Ok(()) },
			TestCase {
				handle: "TEST1",
				expected: Err(Error::<Test>::MSAHandleAlreadyExists.into()),
			},
		];

		for test_case in test_cases {
			let (payload, proof) =
				get_signed_claims_payload(&alice, test_case.handle.as_bytes().to_vec(), expiration);

			assert_eq!(
				Handles::claim_handle(
					RuntimeOrigin::signed(alice.public().into()),
					alice.public().into(),
					proof,
					payload
				),
				test_case.expected
			);
		}
	});
}

#[test]
fn claim_handle_already_claimed_with_homoglyph() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 100;

		let test_cases: [TestCase<DispatchResult>; 2] = [
			TestCase { handle: "test1", expected: Ok(()) },
			TestCase {
				handle: "tést1",
				expected: Err(Error::<Test>::MSAHandleAlreadyExists.into()),
			},
		];

		for test_case in test_cases {
			let (payload, proof) =
				get_signed_claims_payload(&alice, test_case.handle.as_bytes().to_vec(), expiration);

			assert_eq!(
				Handles::claim_handle(
					RuntimeOrigin::signed(alice.public().into()),
					alice.public().into(),
					proof,
					payload
				),
				test_case.expected
			);
		}
	});
}

#[test]
fn claim_handle_get_msa_handle() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 100;
		let (payload, proof) =
			get_signed_claims_payload(&alice, "test1".as_bytes().to_vec(), expiration);
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
		let expiration = 100;
		let (payload, proof) = get_signed_claims_payload(
			&alice,
			"abcdefghijklmnopqrstuvwxyz0123456789".as_bytes().to_vec(),
			expiration,
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
		let expiration = 100;
		let (payload, proof) =
			get_signed_claims_payload(&alice, "a".as_bytes().to_vec(), expiration);
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
		let expiration = 100;
		let (payload, proof) = get_signed_claims_payload(
			&alice,
			"ℂн𝔸RℒℰᏕ𝔇𝔸𐒴𑣯1𝒩𝓐𑣯𝔸R𝔻Ꮥ".as_bytes().to_vec(),
			expiration,
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
fn test_get_next_suffixes() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 100;
		let (payload, proof) =
			get_signed_claims_payload(&alice, "test1".as_bytes().to_vec(), expiration);
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
		let next_suffixes = Handles::get_next_suffixes(base_handle.try_into().unwrap(), 5);
		assert_eq!(next_suffixes.suffixes.len(), 5);
		let mut presumptive_suffixes = BTreeSet::new();
		for suffix in next_suffixes.suffixes {
			assert!(suffix > 0 && suffix != handle_result.suffix);
			presumptive_suffixes.insert(suffix);
		}
		assert_eq!(presumptive_suffixes.len(), 5);
	});
}

#[test]
fn claim_handle_supports_stripping_diacriticals_from_utf8_with_combining_marks() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 100;

		// Construct a handle "Álvaro" where the first character consists of
		// a base character and a combining mark for an accute accent
		let mut handle_with_combining_mark = String::new();
		handle_with_combining_mark.push('\u{0041}');
		handle_with_combining_mark.push('\u{0301}');
		handle_with_combining_mark.push_str("lvaro");

		let (payload, proof) = get_signed_claims_payload(
			&alice,
			handle_with_combining_mark.as_bytes().to_vec(),
			expiration,
		);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload.clone()
		));
	});
}

#[test]
fn claim_handle_fails_when_handle_contains_unsupported_unicode_characters() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 100;
		let handle_with_unsupported_unicode_characters = "𓅓𓅱𓅱𓆑𓆷";
		let (payload, proof) = get_signed_claims_payload(
			&alice,
			handle_with_unsupported_unicode_characters.as_bytes().to_vec(),
			expiration,
		);
		assert_noop!(
			Handles::claim_handle(
				RuntimeOrigin::signed(alice.public().into()),
				alice.public().into(),
				proof,
				payload.clone()
			),
			Error::<Test>::HandleDoesNotConsistOfSupportedCharacterSets
		);
	});
}

#[test]
fn claim_handle_with_max_bytes_should_get_correct_display_handle() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 100;
		// use 4 bytes character to build a handle with 32 bytes
		let handle = "𝔸".repeat(8);
		let (payload, proof) =
			get_signed_claims_payload(&alice, handle.as_bytes().to_vec(), expiration);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload.clone()
		));
		let msa_id = MessageSourceId::decode(&mut &alice.public().encode()[..]).unwrap();
		let handle = Handles::get_handle_for_msa(msa_id);
		assert!(handle.is_some());
		let handle_result = handle.unwrap();
		assert_eq!(handle_result.base_handle, "𝔸".repeat(8).as_bytes().to_vec());
		assert!(handle_result.suffix > 0);
		let display_handle = "𝔸".repeat(8) + "." + &handle_result.suffix.to_string();
		let display_handle_vec = display_handle.as_bytes().to_vec();
		let msa_id_from_state =
			Handles::get_msa_id_for_handle(display_handle_vec.try_into().unwrap());
		assert!(msa_id_from_state.is_some());
		assert_eq!(msa_id_from_state.unwrap(), msa_id);
	});
}
