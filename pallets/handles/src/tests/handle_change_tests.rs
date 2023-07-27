use crate::{tests::mock::*, Error, Event};
use codec::Decode;
use common_primitives::{handles::SequenceIndex, msa::MessageSourceId};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchResult};
use handles_utils::converter::convert_to_canonical;
use sp_core::{sr25519, Encode, Pair};
use sp_std::collections::btree_set::BTreeSet;

#[test]
fn change_handle_happy_path() {
	new_test_ext().execute_with(|| {
        let handle = "test1";
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
	});
}
