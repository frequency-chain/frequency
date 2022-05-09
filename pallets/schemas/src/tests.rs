use frame_support::assert_ok;
use serial_test::serial;
use sp_core::ed25519::Public;
use sp_keyring::ed25519::Keyring;

use common_primitives::schema::SchemaId;

use crate::{pallet::Error, Event as AnnouncementEvent};

use super::mock::*;

pub mod test {}

//----Tests----
#[test]
fn require_validated_schema() {
	new_test_ext().execute_with(|| {})
}

struct TestCase<T> {
	schema: Vec<u8>,
	expected: T,
}

#[test]
fn require_valid_schema_size_errors() {
	new_test_ext().execute_with(|| {
		let test_cases: [TestCase<Error<Test>>; 2] = [
			TestCase {
				schema: vec![],
				expected: Error::TooShortSchema,
			},
			TestCase {
				schema: Vec::from("foo,bar,bazz,way,wayway,wayway,foo,bar,bazz,way,wayway,wayway,foo,bar,bazz,thisiswaywaywaywaywaywaywaytoolong".as_bytes()),
				expected: Error::TooLongSchema,
			},
		];
		for tc in test_cases {
			let err = SchemasPallet::require_valid_schema_size(tc.schema).unwrap_err();
			assert_eq!(tc.expected, err)
		}
	})
}

#[test]
fn get_latest_schema_count() {
	new_test_ext().execute_with(|| {
		let schema_count = SchemasPallet::schema_count();
		let schema_latest_rpc = SchemasPallet::get_latest_schema_id();
		assert!(schema_count == schema_latest_rpc.unwrap());
	})
}

#[test]
fn register_schema_happy_path() {
	new_test_ext().execute_with(|| {
		let alice = Keyring::Alice.to_raw_public();
		let sender = Public::from_raw(alice);
		let serialized_fields = Vec::from("foo,bar,bazz".as_bytes());

		assert_ok!(SchemasPallet::register_schema(Origin::signed(sender), serialized_fields));
	})
}

#[test]
#[serial]
fn register_schema_id_deposits_events_and_increments_schema_id() {
	new_test_ext().execute_with(|| {
		let alice = Keyring::Alice.to_raw_public();
		let sender = Public::from_raw(alice);
		let mut last_schema_id: SchemaId = 0;
		for fields in ["foo,bar,bazz", "this,is,another,schema", "test,one,two,three"] {
			let serialized_fields = Vec::from(fields.as_bytes());
			assert_ok!(SchemasPallet::register_schema(Origin::signed(sender), serialized_fields));
			let expected_schema_id = last_schema_id + 1;
			System::assert_last_event(
				AnnouncementEvent::SchemaRegistered(sender, expected_schema_id).into(),
			);
			last_schema_id = expected_schema_id;
		}
		let serialized_fields1 = Vec::from("foo,bar,".as_bytes());
		assert_ok!(SchemasPallet::register_schema(Origin::signed(sender), serialized_fields1));
	})
}
