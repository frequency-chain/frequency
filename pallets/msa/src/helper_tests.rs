use frame_support::{
	assert_err, assert_noop, assert_ok, pallet_prelude::InvalidTransaction, BoundedBTreeMap,
};

use sp_core::{crypto::AccountId32, sr25519, Pair};

use crate::testing_utils::set_schema_count;

use crate::{
	ensure,
	mock::*,
	types::{PermittedDelegationSchemas, EMPTY_FUNCTION},
	CheckFreeExtrinsicUse, Config, DispatchResult, Error, ProviderToRegistryEntry, ValidityError,
};

use common_primitives::{
	msa::{
		Delegation, DelegationValidator, DelegatorId, MessageSourceId, ProviderId,
		ProviderRegistryEntry, SchemaGrantValidator,
	},
	schema::SchemaId,
};

#[test]
fn test_ensure_msa_can_retire_fails_if_more_than_one_account_exists() {
	new_test_ext().execute_with(|| {
		let msa_id = 2;
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();

		// Create accounts
		let test_account_1 = AccountId32::new(test_account_1_key_pair.public().into());
		let test_account_2 = AccountId32::new(test_account_2_key_pair.public().into());

		// Add two accounts to the MSA
		assert_ok!(Msa::add_key(msa_id, &test_account_1, EMPTY_FUNCTION));
		assert_ok!(Msa::add_key(msa_id, &test_account_2, EMPTY_FUNCTION));

		// Retire the MSA
		assert_noop!(
			CheckFreeExtrinsicUse::<Test>::ensure_msa_can_retire(&test_account_1),
			InvalidTransaction::Custom(ValidityError::InvalidMoreThanOneKeyExists as u8)
		);
	})
}

#[test]
fn test_ensure_msa_can_retire_fails_if_registered_provider() {
	new_test_ext().execute_with(|| {
		// Create an account
		let (test_account_key_pair, _) = sr25519::Pair::generate();
		let test_account = AccountId32::new(test_account_key_pair.public().into());
		let origin = RuntimeOrigin::signed(test_account.clone());

		// Add an account to the MSA
		assert_ok!(Msa::add_key(2, &test_account, EMPTY_FUNCTION));

		// Register provider
		assert_ok!(Msa::create_provider(origin, Vec::from("Foo")));

		// Retire MSA
		assert_noop!(
			CheckFreeExtrinsicUse::<Test>::ensure_msa_can_retire(&test_account),
			InvalidTransaction::Custom(
				ValidityError::InvalidRegisteredProviderCannotBeRetired as u8
			)
		);
	})
}

#[test]
fn test_ensure_msa_can_retire_fails_if_any_delegations_exist() {
	new_test_ext().execute_with(|| {
		// Create delegator
		let msa_id = 2;
		let (test_account_key_pair, _) = sr25519::Pair::generate();
		let test_account = AccountId32::new(test_account_key_pair.public().into());
		assert_ok!(Msa::add_key(msa_id, &test_account, EMPTY_FUNCTION));

		// Create provider
		let (provider_id, _provider_key) = create_provider_with_name("test");
		let schema_ids = vec![1];
		set_schema_count::<Test>(1);
		assert_ok!(Msa::add_provider(ProviderId(provider_id), DelegatorId(msa_id), schema_ids));

		// Retire the MSA
		assert_err!(
			CheckFreeExtrinsicUse::<Test>::ensure_msa_can_retire(&test_account),
			InvalidTransaction::Custom(ValidityError::InvalidNonZeroProviderDelegations as u8)
		);
	})
}

#[test]
pub fn test_get_owner_of() {
	new_test_ext().execute_with(|| {
		assert_eq!(Msa::get_owner_of(&test_public(1)), None);

		assert_ok!(Msa::create(test_origin_signed(1)));

		assert_eq!(Msa::get_owner_of(&test_public(1)), Some(1));
	});
}

#[test]
pub fn test_delete_key() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::add_key(1, &test_public(1), EMPTY_FUNCTION));

		let info = Msa::get_msa_by_public_key(&test_public(1));

		assert_eq!(info, Some(1 as MessageSourceId));

		assert_ok!(Msa::delete_key_for_msa(info.unwrap(), &test_public(1)));
	});
}

#[test]
pub fn test_delete_key_errors() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::add_key(1, &test_public(1), EMPTY_FUNCTION));

		assert_ok!(Msa::delete_key_for_msa(1, &test_public(1)));
	});
}

#[test]
pub fn test_ensure_msa_owner() {
	new_test_ext().execute_with(|| {
		assert_noop!(Msa::ensure_msa_owner(&test_public(1), 1), Error::<Test>::NoKeyExists);

		assert_ok!(Msa::add_key(1, &test_public(1), EMPTY_FUNCTION));

		assert_eq!(Msa::ensure_msa_owner(&test_public(1), 1), Ok(()));
	});
}

#[test]
pub fn ensure_valid_msa_key_is_successfull() {
	new_test_ext().execute_with(|| {
		assert_noop!(Msa::ensure_valid_msa_key(&test_public(1)), Error::<Test>::NoKeyExists);

		assert_ok!(Msa::create(test_origin_signed(1)));

		assert_ok!(Msa::ensure_valid_msa_key(&test_public(1)));
	});
}

#[test]
pub fn add_key_with_panic_in_on_success_should_revert_everything() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1u64;
		let key = test_public(msa_id as u8);

		// act
		assert_noop!(
			Msa::add_key(msa_id, &key, |new_msa_id| -> DispatchResult {
				ensure!(new_msa_id != msa_id, Error::<Test>::InvalidSelfRemoval);
				Ok(())
			}),
			Error::<Test>::InvalidSelfRemoval
		);

		// assert
		assert_eq!(Msa::get_msa_by_public_key(&key), None);

		// *Temporarily Removed* until https://github.com/LibertyDSNP/frequency/issues/418 is completed
		// assert_eq!(Msa::get_msa_keys(msa_id).into_inner(), vec![])
	});
}

#[test]
pub fn create_account_with_panic_in_on_success_should_revert_everything() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1u64;
		let key = test_public(msa_id as u8);
		let next_msa_id = Msa::get_next_msa_id().unwrap();

		// act
		assert_noop!(
			Msa::create_account(key, |new_msa_id| -> DispatchResult {
				ensure!(new_msa_id != msa_id, Error::<Test>::InvalidSelfRemoval);
				Ok(())
			}),
			Error::<Test>::InvalidSelfRemoval
		);

		// assert
		assert_eq!(next_msa_id, Msa::get_next_msa_id().unwrap());
	});
}

#[test]
pub fn valid_delegation() {
	new_test_ext().execute_with(|| {
		let provider = ProviderId(1);
		let delegator = DelegatorId(2);

		assert_ok!(Msa::add_provider(provider, delegator, Vec::default()));

		System::set_block_number(System::block_number() + 1);

		assert_ok!(Msa::ensure_valid_delegation(provider, delegator, None));
	})
}

#[test]
pub fn delegation_not_found() {
	new_test_ext().execute_with(|| {
		let provider = ProviderId(1);
		let delegator = DelegatorId(2);

		assert_noop!(
			Msa::ensure_valid_delegation(provider, delegator, None),
			Error::<Test>::DelegationNotFound
		);
	})
}
#[test]
pub fn delegation_expired() {
	new_test_ext().execute_with(|| {
		let provider = ProviderId(1);
		let delegator = DelegatorId(2);

		assert_ok!(Msa::add_provider(provider, delegator, Vec::default()));

		System::set_block_number(System::block_number() + 1);
		assert_ok!(Msa::ensure_valid_delegation(provider, delegator, None));

		assert_ok!(Msa::revoke_provider(provider, delegator));

		System::set_block_number(System::block_number() + 1);

		assert_noop!(
			Msa::ensure_valid_delegation(provider, delegator, None),
			Error::<Test>::DelegationRevoked
		);
	})
}

#[test]
pub fn valid_schema_grant() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(2);

		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		let schema_grants = vec![1, 2];
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		System::set_block_number(System::block_number() + 1);

		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 2u16, 1u64));
	})
}

#[test]
pub fn error_invalid_schema_id() {
	struct TestCase<T> {
		schema: Vec<u16>,
		expected: T,
	}
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(12);

		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		let test_cases: [TestCase<Error<Test>>; 3] = [
			TestCase { schema: vec![15, 16], expected: Error::<Test>::InvalidSchemaId },
			TestCase { schema: vec![16, 17], expected: Error::<Test>::InvalidSchemaId },
			TestCase { schema: vec![18], expected: Error::<Test>::InvalidSchemaId },
		];
		for tc in test_cases {
			assert_noop!(Msa::add_provider(provider, delegator, tc.schema), tc.expected);
		}
	})
}

#[test]
pub fn error_exceeding_max_schema_under_minimum_schema_grants() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(16);

		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		assert_noop!(
			Msa::add_provider(provider, delegator, (1..32 as u16).collect::<Vec<_>>()),
			Error::<Test>::ExceedsMaxSchemaGrantsPerDelegation
		);
	})
}

#[test]
pub fn error_not_delegated_rpc() {
	new_test_ext().execute_with(|| {
		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		assert_err!(
			Msa::get_granted_schemas_by_msa_id(delegator, provider),
			Error::<Test>::DelegationNotFound
		);
	})
}

#[test]
pub fn error_schema_not_granted_rpc() {
	new_test_ext().execute_with(|| {
		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		assert_ok!(Msa::add_provider(provider, delegator, Vec::default()));
		assert_err!(
			Msa::get_granted_schemas_by_msa_id(delegator, provider),
			Error::<Test>::SchemaNotGranted
		);
	})
}

#[test]
pub fn schema_granted_success_rpc() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(2);

		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		let schema_grants = vec![1, 2];
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));
		let schemas_granted = Msa::get_granted_schemas_by_msa_id(delegator, provider);
		let expected_schemas_granted = vec![1, 2];
		let output_schemas: Vec<SchemaId> = schemas_granted.unwrap().unwrap();
		assert_eq!(output_schemas, expected_schemas_granted);
	})
}

#[test]
pub fn delegation_expired_long_back() {
	new_test_ext().execute_with(|| {
		let provider = ProviderId(1);
		let delegator = DelegatorId(2);

		assert_ok!(Msa::add_provider(provider, delegator, Vec::default()));

		System::set_block_number(System::block_number() + 100);
		assert_ok!(Msa::ensure_valid_delegation(provider, delegator, None));

		assert_ok!(Msa::revoke_provider(provider, delegator));

		System::set_block_number(System::block_number() + 150);

		assert_noop!(
			Msa::ensure_valid_delegation(provider, delegator, Some(151)),
			Error::<Test>::DelegationRevoked
		);
		assert_ok!(Msa::ensure_valid_delegation(provider, delegator, Some(6)));
		assert_noop!(
			Msa::ensure_valid_delegation(provider, delegator, Some(1000)),
			Error::<Test>::CannotPredictValidityPastCurrentBlock
		);
	})
}

#[test]
pub fn ensure_all_schema_ids_are_valid_errors() {
	new_test_ext().execute_with(|| {
		let schema_ids = vec![1];
		assert_noop!(
			Msa::ensure_all_schema_ids_are_valid(&schema_ids),
			Error::<Test>::InvalidSchemaId
		);

		let schema_ids = (1..32).collect::<Vec<_>>();
		assert_noop!(
			Msa::ensure_all_schema_ids_are_valid(&schema_ids),
			Error::<Test>::ExceedsMaxSchemaGrantsPerDelegation
		);
	})
}
#[test]
pub fn ensure_all_schema_ids_are_valid_success() {
	new_test_ext().execute_with(|| {
		let schema_ids = vec![1];
		set_schema_count::<Test>(1);

		assert_ok!(Msa::ensure_all_schema_ids_are_valid(&schema_ids));
	});
}

#[test]
pub fn is_registered_provider_is_true() {
	new_test_ext().execute_with(|| {
		let provider = ProviderId(1);
		let provider_name = Vec::from("frequency".as_bytes()).try_into().unwrap();

		let provider_meta = ProviderRegistryEntry { provider_name };
		ProviderToRegistryEntry::<Test>::insert(provider, provider_meta);

		assert!(Msa::is_registered_provider(provider.into()));
	});
}

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

		let delegation_relationship = Msa::get_delegation(delegator, provider).unwrap();
		let mut expected = BoundedBTreeMap::<
			SchemaId,
			<Test as frame_system::Config>::BlockNumber,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		>::new();

		expected.try_insert(1, Default::default()).expect("testing expected");

		assert_eq!(delegation_relationship.schema_permissions, expected);

		// Add new schema ids
		let additional_grants = vec![2];
		let result = Msa::grant_permissions_for_schemas(delegator, provider, additional_grants);

		assert_ok!(result);

		let delegation_relationship = Msa::get_delegation(delegator, provider).unwrap();
		let mut expected = BoundedBTreeMap::<
			SchemaId,
			<Test as frame_system::Config>::BlockNumber,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		>::new();

		expected.try_insert(1, Default::default()).expect("testing expected");
		expected.try_insert(2, Default::default()).expect("testing expected");

		assert_eq!(delegation_relationship.schema_permissions, expected);
	});
}

#[test]
fn delegation_default_trait_impl() {
	new_test_ext().execute_with(|| {
		let delegation: Delegation<
			SchemaId,
			<Test as frame_system::Config>::BlockNumber,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		> = Default::default();

		let expected = Delegation {
			schema_permissions: BoundedBTreeMap::<
				SchemaId,
				<Test as frame_system::Config>::BlockNumber,
				<Test as Config>::MaxSchemaGrantsPerDelegation,
			>::default(),
			revoked_at: Default::default(),
		};

		assert_eq!(delegation, expected);
	});
}

#[test]
fn schema_permissions_trait_impl_try_insert_schema_success() {
	new_test_ext().execute_with(|| {
		let mut delegation: Delegation<
			SchemaId,
			<Test as frame_system::Config>::BlockNumber,
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
			<Test as frame_system::Config>::BlockNumber,
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
fn try_mutate_delegation_success() {
	new_test_ext().execute_with(|| {
		let delegator = DelegatorId(1);
		let provider = ProviderId(2);

		assert_ok!(Msa::try_mutate_delegation(
			delegator,
			provider,
			|delegation, _is_new_provider| -> Result<(), &'static str> {
				let schema_id = 1;
				let _a =
					PermittedDelegationSchemas::<Test>::try_insert_schema(delegation, schema_id);

				Ok(())
			},
		));

		assert!(Msa::get_delegation(delegator, provider).is_some());
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

		let delegation_relationship = Msa::get_delegation(delegator, provider).unwrap();
		let mut expected = BoundedBTreeMap::<
			SchemaId,
			<Test as frame_system::Config>::BlockNumber,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		>::new();

		expected.try_insert(1, Default::default()).expect("testing expected");

		assert_eq!(delegation_relationship.schema_permissions, expected);

		// Revoke schema ids
		let schemas_to_be_revoked = vec![1];
		let result =
			Msa::revoke_permissions_for_schemas(delegator, provider, schemas_to_be_revoked);

		assert_ok!(result);

		let delegation_relationship = Msa::get_delegation(delegator, provider).unwrap();
		let mut expected = BoundedBTreeMap::<
			SchemaId,
			<Test as frame_system::Config>::BlockNumber,
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

		let result = Msa::get_delegation(delegator, provider);

		let mut expected = Delegation {
			revoked_at: 0u32.into(),
			schema_permissions: BoundedBTreeMap::<
				SchemaId,
				<Test as frame_system::Config>::BlockNumber,
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
			<Test as frame_system::Config>::BlockNumber,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		> = Default::default();

		let schema_id = 1;
		assert_ok!(PermittedDelegationSchemas::<Test>::try_insert_schema(
			&mut delegation,
			schema_id
		));
		let default_block_number = 0u64;

		assert_eq!(delegation.schema_permissions.len(), 1);
		assert_eq!(delegation.schema_permissions.get(&schema_id).unwrap(), &default_block_number);

		let revoked_block_number = 2u64;

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

		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 1_u16, 1u64));
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
			Msa::ensure_valid_schema_grant(provider, delegator, 3_u16, 1u64),
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
