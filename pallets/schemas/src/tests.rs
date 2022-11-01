use crate::{Config, Error, Event as AnnouncementEvent};
use common_primitives::{
	parquet::{
		column::ParquetColumn,
		column_compression_codec::ColumnCompressionCodec,
		numeric::{ParquetInteger, ParquetNumericType},
		types::ParquetType,
		ParquetModel,
	},
	schema::{ModelType, PayloadLocation, SchemaId},
};
use frame_support::{assert_noop, assert_ok, dispatch::RawOrigin, BoundedVec};
use serial_test::serial;
use sp_runtime::DispatchError::BadOrigin;

use super::mock::*;

fn create_bounded_schema_vec(
	from_string: &str,
) -> BoundedVec<u8, <Test as Config>::SchemaModelMaxBytesBoundedVecLimit> {
	let fields_vec = Vec::from(from_string.as_bytes());
	BoundedVec::try_from(fields_vec).unwrap()
}

fn sudo_set_max_schema_size() {
	assert_ok!(SchemasPallet::set_max_schema_model_bytes(RawOrigin::Root.into(), 70));
}

pub mod test {}

struct TestCase<T> {
	schema: &'static str,
	expected: T,
}

#[test]
fn require_valid_schema_size_errors() {
	new_test_ext().execute_with(|| {
		let sender: AccountId = 1;
		sudo_set_max_schema_size();
		let test_cases: [TestCase<(Error<Test>, u8)>; 2] = [
			TestCase {
				schema: r#"{"a":1}"#,
				expected: (Error::<Test>::LessThanMinSchemaModelBytes,3),
			},
			TestCase {
				schema: r#"{"id": "long", "title": "I am a very very very long schema", "properties": "just way too long to live a long life", "description": "Just a never ending stream of bytes that goes on for a minute too long"}"#,
				expected: (Error::<Test>::ExceedsMaxSchemaModelBytes, 2),
			},
		];
		for tc in test_cases {
			assert_noop!(
				SchemasPallet::create_schema(Origin::signed(sender), create_bounded_schema_vec(tc.schema), ModelType::AvroBinary, PayloadLocation::OnChain),
				tc.expected.0);
		}
	})
}

#[test]
fn register_schema_happy_path() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();
		let sender: AccountId = 1;
		assert_ok!(SchemasPallet::create_schema(
			Origin::signed(sender),
			create_bounded_schema_vec(r#"{"name": "Doe", "type": "lost"}"#),
			ModelType::AvroBinary,
			PayloadLocation::OnChain
		));
	})
}

#[test]
fn register_schema_unhappy_path() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();
		let sender: AccountId = 1;
		assert_noop!(
			SchemasPallet::create_schema(
				Origin::signed(sender),
				// name key does not have a colon
				create_bounded_schema_vec(r#"{"name", 54, "type": "none"}"#),
				ModelType::AvroBinary,
				PayloadLocation::OnChain
			),
			Error::<Test>::InvalidSchema
		);
	})
}

#[test]
fn set_max_schema_size_works_if_root() {
	new_test_ext().execute_with(|| {
		let new_size: u32 = 42;
		assert_ok!(SchemasPallet::set_max_schema_model_bytes(RawOrigin::Root.into(), new_size));
		let new_schema_size = SchemasPallet::get_schema_model_max_bytes();
		assert_eq!(new_size, new_schema_size);
	})
}

#[test]
fn set_max_schema_size_fails_if_not_root() {
	new_test_ext().execute_with(|| {
		let new_size: u32 = 42;
		let sender: AccountId = 1;
		let expected_err = BadOrigin;
		assert_noop!(
			SchemasPallet::set_max_schema_model_bytes(Origin::signed(sender), new_size),
			expected_err
		);
	})
}

#[test]
fn set_max_schema_size_fails_if_larger_than_bound() {
	new_test_ext().execute_with(|| {
		let new_size: u32 = 68_000;
		let expected_err = Error::<Test>::ExceedsMaxSchemaModelBytes;
		assert_noop!(
			SchemasPallet::set_max_schema_model_bytes(RawOrigin::Root.into(), new_size),
			expected_err
		);
	})
}

#[test]
#[serial]
fn register_schema_id_deposits_events_and_increments_schema_id() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();
		let sender: AccountId = 1;
		let mut last_schema_id: SchemaId = 0;
		for fields in [
			r#"{"Name": "Bond", "Code": "007"}"#,
			r#"{"type": "num","minimum": -90,"maximum": 90}"#,
			r#"{"latitude": 48.858093,"longitude": 2.294694}"#,
		] {
			let expected_schema_id = last_schema_id + 1;
			assert_ok!(SchemasPallet::create_schema(
				Origin::signed(sender),
				create_bounded_schema_vec(fields),
				ModelType::AvroBinary,
				PayloadLocation::OnChain
			));
			System::assert_last_event(
				AnnouncementEvent::SchemaRegistered(sender, expected_schema_id).into(),
			);
			last_schema_id = expected_schema_id;
		}
		assert_ok!(SchemasPallet::create_schema(
			Origin::signed(sender),
			create_bounded_schema_vec(r#"{"account":3050}"#),
			ModelType::AvroBinary,
			PayloadLocation::OnChain
		));
	})
}

#[test]
fn get_existing_schema_by_id_should_return_schema() {
	new_test_ext().execute_with(|| {
		let sender: AccountId = 1;
		sudo_set_max_schema_size();
		// arrange
		let test_str = r#"{"foo": "bar", "bar": "buzz"}"#;
		let serialized_fields = Vec::from(test_str.as_bytes());
		assert_ok!(SchemasPallet::create_schema(
			Origin::signed(sender),
			create_bounded_schema_vec(test_str),
			ModelType::AvroBinary,
			PayloadLocation::OnChain
		));

		// act
		let res = SchemasPallet::get_schema_by_id(1);

		// assert
		assert_eq!(res.as_ref().is_some(), true);
		assert_eq!(res.as_ref().unwrap().clone().model, serialized_fields);
	})
}

#[test]
fn get_non_existing_schema_by_id_should_return_none() {
	new_test_ext().execute_with(|| {
		// act
		let res = SchemasPallet::get_schema_by_id(1);

		// assert
		assert_eq!(res.as_ref().is_none(), true);
	})
}

#[test]
fn validate_schema_is_acceptable() {
	new_test_ext().execute_with(|| {
		let test_str_raw = r#"{"name":"John Doe"}"#;
		let result = SchemasPallet::ensure_valid_model(
			&ModelType::AvroBinary,
			&create_bounded_schema_vec(test_str_raw),
		);
		assert_ok!(result);
	});
}

#[test]
fn reject_null_json_schema() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			SchemasPallet::ensure_valid_model(
				&ModelType::AvroBinary,
				&create_bounded_schema_vec("")
			),
			Error::<Test>::InvalidSchema
		);
	})
}

#[test]
fn serialize_parquet_column() {
	new_test_ext().execute_with(|| {
		let p: ParquetColumn = ParquetColumn::new(
			"Foo".to_string(),
			ParquetType::default(),
			ColumnCompressionCodec::default(),
			true,
		);
		assert_eq!(
			serde_json::to_string(&p).unwrap(),
			r#"{"name":"Foo","column_type":"BOOLEAN","compression":"UNCOMPRESSED","bloom_filter":true}"#
		);
	})
}

#[test]
fn validate_parquet_model() {
	new_test_ext().execute_with(|| {
		let test_str_raw = r#"[{"name": "Foo", "column_type": "BOOLEAN", "compression": "UNCOMPRESSED", "bloom_filter": true}]"#;
		let result = SchemasPallet::ensure_valid_model(&ModelType::Parquet, &create_bounded_schema_vec(test_str_raw));
		assert_ok!(result);
	});
}

#[test]
fn reject_incorrect_parquet_model() {
	new_test_ext().execute_with(|| {
		let test_str_raw = r#"{"name":"John Doe"}"#;
		assert_noop!(
			SchemasPallet::ensure_valid_model(
				&ModelType::Parquet,
				&create_bounded_schema_vec(test_str_raw)
			),
			Error::<Test>::InvalidSchema
		);
	})
}

#[test]
fn serialize_parquet_model() {
	new_test_ext().execute_with(|| {
		let p: ParquetModel = vec![ParquetColumn::new(
			"Baz".to_string(),
			ParquetType::default(),
			ColumnCompressionCodec::default(),
			true,
		)];
		assert_eq!(
			serde_json::to_string(&p).unwrap(),
			r#"[{"name":"Baz","column_type":"BOOLEAN","compression":"UNCOMPRESSED","bloom_filter":true}]"#
		);
	});
}

#[test]
fn serialize_parquet_model_integer() {
	new_test_ext().execute_with(|| {
		let p: ParquetModel = vec![ParquetColumn::new(
			"Baz".to_string(),
			ParquetType::NumericType(ParquetNumericType::Integer(
				ParquetInteger {
					bit_width: 32,
					sign: false
				}
			)),
			ColumnCompressionCodec::default(),
			true,
		)];
		assert_eq!(
			serde_json::to_string(&p).unwrap(),
			r#"[{"name":"Baz","column_type":{"INTEGER":{"bit_width":32,"sign":false}},"compression":"UNCOMPRESSED","bloom_filter":true}]"#
		);
	});
}

#[test]
fn validate_parquet_model_integer() {
	new_test_ext().execute_with(|| {
		let test_str_raw = r#"[{"name":"Baz","column_type":{"INTEGER":{"bit_width":32,"sign":false}},"compression":"UNCOMPRESSED","bloom_filter":true}]"#;
		let result = SchemasPallet::ensure_valid_model(&ModelType::Parquet, &create_bounded_schema_vec(test_str_raw));
		assert_ok!(result);
	});
}

#[test]
fn dsnp_broadcast() {
	let test_str_raw = r#"
	[
		{
			"name": "announcementType",
			"column_type": {
				"INTEGER": {
					"bit_width": 32,
					"sign": true
				}
			},
			"compression": "GZIP",
			"bloom_filter": false
		},
		{
			"name": "contentHash",
			"column_type": "BYTE_ARRAY",
			"compression": "SNAPPY",
			"bloom_filter": true
		},
		{
			"name": "fromId",
			"column_type": {
				"INTEGER": {
					"bit_width": 64,
					"sign": false
				}
			},
			"compression": "UNCOMPRESSED",
			"bloom_filter": true
		},
		{
			"name": "url",
			"column_type": "STRING",
			"compression": "LZO",
			"bloom_filter": false
		}
	]
	"#;
	let result = SchemasPallet::ensure_valid_model(
		&ModelType::Parquet,
		&create_bounded_schema_vec(test_str_raw),
	);
	assert_ok!(result);
}
