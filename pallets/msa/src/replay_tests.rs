use crate::{mock::*, AddKeyData, AddProvider, Error};

use common_primitives::{node::BlockNumber, utils::wrap_binary_data};
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::{sr25519, Encode, Pair};
use sp_runtime::MultiSignature;

// This scenario must fail:
// 1. User creates MSA and delegates to provider
// 2. User revokes msa delegation
// 3. User adds a key to their msa
// 4. User deletes first key from msa
// 5. Provider successfully calls "create_sponsored_account_with_delegation", OR
//    Provider successfully calls "grant_delegation" with same payload and proof/signature.
#[test]
pub fn replaying_create_sponsored_account_with_delegation_fails() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_key = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_key = key_pair_delegator.public();

		let expiration: BlockNumber = 100;
		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		// create MSA for provider and register them
		assert_ok!(Msa::create(Origin::signed(provider_key.into())));
		assert_ok!(Msa::register_provider(Origin::signed(provider_key.into()), Vec::from("Foo")));

		// Step 1
		assert_ok!(Msa::create_sponsored_account_with_delegation(
			Origin::signed(provider_key.into()),
			delegator_key.into(),
			signature.clone(),
			add_provider_payload.clone()
		));
		run_to_block(25);

		// Step 2
		assert_ok!(Msa::revoke_delegation_by_delegator(Origin::signed(delegator_key.into()), 1));
		run_to_block(40);

		// Step 3
		let (key_pair_delegator2, _) = sr25519::Pair::generate();
		let delegator_account2 = key_pair_delegator2.public();

		let add_key_payload: AddKeyData = AddKeyData { msa_id: 2, nonce: 0, expiration: 110 };
		let encode_add_key_data = wrap_binary_data(add_key_payload.encode());
		let add_key_signature_delegator = key_pair_delegator.sign(&encode_add_key_data);
		let add_key_signature_new_key = key_pair_delegator2.sign(&encode_add_key_data);

		run_to_block(55);

		assert_ok!(Msa::add_key_to_msa(
			Origin::signed(delegator_key.into()),
			delegator_key.into(),
			add_key_signature_delegator.into(),
			delegator_account2.into(),
			add_key_signature_new_key.into(),
			add_key_payload
		));
		run_to_block(60);

		assert_ok!(Msa::delete_msa_public_key(
			Origin::signed(delegator_account2.into()),
			delegator_key.into(),
		));
		run_to_block(75);

		// expect call create with same signature to fail
		assert_err!(
			Msa::create_sponsored_account_with_delegation(
				Origin::signed(provider_key.into()),
				delegator_key.into(),
				signature.clone(),
				add_provider_payload.clone(),
			),
			Error::<Test>::SignatureAlreadySubmitted
		);
		run_to_block(99);

		// expect this to fail for the same reason
		assert_err!(
			Msa::grant_delegation(
				Origin::signed(provider_key.into()),
				delegator_key.into(),
				signature.clone(),
				add_provider_payload.clone(),
			),
			Error::<Test>::SignatureAlreadySubmitted
		);
	})
}

// This scenario should fail:
//   1. provider authorizes being added as provider to MSA and MSA account adds them.
//   2. provider removes them as MSA (say by quickly discovering MSA is undesirable)
//   3. MSA account replays the add, using the previous signed payload + signature.
#[test]
fn replaying_grant_delegation_fails() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_key = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_key = key_pair_delegator.public();

		// add_provider_payload in this case has delegator's msa_id as authorized_msa_id
		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		// DELEGATOR signs to add the provider
		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		// create MSA for provider and register them
		assert_ok!(Msa::create(Origin::signed(provider_key.into())));
		assert_ok!(Msa::register_provider(Origin::signed(provider_key.into()), Vec::from("Foo")));

		// create MSA for delegator
		assert_ok!(Msa::create(Origin::signed(delegator_key.into())));

		assert_ok!(Msa::grant_delegation(
			Origin::signed(provider_key.into()),
			delegator_key.into(),
			signature.clone(),
			add_provider_payload.clone(),
		));

		// provider revokes the delegation.
		assert_ok!(Msa::revoke_delegation_by_provider(Origin::signed(provider_key.into()), 2));
		System::set_block_number(System::block_number() + 1);

		// Expected to fail because revoking the delegation just expires it at a given block number.
		assert_err!(
			Msa::grant_delegation(
				Origin::signed(provider_key.into()),
				delegator_key.into(),
				signature.clone(),
				add_provider_payload.clone(),
			),
			Error::<Test>::SignatureAlreadySubmitted
		);
	})
}

#[test]
pub fn add_signature_replay_fails() {
	struct TestCase {
		current: u64,
		mortality: u64,
		run_to: u64,
	}
	new_test_ext().execute_with(|| {
		// these should all fail replay
		let test_cases: Vec<TestCase> = vec![
			TestCase { current: 10_849u64, mortality: 11_001u64, run_to: 11_000u64 }, // fails test
			TestCase { current: 1u64, mortality: 3u64, run_to: 2u64 },
			TestCase { current: 99u64, mortality: 101u64, run_to: 100u64 },
			TestCase { current: 1_000u64, mortality: 1_199u64, run_to: 1_198u64 },
			TestCase { current: 1_002u64, mortality: 1_201u64, run_to: 1_200u64 },
			TestCase { current: 999u64, mortality: 1_148u64, run_to: 1_101u64 },
		];
		for tc in test_cases {
			System::set_block_number(tc.current);
			let sig1 = &generate_test_signature();
			assert_ok!(Msa::register_signature(sig1, tc.mortality));
			run_to_block(tc.run_to);
			assert_noop!(
				Msa::register_signature(sig1, tc.mortality),
				Error::<Test>::SignatureAlreadySubmitted,
			);
		}
	});
}

#[test]
// This scenario must fail:
//     1. User Signed Request to Provider: create_sponsored_account_with_delegation
//     2. User Request Direct to Chain: retire_msa
//     3. The Provider from Step 1 attempts to create a NEW MSA with (key from Step 1)
//     4. Transaction 1-3 all executed before Step 1's payload expireBlock
fn replaying_create_sponsored_account_with_delegation_fails_02() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_key = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_key = key_pair_delegator.public();
		let expiration: BlockNumber = 100;
		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		// create MSA for provider and register them
		assert_ok!(Msa::create(Origin::signed(provider_key.into())));
		assert_ok!(Msa::register_provider(Origin::signed(provider_key.into()), Vec::from("Foo")));

		// Step 1
		assert_ok!(Msa::create_sponsored_account_with_delegation(
			Origin::signed(provider_key.into()),
			delegator_key.into(),
			signature.clone(),
			add_provider_payload.clone()
		));
		run_to_block(2);

		// Step 2
		assert_ok!(Msa::retire_msa(Origin::signed(delegator_key.into())));
		run_to_block(3);

		// Step 3
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				Origin::signed(provider_key.into()),
				delegator_key.into(),
				signature.clone(),
				add_provider_payload.clone()
			),
			Error::<Test>::SignatureAlreadySubmitted
		);
	});
}

#[test]
// This scenario must fail:
//     1. User Signed Request to Provider: create_sponsored_account_with_delegation
//     2. User Request Direct to Chain: add_key_to_msa
//     3. User Wallet: delete_key_from_msa (using the original delegator key from Step 1)
//     4. The Provider from Step 1 attempts to create a NEW MSA with the original delegator key from Step 1
// Transaction 1-4 all executed before Step 1's payload expireBlock
fn replaying_create_sponsored_account_with_delegation_fails_03() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_key = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_key = key_pair_delegator.public();
		let add_provider_payload = AddProvider::new(1u64, None, 100);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
		let original_msa_creation_signature: MultiSignature =
			key_pair_delegator.sign(&encode_add_provider_data).into();

		// create MSA for provider and register them
		assert_ok!(Msa::create(Origin::signed(provider_key.into())));
		assert_ok!(Msa::register_provider(Origin::signed(provider_key.into()), Vec::from("Foo")));
		// Step 1
		assert_ok!(Msa::create_sponsored_account_with_delegation(
			Origin::signed(provider_key.into()),
			delegator_key.into(),
			original_msa_creation_signature.clone(),
			add_provider_payload.clone()
		));
		run_to_block(5);

		let (new_key_pair, _) = sr25519::Pair::generate();
		let new_public_key = new_key_pair.public();
		let add_new_key_data = AddKeyData { nonce: 1, msa_id: 2, expiration: 10 };
		let encode_add_key_data = wrap_binary_data(add_new_key_data.encode());

		let add_key_signature_delegator = key_pair_delegator.sign(&encode_add_key_data);
		let add_key_signature: MultiSignature = new_key_pair.sign(&encode_add_key_data).into();

		// Step 2.
		assert_ok!(Msa::add_key_to_msa(
			Origin::signed(delegator_key.into()),
			delegator_key.into(),
			add_key_signature_delegator.into(),
			new_public_key.into(),
			add_key_signature,
			add_new_key_data.clone()
		));
		run_to_block(75);

		// Step 3
		assert_ok!(Msa::delete_msa_public_key(
			Origin::signed(new_public_key.into()),
			delegator_key.into()
		));
		run_to_block(99);

		// Step 4
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				Origin::signed(provider_key.into()),
				delegator_key.into(),
				original_msa_creation_signature.clone(),
				add_provider_payload.clone()
			),
			Error::<Test>::SignatureAlreadySubmitted
		);
	});
}
