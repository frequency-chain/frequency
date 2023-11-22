use crate::{
	migration::v3,
	pallet::SchemaNameToIds,
	tests::mock::{
		create_bounded_schema_vec, new_test_ext, sudo_set_max_schema_size, test_public,
		RuntimeOrigin, SchemasPallet, Test,
	},
	SchemaName,
};
use common_primitives::{node::AccountId, schema::*};
use frame_support::{
	assert_ok, pallet_prelude::StorageVersion, traits::GetStorageVersion, BoundedVec,
};

#[test]
fn schemas_migration_to_v3_should_work_as_expected() {
	new_test_ext().execute_with(|| {
		// Arrange
		sudo_set_max_schema_size();
		let sender: AccountId = test_public(5);
		let schemas = vec![r#"{"latitude": 48.858093,"longitude": 2.294694}"#; 20];
		for fields in schemas.iter() {
			assert_ok!(SchemasPallet::create_schema_v3(
				RuntimeOrigin::signed(sender.clone()),
				create_bounded_schema_vec(fields),
				ModelType::AvroBinary,
				PayloadLocation::OnChain,
				BoundedVec::default(),
				None,
			));
		}

		// Act
		let _ = v3::migrate_to_v3::<Test>();

		// Assert
		let current_version = SchemasPallet::current_storage_version();
		assert_eq!(current_version, StorageVersion::new(3));

		let known_schemas = v3::get_known_schemas();
		let versions_count = SchemaNameToIds::<Test>::iter().count();
		assert_eq!(known_schemas.len(), versions_count);

		for (_, schema_name) in known_schemas.iter() {
			let bounded_name = BoundedVec::try_from(schema_name.clone()).expect("should work");
			let parsed_name =
				SchemaName::try_parse::<Test>(bounded_name, true).expect("should parse");
			let val = SchemaNameToIds::<Test>::get(&parsed_name.namespace, &parsed_name.descriptor);
			assert_eq!(val.ids.len(), 1usize);
		}
	});
}
