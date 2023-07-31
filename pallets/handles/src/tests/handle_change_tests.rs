use crate::{tests::mock::*, Error, Event};
use codec::Decode;
use common_primitives::{handles::HANDLE_BASE_BYTES_MAX, msa::MessageSourceId};
use frame_support::{assert_err, assert_ok};
use sp_core::{sr25519, Encode, Pair};

#[test]
fn change_handle_happy_path() {
	new_test_ext().execute_with(|| {
		let handle_str = "MyHandle";
		let handle = handle_str.as_bytes().to_vec();
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiry = 100;
		let (payload, proof) = get_signed_claims_payload(&alice, handle.clone(), expiry);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload
		));

		// Confirm that HandleClaimed event was deposited
		let msa_id = MessageSourceId::decode(&mut &alice.public().encode()[..]).unwrap();
		let full_handle = create_full_handle_for_index(handle_str, 0);
		System::assert_last_event(
			Event::HandleClaimed { msa_id, handle: full_handle.clone() }.into(),
		);

		let new_handle = "MyNewHandle";
		let (payload, proof) =
			get_signed_claims_payload(&alice, new_handle.as_bytes().to_vec(), expiry);
		assert_ok!(Handles::change_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload
		));
		let changed_handle = create_full_handle_for_index(new_handle, 0);
		System::assert_has_event(Event::HandleRetired { msa_id, handle: full_handle }.into());
		System::assert_last_event(Event::HandleClaimed { msa_id, handle: changed_handle }.into());
	});
}

#[test]
fn change_handle_no_handle() {
	new_test_ext().execute_with(|| {
		let handle = "MyNewHandleWithoutAnOldOne";
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiry = 100;
		let (payload, proof) =
			get_signed_claims_payload(&alice, handle.as_bytes().to_vec(), expiry);
		assert_err!(
			Handles::change_handle(
				RuntimeOrigin::signed(alice.public().into()),
				alice.public().into(),
				proof,
				payload
			),
			Error::<Test>::MSAHandleDoesNotExist
		);
	});
}

#[test]
fn test_max_handle_length_in_bytes() {
	new_test_ext().execute_with(|| {
		// Max bytes handle is ok
		let handle_str: String =
			std::iter::repeat('*').take((HANDLE_BASE_BYTES_MAX) as usize).collect();
		let handle = handle_str.as_bytes().to_vec();
		assert_ok!(Handles::verify_handle_length(handle));

		// However, max bytes handle plus 1 is not ok
		let handle_str: String =
			std::iter::repeat('*').take((HANDLE_BASE_BYTES_MAX + 1) as usize).collect();
		let handle = handle_str.as_bytes().to_vec();
		assert_err!(Handles::verify_handle_length(handle), Error::<Test>::InvalidHandleByteLength);
	});
}
