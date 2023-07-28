use crate::{tests::mock::*, Error, Event};
use codec::Decode;
use common_primitives::msa::MessageSourceId;
use frame_support::{assert_err, assert_ok};
use sp_core::{sr25519, Encode, Pair};

#[test]
fn change_handle_happy_path() {
	new_test_ext().execute_with(|| {
		let handle = "MyHandle";
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiry = 100;
		let (payload, proof) =
			get_signed_claims_payload(&alice, handle.as_bytes().to_vec(), expiry);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload
		));

		// Confirm that HandleClaimed event was deposited
		let msa_id = MessageSourceId::decode(&mut &alice.public().encode()[..]).unwrap();
		let full_handle = create_full_handle_for_index(handle, 0);
		System::assert_last_event(Event::HandleClaimed { msa_id, handle: full_handle }.into());

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
		assert_err!(Handles::change_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload
		), Error::<Test>::MSAHandleDoesNotExist);
	});
}
