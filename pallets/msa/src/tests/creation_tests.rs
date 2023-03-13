use frame_support::{
	assert_noop, assert_ok,
	dispatch::{GetDispatchInfo, Weight},
};

use crate::{tests::mock::*, CurrentMsaIdentifierMaximum, Error, Event};

use common_primitives::msa::MessageSourceId;

#[test]
fn it_create_has_weight() {
	new_test_ext().execute_with(|| {
		let call = MsaCall::<Test>::create {};
		let dispatch_info = call.get_dispatch_info();

		assert!(dispatch_info.weight.ref_time() > Weight::from_ref_time(10_000 as u64).ref_time());
	});
}

#[test]
fn it_creates_an_msa_account() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::create(test_origin_signed(1)));

		assert_eq!(Msa::get_msa_by_public_key(test_public(1)), Some(1 as MessageSourceId));

		assert_eq!(Msa::get_current_msa_identifier_maximum(), 1);

		System::assert_last_event(Event::MsaCreated { msa_id: 1, key: test_public(1) }.into());
	});
}

#[test]
fn it_throws_msa_identifier_overflow() {
	new_test_ext().execute_with(|| {
		CurrentMsaIdentifierMaximum::<Test>::set(u64::MAX);

		assert_noop!(Msa::create(test_origin_signed(1)), Error::<Test>::MsaIdOverflow);
	});
}

#[test]
#[allow(unused_must_use)]
fn it_does_not_allow_duplicate_keys() {
	new_test_ext().execute_with(|| {
		Msa::create(test_origin_signed(1));

		assert_noop!(Msa::create(test_origin_signed(1)), Error::<Test>::KeyAlreadyRegistered);

		assert_eq!(Msa::get_current_msa_identifier_maximum(), 1);
	});
}
