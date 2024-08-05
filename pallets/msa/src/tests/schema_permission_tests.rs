use frame_support::{assert_err, assert_noop, assert_ok, BoundedBTreeMap};
use frame_system::pallet_prelude::BlockNumberFor;

use crate::{
	tests::{mock::*, other_tests::set_schema_count},
	types::PermittedDelegationSchemas,
	Config, DelegatorAndProviderToDelegation, Error,
};

use common_primitives::{
	msa::{Delegation, DelegatorId, ProviderId, SchemaGrantValidator},
	schema::SchemaId,
};

#[test]
fn grant_permissions_for_schemas_errors_when_no_delegation() {
	new_test_ext().execute_with(|| {
		let delegator = DelegatorId(2);
		let provider = ProviderId(1);
		let schema_ids = vec![1, 2];
		let result = Msa::grant_permissions_for_schemas(delegator, provider, schema_ids);

		assert_noop!(result, Error::<Test>::DelegationNotFound);
	});
}

#[test]
fn grant_permissions_for_schemas_errors_when_invalid_schema_id() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(1);
		let delegator = DelegatorId(2);
		let provider = ProviderId(1);
		let schema_grants = vec![1];

		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		let additional_grants = vec![2];
		let result = Msa::grant_permissions_for_schemas(delegator, provider, additional_grants);

		assert_noop!(result, Error::<Test>::InvalidSchemaId);
	});
}

#[test]
fn grant_permissions_for_schemas_errors_when_exceeds_max_schema_grants() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(31);

		let delegator = DelegatorId(2);
		let provider = ProviderId(1);
		let schema_grants = vec![1];

		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		let additional_grants = (2..32 as u16).collect::<Vec<_>>();
		let result = Msa::grant_permissions_for_schemas(delegator, provider, additional_grants);

		assert_noop!(result, Error::<Test>::ExceedsMaxSchemaGrantsPerDelegation);
	});
}

#[test]
fn grant_permissions_for_schema_success() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(3);

		let delegator = DelegatorId(2);
		let provider = ProviderId(1);
		let schema_grants = vec![1];

		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		let delegation_relationship =
			DelegatorAndProviderToDelegation::<Test>::get(delegator, provider).unwrap();
		let mut expected = BoundedBTreeMap::<
			SchemaId,
			BlockNumberFor<Test>,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		>::new();

		expected.try_insert(1, Default::default()).expect("testing expected");

		assert_eq!(delegation_relationship.schema_permissions, expected);

		// Add new schema ids
		let additional_grants = vec![2];
		let result = Msa::grant_permissions_for_schemas(delegator, provider, additional_grants);

		assert_ok!(result);

		let delegation_relationship =
			DelegatorAndProviderToDelegation::<Test>::get(delegator, provider).unwrap();
		let mut expected = BoundedBTreeMap::<
			SchemaId,
			BlockNumberFor<Test>,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		>::new();

		expected.try_insert(1, Default::default()).expect("testing expected");
		expected.try_insert(2, Default::default()).expect("testing expected");

		assert_eq!(delegation_relationship.schema_permissions, expected);
	});
}

#[test]
fn schema_permissions_trait_impl_try_insert_schema_success() {
	new_test_ext().execute_with(|| {
		let mut delegation: Delegation<
			SchemaId,
			BlockNumberFor<Test>,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		> = Default::default();

		let schema_id = 1;
		assert_ok!(PermittedDelegationSchemas::<Test>::try_insert_schema(
			&mut delegation,
			schema_id
		));
		assert_eq!(delegation.schema_permissions.len(), 1);
	});
}

#[test]
fn schema_permissions_trait_impl_try_insert_schemas_errors_when_exceeds_max_schema_grants() {
	new_test_ext().execute_with(|| {
		let mut delegation: Delegation<
			SchemaId,
			BlockNumberFor<Test>,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		> = Default::default();

		let schema_ids = (1..32).collect::<Vec<_>>();
		assert_noop!(
			PermittedDelegationSchemas::<Test>::try_insert_schemas(&mut delegation, schema_ids),
			Error::<Test>::ExceedsMaxSchemaGrantsPerDelegation
		);
	});
}

#[test]
fn revoke_permissions_for_schema_success() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(3);

		let delegator = DelegatorId(2);
		let provider = ProviderId(1);
		let schema_grants = vec![1];

		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		let delegation_relationship =
			DelegatorAndProviderToDelegation::<Test>::get(delegator, provider).unwrap();
		let mut expected = BoundedBTreeMap::<
			SchemaId,
			BlockNumberFor<Test>,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		>::new();

		expected.try_insert(1, Default::default()).expect("testing expected");

		assert_eq!(delegation_relationship.schema_permissions, expected);

		// Revoke schema ids
		let schemas_to_be_revoked = vec![1];
		let result =
			Msa::revoke_permissions_for_schemas(delegator, provider, schemas_to_be_revoked);

		assert_ok!(result);

		let delegation_relationship =
			DelegatorAndProviderToDelegation::<Test>::get(delegator, provider).unwrap();
		let mut expected = BoundedBTreeMap::<
			SchemaId,
			BlockNumberFor<Test>,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		>::new();

		expected.try_insert(1, 1u32.into()).expect("testing expected");

		assert_eq!(delegation_relationship.schema_permissions, expected);
	});
}

#[test]
fn revoke_permissions_for_schemas_errors_when_no_delegation() {
	new_test_ext().execute_with(|| {
		let delegator = DelegatorId(2);
		let provider = ProviderId(1);
		let schema_ids = vec![1, 2];
		let result = Msa::revoke_permissions_for_schemas(delegator, provider, schema_ids);

		assert_noop!(result, Error::<Test>::DelegationNotFound);
	});
}

#[test]
fn revoke_permissions_for_schemas_errors_when_schema_does_not_exist_in_list_of_schema_grants() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(31);

		let delegator = DelegatorId(2);
		let provider = ProviderId(1);
		let schema_grants = vec![1, 2];

		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		let additional_grants = (3..32 as u16).collect::<Vec<_>>();
		let result = Msa::revoke_permissions_for_schemas(delegator, provider, additional_grants);

		assert_noop!(result, Error::<Test>::SchemaNotGranted);

		let result = DelegatorAndProviderToDelegation::<Test>::get(delegator, provider);

		let mut expected = Delegation {
			revoked_at: 0u32.into(),
			schema_permissions: BoundedBTreeMap::<
				SchemaId,
				BlockNumberFor<Test>,
				<Test as Config>::MaxSchemaGrantsPerDelegation,
			>::new(),
		};

		expected
			.schema_permissions
			.try_insert(1, 0u32.into())
			.expect("testing expected");

		expected
			.schema_permissions
			.try_insert(2, 0u32.into())
			.expect("testing expected");

		assert_eq!(result.unwrap(), expected);
	});
}

#[test]
fn schema_permissions_trait_impl_try_get_mut_schema_success() {
	new_test_ext().execute_with(|| {
		let mut delegation: Delegation<
			SchemaId,
			BlockNumberFor<Test>,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		> = Default::default();

		let schema_id = 1;
		assert_ok!(PermittedDelegationSchemas::<Test>::try_insert_schema(
			&mut delegation,
			schema_id
		));
		let default_block_number = 0u32;

		assert_eq!(delegation.schema_permissions.len(), 1);
		assert_eq!(delegation.schema_permissions.get(&schema_id).unwrap(), &default_block_number);

		let revoked_block_number = 2u32;

		assert_ok!(PermittedDelegationSchemas::<Test>::try_get_mut_schema(
			&mut delegation,
			schema_id,
			revoked_block_number.clone()
		));

		assert_eq!(delegation.schema_permissions.get(&schema_id).unwrap(), &revoked_block_number);
	});
}

#[test]
pub fn ensure_valid_schema_grant_success() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(2);

		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		let schema_grants = vec![1, 2];
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		System::set_block_number(System::block_number() + 1);

		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 1_u16, 1u32));
	})
}

#[test]
pub fn ensure_valid_schema_grant_errors_when_delegation_relationship_is_valid_and_grant_does_not_exist(
) {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(2);

		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		let schema_grants = vec![1, 2];

		// Add delegation relationship with schema grants.
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		// Set block number to 2.
		System::set_block_number(System::block_number() + 1);

		assert_err!(
			Msa::ensure_valid_schema_grant(provider, delegator, 3_u16, 1u32),
			Error::<Test>::SchemaNotGranted
		);
	})
}

#[test]
pub fn ensure_valid_schema_grant_errors_when_delegation_relationship_is_valid_and_schema_grant_is_revoked(
) {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(2);

		// Create delegation relationship.
		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		let schema_grants = vec![1, 2];
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		// Set block number to 6.
		System::set_block_number(System::block_number() + 5);

		// revoke schema permission at block 6.
		assert_ok!(Msa::revoke_permissions_for_schemas(delegator, provider, vec![1]));

		// Schemas is valid for the current block that is revoked 6
		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 1, 6));

		// Checking that asking for validity past the current block, 6, errors.
		assert_noop!(
			Msa::ensure_valid_schema_grant(provider, delegator, 1, 7),
			Error::<Test>::CannotPredictValidityPastCurrentBlock
		);

		// Set block number to 6.
		System::set_block_number(System::block_number() + 5);
		assert_eq!(System::block_number(), 11);

		assert_noop!(
			Msa::ensure_valid_schema_grant(provider, delegator, 1, 7),
			Error::<Test>::SchemaNotGranted
		);
	});
}

#[test]
pub fn ensure_valid_schema_grant_errors_delegation_revoked_when_delegation_relationship_has_been_revoked(
) {
	new_test_ext().execute_with(|| {
		// Set the schemas counts so that it passes validation.
		set_schema_count::<Test>(2);

		// Create delegation relationship.
		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		let schema_grants = vec![1, 2];

		// Create delegation relationship.
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		// Move forward to block 6.
		System::set_block_number(System::block_number() + 5);

		// Revoke delegation relationship at block 6.
		assert_ok!(Msa::revoke_provider(provider, delegator));

		// Schemas is valid for the current block that is revoked 6.
		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 1, 6));
		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 1, 5));

		// Checking that asking for validity past the current block, 6, errors.
		assert_noop!(
			Msa::ensure_valid_schema_grant(provider, delegator, 1, 8),
			Error::<Test>::CannotPredictValidityPastCurrentBlock
		);

		// Move forward to block 11.
		System::set_block_number(System::block_number() + 5);

		// Check that schema is not valid after delegation revocation
		assert_noop!(
			Msa::ensure_valid_schema_grant(provider, delegator, 1, 7),
			Error::<Test>::DelegationRevoked
		);
	});
}

#[test]
pub fn ensure_delegation_revocation_reflects_in_schema_permissions() {
	new_test_ext().execute_with(|| {
		// Set the schemas counts so that it passes validation.
		set_schema_count::<Test>(2);

		// Create delegation relationship.
		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		let schema_grants = vec![1, 2];

		// Create delegation relationship.
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		// Move forward to block 6.
		System::set_block_number(System::block_number() + 5);

		// Revoke delegation relationship at block 6.
		assert_ok!(Msa::revoke_provider(provider, delegator));

		let grants_result = Msa::get_granted_schemas_by_msa_id(delegator, Some(provider));
		assert!(grants_result.is_ok());
		let grants_option = grants_result.unwrap();
		assert!(grants_option.len() == 1);
		let grants = grants_option.into_iter().next().unwrap();
		assert!(grants.permissions[0].revoked_at == 6);
		assert!(grants.permissions[1].revoked_at == 6);
	});
}
