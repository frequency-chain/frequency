use crate::{mock::*, Error, Event, MsaIdentifier};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_creates_an_msa_account() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::create(Origin::signed(1)));

		assert_eq!(Msa::get_owner_of(1), Some(1));

		assert_eq!(Msa::get_identifier(), 1);

		System::assert_last_event(Event::MsaCreated { msa_id: 1, key: 1 }.into());
	});
}

#[test]
fn it_throws_msa_identifier_overflow() {
	new_test_ext().execute_with(|| {
		MsaIdentifier::<Test>::set(u64::MAX);

		assert_noop!(Msa::create(Origin::signed(1)), Error::<Test>::MsaIdOverflow);
	});
}

#[test]
#[allow(unused_must_use)]
fn it_does_not_allow_duplicate_keys() {
	new_test_ext().execute_with(|| {
		Msa::create(Origin::signed(1));

		assert_noop!(Msa::create(Origin::signed(1)), Error::<Test>::DuplicatedKey);

		assert_eq!(Msa::get_identifier(), 1);
	});
}
