use crate::{mock::*, pallet::*};
use frame_support::{
	assert_ok,  assert_err, assert_noop, BoundedVec,
	traits::{OnFinalize,OnInitialize}
};
// use frame_system::{EventRecord, Phase};
// use sp_core::{Hasher, H256, KeccakHasher};
// use sp_runtime::DispatchError;
// use sp_std::convert::TryInto;
//
// use sp_core::ed25519::Public;

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

