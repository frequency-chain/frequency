use frame_support::{
	assert_noop, assert_ok,
	dispatch::{GetDispatchInfo, Pays},
};

use sp_core::{crypto::AccountId32, sr25519, Encode, Pair};
use sp_runtime::{ArithmeticError, MultiSignature};

use crate::{
	tests::mock::*,
	types::{AddKeyData, EMPTY_FUNCTION},
	Config, Error, Event, PayloadSignatureRegistryList, PayloadSignatureRegistryPointer,
	PublicKeyCountForMsaId, PublicKeyToMsaId,
};

use common_primitives::{
	msa::{MessageSourceId, SignatureRegistryPointer},
	utils::wrap_binary_data,
};

#[test]
fn it_throws_error_when_new_key_verification_fails() {
	new_test_ext().execute_with(|| {
		let (new_msa_id, owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();
		let (fake_key_pair, _) = sr25519::Pair::generate();

		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 10,
			new_public_key: new_key_pair.public().into(),
		};

		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let fake_new_key_signature: MultiSignature =
			fake_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				fake_new_key_signature,
				add_new_key_data
			),
			Error::<Test>::NewKeyOwnershipInvalidSignature
		);
	});
}

#[test]
fn it_throws_error_when_msa_ownership_verification_fails() {
	new_test_ext().execute_with(|| {
		let (new_msa_id, owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();
		let (fake_owner_pair, _) = sr25519::Pair::generate();

		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 10,
			new_public_key: new_key_pair.public().into(),
		};

		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let fake_owner_signature: MultiSignature =
			fake_owner_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				fake_owner_signature,
				new_key_signature,
				add_new_key_data
			),
			Error::<Test>::MsaOwnershipInvalidSignature
		);
	});
}

#[test]
fn it_throws_error_when_not_msa_owner() {
	new_test_ext().execute_with(|| {
		let (new_msa_id, _) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();
		let (_fake_msa_id, fake_key_pair) = create_account();

		assert_ok!(Msa::create_account(test_public(1), EMPTY_FUNCTION));

		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 10,
			new_public_key: new_key_pair.public().into(),
		};

		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let fake_owner_signature: MultiSignature =
			fake_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				fake_key_pair.public().into(),
				fake_owner_signature,
				new_key_signature,
				add_new_key_data
			),
			Error::<Test>::NotMsaOwner
		);
	});
}

#[test]
fn it_throws_error_when_for_duplicate_key() {
	new_test_ext().execute_with(|| {
		let (new_msa_id, owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();

		let _ = Msa::add_key(new_msa_id, &new_key_pair.public().into(), EMPTY_FUNCTION);

		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 10,
			new_public_key: new_key_pair.public().into(),
		};

		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				new_key_signature,
				add_new_key_data
			),
			Error::<Test>::KeyAlreadyRegistered
		);
	});
}

#[test]
fn add_key_with_more_than_allowed_should_panic() {
	new_test_ext().execute_with(|| {
		// arrange
		let (new_msa_id, owner_key_pair) = create_account();

		for _ in 1..<Test as Config>::MaxPublicKeysPerMsa::get() {
			let (new_key_pair, _) = sr25519::Pair::generate();

			let add_new_key_data = AddKeyData {
				msa_id: new_msa_id,
				expiration: 10,
				new_public_key: new_key_pair.public().into(),
			};
			let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

			let owner_signature: MultiSignature =
				owner_key_pair.sign(&encode_data_new_key_data).into();

			let public_key_ownership_signature =
				new_key_pair.sign(&encode_data_new_key_data).into();

			assert_ok!(Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				public_key_ownership_signature,
				add_new_key_data
			));
		}

		// act
		let (final_key_pair, _) = sr25519::Pair::generate();

		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 10,
			new_public_key: final_key_pair.public().into(),
		};
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature =
			final_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				new_key_signature,
				add_new_key_data
			),
			ArithmeticError::Overflow
		);
	});
}

#[test]
fn add_key_with_valid_request_should_store_value_and_event() {
	new_test_ext().execute_with(|| {
		// arrange
		let (new_msa_id, owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();

		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 10,
			new_public_key: new_key_pair.public().into(),
		};
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		// act
		assert_ok!(Msa::add_public_key_to_msa(
			test_origin_signed(1),
			owner_key_pair.public().into(),
			owner_signature,
			new_key_signature,
			add_new_key_data
		));

		// assert
		let keys_count = PublicKeyCountForMsaId::<Test>::get(new_msa_id);
		assert_eq!(keys_count, 2);
		System::assert_last_event(
			Event::PublicKeyAdded { msa_id: 1, key: new_key_pair.public().into() }.into(),
		);
	});
}

/// Assert that when attempting to add a key to an MSA with an expired proof that the key is NOT added.
/// Expected error: ProofHasExpired
#[test]
fn add_key_with_expired_proof_fails() {
	new_test_ext().execute_with(|| {
		// arrange
		let (new_msa_id, owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();

		// The current block is 1, therefore setting the proof expiration to 1 should cause
		// the extrinsic to fail because the proof has expired.
		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 1,
			new_public_key: new_key_pair.public().into(),
		};

		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				new_key_signature,
				add_new_key_data
			),
			Error::<Test>::ProofHasExpired
		);
	})
}

/// Assert that when attempting to add a key to an MSA with a proof expiration too far into the future the key is NOT added.
/// Expected error: ProofNotYetValid
#[test]
fn add_key_with_proof_too_far_into_future_fails() {
	new_test_ext().execute_with(|| {
		// arrange
		let (new_msa_id, owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();

		// The current block is 1, therefore setting the proof expiration to EXPIRATION_BLOCK_VALIDITY_GAP + 1
		// should cause the extrinsic to fail because the proof is only valid for EXPIRATION_BLOCK_VALIDITY_GAP
		// more blocks.
		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 202,
			new_public_key: new_key_pair.public().into(),
		};

		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				new_key_signature,
				add_new_key_data
			),
			Error::<Test>::ProofNotYetValid
		);
	})
}

/// Assert that when a key has been added to an MSA, that it my NOT be added to any other MSA.
/// Expected error: KeyAlreadyRegistered
#[test]
fn double_add_key_two_msa_fails() {
	new_test_ext().execute_with(|| {
		let (msa_id1, owner_key_pair) = create_account();
		let (_msa_id2, msa_2_owner_key_pair) = create_account();

		let add_new_key_data = AddKeyData {
			msa_id: msa_id1,
			expiration: 10,
			new_public_key: msa_2_owner_key_pair.public().into(),
		};
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature =
			msa_2_owner_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				new_key_signature,
				add_new_key_data
			),
			Error::<Test>::KeyAlreadyRegistered
		);
	})
}

#[test]
fn add_public_key_to_msa_registers_two_signatures() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let (msa_id1, owner_key_pair) = create_account();
		let (_msa_id2, _msa_2_owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();

		let add_new_key_data = AddKeyData {
			msa_id: msa_id1,
			expiration: 10,
			new_public_key: new_key_pair.public().into(),
		};
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_ok!(Msa::add_public_key_to_msa(
			test_origin_signed(1),
			owner_key_pair.public().into(),
			owner_signature.clone(),
			new_key_signature.clone(),
			add_new_key_data
		));

		assert_eq!(
			PayloadSignatureRegistryList::<Test>::get(owner_signature.clone()).unwrap().0,
			10
		);
		assert_eq!(
			PayloadSignatureRegistryPointer::<Test>::get().unwrap(),
			SignatureRegistryPointer {
				newest: new_key_signature,
				newest_expires_at: 10u32,
				oldest: owner_signature,
				count: 2,
			}
		);
	});
}

/// Assert that when a key has been deleted from one MSA, that it may be added to a different MSA.
#[test]
fn add_removed_key_to_msa_pass() {
	new_test_ext().execute_with(|| {
		let (msa_getting_a_second_key, owner_key_pair) = create_account();
		let (msa_used_to_have_a_key, prior_msa_key) = create_account();

		assert_ok!(Msa::delete_key_for_msa(msa_used_to_have_a_key, &prior_msa_key.public().into()));

		let add_new_key_data = AddKeyData {
			msa_id: msa_getting_a_second_key,
			expiration: 10,
			new_public_key: prior_msa_key.public().into(),
		};
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());
		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature =
			prior_msa_key.sign(&encode_data_new_key_data).into();

		assert_ok!(Msa::add_public_key_to_msa(
			test_origin_signed(1),
			owner_key_pair.public().into(),
			owner_signature,
			new_key_signature,
			add_new_key_data
		));
	});
}

#[test]
fn it_deletes_msa_key_successfully() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::add_key(2, &test_public(1), EMPTY_FUNCTION));
		assert_ok!(Msa::add_key(2, &test_public(2), EMPTY_FUNCTION));

		assert_ok!(Msa::delete_msa_public_key(test_origin_signed(1), test_public(2)));

		let info = PublicKeyToMsaId::<Test>::get(test_public(2));

		assert_eq!(info, None);

		System::assert_last_event(Event::PublicKeyDeleted { key: test_public(2) }.into());
	})
}

#[test]
pub fn delete_msa_public_key_call_has_correct_costs() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let new_key = key_pair.public();

		let call = MsaCall::<Test>::delete_msa_public_key {
			public_key_to_delete: AccountId32::from(new_key),
		};
		let dispatch_info = call.get_dispatch_info();
		assert_eq!(dispatch_info.pays_fee, Pays::No);
	})
}

#[test]
pub fn test_delete_key() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::add_key(1, &test_public(1), EMPTY_FUNCTION));

		let info = PublicKeyToMsaId::<Test>::get(test_public(1));

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
