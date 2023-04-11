use super::*;
use crate::{self as pallet_frequency_tx_payment, mock::*, ChargeFrqTransactionPayment};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchErrorWithPostInfo, weights::Weight};
use frame_system::RawOrigin;
use pallet_capacity::{CapacityDetails, CurrentEpoch, Nontransferable};

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
			//   - extrinsic_weight(11) * WeightToFee(1)
			//   - TransactionByteFee(1)* len(10) = 80
			assert_eq!(Capacity::balance(1), 100 - 5 - 11 - 10);

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
			assert_eq!(Capacity::balance(1), 100 - 5 - 11 - 10);
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
			let key_without_msa = 20u64;
			let create_msa_call = Box::new(RuntimeCall::Msa(MsaCall::<Test>::create {}));

			assert_ok!(FrequencyTxPayment::pay_with_capacity(
				RuntimeOrigin::signed(key_without_msa),
				create_msa_call
			));
		});
}

#[test]
fn pay_with_capacity_errors_with_call_error() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let existing_key_with_msa = 1u64;
			let create_msa_call = Box::new(RuntimeCall::Msa(MsaCall::<Test>::create {}));

			assert_noop!(
				FrequencyTxPayment::pay_with_capacity(
					RuntimeOrigin::signed(existing_key_with_msa),
					create_msa_call
				),
				pallet_msa::Error::<Test>::KeyAlreadyRegistered
			);
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
fn charge_frq_transaction_payment_withdraw_fee_for_capacity_batch_tx_returns_tupple_with_fee_and_enum(
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
				&RuntimeCall::FrequencyTxPayment(Call::pay_with_capacity_batch_all {
					calls: vec![RuntimeCall::Balances(BalancesCall::transfer {
						dest: 2,
						value: 100,
					})],
				});

			let info = DispatchInfo { weight: Weight::from_ref_time(5), ..Default::default() };
			let len = 10;

			// fee = base_weight(5)
			//   + extrinsic_weight(11) * WeightToFee(1)
			//   + TransactionByteFee(1)* len(10) = 26
			assert_eq!(charge_tx_payment.withdraw_fee(&who, call, &info, len).unwrap().0, 26u64);
			assert_eq!(
				charge_tx_payment.withdraw_fee(&who, call, &info, len).unwrap().1.is_capacity(),
				true
			);
		});
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
			//   + extrinsic_weight(11) * WeightToFee(1)
			//   + TransactionByteFee(1)* len(10) = 20
			assert_eq!(charge_tx_payment.withdraw_fee(&who, call, &info, len).unwrap().0, 26u64);
			assert_eq!(
				charge_tx_payment.withdraw_fee(&who, call, &info, len).unwrap().1.is_capacity(),
				true
			);
		});
}

#[test]
fn charge_frq_transaction_payment_withdraw_fee_errors_for_capacity_tx_when_user_does_not_have_enough_funds(
) {
	let balance_factor = 1;

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
			let result = charge_tx_payment.withdraw_fee(&who, call, &info, len);
			assert_eq!(
				result.unwrap_err(),
				TransactionValidityError::Invalid(InvalidTransaction::Payment)
			);
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
	account_id: <Test as frame_system::Config>::AccountId,
	call: &<Test as Config>::RuntimeCall,
	expected_err: Option<TransactionValidityError>,
) {
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
			let account_id = 1u64;
			let allowed_call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });

			let forbidden_call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_all { dest: 2, keep_alive: false });

			assert_withdraw_fee_result(account_id, allowed_call, None);

			let expected_err = TransactionValidityError::Invalid(InvalidTransaction::Custom(
				ChargeFrqTransactionPaymentError::CallIsNotCapacityEligible as u8,
			));
			assert_withdraw_fee_result(account_id, forbidden_call, Some(expected_err));
		});
}

#[test]
fn withdraw_fee_returns_custom_error_when_the_account_key_does_not_have_the_required_deposit() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			// An account that has an MSA but has not bet the min balance for key deposit.
			let account_id = 10u64;
			let _ = mock::create_msa_account(account_id);

			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });

			let expected_err = TransactionValidityError::Invalid(InvalidTransaction::Custom(
				ChargeFrqTransactionPaymentError::BelowMinDeposit as u8,
			));

			assert_withdraw_fee_result(account_id, call, Some(expected_err));
		});
}

#[test]
fn withdraw_fee_returns_custom_error_when_the_account_key_is_not_associated_with_an_msa() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let account_id_not_associated_with_msa = 10u64;

			// This allows it not to fail on the requirement of an existential deposit.
			assert_ok!(Balances::set_balance(
				RawOrigin::Root.into(),
				account_id_not_associated_with_msa,
				1u32.into(),
				Zero::zero(),
			));

			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });

			let expected_err = TransactionValidityError::Invalid(InvalidTransaction::Custom(
				ChargeFrqTransactionPaymentError::InvalidMsaKey as u8,
			));

			assert_withdraw_fee_result(
				account_id_not_associated_with_msa,
				call,
				Some(expected_err),
			);
		});
}

#[test]
fn withdraw_fee_replenishes_capacity_account_on_new_epoch_before_deducting_fee() {
	let balance_factor = 10;

	// uses funded account with MSA Id
	let provider_msa_id = 2u64;
	let provider_account_id = 2u64;
	let current_epoch = 11u32;
	let total_capacity_issued = 30u64;
	let total_tokens_staked = 30u64;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			CurrentEpoch::<Test>::set(current_epoch);

			let capacity_details = CapacityDetails {
				remaining_capacity: 1,
				total_tokens_staked,
				total_capacity_issued,
				last_replenished_epoch: 10,
			};
			Capacity::set_capacity_for(provider_msa_id, capacity_details);

			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });

			assert_withdraw_fee_result(provider_msa_id, call, None);

			let actual_capacity = Capacity::get_capacity_for(provider_account_id).unwrap();

			assert_eq!(
				actual_capacity,
				CapacityDetails {
					remaining_capacity: total_capacity_issued.saturating_sub(26),
					total_tokens_staked,
					total_capacity_issued,
					last_replenished_epoch: current_epoch,
				}
			);
		});
}

#[test]
fn withdraw_fee_does_not_replenish_if_not_new_epoch() {
	let balance_factor = 10;

	// uses funded account with MSA Id
	let provider_msa_id = 2u64;
	let provider_account_id = 2u64;
	let total_capacity_issued = 30u64;
	let total_tokens_staked = 30u64;
	let last_replenished_epoch = 10u32;
	let current_epoch = last_replenished_epoch;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			CurrentEpoch::<Test>::set(current_epoch);

			let capacity_details = CapacityDetails {
				remaining_capacity: 27,
				total_tokens_staked,
				total_capacity_issued,
				last_replenished_epoch,
			};
			Capacity::set_capacity_for(provider_msa_id, capacity_details);

			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });

			assert_withdraw_fee_result(provider_msa_id, call, None);

			let actual_capacity = Capacity::get_capacity_for(provider_account_id).unwrap();

			// Capacity details should have only the fee taken out
			assert_eq!(
				actual_capacity,
				CapacityDetails {
					remaining_capacity: 1u64, // fee is 26
					total_tokens_staked,
					total_capacity_issued,
					last_replenished_epoch,
				}
			);
		});
}

#[test]
fn compute_capacity_fee_successful() {
	let balance_factor = 10;
	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });

			// fee = base_weight + extrinsic weight + len = 5 + 11 + 10 = 26
			let fee = FrequencyTxPayment::compute_capacity_fee(
				10u32,
				DispatchClass::Normal,
				<Test as Config>::CapacityCalls::get_stable_weight(call).unwrap(),
			);

			assert_eq!(fee, 26);
		});
}

#[test]
fn pay_with_capacity_batch_all_happy_path() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let origin = 1u64;

			let calls = vec![
				RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 10 }),
				RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 10 }),
			];

			let token_balance_before_call = Balances::free_balance(origin);

			assert_ok!(FrequencyTxPayment::pay_with_capacity_batch_all(
				RuntimeOrigin::signed(origin),
				calls
			));

			let token_balance_after_call = Balances::free_balance(origin);
			assert_eq!(token_balance_before_call - 20u64, token_balance_after_call);
		});
}

#[test]
fn pay_with_capacity_batch_all_errors_when_transaction_amount_exceeds_maximum() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let origin = 1u64;

			let token_balance_before_call = Balances::free_balance(origin);

			let too_many_calls = vec![
				RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 }),
				RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 }),
				RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 }),
			];
			assert_noop!(
				FrequencyTxPayment::pay_with_capacity_batch_all(
					RuntimeOrigin::signed(origin),
					too_many_calls
				),
				Error::<Test>::BatchedCallAmountExceedsMaximum
			);

			let token_balance_after_call = Balances::free_balance(origin);

			assert_eq!(token_balance_before_call, token_balance_after_call);
		});
}

#[test]
fn pay_with_capacity_batch_all_transactions_will_all_fail_if_one_fails() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_ref_time(5))
		.build()
		.execute_with(|| {
			let origin = 1u64;
			let successful_balance_transfer_call =
				RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100 });

			let balance_transfer_call_insufficient_funds =
				RuntimeCall::Balances(BalancesCall::transfer { dest: 2, value: 100000000 });

			let token_balance_before_call = Balances::free_balance(origin);

			let calls_to_batch =
				vec![successful_balance_transfer_call, balance_transfer_call_insufficient_funds];

			let result = FrequencyTxPayment::pay_with_capacity_batch_all(
				RuntimeOrigin::signed(origin),
				calls_to_batch,
			);

			assert!(match result {
				Err(DispatchErrorWithPostInfo { .. }) => {
					true
				},
				_ => {
					false
				},
			});

			let token_balance_after_call = Balances::free_balance(origin);

			assert_eq!(token_balance_before_call, token_balance_after_call);
		});
}
