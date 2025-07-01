use frame_support::{
	assert_err, assert_noop, assert_ok,
	dispatch::{GetDispatchInfo, Pays},
	BoundedBTreeMap,
};

use frame_system::pallet_prelude::BlockNumberFor;

use sp_core::{crypto::AccountId32, ecdsa, sr25519, Encode, Pair};
use sp_runtime::{traits::Zero, MultiSignature};

use crate::{
	ensure,
	tests::mock::*,
	types::{AddProvider, PermittedDelegationSchemas, EMPTY_FUNCTION},
	AddKeyData, AuthorizedKeyData, Config, DelegatorAndProviderToDelegation, DispatchResult, Error,
	Event, ProviderToRegistryEntry, PublicKeyToMsaId,
};
use common_primitives::signatures::AccountAddressMapper;

use common_primitives::{
	msa::{
		Delegation, DelegationResponse, DelegatorId, MessageSourceId, ProviderId,
		ProviderRegistryEntry, SchemaGrant, SchemaGrantValidator, H160,
	},
	node::{BlockNumber, EIP712Encode},
	schema::{SchemaId, SchemaValidator},
	signatures::{EthereumAddressMapper, UnifiedSignature, UnifiedSigner},
	utils::wrap_binary_data,
};
use pretty_assertions::assert_eq;
use sp_core::bytes::from_hex;
use sp_runtime::traits::{IdentifyAccount, Verify};
extern crate alloc;
use crate::types::PayloadTypeDiscriminator;
use alloc::vec;

pub fn assert_revoke_delegation_by_delegator_no_effect(
	test_account: AccountId32,
	provider_msa_id: u64,
) {
	let event_count = System::event_count();
	assert_ok!(Msa::revoke_delegation_by_delegator(
		RuntimeOrigin::signed(test_account.clone()),
		provider_msa_id
	));
	assert_eq!(event_count, System::event_count())
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
pub fn test_ensure_msa_owner() {
	new_test_ext().execute_with(|| {
		assert_noop!(Msa::ensure_msa_owner(&test_public(1), 1), Error::<Test>::NoKeyExists);

		assert_ok!(Msa::add_key(1, &test_public(1), EMPTY_FUNCTION));

		assert_eq!(Msa::ensure_msa_owner(&test_public(1), 1), Ok(()));
	});
}

#[test]
pub fn add_provider_to_msa_is_success() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		// Create provider account and get its MSA ID (u64)
		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));
		let provider_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(provider_account.0)).unwrap();

		// Create delegator account and get its MSA ID (u64)
		assert_ok!(Msa::create(RuntimeOrigin::signed(delegator_account.into())));
		let delegator_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(delegator_account.0)).unwrap();

		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		set_schema_count::<Test>(10);

		assert_ok!(Msa::grant_delegation(
			RuntimeOrigin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		let provider = ProviderId(provider_msa);
		let delegator = DelegatorId(delegator_msa);

		assert_eq!(
			DelegatorAndProviderToDelegation::<Test>::get(delegator, provider),
			Some(Delegation { revoked_at: 0, schema_permissions: Default::default() })
		);

		System::assert_last_event(
			Event::DelegationGranted {
				delegator_id: delegator_msa.into(),
				provider_id: provider_msa.into(),
			}
			.into(),
		);
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
		assert_eq!(PublicKeyToMsaId::<Test>::get(&key), None);
	});
}

#[test]
pub fn revoke_delegation_by_delegator_is_successful() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		assert_ok!(Msa::create(RuntimeOrigin::signed(delegator_account.into())));
		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));

		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		let provider_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(provider_account.0)).unwrap();

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		assert_ok!(Msa::grant_delegation(
			RuntimeOrigin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		assert_ok!(Msa::revoke_delegation_by_delegator(
			RuntimeOrigin::signed(delegator_account.into()),
			2
		));

		System::assert_last_event(
			Event::DelegationRevoked { delegator_id: 1.into(), provider_id: 2.into() }.into(),
		);
	});
}

#[test]
pub fn revoke_provider_is_successful() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		assert_ok!(Msa::create(RuntimeOrigin::signed(delegator_account.into())));
		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));

		let provider_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(provider_account.0)).unwrap();

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		assert_ok!(Msa::grant_delegation(
			RuntimeOrigin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		let delegator_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(delegator_account.0)).unwrap();

		let provider = ProviderId(provider_msa);
		let delegator = DelegatorId(delegator_msa);

		assert_ok!(Msa::revoke_provider(provider, delegator));

		assert_eq!(
			DelegatorAndProviderToDelegation::<Test>::get(delegator, provider).unwrap(),
			Delegation { revoked_at: 1, schema_permissions: Default::default() },
		);
	});
}

#[test]
fn revoke_delegation_by_delegator_does_nothing_when_no_msa() {
	new_test_ext()
		.execute_with(|| assert_revoke_delegation_by_delegator_no_effect(test_public(3), 333u64));
}

#[test]
pub fn revoke_delegation_by_delegator_does_nothing_if_only_key_is_revoked() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::create(RuntimeOrigin::signed(test_public(2))));
		assert_ok!(Msa::delete_key_for_msa(1, &test_public(2)));
		assert_revoke_delegation_by_delegator_no_effect(test_public(2), 1u64)
	})
}

#[test]
pub fn revoke_delegation_by_delegator_fails_if_has_msa_but_no_delegation() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(test_origin_signed(2)));
		assert_noop!(
			Msa::revoke_delegation_by_delegator(test_origin_signed(1), 2),
			Error::<Test>::DelegationNotFound
		);
	})
}

#[test]
fn revoke_delegation_by_delegator_throws_error_when_delegation_already_revoked() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		assert_ok!(Msa::create(RuntimeOrigin::signed(delegator_account.into())));
		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));

		let provider_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(provider_account.0)).unwrap();

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		assert_ok!(Msa::grant_delegation(
			RuntimeOrigin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		assert_ok!(Msa::revoke_delegation_by_delegator(
			RuntimeOrigin::signed(delegator_account.into()),
			provider_msa
		));

		assert_noop!(
			Msa::revoke_delegation_by_delegator(
				RuntimeOrigin::signed(delegator_account.into()),
				provider_msa
			),
			Error::<Test>::DelegationRevoked
		);
	});
}

/// Assert that the call to revoke a delegation is free.
#[test]
pub fn revoke_provider_call_has_no_cost() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(1, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));

		// Register provider
		assert_ok!(Msa::create_provider(test_origin_signed(1), Vec::from("Foo")));

		assert_ok!(Msa::grant_delegation(
			test_origin_signed(1),
			provider_account.into(),
			signature,
			add_provider_payload
		));

		let call = MsaCall::<Test>::revoke_delegation_by_delegator { provider_msa_id: 2 };
		let dispatch_info = call.get_dispatch_info();

		assert_eq!(dispatch_info.pays_fee, Pays::No);
	})
}

#[test]
fn create_provider() {
	new_test_ext().execute_with(|| {
		let (_new_msa_id, key_pair) = create_account();

		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(key_pair.public().into()),
			Vec::from("Foo")
		));
	})
}

#[test]
fn create_provider_max_size_exceeded() {
	new_test_ext().execute_with(|| {
		let (_new_msa_id, key_pair) = create_account();

		assert_err!(
			Msa::create_provider(
				RuntimeOrigin::signed(key_pair.public().into()),
				Vec::from("12345678901234567")
			),
			Error::<Test>::ExceedsMaxProviderNameSize
		);
	})
}

#[test]
fn create_provider_duplicate() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let (_new_msa_id, _) =
			Msa::create_account(key_pair.public().into(), EMPTY_FUNCTION).unwrap();
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(key_pair.public().into()),
			Vec::from("Foo")
		));

		assert_err!(
			Msa::create_provider(RuntimeOrigin::signed(key_pair.public().into()), Vec::from("Foo")),
			Error::<Test>::DuplicateProviderRegistryEntry
		)
	})
}

pub fn set_schema_count<T: Config>(n: u16) {
	<T>::SchemaValidator::set_schema_count(n);
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

		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 2u16, 1u32));
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
			Msa::add_provider(provider, delegator, (1..32_u16).collect::<Vec<_>>()),
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
			Msa::get_granted_schemas_by_msa_id(delegator, Some(provider)),
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
			Msa::get_granted_schemas_by_msa_id(delegator, Some(provider)),
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
		let schemas_granted = Msa::get_granted_schemas_by_msa_id(delegator, Some(provider));
		let expected_schemas_granted = vec![
			SchemaGrant::new(1, BlockNumber::zero()),
			SchemaGrant::new(2, BlockNumber::zero()),
		];
		let expected_delegations = vec![DelegationResponse {
			provider_id: provider,
			permissions: expected_schemas_granted,
		}];
		let output_schemas = schemas_granted.unwrap();
		assert_eq!(output_schemas, expected_delegations);
	})
}

#[test]
pub fn schema_granted_success_multiple_providers_rpc() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(4);

		let provider_1 = ProviderId(1);
		let provider_2 = ProviderId(2);
		let delegator = DelegatorId(2);
		let schema_grants_1 = vec![1, 2];
		let schema_grants_2 = vec![3, 4];
		assert_ok!(Msa::add_provider(provider_1, delegator, schema_grants_1));
		assert_ok!(Msa::add_provider(provider_2, delegator, schema_grants_2));
		let schemas_granted = Msa::get_granted_schemas_by_msa_id(delegator, None);
		let expected_schemas_granted_1 = vec![
			SchemaGrant::new(1, BlockNumber::zero()),
			SchemaGrant::new(2, BlockNumber::zero()),
		];
		let expected_schemas_granted_2 = vec![
			SchemaGrant::new(3, BlockNumber::zero()),
			SchemaGrant::new(4, BlockNumber::zero()),
		];
		let expected_delegation_1 =
			DelegationResponse { provider_id: provider_1, permissions: expected_schemas_granted_1 };
		let expected_delegation_2 =
			DelegationResponse { provider_id: provider_2, permissions: expected_schemas_granted_2 };
		let output_schemas = schemas_granted.unwrap();
		assert_eq!(output_schemas, vec![expected_delegation_1, expected_delegation_2]);
	})
}

#[test]
pub fn schema_revoked_rpc() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(2);

		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		let mut schema_grants = vec![1, 2];
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));
		let mut schemas_granted = Msa::get_granted_schemas_by_msa_id(delegator, Some(provider));
		let mut expected_schemas_granted = vec![
			SchemaGrant::new(1, BlockNumber::zero()),
			SchemaGrant::new(2, BlockNumber::zero()),
		];
		let expected_delegations = vec![DelegationResponse {
			provider_id: provider,
			permissions: expected_schemas_granted,
		}];
		let mut output_schemas = schemas_granted.unwrap();
		assert_eq!(output_schemas, expected_delegations);

		// Now revoke a schema and check that it is reported correctly by the RPC
		run_to_block(5);
		schema_grants = vec![1];
		assert_ok!(Msa::upsert_schema_permissions(provider, delegator, schema_grants));
		schemas_granted = Msa::get_granted_schemas_by_msa_id(delegator, Some(provider));
		expected_schemas_granted =
			vec![SchemaGrant::new(1, BlockNumber::zero()), SchemaGrant::new(2, 5)];
		let expected_delegations = vec![DelegationResponse {
			provider_id: provider,
			permissions: expected_schemas_granted,
		}];
		output_schemas = schemas_granted.unwrap();
		assert_eq!(output_schemas, expected_delegations);
	})
}

#[test]
pub fn add_provider_expired() {
	new_test_ext().execute_with(|| {
		// 1. create two key pairs
		let (provider_pair, _) = sr25519::Pair::generate();
		let (user_pair, _) = sr25519::Pair::generate();

		let provider_key = provider_pair.public();
		let delegator_key = user_pair.public();

		// 2. create provider MSA
		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_key.into()))); // MSA = 1

		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_key.into()),
			Vec::from("Foo")
		));

		// 3. create delegator MSA and provider to provider
		let expiration: BlockNumber = 0;

		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
		let signature: MultiSignature = user_pair.sign(&encode_add_provider_data).into();
		// 3.5 create the user's MSA + add provider as provider
		assert_err!(
			Msa::grant_delegation(
				test_origin_signed(1),
				delegator_key.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::ProofHasExpired
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
fn delegation_default_trait_impl() {
	new_test_ext().execute_with(|| {
		let delegation: Delegation<
			SchemaId,
			BlockNumberFor<Test>,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		> = Default::default();

		let expected = Delegation {
			schema_permissions: BoundedBTreeMap::<
				SchemaId,
				BlockNumberFor<Test>,
				<Test as Config>::MaxSchemaGrantsPerDelegation,
			>::default(),
			revoked_at: Default::default(),
		};

		assert_eq!(delegation, expected);
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

		assert!(DelegatorAndProviderToDelegation::<Test>::get(delegator, provider).is_some());
	});
}

#[test]
fn msa_id_to_eth_address_binary() {
	let msa_ids: [MessageSourceId; 2] = [1234u64, 4321u64];
	let expected = [
		H160(
			hex::decode("65928b9a88db189eea76f72d86128af834d64c32")
				.unwrap()
				.try_into()
				.unwrap(),
		),
		H160(
			hex::decode("f2f77409b0054b4b14911f00961140deb316ab39")
				.unwrap()
				.try_into()
				.unwrap(),
		),
	];

	for i in 0..msa_ids.len() {
		let eth_address = Msa::msa_id_to_eth_address(msa_ids[i]);
		assert_eq!(eth_address, expected[i]);
	}
}

#[test]
fn validate_eth_address_for_msa_good() {
	let msa_ids: [MessageSourceId; 2] = [1234u64, 4321u64];
	let expected = msa_ids.map(|msa_id| Msa::msa_id_to_eth_address(msa_id));

	for i in 0..msa_ids.len() {
		let status = Msa::validate_eth_address_for_msa(&expected[i], msa_ids[i]);
		assert_eq!(status, true);
	}
}

#[test]
fn validate_eth_address_for_msa_bad() {
	let msa_ids = [1234u64, 4321u64];
	let expected: [H160; 2] = msa_ids
		.map(|msa_id| Msa::msa_id_to_eth_address(msa_id))
		.iter()
		.rev()
		.cloned()
		.collect::<Vec<_>>()
		.try_into()
		.unwrap();

	for i in 0..msa_ids.len() {
		let status = Msa::validate_eth_address_for_msa(&expected[i], msa_ids[i]);
		assert_eq!(status, false);
	}
}

#[test]
fn eth_address_to_checksummed_string() {
	let eth_addresses: [H160; 5] = [
		H160(
			hex::decode("5aaeb6053f3e94c9b9a09f33669435e7ef1beaed")
				.unwrap()
				.try_into()
				.unwrap(),
		),
		H160(
			hex::decode("fb6916095ca1df60bb79ce92ce3ea74c37c5d359")
				.unwrap()
				.try_into()
				.unwrap(),
		),
		H160(
			hex::decode("dbf03b407c01e7cd3cbea99509d93f8dddc8c6fb")
				.unwrap()
				.try_into()
				.unwrap(),
		),
		H160(
			hex::decode("d1220a0cf47c7b9be7a2e6ba89f429762e7b9adb")
				.unwrap()
				.try_into()
				.unwrap(),
		),
		H160(
			hex::decode("f5b82ff246a2f4226749bd78b1bdae28cfffb9f7")
				.unwrap()
				.try_into()
				.unwrap(),
		),
	];

	// Test values from https://github.com/ethereum/ercs/blob/master/ERCS/erc-55.md
	let eth_results: [alloc::string::String; 5] = [
		"0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed".to_string(),
		"0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359".to_string(),
		"0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB".to_string(),
		"0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb".to_string(),
		"0xF5b82ff246a2F4226749bd78B1bDaE28Cfffb9f7".to_string(),
	];

	for i in 0..eth_addresses.len() {
		let generated_result = Msa::eth_address_to_checksummed_string(&eth_addresses[i]);
		assert_eq!(generated_result, eth_results[i]);
	}
}

// Test data generated by [this tool](../../eth-migration/metamask.html)
#[test]
fn ethereum_eip712_signatures_for_add_key_should_work() {
	new_test_ext().execute_with(|| {
		let address = EthereumAddressMapper::to_account_id(&from_hex("0x7A23F8D62589aB9651722C7F4a0E998D7d3Ef2A9").unwrap_or_default());
		let payload: AddKeyData<Test> = AddKeyData {
			msa_id: 12876327,
			expiration: 100,
			new_public_key: address.into(),
		};
		let encoded_payload = payload.encode_eip_712(420420420u32);

		// following signature is generated via Metamask using the same input to check compatibility
		let signature_raw = from_hex("0x7fb9df5e7f51875509456fe24de92c256c4dcaaaeb952fe36bb30f79c8cc3bbf2f988fa1c55efb6bf20825e98de5cc1ac0bdcf036ad1e0f9ee969a729540ff8d1c").expect("Should convert");
		let unified_signature = UnifiedSignature::from(ecdsa::Signature::from_raw(
			signature_raw.try_into().expect("should convert"),
		));

		// Non-compressed public key associated with the keypair used in Metamask
		// 0x509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9fa213197dc0666e85529d6c9dda579c1295d61c417f01505765481e89a4016f02
		let public_key = ecdsa::Public::from_raw(
			from_hex("0x02509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9f")
				.expect("should convert")
				.try_into()
				.expect("invalid size"),
		);
		let unified_signer = UnifiedSigner::from(public_key);
		assert!(unified_signature.verify(&encoded_payload[..], &unified_signer.into_account()));
	});
}

#[test]
fn ethereum_eip712_signatures_for_authorized_key_should_work() {
	new_test_ext().execute_with(|| {
		let address = EthereumAddressMapper::to_account_id(&from_hex("0x7A23F8D62589aB9651722C7F4a0E998D7d3Ef2A9").unwrap_or_default());
		let payload: AuthorizedKeyData<Test> = AuthorizedKeyData {
			discriminant: PayloadTypeDiscriminator::AuthorizedKeyData,
			msa_id: 12876327,
			expiration: 100,
			authorized_public_key: address.into(),
		};
		let encoded_payload = payload.encode_eip_712(420420420u32);

		// following signature is generated via Metamask using the same input to check compatibility
		let signature_raw = from_hex("0x9dec03e5c93e2b9619cc8fd77383a8fc38d4aa3dc20fa26be436d386acb380260e2a82e677b71f28adc7cc63b60855ccc481057307a1b05dbb2f5af19c66b5461c").expect("Should convert");
		let unified_signature = UnifiedSignature::from(ecdsa::Signature::from_raw(
			signature_raw.try_into().expect("should convert"),
		));

		// Non-compressed public key associated with the keypair used in Metamask
		// 0x509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9fa213197dc0666e85529d6c9dda579c1295d61c417f01505765481e89a4016f02
		let public_key = ecdsa::Public::from_raw(
			from_hex("0x02509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9f")
				.expect("should convert")
				.try_into()
				.expect("invalid size"),
		);
		let unified_signer = UnifiedSigner::from(public_key);
		assert!(unified_signature.verify(&encoded_payload[..], &unified_signer.into_account()));
	});
}

#[test]
fn ethereum_eip712_signatures_for_add_provider_should_work() {
	new_test_ext().execute_with(|| {
		let payload = AddProvider {
			authorized_msa_id: 12876327,
			schema_ids: vec![2,4,5,6,7,8],
			expiration: 100,
		};
		let encoded_payload = payload.encode_eip_712(420420420u32);

		// following signature is generated via Metamask using the same input to check compatibility
		let signature_raw = from_hex("0x34ed5cc291815bdc7d95b418b341bbd3d9ca82c284d5f22d8016c27bb9d4eef8507cdb169a40e69dc5d7ee8ff0bff29fa0d8fc4e73cad6fc9bf1bf076f8e0a741c").expect("Should convert");
		let unified_signature = UnifiedSignature::from(ecdsa::Signature::from_raw(
			signature_raw.try_into().expect("should convert"),
		));

		// Non-compressed public key associated with the keypair used in Metamask
		// 0x509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9fa213197dc0666e85529d6c9dda579c1295d61c417f01505765481e89a4016f02
		let public_key = ecdsa::Public::from_raw(
			from_hex("0x02509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9f")
				.expect("should convert")
				.try_into()
				.expect("invalid size"),
		);
		let unified_signer = UnifiedSigner::from(public_key);
		assert!(unified_signature.verify(&encoded_payload[..], &unified_signer.into_account()));
	});
}
