use crate::{homoglyphs::canonical::HandleConverter, tests::mock::*, Error};
use common_primitives::{handles::*, utils::wrap_binary_data};
use frame_support::{assert_noop, assert_ok};
use sp_core::{sr25519, Encode, Pair};
use sp_runtime::MultiSignature;

fn get_signed_claims_payload(
	account: &sr25519::Pair,
	handle: Vec<u8>,
) -> (ClaimHandlePayload, MultiSignature) {
	let base_handle = handle;
	let payload = ClaimHandlePayload::new(base_handle.clone());
	let encoded_payload = wrap_binary_data(payload.encode());
	let proof: MultiSignature = account.sign(&encoded_payload).into();

	(payload, proof)
}

#[test]
fn claim_handle() {
	new_test_ext().execute_with(|| {
		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let (payload, proof) = get_signed_claims_payload(&alice, "test1".as_bytes().to_vec());
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload
		));
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

		// Try to claim the same handle again with a different account
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

		// Try to claim the same handle again with a different account
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

		// Try to claim the same handle again with a different account
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
