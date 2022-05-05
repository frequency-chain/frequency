use crate::{mock::*, pallet::Error};
use frame_support::{
	assert_err, assert_noop, assert_ok,
	traits::{OnFinalize, OnInitialize},
	BoundedVec,
};

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
		let test_cases: [TestCase<Error<Test>>; 3] = [
			TestCase {
				schema: vec![],
				expected: Error::TooShortSchema,
			},
			TestCase {
				schema: Vec::from("foo".as_bytes()),
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
