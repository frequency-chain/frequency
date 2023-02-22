use super::*;
use crate::{self as pallet_frequency_tx_payment, mock::*, ChargeFrqTransactionPayment};
use frame_support::{assert_noop, assert_ok, weights::Weight};

use pallet_balances::Call as BalancesCall;
use pallet_frequency_tx_payment::Call as FrequencyTxPaymentCall;
use pallet_msa::Call as MsaCall;

#[test]
fn transaction_payment_validate_is_succesful() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let account_id = 1u64;
			let balances_call: &<Test as frame_system::Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });
			let dispatch_info =
				DispatchInfo { weight: Weight::from_ref_time(5), ..Default::default() };
			let len = 10;

			assert_ok!(ChargeFrqTransactionPayment::<Test>::from(0u64).validate(
				&account_id,
				balances_call,
				&dispatch_info,
				len,
			));
		});
}

#[test]
fn transaction_payment_validate_errors_when_balance_is_cannot_pay_for_fee() {
	let balance_factor = 1;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let account_id = 1u64;
			let balances_call: &<Test as frame_system::Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });
			let dispatch_info =
				DispatchInfo { weight: Weight::from_ref_time(5), ..Default::default() };
			let len = 10;

			assert_noop!(
				ChargeFrqTransactionPayment::<Test>::from(0u64).validate(
					&account_id,
					balances_call,
					&dispatch_info,
					len,
				),
				TransactionValidityError::Invalid(InvalidTransaction::Payment)
			);
		});
}

#[test]
fn transaction_payment_with_token_and_no_overcharge_post_dispatch_refund_is_succesful() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let account_id = 1u64;
			let balances_call: &<Test as frame_system::Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });
			let dispatch_info =
				DispatchInfo { weight: Weight::from_ref_time(5), ..Default::default() };
			let len = 10;

			assert_eq!(Balances::free_balance(1), 100);

			let pre = ChargeFrqTransactionPayment::<Test>::from(0u64)
				.pre_dispatch(&account_id, balances_call, &dispatch_info, len)
				.unwrap();

			// account_balance = free_balance - base_weight(5)
			//   - extrinsic_weight(5) * WeightToFee(1)
			//   - TransactionByteFee(1)* len(10) = 80
			assert_eq!(Balances::free_balance(1), 100 - 5 - 5 - 10);

			let post_info: PostDispatchInfo =
				PostDispatchInfo { actual_weight: None, pays_fee: Default::default() };

			assert_ok!(ChargeFrqTransactionPayment::<Test>::post_dispatch(
				Some(pre),
				&dispatch_info,
				&post_info,
				len,
				&Ok(()),
			));

			// Checking balance was not modified after post-dispatch.
			assert_eq!(Balances::free_balance(1), 100 - 5 - 5 - 10);
		});
}

#[test]
fn transaction_payment_with_token_and_post_dispatch_refund_is_succesful() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let account_id = 1u64;
			let balances_call: &<Test as frame_system::Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });
			let dispatch_info =
				DispatchInfo { weight: Weight::from_ref_time(5), ..Default::default() };
			let len = 10;

			assert_eq!(Balances::free_balance(1), 100);

			let pre = ChargeFrqTransactionPayment::<Test>::from(0u64)
				.pre_dispatch(&account_id, balances_call, &dispatch_info, len)
				.unwrap();

			// account_balance = free_balance - base_weight(5)
			//   - extrinsic_weight(5) * WeightToFee(1)
			//   - TransactionByteFee(1)* len(10) = 80
			assert_eq!(Balances::free_balance(1), 100 - 5 - 5 - 10);

			let post_info: PostDispatchInfo = PostDispatchInfo {
				actual_weight: Some(Weight::from_ref_time(2)),
				pays_fee: Default::default(),
			};

			assert_ok!(ChargeFrqTransactionPayment::<Test>::post_dispatch(
				Some(pre),
				&dispatch_info,
				&post_info,
				len,
				&Ok(()),
			));

			// account_balance = free_balance - base_weight(5)
			//   - extrinsic_weight(5) * WeightToFee(1)
			//   - TransactionByteFee(1)* len(10)
			//   + difference_of_actual_weight(5 - 2) = 83
			assert_eq!(Balances::free_balance(1), 100 - 5 - 5 - 10 + 3);
		});
}

#[test]
fn pay_with_capacity_happy_path() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let signer = 1u64;
			let create_msa_call = Box::new(RuntimeCall::Msa(MsaCall::<Test>::create {}));

			assert_ok!(FrequencyTxPayment::pay_with_capacity(
				RuntimeOrigin::signed(signer),
				create_msa_call
			));
		});
}

#[test]
fn pay_with_capacity_returns_weight_of_child_call() {
	let create_msa_call = Box::new(RuntimeCall::Msa(MsaCall::<Test>::create {}));
	let create_msa_dispatch_info = create_msa_call.get_dispatch_info();

	let pay_with_capacity_call = Box::new(RuntimeCall::FrequencyTxPayment(
		FrequencyTxPaymentCall::<Test>::pay_with_capacity { call: create_msa_call },
	));
	let pay_with_capacity_dispatch_info = pay_with_capacity_call.get_dispatch_info();

	assert_eq!(create_msa_dispatch_info.weight, pay_with_capacity_dispatch_info.weight);
}
