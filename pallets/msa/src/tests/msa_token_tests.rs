use frame_support::{
	assert_noop, assert_ok,
	traits::{
		tokens::{fungible::Inspect, Fortitude, Preservation},
		Currency,
	},
};

use sp_core::{sr25519, Encode, Pair};
use sp_runtime::MultiSignature;

use crate::{tests::mock::*, types::AddKeyData, Config, Error};

use common_primitives::{
	msa::{MessageSourceId, H160},
	node::BlockNumber,
	signatures::{AccountAddressMapper, EthereumAddressMapper},
	utils::wrap_binary_data,
};

use pallet_balances::Event as BalancesEvent;

fn generate_payload(
	msa_id: MessageSourceId,
	msa_owner_keys: &sr25519::Pair,
	new_public_key: &sr25519::Pair,
	expiration: Option<BlockNumber>,
) -> (AddKeyData<Test>, Vec<u8>, MultiSignature) {
	let payload = AddKeyData::<Test> {
		msa_id,
		expiration: match expiration {
			Some(block_number) => block_number,
			None => 10,
		},
		new_public_key: new_public_key.public().into(),
	};

	let encoded_payload = wrap_binary_data(payload.encode());
	let signature: MultiSignature = msa_owner_keys.sign(&encoded_payload).into();

	(payload, encoded_payload, signature)
}

#[test]
fn it_fails_when_caller_key_does_not_match_payload() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();
		let (other_key_pair, _) = sr25519::Pair::generate();

		let (payload, _, msa_signature) =
			generate_payload(msa_id, &owner_key_pair, &other_key_pair, None);

		assert_noop!(
			Msa::withdraw_tokens(
				RuntimeOrigin::signed(origin_key_pair.public().into()),
				owner_key_pair.public().into(),
				msa_signature,
				payload
			),
			Error::<Test>::NotKeyOwner
		);
	});
}

#[test]
fn it_fails_when_payload_signature_is_invalid() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();
		let (other_key_pair, _) = sr25519::Pair::generate();

		let (payload, _, msa_signature) =
			generate_payload(msa_id, &other_key_pair, &origin_key_pair, None);

		assert_noop!(
			Msa::withdraw_tokens(
				RuntimeOrigin::signed(origin_key_pair.public().into()),
				owner_key_pair.public().into(),
				msa_signature,
				payload
			),
			Error::<Test>::MsaOwnershipInvalidSignature
		);
	});
}

#[test]
fn it_fails_when_proof_is_expired() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();

		// The current block is 1, therefore setting the proof expiration to 1 should cause
		// the extrinsic to fail because the proof has expired.
		let (payload, _, msa_signature) =
			generate_payload(msa_id, &owner_key_pair, &origin_key_pair, Some(1));

		assert_noop!(
			Msa::withdraw_tokens(
				RuntimeOrigin::signed(origin_key_pair.public().into()),
				owner_key_pair.public().into(),
				msa_signature,
				payload
			),
			Error::<Test>::ProofHasExpired
		);
	});
}

#[test]
fn it_fails_when_proof_is_not_yet_valid() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();

		// The current block is 1, therefore setting the proof expiration to the max mortality period
		// should cause the extrinsic to fail
		let (payload, _, msa_signature) = generate_payload(
			msa_id,
			&owner_key_pair,
			&origin_key_pair,
			Some(Msa::mortality_block_limit(1)),
		);

		assert_noop!(
			Msa::withdraw_tokens(
				RuntimeOrigin::signed(origin_key_pair.public().into()),
				owner_key_pair.public().into(),
				msa_signature,
				payload
			),
			Error::<Test>::ProofNotYetValid
		);
	});
}

#[test]
fn it_fails_when_msa_key_is_not_an_msa_control_key() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();

		let (payload, _, msa_signature) =
			generate_payload(msa_id + 1, &owner_key_pair, &origin_key_pair, None);

		assert_noop!(
			Msa::withdraw_tokens(
				RuntimeOrigin::signed(origin_key_pair.public().into()),
				owner_key_pair.public().into(),
				msa_signature,
				payload
			),
			Error::<Test>::NotMsaOwner
		);
	})
}

#[test]
fn it_fails_when_msa_key_does_not_control_msa_in_payload() {
	new_test_ext().execute_with(|| {
		let (msa_id, _) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();
		let (other_key_pair, _) = sr25519::Pair::generate();

		let (payload, _, msa_signature) =
			generate_payload(msa_id, &other_key_pair, &origin_key_pair, None);

		assert_noop!(
			Msa::withdraw_tokens(
				RuntimeOrigin::signed(origin_key_pair.public().into()),
				other_key_pair.public().into(),
				msa_signature,
				payload
			),
			Error::<Test>::NoKeyExists
		);
	})
}

#[test]
fn it_fails_when_msa_does_not_have_a_balance() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();

		let (payload, _, msa_signature) =
			generate_payload(msa_id, &owner_key_pair, &origin_key_pair, None);

		assert_noop!(
			Msa::withdraw_tokens(
				RuntimeOrigin::signed(origin_key_pair.public().into()),
				owner_key_pair.public().into(),
				msa_signature,
				payload
			),
			Error::<Test>::InsufficientBalanceToWithdraw
		);
	})
}

#[test]
fn it_succeeds_when_balance_is_sufficient() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();
		let eth_account_id: H160 = Msa::msa_id_to_eth_address(msa_id);
		let bytes: [u8; 32] = EthereumAddressMapper::to_bytes32(&eth_account_id.0);
		let msa_account_id = <Test as frame_system::Config>::AccountId::from(bytes);

		let (payload, _, msa_signature) =
			generate_payload(msa_id, &owner_key_pair, &origin_key_pair, None);

		let transfer_amount = 10_000_000;

		// Fund MSA
		let _ = <Test as Config>::Currency::deposit_creating(&msa_account_id, transfer_amount);

		assert_ok!(Msa::withdraw_tokens(
			RuntimeOrigin::signed(origin_key_pair.public().into()),
			owner_key_pair.public().into(),
			msa_signature,
			payload
		));

		let receiver_balance = <Test as Config>::Currency::reducible_balance(
			&origin_key_pair.public().into(),
			Preservation::Expendable,
			Fortitude::Polite,
		);
		assert_eq!(
			receiver_balance, transfer_amount,
			"transfer amount {} does not equal new balance {}",
			transfer_amount, receiver_balance
		);

		System::assert_last_event(RuntimeEvent::Balances(BalancesEvent::Transfer {
			from: msa_account_id,
			to: origin_key_pair.public().into(),
			amount: transfer_amount,
		}));
	})
}
