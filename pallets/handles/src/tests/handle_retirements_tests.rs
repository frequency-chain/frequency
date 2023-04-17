use crate::{handles_signed_extension::HandlesSignedExtension, tests::mock::*, Error, Event};
use codec::Decode;
use common_primitives::{handles::*, msa::MessageSourceId};
use frame_support::{
	assert_noop, assert_ok,
	dispatch::{DispatchInfo, GetDispatchInfo, Pays},
};
use numtoa::*;
use sp_core::{sr25519, Encode, Pair};
use sp_runtime::traits::SignedExtension;

#[test]
fn claim_and_retire_handle_happy_path() {
	new_test_ext().execute_with(|| {
		let base_handle_str = "test1";

		let alice = sr25519::Pair::from_seed(&[0; 32]);
		let expiration = 10;
		let (payload, proof) =
			get_signed_claims_payload(&alice, base_handle_str.as_bytes().to_vec(), expiration);
		assert_ok!(Handles::claim_handle(
			RuntimeOrigin::signed(alice.public().into()),
			alice.public().into(),
			proof,
			payload
		));
		let msa_id = MessageSourceId::decode(&mut &alice.public().encode()[..]).unwrap();
		let handle = Handles::get_handle_for_msa(msa_id);
		assert!(handle.is_some());

		// Compose the full display handle from the base handle, "." delimiter and suffix
		let handle_result = handle.unwrap();
		let suffix = handle_result.suffix;
		let mut full_handle_vec: Vec<u8> = vec![];
		full_handle_vec.extend(base_handle_str.as_bytes());
		full_handle_vec.extend(b"."); // The delimiter
		let mut buff = [0u8; SUFFIX_MAX_DIGITS];
		full_handle_vec.extend(suffix.numtoa(10, &mut buff)); // Use base 10

		run_to_block(200);
		// Retire the handle
		assert_ok!(Handles::retire_handle(RuntimeOrigin::signed(alice.public().into())));

		// Confirm that HandleRetired event was deposited
		System::assert_last_event(
			Event::HandleRetired { msa_id, handle: full_handle_vec.clone() }.into(),
		);

		// Try to retire again which should fail
		assert_noop!(
			Handles::retire_handle(RuntimeOrigin::signed(alice.public().into()),),
			Error::<Test>::MSAHandleDoesNotExist
		);
	});
}

#[test]
fn test_handle_early_retirement_fails() {
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
fn retire_handle_no_handle() {
	new_test_ext().execute_with(|| {
		let delegator_key_pair = sr25519::Pair::generate().0;
		let delegator_account = delegator_key_pair.public();

		// Payload
		assert_noop!(
			Handles::retire_handle(RuntimeOrigin::signed(delegator_account.into()),),
			Error::<Test>::MSAHandleDoesNotExist
		);
	});
}

#[test]
fn check_free_extrinsic_use() {
	let call = HandlesCall::<Test>::retire_handle {};
	let dispatch_info = call.get_dispatch_info();
	assert_eq!(dispatch_info.pays_fee, Pays::No);
}
