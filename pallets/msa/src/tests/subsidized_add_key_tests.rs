use crate::{
	tests::mock::{create_account, new_test_ext, test_origin_signed, Msa},
	AddKeyData,
};
use common_primitives::{msa::MsaKeyProvider, utils::wrap_binary_data};
use frame_support::assert_ok;
use sp_core::{bytes::from_hex, crypto::AccountId32, sr25519, Encode, Pair};
use sp_runtime::MultiSignature;

#[test]
fn key_not_eligible_for_subsidized_addition_when_more_than_one_key() {
	new_test_ext().execute_with(|| {
		let (msa_id, key_pair) = create_account();
		let public_key = key_pair.public();
		let account_id32 = AccountId32::from(public_key);

		let (new_key_pair, _) = sr25519::Pair::generate();
		let new_public_key: AccountId32 = new_key_pair.public().into();

		let add_new_key_data =
			AddKeyData { msa_id, expiration: 10, new_public_key: new_public_key.clone() };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());
		let owner_signature: MultiSignature = key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_ok!(Msa::add_public_key_to_msa(
			test_origin_signed(1),
			key_pair.public().into(),
			owner_signature,
			new_key_signature,
			add_new_key_data
		));

		let valid_eth_address =
			from_hex("0x9999999999999999999999999999999999999999eeeeeeeeeeeeeeeeeeeeeeee")
				.expect("should be hex");
		let ethereum_key = AccountId32::new(valid_eth_address.clone().try_into().unwrap());

		pretty_assertions::assert_eq!(
			Msa::key_eligible_for_subsidized_addition(account_id32.into(), ethereum_key, msa_id),
			false
		);
	});
}

#[test]
fn key_eligible_for_subsidized_addition_is_false_when() {
	new_test_ext().execute_with(|| {
		// Set up valid msa_id and control keys
		let (msa_id, valid_keypair) = create_account();
		let msa_control_key = AccountId32::from(valid_keypair.public());

		// set up msa_id not associated with the control key
		let invalid_msa_id = msa_id + 1;

		// control key not associated with the msa id
		let (key_pair1, _) = sr25519::Pair::generate();
		let not_msa_control_key = AccountId32::from(key_pair1.public());

		// a new key that isn't an ethereum compatible key
		let (key_pair2, _) = sr25519::Pair::generate();
		let non_ethereum_key = AccountId32::from(key_pair2.public());

		// a new key that is an ethereum compatible key
		let valid_eth_address =
			from_hex("0x917B536617B0A42B2ABE85AC88788825F29F0B29eeeeeeeeeeeeeeeeeeeeeeee")
				.expect("should be hex");
		let ethereum_key = AccountId32::new(valid_eth_address.clone().try_into().unwrap());

		// can't get a free transaction when msa_id exists but this is not the correct owner key
		pretty_assertions::assert_eq!(
			Msa::key_eligible_for_subsidized_addition(
				not_msa_control_key.clone(),
				ethereum_key.clone(),
				msa_id
			),
			false
		);
		// can't get a free transaction if the new key isn't an ethereum-compatible key
		pretty_assertions::assert_eq!(
			Msa::key_eligible_for_subsidized_addition(
				msa_control_key.clone(),
				non_ethereum_key.clone(),
				msa_id
			),
			false
		);
		// can't get a free transaction if the owner key exists but msa id provided is wrong
		pretty_assertions::assert_eq!(
			Msa::key_eligible_for_subsidized_addition(
				msa_control_key.clone(),
				ethereum_key.clone(),
				invalid_msa_id
			),
			false
		);
	});
}

#[test]
fn key_eligible_for_subsidized_addition_when_only_one_key_and_ethereum_compatible() {
	new_test_ext().execute_with(|| {
		let (msa_id, key_pair) = create_account();
		let account_id = key_pair.public();
		let valid_eth_address =
			from_hex("0x1111111111111111111111111111111111111111eeeeeeeeeeeeeeeeeeeeeeee")
				.expect("should be hex");
		let ethereum_key = AccountId32::new(valid_eth_address.clone().try_into().unwrap());
		assert!(Msa::key_eligible_for_subsidized_addition(account_id.into(), ethereum_key, msa_id));
	});
}
