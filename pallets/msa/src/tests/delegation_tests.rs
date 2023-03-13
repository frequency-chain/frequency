use frame_support::{assert_noop, assert_ok, BoundedBTreeMap};

use sp_runtime::MultiSignature;

use crate::{
	tests::{common_tests::set_schema_count, mock::*},
	types::AddProvider,
	Error, Event,
};
use common_primitives::{
	msa::{Delegation, DelegatorId, ProviderId},
	node::BlockNumber,
	schema::SchemaId,
	utils::wrap_binary_data,
};
use sp_core::{crypto::AccountId32, sr25519, Encode, Pair};

#[test]
pub fn grant_delegation_changes_schema_permissions() {
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

		let block_expiration: BlockNumber = 110;

		System::set_block_number(90);
		set_schema_count::<Test>(10);

		// Create delegation without any schema permissions
		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload_with_schemas(
				delegator_pair.clone(),
				provider_msa,
				None,
				block_expiration,
			);

		assert_ok!(Msa::grant_delegation(
			RuntimeOrigin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		let provider = ProviderId(provider_msa);
		let delegator = DelegatorId(delegator_msa);

		assert_eq!(
			Msa::get_delegation(delegator, provider),
			Some(Delegation { revoked_at: 0, schema_permissions: Default::default() })
		);

		System::assert_last_event(
			Event::DelegationGranted {
				delegator_id: delegator_msa.into(),
				provider_id: provider_msa.into(),
			}
			.into(),
		);

		// Grant delegation w/schemas 1, 2, and 3 (implies block 0)
		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload_with_schemas(
				delegator_pair.clone(),
				provider_msa,
				Some(vec![1, 2, 3]),
				block_expiration,
			);

		assert_ok!(Msa::grant_delegation(
			RuntimeOrigin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		let mut sp = BoundedBTreeMap::<SchemaId, u64, MaxSchemaGrantsPerDelegation>::new();
		assert_ok!(sp.try_insert(1u16, 0u64));
		assert_ok!(sp.try_insert(2u16, 0u64));
		assert_ok!(sp.try_insert(3u16, 0u64));

		let expected = Delegation { revoked_at: 0, schema_permissions: sp };

		assert_eq!(Msa::get_delegation(delegator, provider), Some(expected));

		let revoked_block_number: u64 = 100;
		System::set_block_number(revoked_block_number);
		// Revoke all schema ids.
		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload_with_schemas(
				delegator_pair.clone(),
				provider_msa,
				None,
				block_expiration,
			);

		assert_ok!(Msa::grant_delegation(
			RuntimeOrigin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		let mut sp = BoundedBTreeMap::<SchemaId, u64, MaxSchemaGrantsPerDelegation>::new();
		assert_ok!(sp.try_insert(1u16, revoked_block_number)); // schema id 1 revoked at revoked_block_number
		assert_ok!(sp.try_insert(2u16, revoked_block_number)); // schema id 2 revoked at revoked_block_number
		assert_ok!(sp.try_insert(3u16, revoked_block_number)); // schema id 3 revoked at revoked_block_number

		let expected = Delegation { revoked_at: 0, schema_permissions: sp };

		assert_eq!(Msa::get_delegation(delegator, provider), Some(expected));

		System::set_block_number(revoked_block_number + 1);
		// Grant 2, 3, 4
		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload_with_schemas(
				delegator_pair.clone(),
				provider_msa,
				Some(vec![2, 3, 4]),
				block_expiration,
			);

		assert_ok!(Msa::grant_delegation(
			RuntimeOrigin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		let mut sp = BoundedBTreeMap::<SchemaId, u64, MaxSchemaGrantsPerDelegation>::new();
		assert_ok!(sp.try_insert(1u16, 100u64)); // schema id 1 revoked at block 100
		assert_ok!(sp.try_insert(2u16, 0u64)); // schema id 2 granted (block 0)
		assert_ok!(sp.try_insert(3u16, 0u64)); // schema id 3 granted (block 0)
		assert_ok!(sp.try_insert(4u16, 0u64)); // schema id 4 granted (block 0)

		let expected = Delegation { revoked_at: 0, schema_permissions: sp };
		assert_eq!(Msa::get_delegation(delegator, provider), Some(expected));
	});
}

#[test]
pub fn grant_delegation_to_msa_throws_add_provider_verification_failed() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let account = key_pair.public();
		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(2, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();
		let fake_provider_payload = AddProvider::new(3, None, expiration);
		assert_noop!(
			Msa::grant_delegation(
				RuntimeOrigin::signed(account.into()),
				account.into(),
				signature,
				fake_provider_payload
			),
			Error::<Test>::AddProviderSignatureVerificationFailed
		);
	});
}

#[test]
pub fn grant_delegation_throws_no_key_exist_error() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(2, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_noop!(
			Msa::grant_delegation(
				test_origin_signed(1),
				provider_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::NoKeyExists
		);
	});
}

#[test]
pub fn grant_delegation_throws_key_revoked_error() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(2, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));
		assert_ok!(Msa::delete_key_for_msa(1, &test_public(1)));

		assert_noop!(
			Msa::grant_delegation(
				test_origin_signed(1),
				provider_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::NoKeyExists
		);
	});
}

#[test]
pub fn grant_delegation_throws_invalid_self_provider_error() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(1, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));

		assert_noop!(
			Msa::grant_delegation(
				RuntimeOrigin::signed(provider_account.into()),
				provider_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::InvalidSelfProvider
		);
	});
}

#[test]
pub fn grant_delegation_throws_unauthorized_delegator_error() {
	new_test_ext().execute_with(|| {
		// Generate a key pair for the provider
		let (provider_key_pair, _) = sr25519::Pair::generate();
		let provider_account = provider_key_pair.public();

		// Generate a key pair for the delegator
		let (delegator_key_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_key_pair.public();
		assert_ok!(Msa::create(RuntimeOrigin::signed(delegator_account.into())));
		let delegator_msa_id =
			Msa::ensure_valid_msa_key(&AccountId32::new(delegator_account.0)).unwrap();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(delegator_msa_id, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
		let signature: MultiSignature = delegator_key_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));

		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		assert_noop!(
			Msa::grant_delegation(
				RuntimeOrigin::signed(provider_account.into()),
				delegator_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::UnauthorizedDelegator
		);
	});
}
