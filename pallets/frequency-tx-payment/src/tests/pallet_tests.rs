use crate::{
	self as pallet_frequency_tx_payment, tests::mock::*, ChargeFrqTransactionPayment, DispatchInfo,
	*,
};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchErrorWithPostInfo, weights::Weight};
use frame_system::RawOrigin;
use pallet_capacity::{CapacityDetails, CurrentEpoch, Nontransferable};

use sp_runtime::{testing::TestXt, transaction_validity::TransactionValidityError, MultiSignature};

use pallet_balances::Call as BalancesCall;
use pallet_capacity::CapacityLedger;
use pallet_frequency_tx_payment::Call as FrequencyTxPaymentCall;
use pallet_msa::{AddKeyData, Call as MsaCall};
use sp_core::{sr25519, Pair, H256};

#[test]
#[allow(deprecated)]
fn transaction_payment_validate_is_succesful() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let account_id = 1u64;
			let balances_call: &<Test as frame_system::Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });
			let dispatch_info =
				DispatchInfo { call_weight: Weight::from_parts(5, 0), ..Default::default() };
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
#[allow(deprecated)]
fn transaction_payment_validate_errors_when_balance_is_cannot_pay_for_fee() {
	let balance_factor = 1;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let account_id = 1u64;
			let balances_call: &<Test as frame_system::Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });
			let dispatch_info =
				DispatchInfo { call_weight: Weight::from_parts(5, 0), ..Default::default() };
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
#[allow(deprecated)]
fn transaction_payment_with_token_and_no_overcharge_post_dispatch_refund_is_succesful() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let account_id = 1u64;
			let balances_call: &<Test as frame_system::Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });
			let dispatch_info =
				DispatchInfo { call_weight: Weight::from_parts(5, 0), ..Default::default() };
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
#[allow(deprecated)]
fn transaction_payment_with_token_and_post_dispatch_refund_is_succesful() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let account_id = 1u64;
			let balances_call: &<Test as frame_system::Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });
			let dispatch_info =
				DispatchInfo { call_weight: Weight::from_parts(5, 0), ..Default::default() };
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
				actual_weight: Some(Weight::from_parts(2, 0)),
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
#[allow(deprecated)]
fn transaction_payment_with_capacity_and_no_overcharge_post_dispatch_refund_is_succesful() {
	let balance_factor = 100_000_000;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let account_id = 1u64;
			let balances_call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::FrequencyTxPayment(Call::pay_with_capacity {
					call: Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
						dest: 2,
						value: 100,
					})),
				});

			let dispatch_info =
				DispatchInfo { call_weight: Weight::from_parts(5, 0), ..Default::default() };
			let len = 10;

			assert_eq!(Capacity::balance(1), 1_000_000_000);

			let pre = ChargeFrqTransactionPayment::<Test>::from(0u64)
				.pre_dispatch(&account_id, balances_call, &dispatch_info, len)
				.unwrap();

			// Token account Balance is not effected
			assert_eq!(Balances::free_balance(1), 1_000_000_000);

			// capacity_balance = free_balance - base_weight(CAPACITY_EXTRINSIC_BASE_WEIGHT)
			//   - extrinsic_weight(11) * WeightToFee(1)
			//   - TransactionByteFee(1)* len(10) = 80
			assert_eq!(Capacity::balance(1), 1_000_000_000 - 105_455_000 - 11 - 10);

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
			assert_eq!(Capacity::balance(1), 1_000_000_000 - 105_455_000 - 11 - 10);
		});
}

#[test]
#[allow(deprecated)]
fn pay_with_capacity_happy_path() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
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
#[allow(deprecated)]
fn pay_with_capacity_errors_with_call_error() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
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
#[allow(deprecated)]
fn pay_with_capacity_returns_weight_of_child_call() {
	let create_msa_call = Box::new(RuntimeCall::Msa(MsaCall::<Test>::create {}));
	let create_msa_dispatch_info = create_msa_call.get_dispatch_info();

	let pay_with_capacity_call = Box::new(RuntimeCall::FrequencyTxPayment(
		FrequencyTxPaymentCall::<Test>::pay_with_capacity { call: create_msa_call },
	));
	let pay_with_capacity_dispatch_info = pay_with_capacity_call.get_dispatch_info();

	assert!(pay_with_capacity_dispatch_info
		.call_weight
		.ref_time()
		.gt(&create_msa_dispatch_info.call_weight.ref_time()));
}

#[test]
#[allow(deprecated)]
fn charge_frq_transaction_payment_withdraw_fee_for_capacity_batch_tx_returns_tuple_with_fee_and_enum(
) {
	let balance_factor = 100_000_000;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let charge_tx_payment = ChargeFrqTransactionPayment::<Test>::from(0u64);
			let who = 1u64;
			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::FrequencyTxPayment(Call::pay_with_capacity_batch_all {
					calls: vec![RuntimeCall::Balances(BalancesCall::transfer_allow_death {
						dest: 2,
						value: 100,
					})],
				});

			let info = DispatchInfo { call_weight: Weight::from_parts(5, 0), ..Default::default() };
			let len = 10;

			// fee = base_weight(CAPACITY_EXTRINSIC_BASE_WEIGHT)
			//   + extrinsic_weight(11) * WeightToFee(1)
			//   + TransactionByteFee(1)* len(10) = CAPACITY_EXTRINSIC_BASE_WEIGHT + 21
			let res = charge_tx_payment.withdraw_fee(&who, call, &info, len);
			assert_ok!(&res);
			assert_eq!(res.unwrap().0, 105_455_000 + 21);
			assert!(charge_tx_payment
				.withdraw_fee(&who, call, &info, len)
				.unwrap()
				.1
				.is_capacity());
		});
}

#[test]
#[allow(deprecated)]
fn charge_frq_transaction_payment_withdraw_fee_for_capacity_tx_returns_tupple_with_fee_and_enum() {
	let balance_factor = 100_000_000;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let charge_tx_payment = ChargeFrqTransactionPayment::<Test>::from(0u64);
			let who = 1u64;
			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::FrequencyTxPayment(Call::pay_with_capacity {
					call: Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
						dest: 2,
						value: 100,
					})),
				});

			let info = DispatchInfo { call_weight: Weight::from_parts(5, 0), ..Default::default() };
			let len = 10;

			// fee = base_weight(CAPACITY_EXTRINSIC_BASE_WEIGHT)
			//   + extrinsic_weight(11) * WeightToFee(1)
			//   + TransactionByteFee(1)* len(10) = 20
			assert_eq!(
				charge_tx_payment.withdraw_fee(&who, call, &info, len).unwrap().0,
				(105_455_000 + 21u64)
			);
			assert!(charge_tx_payment
				.withdraw_fee(&who, call, &info, len)
				.unwrap()
				.1
				.is_capacity());
		});
}

#[test]
fn charge_frq_transaction_payment_withdraw_fee_errors_for_capacity_tx_when_user_does_not_have_enough_funds(
) {
	let balance_factor = 1;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(100, 0))
		.build()
		.execute_with(|| {
			let charge_tx_payment = ChargeFrqTransactionPayment::<Test>::from(0u64);
			let who = 1u64;
			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::FrequencyTxPayment(Call::pay_with_capacity {
					call: Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
						dest: 2,
						value: 100,
					})),
				});

			let info = DispatchInfo { call_weight: Weight::from_parts(5, 0), ..Default::default() };
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
		.base_weight(Weight::from_parts(100, 0))
		.build()
		.execute_with(|| {
			let charge_tx_payment = ChargeFrqTransactionPayment::<Test>::from(0u64);
			let who = 1u64;
			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });

			let info = DispatchInfo { call_weight: Weight::from_parts(5, 0), ..Default::default() };
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
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let charge_tx_payment = ChargeFrqTransactionPayment::<Test>::from(0u64);
			let who = 1u64;
			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });

			let info = DispatchInfo { call_weight: Weight::from_parts(5, 0), ..Default::default() };
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
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let charge_tx_payment = ChargeFrqTransactionPayment::<Test>::from(0u64);
			let who = 1u64;
			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });

			let info = DispatchInfo {
				call_weight: Weight::from_parts(5, 0),
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
			call: Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: 2,
				value: 100,
			})),
		});

	let result = charge_tx_payment.tip(call);

	assert_eq!(result, 0u64);
}

#[test]
fn charge_frq_transaction_payment_tip_is_some_amount_for_non_capacity_calls() {
	let tip = 200;
	let charge_tx_payment = ChargeFrqTransactionPayment::<Test>::from(tip);
	let call: &<Test as Config>::RuntimeCall =
		&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });

	let result = charge_tx_payment.tip(call);

	assert_eq!(result, 200u64);
}

/// Test Helper Function
/// Asserts that the `withdraw_fee` function returns the expected result.
pub fn assert_withdraw_fee_result(
	account_id: <Test as frame_system::Config>::AccountId,
	call: &<Test as Config>::RuntimeCall,
	expected_err: Option<TransactionValidityError>,
) {
	let dispatch_info =
		DispatchInfo { call_weight: Weight::from_parts(5, 0), ..Default::default() };

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
	let balance_factor = 100_000_000;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let account_id = 1u64;
			let allowed_call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });

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
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			// An account that has an MSA but has not bet the min balance for key deposit.
			let account_id = 10u64;
			let _ = tests::mock::create_msa_account(account_id);

			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });

			let expected_err = TransactionValidityError::Invalid(InvalidTransaction::Payment);
			assert_withdraw_fee_result(account_id, call, Some(expected_err));
		});
}

#[test]
fn withdraw_fee_returns_custom_error_when_the_account_key_is_not_associated_with_an_msa() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let account_id_not_associated_with_msa = 10u64;

			// This allows it not to fail on the requirement of an existential deposit.
			assert_ok!(Balances::force_set_balance(
				RawOrigin::Root.into(),
				account_id_not_associated_with_msa,
				1u32.into(),
			));

			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });

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
	let balance_factor = 100_000_000;

	// uses funded account with MSA Id
	let provider_msa_id = 2u64;
	let provider_account_id = 2u64;
	let current_epoch = 11u32;
	let total_capacity_issued = 3_000_000_000u64;
	let total_tokens_staked = 3_000_000_000u64;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			CurrentEpoch::<Test>::set(current_epoch);

			let capacity_details = CapacityDetails {
				remaining_capacity: 1_000_000_000,
				total_tokens_staked,
				total_capacity_issued,
				last_replenished_epoch: 10,
			};
			Capacity::set_capacity_for(provider_msa_id, capacity_details);

			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });

			assert_withdraw_fee_result(provider_msa_id, call, None);

			let actual_capacity = CapacityLedger::<Test>::get(provider_account_id).unwrap();

			assert_eq!(
				actual_capacity,
				CapacityDetails {
					remaining_capacity: total_capacity_issued.saturating_sub(105_455_000 + 21),
					total_tokens_staked,
					total_capacity_issued,
					last_replenished_epoch: current_epoch,
				}
			);
		});
}

#[test]
fn withdraw_fee_does_not_replenish_if_not_new_epoch() {
	let balance_factor = 100_000_000;

	// uses funded account with MSA Id
	let provider_msa_id = 2u64;
	let provider_account_id = 2u64;
	let total_capacity_issued = 3_000_000_000u64;
	let total_tokens_staked = 3_000_000_000u64;
	let last_replenished_epoch = 10u32;
	let current_epoch = last_replenished_epoch;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			CurrentEpoch::<Test>::set(current_epoch);

			let capacity_details = CapacityDetails {
				remaining_capacity: 2_700_000_000,
				total_tokens_staked,
				total_capacity_issued,
				last_replenished_epoch,
			};
			Capacity::set_capacity_for(provider_msa_id, capacity_details);

			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });

			assert_withdraw_fee_result(provider_msa_id, call, None);

			let actual_capacity = CapacityLedger::<Test>::get(provider_account_id).unwrap();

			// Capacity details should have only the fee taken out
			assert_eq!(
				actual_capacity,
				CapacityDetails {
					remaining_capacity: 2_700_000_000.saturating_sub(105_455_000 + 21),
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
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });

			// fee = base_weight + extrinsic weight + len = CAPACITY_EXTRINSIC_BASE_WEIGHT + 11 + 10 = CAPACITY_EXTRINSIC_BASE_WEIGHT + 21
			let fee = FrequencyTxPayment::compute_capacity_fee(
				10u32,
				<Test as Config>::CapacityCalls::get_stable_weight(call).unwrap(),
			);

			assert_eq!(fee, 105_455_000 + 21);
		});
}

#[test]
fn pay_with_capacity_batch_all_happy_path() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let origin = 1u64;

			let calls = vec![
				RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 10 }),
				RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 10 }),
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
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let origin = 1u64;

			let token_balance_before_call = Balances::free_balance(origin);

			let too_many_calls = vec![
				RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 }),
				RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 }),
				RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 }),
				RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 }),
				RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 }),
				RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 }),
				RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 }),
				RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 }),
				RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 }),
				RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 }),
				RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 }),
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
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let origin = 1u64;
			let successful_balance_transfer_call =
				RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });

			let balance_transfer_call_insufficient_funds =
				RuntimeCall::Balances(BalancesCall::transfer_allow_death {
					dest: 2,
					value: 100000000,
				});

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

#[test]
fn compute_capacity_fee_returns_zero_when_call_is_not_capacity_eligible() {
	let balance_factor = 10;
	let call: &<Test as Config>::RuntimeCall =
		&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });
	// since we are not checking the signature in FrequencyTxPayment here we can use TestXt::new_bare for simplicity eventhough the Call would be signed one in reality
	let xt: TestXt<RuntimeCallFor<Test>, ()> = TestXt::new_bare(call.clone());
	let ext = xt.encode();
	let len = ext.len() as u32;
	let dispatch_info = call.get_dispatch_info();

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let fee = FrequencyTxPayment::compute_capacity_fee_details(
				call,
				&dispatch_info.call_weight,
				len,
			);
			assert!(fee.inclusion_fee.is_some());
			assert!(fee.tip == 0);
		});
}

#[test]
fn compute_capacity_fee_returns_fee_when_call_is_capacity_eligible() {
	let balance_factor = 10;
	let call: &<Test as Config>::RuntimeCall =
		&RuntimeCall::FrequencyTxPayment(Call::pay_with_capacity {
			call: Box::new(RuntimeCall::Msa(MsaCall::<Test>::create {})),
		});
	// since we are not checking the signature in FrequencyTxPayment here we can use TestXt::new_bare for simplicity eventhough the Call would be signed one in reality
	let xt: TestXt<RuntimeCallFor<Test>, ()> = TestXt::new_bare(call.clone());
	let ext = xt.encode();
	let len = ext.len() as u32;
	let dispatch_info = call.get_dispatch_info();
	assert!(!dispatch_info.call_weight.is_zero());

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let fee_res = FrequencyTxPayment::compute_capacity_fee_details(
				call,
				&dispatch_info.call_weight,
				len,
			);
			assert!(fee_res.inclusion_fee.is_some());
		});
}

pub fn assert_dryrun_withdraw_fee_result(
	account_id: <Test as frame_system::Config>::AccountId,
	call: &<Test as Config>::RuntimeCall,
	expected_err: Option<TransactionValidityError>,
) {
	let dispatch_info =
		DispatchInfo { call_weight: Weight::from_parts(5, 0), ..Default::default() };

	let call: &<Test as Config>::RuntimeCall =
		&RuntimeCall::FrequencyTxPayment(Call::pay_with_capacity { call: Box::new(call.clone()) });

	let dryrun_withdraw_fee = ChargeFrqTransactionPayment::<Test>::from(0u64).dryrun_withdraw_fee(
		&account_id,
		call,
		&dispatch_info,
		10,
	);

	match expected_err {
		None => assert!(dryrun_withdraw_fee.is_ok()),
		Some(err) => {
			assert!(dryrun_withdraw_fee.is_err());
			assert_eq!(err, dryrun_withdraw_fee.err().unwrap())
		},
	}
}

/// can_withdraw_fee, token transactions
#[test]
fn can_withdraw_fee_allows_configured_capacity_calls() {
	let balance_factor = 100_000_000;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let account_id = 1u64;
			let allowed_call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });

			let forbidden_call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_all { dest: 2, keep_alive: false });

			assert_dryrun_withdraw_fee_result(account_id, allowed_call, None);

			let expected_err = TransactionValidityError::Invalid(InvalidTransaction::Custom(
				ChargeFrqTransactionPaymentError::CallIsNotCapacityEligible as u8,
			));
			assert_dryrun_withdraw_fee_result(account_id, forbidden_call, Some(expected_err));
		});
}
#[test]
fn can_withdraw_fee_errors_on_capacity_transaction_without_enough_funds() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			// An account that has an MSA but has not bet the min balance for key deposit.
			let account_id = 10u64;
			let _ = tests::mock::create_msa_account(account_id);

			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });

			let expected_err = TransactionValidityError::Invalid(InvalidTransaction::Payment);
			assert_dryrun_withdraw_fee_result(account_id, call, Some(expected_err));
		});
}
#[test]
fn can_withdraw_fee_errors_for_capacity_txn_when_invalid_msa() {
	let balance_factor = 100_000_000;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			// An account that has an MSA but has not bet the min balance for key deposit.
			let account_id_not_associated_with_msa = 10u64;
			// This allows it not to fail on the requirement of an existential deposit.
			assert_ok!(Balances::force_set_balance(
				RawOrigin::Root.into(),
				account_id_not_associated_with_msa,
				1u32.into(),
			));

			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });

			let expected_err = TransactionValidityError::Invalid(InvalidTransaction::Custom(
				ChargeFrqTransactionPaymentError::InvalidMsaKey as u8,
			));
			assert_dryrun_withdraw_fee_result(
				account_id_not_associated_with_msa,
				call,
				Some(expected_err),
			);
		});
}

#[test]
fn can_withdraw_fee_errors_on_token_txn_witout_enough_funds() {
	let balance_factor = 10;

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(100, 0))
		.build()
		.execute_with(|| {
			let charge_tx_payment = ChargeFrqTransactionPayment::<Test>::from(0u64);
			let who = 1u64;
			let call: &<Test as Config>::RuntimeCall =
				&RuntimeCall::Balances(BalancesCall::transfer_allow_death { dest: 2, value: 100 });

			let info = DispatchInfo { call_weight: Weight::from_parts(5, 0), ..Default::default() };
			let len = 10;
			let error = charge_tx_payment.dryrun_withdraw_fee(&who, call, &info, len).unwrap_err();
			assert_eq!(error, TransactionValidityError::Invalid(InvalidTransaction::Payment));
		});
}

pub fn generate_test_signature() -> MultiSignature {
	let (key_pair, _) = sr25519::Pair::generate();
	let fake_data = H256::random();
	key_pair.sign(fake_data.as_bytes()).into()
}

pub fn generate_add_public_key_call(msa_id: u64, owner_id: u64) -> Box<RuntimeCall> {
	let designated_ethereum_key = 999u64;
	let proof1: MultiSignature = generate_test_signature();
	let proof2: MultiSignature = generate_test_signature();
	let payload: AddKeyData<Test> =
		AddKeyData { msa_id, expiration: 99u32, new_public_key: designated_ethereum_key };
	let add_public_key_inner = RuntimeCall::Msa(MsaCall::<Test>::add_public_key_to_msa {
		msa_owner_public_key: owner_id,
		msa_owner_proof: proof1,
		new_key_owner_proof: proof2,
		add_key_payload: payload.into(),
	});

	let add_public_key_call = Box::new(add_public_key_inner);
	Box::new(RuntimeCall::FrequencyTxPayment(FrequencyTxPaymentCall::<Test>::pay_with_capacity {
		call: add_public_key_call,
	}))
}

#[test]
fn add_public_key_to_msa_has_lower_capacity_charge_if_is_ethereum_compatible() {
	let balance_factor = 100_000_000;
	let dispatch_info =
		DispatchInfo { call_weight: Weight::from_parts(5, 0), ..Default::default() };

	// uses funded account already with MSA Id
	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let msa_id = 2u64;
			let owner_id = 2u64;
			let pay_with_capacity_add_public_key_call =
				generate_add_public_key_call(msa_id, owner_id);
			// ask if the returned weight is much less
			let withdraw_fee = ChargeFrqTransactionPayment::<Test>::from(0u64)
				.withdraw_fee(&owner_id, &pay_with_capacity_add_public_key_call, &dispatch_info, 10)
				.unwrap();
			assert!(withdraw_fee.0.lt(&106_000_000u64));
		});
}

#[test]
fn add_public_key_to_msa_not_free_if_mismatched_msa_to_account_id() {
	// using same as prior test for comparison
	let balance_factor = 100_000_000;
	let dispatch_info =
		DispatchInfo { call_weight: Weight::from_parts(5, 0), ..Default::default() };

	ExtBuilder::default()
		.balance_factor(balance_factor)
		.base_weight(Weight::from_parts(5, 0))
		.build()
		.execute_with(|| {
			let msa_id = 99u64;
			let owner_id = 2u64;
			let pay_with_capacity_add_public_key_call =
				generate_add_public_key_call(msa_id, owner_id);
			// ask if the returned weight is much less
			let withdraw_fee = ChargeFrqTransactionPayment::<Test>::from(0u64)
				.withdraw_fee(&owner_id, &pay_with_capacity_add_public_key_call, &dispatch_info, 10)
				.unwrap();
			assert!(withdraw_fee.0.gt(&270_000_000u64));
		})
}
