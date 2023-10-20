use frame_support::{
	assert_noop, assert_ok,
	dispatch::RawOrigin,
	traits::{ChangeMembers, Hash},
	BoundedVec,
};
use serial_test::serial;
use sp_core::{crypto::AccountId32, Encode};
use sp_weights::Weight;

use common_primitives::{
	node::AccountId,
	parquet::{
		column::ParquetColumn,
		column_compression_codec::ColumnCompressionCodec,
		numeric::{ParquetInteger, ParquetNumericType},
		types::ParquetType,
		ParquetModel,
	},
	schema::{ModelType, PayloadLocation, SchemaId, SchemaSetting},
};
use sp_runtime::DispatchError::BadOrigin;

use crate::{Config, Error, Event as AnnouncementEvent};

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

/// Create and return a simple test AccountId32 constructed with the desired integer.
pub fn test_public(n: u8) -> AccountId32 {
	AccountId32::new([n; 32])
}

#[test]
#[allow(deprecated)]
fn require_valid_schema_size_errors() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();
		let test_cases: [TestCase<(Error<Test>, u8)>; 2] = [
			TestCase {
				schema: r#"{"a":1}"#,
				expected: (Error::<Test>::LessThanMinSchemaModelBytes, 3),
			},
			TestCase {
				schema: r#"{"id": "long", "title": "I am a very very very long schema", "properties": "just way too long to live a long life", "description": "Just a never ending stream of bytes that goes on for a minute too long"}"#,
				expected: (Error::<Test>::ExceedsMaxSchemaModelBytes, 2),
			},
		];
		for tc in test_cases {
			assert_noop!(
				SchemasPallet::create_schema(RuntimeOrigin::signed(test_public(1)), create_bounded_schema_vec(tc.schema), ModelType::AvroBinary, PayloadLocation::OnChain),
				tc.expected.0);
		}
	})
}

#[test]
fn create_schema_v2_requires_valid_schema_size() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();
		let test_cases: [TestCase<(Error<Test>, u8)>; 2] = [
			TestCase {
				schema: r#"{"a":1}"#,
				expected: (Error::<Test>::LessThanMinSchemaModelBytes, 3),
			},
			TestCase {
				schema: r#"{"id": "long", "title": "I am a very very very long schema", "properties": "just way too long to live a long life", "description": "Just a never ending stream of bytes that goes on for a minute too long"}"#,
				expected: (Error::<Test>::ExceedsMaxSchemaModelBytes, 2),
			},
		];
		for tc in test_cases {
			assert_noop!(
				SchemasPallet::create_schema_v2(RuntimeOrigin::signed(test_public(1)), create_bounded_schema_vec(tc.schema), ModelType::AvroBinary, PayloadLocation::OnChain, BoundedVec::default()),
				tc.expected.0);
		}
	})
}

#[test]
fn create_schema_via_governance_happy_path() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();
		let sender: AccountId = test_public(5);
		assert_ok!(SchemasPallet::create_schema_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
			sender,
			create_bounded_schema_vec(r#"{"name": "Doe", "type": "lost"}"#),
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
			BoundedVec::default(),
		));
	})
}

/// Test that a request to be a provider, makes the MSA a provider after the council approves it.
#[test]
fn propose_to_create_schema_happy_path() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();

		let test_model = r#"{"foo": "bar", "bar": "buzz"}"#;
		let serialized_fields = Vec::from(test_model.as_bytes());
		// Propose a new schema
		_ = SchemasPallet::propose_to_create_schema(
			test_origin_signed(5),
			create_bounded_schema_vec(test_model),
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
			BoundedVec::default(),
		);

		// Find the Proposed event and get it's hash and index so it can be voted on
		let proposed_events: Vec<(u32, Hash)> = System::events()
			.iter()
			.filter_map(|event| match event.event {
				RuntimeEvent::Council(pallet_collective::Event::Proposed {
					account: _,
					proposal_index,
					proposal_hash,
					threshold: _,
				}) => Some((proposal_index, proposal_hash)),
				_ => None,
			})
			.collect();

		assert_eq!(proposed_events.len(), 1);

		let proposal_index = proposed_events[0].0;
		let proposal_hash = proposed_events[0].1;
		let proposal = Council::proposal_of(proposal_hash).unwrap();
		let proposal_len: u32 = proposal.encoded_size() as u32;

		// Set up the council members
		let council_member_1 = test_public(1); // Use ALICE as a council member
		let council_member_2 = test_public(2); // Use BOB as a council member
		let council_member_3 = test_public(3); // Use CHARLIE as a council member

		let incoming = vec![];
		let outgoing = vec![];
		Council::change_members(
			&incoming,
			&outgoing,
			vec![council_member_1.clone(), council_member_2.clone(), council_member_3.clone()],
		);

		// Council member #1 votes AYE on the proposal
		assert_ok!(Council::vote(
			RuntimeOrigin::signed(council_member_1.clone()),
			proposal_hash,
			proposal_index,
			true
		));
		// Council member #2 votes AYE on the proposal
		assert_ok!(Council::vote(
			RuntimeOrigin::signed(council_member_2.clone()),
			proposal_hash,
			proposal_index,
			true
		));
		// Council member #3 votes NAY on the proposal
		assert_ok!(Council::vote(
			RuntimeOrigin::signed(council_member_3.clone()),
			proposal_hash,
			proposal_index,
			false
		));

		// Find the Voted event and check if it passed
		let voted_events: Vec<(bool, u32, u32)> = System::events()
			.iter()
			.filter_map(|event| match event.event {
				RuntimeEvent::Council(pallet_collective::Event::Voted {
					account: _,
					proposal_hash: _,
					voted,
					yes,
					no,
				}) => Some((voted, yes, no)),
				_ => None,
			})
			.collect();

		assert_eq!(voted_events.len(), 3);
		assert_eq!(voted_events[1].1, 2); // There should be two AYE (out of three) votes to pass

		// Close the voting
		assert_ok!(Council::close(
			RuntimeOrigin::signed(test_public(5)),
			proposal_hash,
			proposal_index,
			Weight::MAX,
			proposal_len
		));

		// Find the Closed event and check if it passed
		let closed_events: Vec<(u32, u32)> = System::events()
			.iter()
			.filter_map(|event| match event.event {
				RuntimeEvent::Council(pallet_collective::Event::Closed {
					proposal_hash: _,
					yes,
					no,
				}) => Some((yes, no)),
				_ => None,
			})
			.collect();

		assert_eq!(closed_events.len(), 1);
		assert_eq!(closed_events[0].0, 2); // There should be two YES votes to pass

		// Find the SchemaCreated event and check if it passed
		let schema_events: Vec<SchemaId> = System::events()
			.iter()
			.filter_map(|event| match event.event {
				RuntimeEvent::SchemasPallet(AnnouncementEvent::SchemaCreated {
					key: _,
					schema_id,
				}) => Some(schema_id),
				_ => None,
			})
			.collect();

		// Confirm that the schema was created
		assert_eq!(schema_events.len(), 1);

		let last_schema_id = schema_events[0];
		let created_schema = SchemasPallet::get_schema_by_id(last_schema_id);
		assert_eq!(created_schema.as_ref().is_some(), true);
		assert_eq!(created_schema.as_ref().unwrap().clone().model, serialized_fields);
	})
}

#[allow(deprecated)]
#[test]
fn create_schema_happy_path() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();
		let sender: AccountId = test_public(1);
		assert_ok!(SchemasPallet::create_schema(
			RuntimeOrigin::signed(sender),
			create_bounded_schema_vec(r#"{"name": "Doe", "type": "lost"}"#),
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
		));
	})
}

#[test]
fn create_schema_v2_happy_path() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();
		let sender: AccountId = test_public(1);
		assert_ok!(SchemasPallet::create_schema_v2(
			RuntimeOrigin::signed(sender),
			create_bounded_schema_vec(r#"{"name": "Doe", "type": "lost"}"#),
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
			BoundedVec::default()
		));
	})
}

#[allow(deprecated)]
#[test]
fn create_schema_unhappy_path() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();
		let sender: AccountId = test_public(1);
		assert_noop!(
			SchemasPallet::create_schema(
				RuntimeOrigin::signed(sender),
				// name key does not have a colon
				create_bounded_schema_vec(r#"{"name", 54, "type": "none"}"#),
				ModelType::AvroBinary,
				PayloadLocation::OnChain,
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
		let sender: AccountId = test_public(1);
		let expected_err = BadOrigin;
		assert_noop!(
			SchemasPallet::set_max_schema_model_bytes(RuntimeOrigin::signed(sender), new_size),
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

#[allow(deprecated)]
#[test]
#[serial]
fn create_schema_id_deposits_events_and_increments_schema_id() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();
		let sender: AccountId = test_public(1);
		let mut last_schema_id: SchemaId = 0;
		for fields in [
			r#"{"Name": "Bond", "Code": "007"}"#,
			r#"{"type": "num","minimum": -90,"maximum": 90}"#,
			r#"{"latitude": 48.858093,"longitude": 2.294694}"#,
		] {
			let expected_schema_id = last_schema_id + 1;
			assert_ok!(SchemasPallet::create_schema(
				RuntimeOrigin::signed(sender.clone()),
				create_bounded_schema_vec(fields),
				ModelType::AvroBinary,
				PayloadLocation::OnChain,
			));
			System::assert_last_event(
				AnnouncementEvent::SchemaCreated {
					key: sender.clone(),
					schema_id: expected_schema_id,
				}
				.into(),
			);
			last_schema_id = expected_schema_id;
		}
		assert_ok!(SchemasPallet::create_schema(
			RuntimeOrigin::signed(sender.clone()),
			create_bounded_schema_vec(r#"{"account":3050}"#),
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
		));
	})
}

#[test]
#[serial]
fn create_schema_v2_id_deposits_events_and_increments_schema_id() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();
		let sender: AccountId = test_public(1);
		let mut last_schema_id: SchemaId = 0;
		for fields in [
			r#"{"Name": "Bond", "Code": "007"}"#,
			r#"{"type": "num","minimum": -90,"maximum": 90}"#,
			r#"{"latitude": 48.858093,"longitude": 2.294694}"#,
		] {
			let expected_schema_id = last_schema_id + 1;
			assert_ok!(SchemasPallet::create_schema_v2(
				RuntimeOrigin::signed(sender.clone()),
				create_bounded_schema_vec(fields),
				ModelType::AvroBinary,
				PayloadLocation::OnChain,
				BoundedVec::default()
			));
			System::assert_last_event(
				AnnouncementEvent::SchemaCreated {
					key: sender.clone(),
					schema_id: expected_schema_id,
				}
				.into(),
			);
			last_schema_id = expected_schema_id;
		}
		assert_ok!(SchemasPallet::create_schema_v2(
			RuntimeOrigin::signed(sender.clone()),
			create_bounded_schema_vec(r#"{"account":3050}"#),
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
			BoundedVec::default()
		));
	})
}

#[allow(deprecated)]
#[test]
fn get_existing_schema_by_id_should_return_schema() {
	new_test_ext().execute_with(|| {
		let sender: AccountId = test_public(1);
		sudo_set_max_schema_size();
		// arrange
		let test_str = r#"{"foo": "bar", "bar": "buzz"}"#;
		let serialized_fields = Vec::from(test_str.as_bytes());
		assert_ok!(SchemasPallet::create_schema(
			RuntimeOrigin::signed(sender),
			create_bounded_schema_vec(test_str),
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
		));

		// act
		let res = SchemasPallet::get_schema_by_id(1);

		// assert
		assert_eq!(res.as_ref().is_some(), true);
		assert_eq!(res.as_ref().unwrap().clone().model, serialized_fields);
	})
}

#[test]
fn get_existing_schema_by_id_should_return_schema_v2() {
	new_test_ext().execute_with(|| {
		let sender: AccountId = test_public(1);
		sudo_set_max_schema_size();
		// arrange
		let test_str = r#"{"foo": "bar", "bar": "buzz"}"#;
		let serialized_fields = Vec::from(test_str.as_bytes());
		assert_ok!(SchemasPallet::create_schema_v2(
			RuntimeOrigin::signed(sender),
			create_bounded_schema_vec(test_str),
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
			BoundedVec::default()
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
					sign: false,
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

#[test]
fn create_schema_with_settings_should_work() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();

		// arrange
		let settings = vec![SchemaSetting::AppendOnly];
		let sender: AccountId = test_public(1);

		// act and assert
		assert_ok!(SchemasPallet::create_schema_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
			sender,
			create_bounded_schema_vec(r#"{"name":"John Doe"}"#),
			ModelType::AvroBinary,
			PayloadLocation::Itemized,
			BoundedVec::try_from(settings.clone()).unwrap(),
		));

		// assert
		let res = SchemasPallet::get_schema_by_id(1);
		assert_eq!(res.unwrap().settings, settings);
	})
}

#[test]
fn create_schema_with_append_only_setting_and_non_itemized_should_fail() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();

		// arrange
		let settings = vec![SchemaSetting::AppendOnly];
		let sender: AccountId = test_public(1);
		// act and assert
		assert_noop!(
			SchemasPallet::create_schema_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
				sender.clone(),
				create_bounded_schema_vec(r#"{"name":"John Doe"}"#),
				ModelType::AvroBinary,
				PayloadLocation::Paginated,
				BoundedVec::try_from(settings.clone()).unwrap(),
			),
			Error::<Test>::InvalidSetting
		);

		// act and assert
		assert_noop!(
			SchemasPallet::create_schema_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
				sender.clone(),
				create_bounded_schema_vec(r#"{"name":"John Doe"}"#),
				ModelType::AvroBinary,
				PayloadLocation::OnChain,
				BoundedVec::try_from(settings.clone()).unwrap(),
			),
			Error::<Test>::InvalidSetting
		);

		assert_noop!(
			SchemasPallet::create_schema_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
				sender,
				create_bounded_schema_vec(r#"{"name":"John Doe"}"#),
				ModelType::AvroBinary,
				PayloadLocation::IPFS,
				BoundedVec::try_from(settings.clone()).unwrap(),
			),
			Error::<Test>::InvalidSetting
		);
	})
}
