use frame_support::{
	assert_noop, assert_ok,
	dispatch::{GetDispatchInfo, Pays},
	BoundedBTreeMap,
};

use sp_runtime::MultiSignature;

use crate::{
	tests::{mock::*, other_tests::set_schema_count},
	types::AddProvider,
	Error, Event,
};
use common_primitives::{
	msa::{Delegation, DelegationValidator, DelegatorId, ProviderId},
	node::BlockNumber,
	schema::SchemaId,
	utils::wrap_binary_data,
};
use sp_core::{crypto::AccountId32, sr25519, Encode, Pair};

fn create_two_keypairs() -> (sr25519::Pair, sr25519::Pair) {
	// fn create_two_keypairs() -> (Public, Public) {
	let (pair1, _) = sr25519::Pair::generate();
	let (pair2, _) = sr25519::Pair::generate();
	(pair1, pair2)
	// (pair1.public(), pair2.public())
}

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

		let mut sp = BoundedBTreeMap::<SchemaId, u32, MaxSchemaGrantsPerDelegation>::new();
		assert_ok!(sp.try_insert(1u16, 0u32));
		assert_ok!(sp.try_insert(2u16, 0u32));
		assert_ok!(sp.try_insert(3u16, 0u32));

		let expected = Delegation { revoked_at: 0u32, schema_permissions: sp };

		assert_eq!(Msa::get_delegation(delegator, provider), Some(expected));

		let revoked_block_number: u32 = 100;
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

		let mut sp = BoundedBTreeMap::<SchemaId, u32, MaxSchemaGrantsPerDelegation>::new();
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

		let mut sp = BoundedBTreeMap::<SchemaId, u32, MaxSchemaGrantsPerDelegation>::new();
		assert_ok!(sp.try_insert(1u16, 100u32)); // schema id 1 revoked at block 100
		assert_ok!(sp.try_insert(2u16, 0u32)); // schema id 2 granted (block 0)
		assert_ok!(sp.try_insert(3u16, 0u32)); // schema id 3 granted (block 0)
		assert_ok!(sp.try_insert(4u16, 0u32)); // schema id 4 granted (block 0)

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

#[test]
pub fn revoke_delegation_by_provider_happy_path() {
	new_test_ext().execute_with(|| {
		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		let (provider_msa_id, provider_pair) = create_account();
		let provider_account = provider_pair.public();

		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("provider")
		));

		// 3. create delegator MSA and provider to provider
		let (signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa_id);

		// 3.5 create the user's MSA + add provider as provider
		assert_ok!(Msa::create_sponsored_account_with_delegation(
			RuntimeOrigin::signed(AccountId32::from(provider_account)),
			delegator_account.into(),
			signature,
			add_provider_payload
		));
		let retrieved_delegator = Msa::get_owner_of(&AccountId32::from(delegator_account)).unwrap();

		//  4. set some block number to ensure it's not a default value
		System::set_block_number(System::block_number() + 25);

		// 5. assert_ok! fn as 2 to remove provider 1
		assert_ok!(Msa::revoke_delegation_by_provider(
			RuntimeOrigin::signed(AccountId32::from(provider_account)),
			retrieved_delegator
		));

		// 6. verify that the provider is revoked
		let provider_info = Msa::get_delegation(DelegatorId(2), ProviderId(1));
		assert_eq!(
			provider_info,
			Some(Delegation { revoked_at: 26, schema_permissions: Default::default() })
		);

		// 7. verify the event
		System::assert_last_event(
			Event::DelegationRevoked { provider_id: ProviderId(1), delegator_id: DelegatorId(2) }
				.into(),
		);
	})
}

#[test]
pub fn revoke_delegation_by_provider_has_correct_costs() {
	new_test_ext().execute_with(|| {
		let call = MsaCall::<Test>::revoke_delegation_by_provider { delegator: 2 };
		let dispatch_info = call.get_dispatch_info();

		assert_eq!(dispatch_info.pays_fee, Pays::No);
	})
}

#[test]
pub fn revoke_delegation_by_provider_does_nothing_when_no_msa() {
	new_test_ext().execute_with(|| {
		let (delegator_pair, provider_pair) = create_two_keypairs();
		let delegator_account = delegator_pair.public();
		let provider_account = provider_pair.public();

		let none_retrieved_delegator = Msa::get_owner_of(&AccountId32::from(delegator_account));
		assert_eq!(none_retrieved_delegator, None);

		let not_an_msa_id = 777u64;

		assert_ok!(Msa::create(RuntimeOrigin::signed(AccountId32::from(provider_account))));

		System::set_block_number(System::block_number() + 19);

		// 1. when delegator msa_id not found
		assert_noop!(
			Msa::revoke_delegation_by_provider(
				RuntimeOrigin::signed(AccountId32::from(provider_account)),
				not_an_msa_id
			),
			Error::<Test>::DelegationNotFound
		);

		assert_ok!(Msa::create(RuntimeOrigin::signed(AccountId32::from(delegator_account))));
		let delegator_msa_id = Msa::get_owner_of(&AccountId32::from(delegator_account)).unwrap();
		// 2. when no delegation relationship
		assert_noop!(
			Msa::revoke_delegation_by_provider(
				RuntimeOrigin::signed(AccountId32::from(provider_account)),
				delegator_msa_id
			),
			Error::<Test>::DelegationNotFound
		);

		assert_ok!(Msa::add_provider(ProviderId(1), DelegatorId(2), Vec::default()));
		assert_ok!(Msa::revoke_provider(ProviderId(1), DelegatorId(2)));

		// 3. when_delegation_expired
		assert_noop!(
			Msa::revoke_delegation_by_provider(
				RuntimeOrigin::signed(AccountId32::from(provider_account)),
				delegator_msa_id
			),
			Error::<Test>::DelegationRevoked
		);
	})
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
