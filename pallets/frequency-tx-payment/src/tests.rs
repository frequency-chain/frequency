use super::*;
use crate::{self as pallet_frequency_tx_payment, mock::*, ChargeFrqTransactionPayment};
use frame_support::{assert_noop, assert_ok, weights::Weight};
use pallet_capacity::Nontransferable;

use sp_runtime::transaction_validity::TransactionValidityError;

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
fn transaction_payment_with_capacity_and_no_overcharge_post_dispatch_refund_is_succesful() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let account_id = 1u64;
			let balances_call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::FrequencyTxPayment(Call::pay_with_capacity {
					call: Box::new(RuntimeCall::Balances(BalancesCall::transfer {
						dest: 2,
						value: 100,
					})),
				});

			let dispatch_info =
				DispatchInfo { weight: Weight::from_ref_time(5), ..Default::default() };
			let len = 10;

			assert_eq!(Capacity::balance(1), 100);

			let pre = ChargeFrqTransactionPayment::<Test>::from(0u64)
				.pre_dispatch(&account_id, balances_call, &dispatch_info, len)
				.unwrap();

			// Token account Balance is not effected
			assert_eq!(Balances::free_balance(1), 100);

			// capacity_balance = free_balance - base_weight(5)
			//   - extrinsic_weight(5) * WeightToFee(1)
			//   - TransactionByteFee(1)* len(10) = 80
			assert_eq!(Capacity::balance(1), 100 - 5 - 5 - 10);

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
			assert_eq!(Capacity::balance(1), 100 - 5 - 5 - 10);
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

	assert!(pay_with_capacity_dispatch_info
		.weight
		.ref_time()
		.gt(&create_msa_dispatch_info.weight.ref_time()));
}

#[test]
fn charge_frq_transaction_payment_withdraw_fee_for_capacity_tx_returns_tupple_with_fee_and_enum() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let charge_tx_payment = ChargeFrqTransactionPayment::<Test>::from(0u64);
			let who = 1u64;
			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::FrequencyTxPayment(Call::pay_with_capacity {
					call: Box::new(RuntimeCall::Balances(BalancesCall::transfer {
						dest: 2,
						value: 100,
					})),
				});

			let info = DispatchInfo { weight: Weight::from_ref_time(5), ..Default::default() };
			let len = 10;

			// fee = base_weight(5)
			//   + extrinsic_weight(5) * WeightToFee(1)
			//   + TransactionByteFee(1)* len(10) = 20
			assert_eq!(charge_tx_payment.withdraw_fee(&who, call, &info, len).unwrap().0, 20u64);
			assert_eq!(
				charge_tx_payment.withdraw_fee(&who, call, &info, len).unwrap().1.is_capacity(),
				true
			);
		});
}

#[test]
fn charge_frq_transaction_payment_withdraw_fee_errors_for_capacity_tx_when_user_does_not_have_enough_funds(
) {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(100))
		.build()
		.execute_with(|| {
			let charge_tx_payment = ChargeFrqTransactionPayment::<Test>::from(0u64);
			let who = 1u64;
			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::FrequencyTxPayment(Call::pay_with_capacity {
					call: Box::new(RuntimeCall::Balances(BalancesCall::transfer {
						dest: 2,
						value: 100,
					})),
				});

			let info = DispatchInfo { weight: Weight::from_ref_time(5), ..Default::default() };
			let len = 10;
			let error = charge_tx_payment.withdraw_fee(&who, call, &info, len).unwrap_err();
			assert_eq!(error, TransactionValidityError::Invalid(InvalidTransaction::Payment));
		});
}

#[test]
fn charge_frq_transaction_payment_withdraw_fee_errors_for_non_capacity_tx_when_user_does_not_have_enough_funds(
) {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(100))
		.build()
		.execute_with(|| {
			let charge_tx_payment = ChargeFrqTransactionPayment::<Test>::from(0u64);
			let who = 1u64;
			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });

			let info = DispatchInfo { weight: Weight::from_ref_time(5), ..Default::default() };
			let len = 10;
			let error = charge_tx_payment.withdraw_fee(&who, call, &info, len).unwrap_err();
			assert_eq!(error, TransactionValidityError::Invalid(InvalidTransaction::Payment));
		});
}

#[test]
fn charge_frq_transaction_payment_withdraw_fee_for_non_capacity_tx_returns_tupple_with_fee_and_initial_payment_token_enum(
) {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let charge_tx_payment = ChargeFrqTransactionPayment::<Test>::from(0u64);
			let who = 1u64;
			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });

			let info = DispatchInfo { weight: Weight::from_ref_time(5), ..Default::default() };
			let len = 10;
			let result = charge_tx_payment.withdraw_fee(&who, call, &info, len).unwrap();

			// fee = base_weight(5)
			//   + extrinsic_weight(5) * WeightToFee(1)
			//   + TransactionByteFee(1)* len(10) = 20
			assert_eq!(result.0, 20);
			let expected = match result.1 {
				InitialPayment::Token(_) => true,
				_ => false,
			};

			assert!(expected);
		});
}

#[test]
fn charge_frq_transaction_payment_withdraw_fee_for_free_non_capacity_tx_returns_tupple_with_fee_and_free_enum(
) {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let charge_tx_payment = ChargeFrqTransactionPayment::<Test>::from(0u64);
			let who = 1u64;
			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });

			let info = DispatchInfo {
				weight: Weight::from_ref_time(5),
				pays_fee: Pays::No,
				..Default::default()
			};
			let len = 10;
			let result = charge_tx_payment.withdraw_fee(&who, call, &info, len).unwrap();

			// fee = base_weight(5)
			//   + extrinsic_weight(5) * WeightToFee(1)
			//   + TransactionByteFee(1)* len(10) = 20
			assert_eq!(result.0, 0);
			let expected = match result.1 {
				InitialPayment::Free => true,
				_ => false,
			};

			assert!(expected);
		});
}

#[test]
fn charge_frq_transaction_payment_tip_is_zero_for_capacity_calls() {
	let fake_tip = 100;
	let charge_tx_payment = ChargeFrqTransactionPayment::<Test>::from(fake_tip);
	let call: &<Test as Config>::RuntimeCall =
		&RuntimeCall::FrequencyTxPayment(Call::pay_with_capacity {
			call: Box::new(RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 })),
		});

	let result = charge_tx_payment.tip(call);

	assert_eq!(result, 0u64);
}

#[test]
fn charge_frq_transaction_payment_tip_is_some_amount_for_non_capacity_calls() {
	let tip = 200;
	let charge_tx_payment = ChargeFrqTransactionPayment::<Test>::from(tip);
	let call: &<Test as Config>::RuntimeCall =
		&RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });

	let result = charge_tx_payment.tip(call);

	assert_eq!(result, 200u64);
}

pub fn assert_withdraw_fee_result(
	call: &<Test as Config>::RuntimeCall,
	expected_err: Option<TransactionValidityError>,
) {
	let account_id = 1u64;
	let dispatch_info = DispatchInfo { weight: Weight::from_ref_time(5), ..Default::default() };

	let call: &<Test as Config>::RuntimeCall =
		&RuntimeCall::FrequencyTxPayment(Call::pay_with_capacity { call: Box::new(call.clone()) });

	let withdraw_fee = ChargeFrqTransactionPayment::<Test>::from(0u64).withdraw_fee(
		&account_id,
		call,
		&dispatch_info,
		10,
	);

	match expected_err {
		None => assert!(withdraw_fee.is_ok()),
		Some(err) => {
			assert!(withdraw_fee.is_err());
			assert_eq!(err, withdraw_fee.err().unwrap())
		},
	}
}

#[test]
fn withdraw_fee_allows_only_configured_capacity_calls() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let allowed_call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });

			let forbidden_call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_all { dest: 2, keep_alive: false });

			assert_withdraw_fee_result(allowed_call, None);

			let expected_err = TransactionValidityError::Invalid(InvalidTransaction::Custom(
				ChargeFrqTransactionPaymentError::CallIsNotCapacityEligible as u8,
			));
			assert_withdraw_fee_result(forbidden_call, Some(expected_err));
		});
}
