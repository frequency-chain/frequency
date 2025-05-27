use frame_support::{
	assert_err, assert_ok,
	traits::{
		tokens::{fungible::Inspect, Fortitude, Preservation},
		Currency,
	},
};

use sp_core::{sr25519, Pair};

use crate::{tests::mock::*, Config, Error};

use common_primitives::{
	msa::H160,
	signatures::{AccountAddressMapper, EthereumAddressMapper},
};

use pallet_balances::Event as BalancesEvent;

#[test]
fn it_succeeds_when_balance_is_sufficient() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();
		let eth_account_id: H160 = Msa::msa_id_to_eth_address(msa_id);
		let bytes: [u8; 32] = EthereumAddressMapper::to_bytes32(&eth_account_id.0);
		let msa_account_id = <Test as frame_system::Config>::AccountId::from(bytes);

		let (payload, _, msa_signature) = generate_and_sign_authorized_key_payload(
			msa_id,
			&owner_key_pair,
			&origin_key_pair,
			None,
		);

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

#[test]
fn it_fails_when_duplicate_signature_submitted() {
	new_test_ext().execute_with(|| {
		let (msa_id, owner_key_pair) = create_account();
		let (origin_key_pair, _) = sr25519::Pair::generate();
		let eth_account_id: H160 = Msa::msa_id_to_eth_address(msa_id);
		let bytes: [u8; 32] = EthereumAddressMapper::to_bytes32(&eth_account_id.0);
		let msa_account_id = <Test as frame_system::Config>::AccountId::from(bytes);

		let (payload, _, msa_signature) = generate_and_sign_authorized_key_payload(
			msa_id,
			&owner_key_pair,
			&origin_key_pair,
			None,
		);

		let transfer_amount = 10_000_000;

		// Fund MSA
		let _ = <Test as Config>::Currency::deposit_creating(&msa_account_id, transfer_amount);

		assert_ok!(Msa::withdraw_tokens(
			RuntimeOrigin::signed(origin_key_pair.public().into()),
			owner_key_pair.public().into(),
			msa_signature.clone(),
			payload.clone()
		));

		assert_err!(
			Msa::withdraw_tokens(
				RuntimeOrigin::signed(origin_key_pair.public().into()),
				owner_key_pair.public().into(),
				msa_signature.clone(),
				payload.clone()
			),
			Error::<Test>::SignatureAlreadySubmitted
		);
	})
}
