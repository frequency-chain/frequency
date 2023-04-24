use crate::{tests::mock::*, AddKeyData, AddProvider, Error};

use common_primitives::{node::BlockNumber, utils::wrap_binary_data};
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::{sr25519, Encode, Pair};
use sp_runtime::MultiSignature;

fn create_add_provider_payload(signature_expiration: BlockNumber) -> (AddProvider, Vec<u8>) {
	let add_provider_payload = AddProvider::new(1u64, None, signature_expiration);
	let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
	(add_provider_payload, encode_add_provider_data)
}

pub fn user_creates_and_delegates_to_provider(
	delegator_keypair: sp_core::sr25519::Pair,
	provider_keypair: sp_core::sr25519::Pair,
	signature_expiration: BlockNumber,
) -> (MultiSignature, AddProvider) {
	let delegator_key = delegator_keypair.public();
	let provider_key = provider_keypair.public();

	let (payload, encode_add_provider_data) = create_add_provider_payload(signature_expiration);

	let signature: MultiSignature = delegator_keypair.sign(&encode_add_provider_data).into();
	assert_ok!(Msa::create_sponsored_account_with_delegation(
		RuntimeOrigin::signed(provider_key.into()),
		delegator_key.into(),
		signature.clone(),
		payload.clone()
	));
	(signature.clone(), payload.clone())
}

pub fn user_adds_key_to_msa(
	delegator_pair: sp_core::sr25519::Pair,
	new_pair: sp_core::sr25519::Pair,
) {
	let add_key_payload =
		AddKeyData { msa_id: 2, expiration: 109, new_public_key: new_pair.public().into() };
	let encode_add_key_data = wrap_binary_data(add_key_payload.encode());
	let msa_owner_signature = delegator_pair.sign(&encode_add_key_data);
	let signature_new_key: MultiSignature = new_pair.sign(&encode_add_key_data).into();

	assert_ok!(Msa::add_public_key_to_msa(
		RuntimeOrigin::signed(delegator_pair.public().into()),
		delegator_pair.public().into(),
		msa_owner_signature.into(),
		signature_new_key,
		add_key_payload
	));
}

fn create_user_and_provider() -> (sr25519::Pair, sr25519::Pair) {
	let (provider_keypair, _) = sr25519::Pair::generate();

	let (delegator_keypair, _) = sr25519::Pair::generate();

	// create MSA for provider and register them
	assert_ok!(Msa::create(RuntimeOrigin::signed(provider_keypair.public().into())));
	assert_ok!(Msa::create_provider(
		RuntimeOrigin::signed(provider_keypair.public().into()),
		Vec::from("Foo")
	));
	(delegator_keypair, provider_keypair)
}

// This scenario must fail:
// 1. User creates MSA and delegates to provider
// 2. User revokes msa delegation
// 3. User adds a key to their msa
// 4. User deletes first key from msa
// 5. Provider successfully calls "create_sponsored_account_with_delegation", OR
//    Provider successfully calls "grant_delegation" with same payload and proof/signature,
//      using first (deleted) key
#[test]
pub fn replaying_create_sponsored_account_with_delegation_fails() {
	new_test_ext().execute_with(|| {
		let (delegator_keypair, provider_keypair) = create_user_and_provider();
		let provider_key = provider_keypair.public();
		let delegator_key = delegator_keypair.public();

		// Step 1
		let (signature, add_provider_payload) = user_creates_and_delegates_to_provider(
			delegator_keypair.clone(),
			provider_keypair,
			99u32.into(),
		);
		run_to_block(25);

		// Step 2
		assert_ok!(Msa::revoke_delegation_by_delegator(
			RuntimeOrigin::signed(delegator_key.into()),
			1
		));
		run_to_block(40);

		// Step 3
		let (new_keypair, _) = sr25519::Pair::generate();
		user_adds_key_to_msa(delegator_keypair, new_keypair.clone());

		assert_ok!(Msa::delete_msa_public_key(
			RuntimeOrigin::signed(new_keypair.public().into()),
			delegator_key.into(),
		));
		run_to_block(75);

		// expect call create with same signature to fail
		assert_err!(
			Msa::create_sponsored_account_with_delegation(
				RuntimeOrigin::signed(provider_key.into()),
				delegator_key.into(),
				signature.clone(),
				add_provider_payload.clone(),
			),
			Error::<Test>::SignatureAlreadySubmitted
		);
		run_to_block(98);

		// expect this to fail for the same reason
		assert_err!(
			Msa::grant_delegation(
				RuntimeOrigin::signed(provider_key.into()),
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
		let (delegator_keypair, provider_keypair) = create_user_and_provider();
		let provider_key = provider_keypair.public();
		let delegator_key = delegator_keypair.public();

		// add_provider_payload in this case has delegator's msa_id as authorized_msa_id
		let (add_provider_payload, encode_add_provider_data) =
			create_add_provider_payload(99u32.into());

		// DELEGATOR signs to add the provider
		let signature: MultiSignature = delegator_keypair.sign(&encode_add_provider_data).into();

		// create MSA for delegator
		assert_ok!(Msa::create(RuntimeOrigin::signed(delegator_key.into())));

		assert_ok!(Msa::grant_delegation(
			RuntimeOrigin::signed(provider_key.into()),
			delegator_key.into(),
			signature.clone(),
			add_provider_payload.clone(),
		));

		// provider revokes the delegation.
		assert_ok!(Msa::revoke_delegation_by_provider(
			RuntimeOrigin::signed(provider_key.into()),
			2
		));
		System::set_block_number(System::block_number() + 1);

		// Expected to fail because revoking the delegation just expires it at a given block number.
		assert_err!(
			Msa::grant_delegation(
				RuntimeOrigin::signed(provider_key.into()),
				delegator_key.into(),
				signature.clone(),
				add_provider_payload.clone(),
			),
			Error::<Test>::SignatureAlreadySubmitted
		);
	})
}

#[test]
pub fn add_signature_replay_boundary_checks() {
	struct TestCase {
		current: u32,
		mortality: u32,
		run_to: u32,
	}
	new_test_ext().execute_with(|| {
		// This tests signature replay attacks for mortality window size = 100 and 2 buckets,
		// by looking at different boundary cases.  It checks that they can't
		// be resubmitted at the expiration block. We assume if they cannot be replayed at a
		// boundary that they can't be replayed earlier than that boundary, given that we are
		// checking explicitly for the error `SignatureAlreadySubmitted`.
		let test_cases: Vec<TestCase> = vec![
			// 1-block expiration for bucket 0, mortality crosses no boundary
			TestCase { current: 1u32, mortality: 3u32, run_to: 2u32 },
			// expiration for bucket 1, mortality is fast, crosses a 100-block boundary, we check at the boundary.
			// at block 100, on_initialize's bucket-clearing has happened by the time register_extrinsic is called,
			// so this should make sure that the signature is still there and the wrong bucket was not cleared.
			TestCase { current: 99u32, mortality: 101u32, run_to: 100u32 },
			// This does the same as above, only it's a different bucket.
			TestCase { current: 11_149u32, mortality: 11_201u32, run_to: 11_200u32 },
			// This sets mortality at a boundary
			TestCase { current: 11_149u32, mortality: 11_200u32, run_to: 11_199u32 },
			// This does the same as above, but for a different bucket.
			TestCase { current: 11_249u32, mortality: 11_300u32, run_to: 11_299u32 },
			// This case sets current block at a boundary, and sets the mortality to the very last block before the boundary
			TestCase { current: 1_100u32, mortality: 1_199u32, run_to: 1_198u32 },
			// This case has current block right before a boundary, and sets expiration to the very last possible block
			TestCase { current: 1_699u32, mortality: 1_798u32, run_to: 1_797u32 },
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
			run_to_block(tc.mortality);
			assert_noop!(
				Msa::register_signature(sig1, tc.mortality),
				Error::<Test>::ProofHasExpired,
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
		let (delegator_keypair, provider_keypair) = create_user_and_provider();
		let provider_key = provider_keypair.public();
		let delegator_key = delegator_keypair.public();

		// Step 1
		let (signature, add_provider_payload) = user_creates_and_delegates_to_provider(
			delegator_keypair.clone(),
			provider_keypair,
			99u32.into(),
		);
		run_to_block(2);

		// Step 2
		assert_ok!(Msa::retire_msa(RuntimeOrigin::signed(delegator_key.into())));
		run_to_block(3);

		// Step 3
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				RuntimeOrigin::signed(provider_key.into()),
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
//     2. User Request Direct to Chain: add_public_key_to_msa
//     3. User Wallet: delete_key_from_msa (using the original delegator key from Step 1)
//     4. The Provider from Step 1 attempts to create a NEW MSA with the original delegator key from Step 1
// Transaction 1-4 all executed before Step 1's payload expireBlock
fn replaying_create_sponsored_account_with_delegation_fails_03() {
	new_test_ext().execute_with(|| {
		let (delegator_keypair, provider_keypair) = create_user_and_provider();
		let provider_key = provider_keypair.public();
		let delegator_key = delegator_keypair.public();

		// Step 1
		let (original_msa_creation_signature, add_provider_payload) =
			user_creates_and_delegates_to_provider(
				delegator_keypair.clone(),
				provider_keypair,
				99u32.into(),
			);

		run_to_block(5);

		let (new_key_pair, _) = sr25519::Pair::generate();
		let new_public_key = new_key_pair.public();

		let add_new_key_data =
			AddKeyData { msa_id: 2, expiration: 10, new_public_key: new_key_pair.public().into() };

		let encode_add_key_data = wrap_binary_data(add_new_key_data.encode());

		let msa_owner_signature = delegator_keypair.sign(&encode_add_key_data);
		let new_key_owner_signature = new_key_pair.sign(&encode_add_key_data);

		// Step 2.
		assert_ok!(Msa::add_public_key_to_msa(
			RuntimeOrigin::signed(delegator_key.into()),
			delegator_key.into(),
			msa_owner_signature.into(),
			new_key_owner_signature.into(),
			add_new_key_data.clone()
		));
		run_to_block(75);

		// Step 3
		assert_ok!(Msa::delete_msa_public_key(
			RuntimeOrigin::signed(new_public_key.into()),
			delegator_key.into()
		));
		run_to_block(98);

		// Step 4
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				RuntimeOrigin::signed(provider_key.into()),
				delegator_key.into(),
				original_msa_creation_signature.clone(),
				add_provider_payload.clone()
			),
			Error::<Test>::SignatureAlreadySubmitted
		);
	});
}
