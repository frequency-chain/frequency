use crate::{
	migration::v4,
	tests::mock::{
		create_bounded_schema_vec, new_test_ext, sudo_set_max_schema_size, test_public,
		RuntimeOrigin, SchemasPallet, Test,
	},
};
use common_primitives::{node::AccountId, schema::*};
use frame_support::{
	assert_ok, pallet_prelude::StorageVersion, traits::GetStorageVersion, BoundedVec,
};

#[test]
fn schemas_migration_to_v4_should_work_as_expected() {
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
		let _ = v4::migrate_to_v4::<Test>();

		// Assert
		let current_version = SchemasPallet::current_storage_version();
		assert_eq!(current_version, StorageVersion::new(4));

		let known_schemas = v4::get_known_schemas::<Test>();
		assert_eq!(known_schemas.len(), 0);
	});
}
