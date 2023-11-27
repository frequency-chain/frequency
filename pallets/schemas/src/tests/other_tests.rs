use common_primitives::{
	node::AccountId,
	parquet::{
		column::ParquetColumn,
		column_compression_codec::ColumnCompressionCodec,
		numeric::{ParquetInteger, ParquetNumericType},
		types::ParquetType,
		ParquetModel,
	},
	schema::{
		ModelType, PayloadLocation, SchemaId, SchemaSetting, SchemaVersion, SchemaVersionResponse,
	},
};
use frame_support::{
	assert_noop, assert_ok,
	dispatch::RawOrigin,
	traits::{ChangeMembers, Hash},
	weights::Weight,
	BoundedVec,
};
use parity_scale_codec::Encode;
use serial_test::serial;
use sp_runtime::DispatchError::BadOrigin;

use crate::{
	Error, Event as AnnouncementEvent, SchemaDescriptor, SchemaName, SchemaNamePayload,
	SchemaNamespace, SchemaVersionId, MAX_NUMBER_OF_VERSIONS,
};

use super::mock::*;

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
fn schema_name_try_parse_with_strict_invalid_names_should_fail() {
	new_test_ext().execute_with(|| {
		let test_cases = [
			TestCase {
				input: r#"¥¤¤.©©©"#, expected: Error::<Test>::InvalidSchemaNameEncoding
			},
			TestCase {
				input: r#"1asbd.hgd"#,
				expected: Error::<Test>::InvalidSchemaNameCharacters,
			},
			TestCase {
				input: r#"asbd.hg1d"#,
				expected: Error::<Test>::InvalidSchemaNameCharacters,
			},
			TestCase {
				input: r#"asbd.hg@d"#,
				expected: Error::<Test>::InvalidSchemaNameCharacters,
			},
			TestCase { input: r#"asbd"#, expected: Error::<Test>::InvalidSchemaNameStructure },
			TestCase {
				input: r#"asbd.sdhks.shd"#,
				expected: Error::<Test>::InvalidSchemaNameStructure,
			},
			TestCase {
				input: r#"-asbdsdhks.shd"#,
				expected: Error::<Test>::InvalidSchemaNameStructure,
			},
			TestCase {
				input: r#"asbdsdhks-.shd"#,
				expected: Error::<Test>::InvalidSchemaNameStructure,
			},
			TestCase {
				input: r#"asbdsdhks.-shd"#,
				expected: Error::<Test>::InvalidSchemaNameStructure,
			},
			TestCase {
				input: r#"asbdsdhks.shd-"#,
				expected: Error::<Test>::InvalidSchemaNameStructure,
			},
			TestCase {
				input: r#"hjsagdhjsagjhgdshjagsadhjsaaaaa."#,
				expected: Error::<Test>::InvalidSchemaNamespaceLength,
			},
			TestCase { input: r#"a.sdhks"#, expected: Error::<Test>::InvalidSchemaNamespaceLength },
			TestCase {
				input: r#"aa.sdhks"#,
				expected: Error::<Test>::InvalidSchemaNamespaceLength,
			},
			TestCase { input: r#"hjs."#, expected: Error::<Test>::InvalidSchemaDescriptorLength },
		];
		for tc in test_cases {
			let payload: SchemaNamePayload =
				BoundedVec::try_from(tc.input.to_string().into_bytes()).expect("should convert");
			assert_noop!(SchemaName::try_parse::<Test>(payload, true), tc.expected);
		}
	});
}

#[test]
fn schema_name_try_parse_with_non_strict_invalid_names_should_fail() {
	new_test_ext().execute_with(|| {
		let test_cases = [
			TestCase {
				input: r#"¥¤¤.©©©"#, expected: Error::<Test>::InvalidSchemaNameEncoding
			},
			TestCase { input: r#"¥¤¤"#, expected: Error::<Test>::InvalidSchemaNameEncoding },
			TestCase {
				input: r#"1asbd.hgd"#,
				expected: Error::<Test>::InvalidSchemaNameCharacters,
			},
			TestCase { input: r#"1asbd"#, expected: Error::<Test>::InvalidSchemaNameCharacters },
			TestCase {
				input: r#"asbd.hg1d"#,
				expected: Error::<Test>::InvalidSchemaNameCharacters,
			},
			TestCase {
				input: r#"asbd.hg@d"#,
				expected: Error::<Test>::InvalidSchemaNameCharacters,
			},
			TestCase { input: r#"hg@d"#, expected: Error::<Test>::InvalidSchemaNameCharacters },
			TestCase {
				input: r#"asbd.sdhks.shd"#,
				expected: Error::<Test>::InvalidSchemaNameStructure,
			},
			TestCase {
				input: r#"-asbdsdhks.shd"#,
				expected: Error::<Test>::InvalidSchemaNameStructure,
			},
			TestCase {
				input: r#"asbdsdhks-.shd"#,
				expected: Error::<Test>::InvalidSchemaNameStructure,
			},
			TestCase {
				input: r#"hjsagdhjsagjhgdshjagsadhjsaaaaa."#,
				expected: Error::<Test>::InvalidSchemaNamespaceLength,
			},
			TestCase {
				input: r#"hjsagdhjsagjhgdshjagsadhjsaaaaa"#,
				expected: Error::<Test>::InvalidSchemaNamespaceLength,
			},
			TestCase { input: r#"a.sdhks"#, expected: Error::<Test>::InvalidSchemaNamespaceLength },
			TestCase { input: r#"a"#, expected: Error::<Test>::InvalidSchemaNamespaceLength },
			TestCase {
				input: r#"aa.sdhks"#,
				expected: Error::<Test>::InvalidSchemaNamespaceLength,
			},
			TestCase { input: r#"aa"#, expected: Error::<Test>::InvalidSchemaNamespaceLength },
			TestCase { input: r#"hjs."#, expected: Error::<Test>::InvalidSchemaDescriptorLength },
		];
		for tc in test_cases {
			let payload: SchemaNamePayload =
				BoundedVec::try_from(tc.input.to_string().into_bytes()).expect("should convert");
			assert_noop!(SchemaName::try_parse::<Test>(payload, false), tc.expected);
		}
	});
}

#[test]
fn schema_name_try_parse_with_strict_valid_names_should_succeed() {
	new_test_ext().execute_with(|| {
		let valid_names = vec!["Abc.a", "a-v.D-D", "aZxcvBnmkjhgfds.asdfghKkloiuyTre"];
		let parsed_names = vec![
			SchemaName {
				namespace: SchemaNamespace::try_from("abc".to_string().into_bytes()).unwrap(),
				descriptor: SchemaDescriptor::try_from("a".to_string().into_bytes()).unwrap(),
			},
			SchemaName {
				namespace: SchemaNamespace::try_from("a-v".to_string().into_bytes()).unwrap(),
				descriptor: SchemaDescriptor::try_from("d-d".to_string().into_bytes()).unwrap(),
			},
			SchemaName {
				namespace: SchemaNamespace::try_from("azxcvbnmkjhgfds".to_string().into_bytes())
					.unwrap(),
				descriptor: SchemaDescriptor::try_from("asdfghkkloiuytre".to_string().into_bytes())
					.unwrap(),
			},
		];
		for (name, result) in valid_names.iter().zip(parsed_names) {
			let payload: SchemaNamePayload =
				BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");
			assert_eq!(SchemaName::try_parse::<Test>(payload, true), Ok(result));
		}
	});
}

#[test]
fn schema_name_try_parse_with_non_strict_valid_names_should_succeed() {
	new_test_ext().execute_with(|| {
		let valid_names = vec!["Abc", "a-v", "aZxcvBnmkjhgfds"];
		let parsed_names = vec![
			SchemaName {
				namespace: SchemaNamespace::try_from("abc".to_string().into_bytes()).unwrap(),
				descriptor: SchemaDescriptor::default(),
			},
			SchemaName {
				namespace: SchemaNamespace::try_from("a-v".to_string().into_bytes()).unwrap(),
				descriptor: SchemaDescriptor::default(),
			},
			SchemaName {
				namespace: SchemaNamespace::try_from("azxcvbnmkjhgfds".to_string().into_bytes())
					.unwrap(),
				descriptor: SchemaDescriptor::default(),
			},
		];
		for (name, result) in valid_names.iter().zip(parsed_names) {
			let payload: SchemaNamePayload =
				BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");
			assert_eq!(SchemaName::try_parse::<Test>(payload, false), Ok(result));
		}
	});
}

#[test]
fn schema_name_get_combined_name_with_valid_names_should_succeed() {
	new_test_ext().execute_with(|| {
		let valid_names = vec!["Abc.a", "a-v.D-D", "aZxcvBnmkjhgfds.asdfghKkloiuyTre"];
		let results = vec!["abc.a", "a-v.d-d", "azxcvbnmkjhgfds.asdfghkkloiuytre"];
		for (name, result) in valid_names.iter().zip(results) {
			let payload: SchemaNamePayload =
				BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");
			let parsed = SchemaName::try_parse::<Test>(payload, true).expect("should work");
			assert_eq!(parsed.get_combined_name(), result.to_string().into_bytes());
		}
	});
}

#[test]
fn schema_version_id_add_should_work() {
	new_test_ext().execute_with(|| {
		let mut val = SchemaVersionId::default();
		let schema_id_1: SchemaId = 55;
		let schema_id_2: SchemaId = 200;
		let schema_name = SchemaName {
			namespace: SchemaNamespace::try_from("abc".to_string().into_bytes()).unwrap(),
			descriptor: SchemaDescriptor::try_from("d-d".to_string().into_bytes()).unwrap(),
		};
		assert_ok!(val.add::<Test>(schema_id_1));
		assert_ok!(val.add::<Test>(schema_id_2));

		let response = val.convert_to_response(&schema_name);
		assert_eq!(
			response,
			vec![
				SchemaVersionResponse {
					schema_id: schema_id_1,
					schema_version: 1,
					schema_name: schema_name.clone().get_combined_name()
				},
				SchemaVersionResponse {
					schema_id: schema_id_2,
					schema_version: 2,
					schema_name: schema_name.get_combined_name()
				},
			]
		);
	});
}

#[test]
fn schema_version_id_add_with_duplicate_should_fail() {
	new_test_ext().execute_with(|| {
		let mut val = SchemaVersionId::default();
		let schema_id_1: SchemaId = 55;

		assert_ok!(val.add::<Test>(schema_id_1));
		assert_noop!(val.add::<Test>(schema_id_1), Error::<Test>::SchemaIdAlreadyExists);
	});
}

#[test]
fn schema_version_id_add_with_max_len_should_fail() {
	new_test_ext().execute_with(|| {
		let mut val = SchemaVersionId::default();
		for i in 1..=MAX_NUMBER_OF_VERSIONS {
			let res = val.add::<Test>(i as SchemaId);
			assert_eq!(res, Ok(i as SchemaVersion));
		}

		assert_noop!(
			val.add::<Test>((MAX_NUMBER_OF_VERSIONS + 1) as SchemaId),
			Error::<Test>::ExceedsMaxNumberOfVersions
		);
	});
}

#[test]
fn create_schema_v3_requires_valid_schema_size() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();
		let test_cases: [TestCase<(Error<Test>, u8)>; 2] = [
			TestCase {
				input: r#"{"a":1}"#,
				expected: (Error::<Test>::LessThanMinSchemaModelBytes, 3),
			},
			TestCase {
				input: r#"{"id": "long", "title": "I am a very very very long schema", "properties": "just way too long to live a long life", "description": "Just a never ending stream of bytes that goes on for a minute too long"}"#,
				expected: (Error::<Test>::ExceedsMaxSchemaModelBytes, 2),
			},
		];
		for tc in test_cases {
			assert_noop!(
				SchemasPallet::create_schema_v3(RuntimeOrigin::signed(test_public(1)), create_bounded_schema_vec(tc.input), ModelType::AvroBinary, PayloadLocation::OnChain, BoundedVec::default(), None),
				tc.expected.0);
		}
	})
}

#[test]
fn create_schema_v3_happy_path() {
	new_test_ext().execute_with(|| {
		// arrange
		sudo_set_max_schema_size();
		let sender: AccountId = test_public(1);
		let name = "namespace.descriptor";
		let schema_name: SchemaNamePayload =
			BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");

		// act
		assert_ok!(SchemasPallet::create_schema_v3(
			RuntimeOrigin::signed(sender.clone()),
			create_bounded_schema_vec(r#"{"name": "Doe", "type": "lost"}"#),
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
			BoundedVec::default(),
			Some(schema_name.clone()),
		));
		let res = SchemasPallet::get_schema_by_id(1);
		let versions = SchemasPallet::get_schema_versions(schema_name.clone().into_inner());

		// assert
		System::assert_has_event(
			AnnouncementEvent::SchemaCreated { key: sender, schema_id: 1 }.into(),
		);
		System::assert_last_event(
			AnnouncementEvent::SchemaNameCreated {
				schema_id: 1,
				name: name.to_string().into_bytes(),
			}
			.into(),
		);
		assert_eq!(res.as_ref().is_some(), true);
		assert_eq!(
			versions,
			Some(vec![SchemaVersionResponse {
				schema_id: 1,
				schema_name: schema_name.into_inner(),
				schema_version: 1
			}])
		);
	})
}

#[test]
#[serial]
fn create_schema_v3_increments_schema_id_and_version_for_same_name() {
	new_test_ext().execute_with(|| {
		// arrange
		sudo_set_max_schema_size();
		let sender: AccountId = test_public(1);
		let name = "namespace.descriptor";
		let schema_name: SchemaNamePayload =
			BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");
		let mut last_schema_id: SchemaId = 0;

		// act and assert
		for fields in [
			r#"{"Name": "Bond", "Code": "007"}"#,
			r#"{"type": "num","minimum": -90,"maximum": 90}"#,
			r#"{"latitude": 48.858093,"longitude": 2.294694}"#,
		] {
			let expected_schema_id = last_schema_id + 1;
			assert_ok!(SchemasPallet::create_schema_v3(
				RuntimeOrigin::signed(sender.clone()),
				create_bounded_schema_vec(fields),
				ModelType::AvroBinary,
				PayloadLocation::OnChain,
				BoundedVec::default(),
				Some(schema_name.clone()),
			));
			System::assert_has_event(
				AnnouncementEvent::SchemaCreated {
					key: sender.clone(),
					schema_id: expected_schema_id,
				}
				.into(),
			);
			System::assert_last_event(
				AnnouncementEvent::SchemaNameCreated {
					schema_id: expected_schema_id,
					name: name.to_string().into_bytes(),
				}
				.into(),
			);
			last_schema_id = expected_schema_id;
		}
		let versions = SchemasPallet::get_schema_versions(schema_name.clone().into_inner());
		assert_eq!(
			versions,
			Some(vec![
				SchemaVersionResponse {
					schema_id: 1,
					schema_name: schema_name.clone().into_inner(),
					schema_version: 1
				},
				SchemaVersionResponse {
					schema_id: 2,
					schema_name: schema_name.clone().into_inner(),
					schema_version: 2
				},
				SchemaVersionResponse {
					schema_id: 3,
					schema_name: schema_name.into_inner(),
					schema_version: 3
				}
			])
		);
	})
}

#[test]
fn get_schema_versions_for_namespace_should_return_all_descriptors() {
	new_test_ext().execute_with(|| {
		// arrange
		sudo_set_max_schema_size();
		let sender: AccountId = test_public(1);
		let namespace = "namespace";
		let name_1 = format!("{}.alice", namespace);
		let schema_name_1: SchemaNamePayload =
			BoundedVec::try_from(name_1.to_string().into_bytes()).expect("should convert");
		let name_2 = format!("{}.bob", namespace);
		let schema_name_2: SchemaNamePayload =
			BoundedVec::try_from(name_2.to_string().into_bytes()).expect("should convert");
		assert_ok!(SchemasPallet::create_schema_v3(
			RuntimeOrigin::signed(sender.clone()),
			create_bounded_schema_vec(r#"{"name": "Doe", "type": "lost"}"#),
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
			BoundedVec::default(),
			Some(schema_name_1.clone()),
		));
		assert_ok!(SchemasPallet::create_schema_v3(
			RuntimeOrigin::signed(sender.clone()),
			create_bounded_schema_vec(r#"{"name": "Doe", "type": "lost"}"#),
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
			BoundedVec::default(),
			Some(schema_name_2.clone()),
		));

		// act
		let versions = SchemasPallet::get_schema_versions(String::from(namespace).into_bytes());

		// assert
		assert!(versions.is_some());

		let mut inner = versions.clone().unwrap();
		inner.sort_by(|a, b| a.schema_name.cmp(&b.schema_name));
		assert_eq!(
			versions,
			Some(vec![
				SchemaVersionResponse {
					schema_id: 1,
					schema_name: schema_name_1.into_inner(),
					schema_version: 1
				},
				SchemaVersionResponse {
					schema_id: 2,
					schema_name: schema_name_2.into_inner(),
					schema_version: 1
				},
			])
		);
	})
}

#[test]
fn create_schema_via_governance_v2_happy_path() {
	new_test_ext().execute_with(|| {
		// arrange
		sudo_set_max_schema_size();
		let settings = vec![SchemaSetting::AppendOnly];
		let sender: AccountId = test_public(5);
		let name = "namespace.descriptor";
		let schema_name: SchemaNamePayload =
			BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");

		// act
		assert_ok!(SchemasPallet::create_schema_via_governance_v2(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
			sender,
			create_bounded_schema_vec(r#"{"name": "Doe", "type": "lost"}"#),
			ModelType::AvroBinary,
			PayloadLocation::Itemized,
			BoundedVec::try_from(settings.clone()).unwrap(),
			Some(schema_name.clone()),
		));

		// assert
		let res = SchemasPallet::get_schema_by_id(1);
		let versions = SchemasPallet::get_schema_versions(schema_name.clone().into_inner());

		assert_eq!(res.unwrap().settings, settings);
		assert_eq!(
			versions,
			Some(vec![SchemaVersionResponse {
				schema_id: 1,
				schema_name: schema_name.into_inner(),
				schema_version: 1
			}])
		);
	})
}

#[test]
fn create_schema_via_governance_v2_with_append_only_setting_and_non_itemized_should_fail() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();

		// arrange
		let settings = vec![SchemaSetting::AppendOnly];
		let sender: AccountId = test_public(1);
		// act and assert
		assert_noop!(
			SchemasPallet::create_schema_via_governance_v2(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
				sender.clone(),
				create_bounded_schema_vec(r#"{"name":"John Doe"}"#),
				ModelType::AvroBinary,
				PayloadLocation::Paginated,
				BoundedVec::try_from(settings.clone()).unwrap(),
				None,
			),
			Error::<Test>::InvalidSetting
		);

		// act and assert
		assert_noop!(
			SchemasPallet::create_schema_via_governance_v2(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
				sender.clone(),
				create_bounded_schema_vec(r#"{"name":"John Doe"}"#),
				ModelType::AvroBinary,
				PayloadLocation::OnChain,
				BoundedVec::try_from(settings.clone()).unwrap(),
				None,
			),
			Error::<Test>::InvalidSetting
		);

		assert_noop!(
			SchemasPallet::create_schema_via_governance_v2(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
				sender,
				create_bounded_schema_vec(r#"{"name":"John Doe"}"#),
				ModelType::AvroBinary,
				PayloadLocation::IPFS,
				BoundedVec::try_from(settings.clone()).unwrap(),
				None,
			),
			Error::<Test>::InvalidSetting
		);
	})
}

/// Test that a request to be a provider, makes the MSA a provider after the council approves it.
#[test]
fn propose_to_create_schema_v2_happy_path() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();

		let test_model = r#"{"foo": "bar", "bar": "buzz"}"#;
		let serialized_fields = Vec::from(test_model.as_bytes());
		let schema_name =
			SchemaNamePayload::try_from("namespace.descriptor".to_string().into_bytes())
				.expect("should work");
		// Propose a new schema
		_ = SchemasPallet::propose_to_create_schema_v2(
			test_origin_signed(5),
			create_bounded_schema_vec(test_model),
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
			BoundedVec::default(),
			Some(schema_name.clone()),
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
				}) |
				RuntimeEvent::SchemasPallet(AnnouncementEvent::SchemaNameCreated {
					name: _,
					schema_id,
				}) => Some(schema_id),
				_ => None,
			})
			.collect();

		// Confirm that the schema was created
		assert_eq!(schema_events.len(), 2);

		let last_schema_id = schema_events[0];
		let created_schema = SchemasPallet::get_schema_by_id(last_schema_id);
		assert_eq!(created_schema.as_ref().is_some(), true);
		assert_eq!(created_schema.as_ref().unwrap().clone().model, serialized_fields);
	})
}

#[test]
fn propose_to_create_schema_name_happy_path() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();
		let schema_name =
			SchemaNamePayload::try_from("namespace.descriptor".to_string().into_bytes())
				.expect("should work");
		let sender: AccountId = test_public(1);

		assert_ok!(SchemasPallet::create_schema_v3(
			RuntimeOrigin::signed(sender.clone()),
			create_bounded_schema_vec(r#"{"name": "Doe", "type": "lost"}"#),
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
			BoundedVec::default(),
			None,
		));
		// Propose a new schema
		_ = SchemasPallet::propose_to_create_schema_name(
			test_origin_signed(5),
			1,
			schema_name.clone(),
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
				RuntimeEvent::SchemasPallet(AnnouncementEvent::SchemaNameCreated {
					name: _,
					schema_id,
				}) => Some(schema_id),
				_ => None,
			})
			.collect();

		// Confirm that the schema was created
		assert_eq!(schema_events.len(), 1);

		let versions = SchemasPallet::get_schema_versions(schema_name.clone().into_inner());
		assert_eq!(
			versions,
			Some(vec![SchemaVersionResponse {
				schema_id: 1,
				schema_name: schema_name.into_inner(),
				schema_version: 1
			}])
		);
	})
}
