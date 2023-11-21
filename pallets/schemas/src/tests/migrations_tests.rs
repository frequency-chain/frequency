// #[test]
// fn schemas_migration_to_v2_should_work_as_expected() {
// 	new_test_ext().execute_with(|| {
// 		// Arrange
// 		sudo_set_max_schema_size();
// 		let sender: AccountId = test_public(5);
// 		let schemas = vec![
// 			r#"{"Name": "Bond", "Code": "007"}"#,
// 			r#"{"type": "num","minimum": -90,"maximum": 90}"#,
// 			r#"{"latitude": 48.858093,"longitude": 2.294694}"#,
// 		];
// 		for (idx, fields) in schemas.iter().enumerate() {
// 			assert_ok!(SchemasPallet::create_schema_v2(
// 				RuntimeOrigin::signed(sender.clone()),
// 				create_bounded_schema_vec(fields),
// 				ModelType::AvroBinary,
// 				PayloadLocation::OnChain,
// 				BoundedVec::default()
// 			));
// 			v2::old::Schemas::<Test>::insert(
// 				idx as u16 + 1,
// 				v2::old::Schema {
// 					model_type: ModelType::AvroBinary,
// 					payload_location: PayloadLocation::OnChain,
// 					settings: SchemaSettings::all_disabled(),
// 					model: BoundedVec::try_from(fields.as_bytes().to_vec())
// 						.expect("should have value"),
// 				},
// 			);
// 		}
// 		let old_schema_1 = v2::old::Schemas::<Test>::get(1u16).expect("should have value");
// 		let old_schema_2 = v2::old::Schemas::<Test>::get(2u16).expect("should have value");
// 		let old_schema_3 = v2::old::Schemas::<Test>::get(3u16).expect("should have value");
//
// 		// Act
// 		let _ = v2::migrate_to_v2::<Test>();
//
// 		// Assert
// 		let old_count = v2::old::Schemas::<Test>::iter().count();
// 		let new_info_count = SchemaInfos::<Test>::iter().count();
// 		let new_payload_count = SchemaPayloads::<Test>::iter().count();
// 		let current_version = SchemasPallet::current_storage_version();
//
// 		assert_eq!(old_count, 0);
// 		assert_eq!(new_info_count, schemas.len());
// 		assert_eq!(new_payload_count, schemas.len());
// 		assert_eq!(current_version, StorageVersion::new(2));
//
// 		let schema_info_1 = SchemaInfos::<Test>::get(1).expect("should have value");
// 		let schema_payload_1 = SchemaPayloads::<Test>::get(1u16).expect("should have value");
// 		assert_eq!(schema_info_1.model_type, old_schema_1.model_type);
// 		assert_eq!(schema_info_1.payload_location, old_schema_1.payload_location);
// 		assert_eq!(schema_info_1.settings, old_schema_1.settings);
// 		assert_eq!(schema_payload_1.into_inner(), old_schema_1.model.into_inner());
//
// 		let schema_info_2 = SchemaInfos::<Test>::get(2).expect("should have value");
// 		let schema_payload_2 = SchemaPayloads::<Test>::get(2u16).expect("should have value");
// 		assert_eq!(schema_info_2.model_type, old_schema_2.model_type);
// 		assert_eq!(schema_info_2.payload_location, old_schema_2.payload_location);
// 		assert_eq!(schema_info_2.settings, old_schema_2.settings);
// 		assert_eq!(schema_payload_2.into_inner(), old_schema_2.model.into_inner());
//
// 		let schema_info_3 = SchemaInfos::<Test>::get(3).expect("should have value");
// 		let schema_payload_3 = SchemaPayloads::<Test>::get(3u16).expect("should have value");
// 		assert_eq!(schema_info_3.model_type, old_schema_3.model_type);
// 		assert_eq!(schema_info_3.payload_location, old_schema_3.payload_location);
// 		assert_eq!(schema_info_3.settings, old_schema_3.settings);
// 		assert_eq!(schema_payload_3.into_inner(), old_schema_3.model.into_inner());
// 	});
// }
