use super::*;
use crate as pallet_capacity;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use testing_utils::{create_capacity_account_and_fund, register_provider, staking_events};

struct TestCase<T: Config> {
	name: &'static str,
	starting_epoch: <T>::EpochNumber,
	epoch_start_block: <T>::BlockNumber,
	expected_epoch: <T>::EpochNumber,
	expected_epoch_start_block: <T>::BlockNumber,
	expected_capacity: u64,
	at_block: <T>::BlockNumber,
}

#[test]
fn start_new_epoch_works() {
	new_test_ext().execute_with(|| {
		// assumes the mock epoch length is 100 blocks.
		let test_cases: Vec<TestCase<Test>> = vec![
			TestCase {
				name: "epoch changes at the right time",
				starting_epoch: 2,
				epoch_start_block: 201,
				expected_epoch: 3,
				expected_epoch_start_block: 301,
				expected_capacity: 0,
				at_block: 301,
			},
			TestCase {
				//
				name: "epoch does not change",
				starting_epoch: 2,
				epoch_start_block: 201,
				expected_epoch: 2,
				expected_epoch_start_block: 201,
				expected_capacity: 45,
				at_block: 215,
			},
		];
		for tc in test_cases {
			CurrentEpoch::<Test>::set(tc.starting_epoch.clone());
			CurrentEpochInfo::<Test>::set(EpochInfo { epoch_start: tc.epoch_start_block });
			CurrentEpochUsedCapacity::<Test>::set(45); // just some non-zero value
			Capacity::start_new_epoch_if_needed(tc.at_block);
			assert_eq!(
				tc.expected_epoch,
				Capacity::get_current_epoch(),
				"{}: had wrong current epoch",
				tc.name
			);
			assert_eq!(
				tc.expected_epoch_start_block,
				CurrentEpochInfo::<Test>::get().epoch_start,
				"{}: had wrong epoch start block",
				tc.name
			);
			assert_eq!(
				tc.expected_capacity,
				Capacity::get_current_epoch_used_capacity(),
				"{}: had wrong used capacity",
				tc.name
			);
		}
	})
}

#[test]
fn ensure_can_stake_errors_with_zero_amount_not_allowed() {
	new_test_ext().execute_with(|| {
		let account = 100;
		let target: MessageSourceId = 1;
		let amount = 0;
		assert_noop!(
			Capacity::ensure_can_stake(&account, target, amount),
			Error::<Test>::ZeroAmountNotAllowed
		);
	});
}

#[test]
fn increase_stake_and_issue_capacity_errors_with_overflow() {
	new_test_ext().execute_with(|| {
		let target: MessageSourceId = 1;
		let staker = 100;
		let amount = 10;
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker), target, amount));
		let mut staking_account = Capacity::get_staking_account_for(staker).unwrap_or_default();

		let overflow_amount = u64::MAX;

		assert_noop!(
			Capacity::increase_stake_and_issue_capacity(
				&staker,
				&mut staking_account,
				target,
				overflow_amount
			),
			ArithmeticError::Overflow
		);
	});
}

#[test]
fn ensure_can_stake_errors_invalid_target() {
	new_test_ext().execute_with(|| {
		let account = 100;
		let target: MessageSourceId = 1;
		let amount = 1;

		assert_noop!(
			Capacity::ensure_can_stake(&account, target, amount),
			Error::<Test>::InvalidTarget
		);
	});
}

#[test]
fn ensure_can_stake_errors_insufficient_staking_amount() {
	new_test_ext().execute_with(|| {
		let account = 100;
		let target: MessageSourceId = 1;
		let amount = 4;
		register_provider(target, String::from("Foo"));

		assert_noop!(
			Capacity::ensure_can_stake(&account, target, amount),
			Error::<Test>::InsufficientStakingAmount
		);
	});
}

#[test]
fn ensure_can_stake_is_successful() {
	new_test_ext().execute_with(|| {
		let account = 100;
		let target: MessageSourceId = 1;
		let amount = 10;
		register_provider(target, String::from("Foo"));

		let staking_details = StakingAccountDetails::<Test>::default();
		assert_ok!(
			Capacity::ensure_can_stake(&account, target, amount),
			(staking_details, BalanceOf::<Test>::from(10u64))
		);
	});
}

#[test]
fn increase_stake_and_issue_capacity_is_successful() {
	new_test_ext().execute_with(|| {
		let staker = 100;
		let target: MessageSourceId = 1;
		let amount = 55;
		let mut staking_account = StakingAccountDetails::<Test>::default();

		assert_ok!(Capacity::increase_stake_and_issue_capacity(
			&staker,
			&mut staking_account,
			target,
			amount
		));

		assert_eq!(staking_account.total, 55);
		assert_eq!(staking_account.active, 55);
		assert_eq!(staking_account.unlocking.len(), 0);

		let capacity_details = Capacity::get_capacity_for(&target).unwrap();

		assert_eq!(capacity_details.remaining, 55);
		assert_eq!(capacity_details.total_available, 55);
		assert_eq!(capacity_details.last_replenished_epoch, 0);

		let target_details = Capacity::get_target_for(&staker, &target).unwrap();

		assert_eq!(target_details.amount, 55);
		assert_eq!(target_details.capacity, 55);
	});
}

#[test]
fn set_staking_account_is_succesful() {
	new_test_ext().execute_with(|| {
		let staker = 100;
		let mut staking_account = StakingAccountDetails::<Test>::default();
		staking_account.deposit(55);

		Capacity::set_staking_account(&staker, &staking_account);

		assert_eq!(Balances::locks(&staker)[0].amount, 55);
	});
}

#[test]
fn set_target_details_is_successful() {
	new_test_ext().execute_with(|| {
		let staker = 100;
		let target: MessageSourceId = 1;

		assert_eq!(Capacity::get_target_for(&staker, target), None);

		let mut target_details = StakingTargetDetails::<BalanceOf<Test>>::default();
		target_details.amount = 10;
		target_details.capacity = 10;

		Capacity::set_target_details_for(&staker, target, target_details);

		let stored_target_details = Capacity::get_target_for(&staker, target).unwrap();

		assert_eq!(stored_target_details.amount, 10);
		assert_eq!(stored_target_details.capacity, 10);
	});
}

#[test]
fn set_capacity_details_is_successful() {
	new_test_ext().execute_with(|| {
		let target: MessageSourceId = 1;

		assert_eq!(Capacity::get_capacity_for(target), None);

		let capacity_details: CapacityDetails<BalanceOf<Test>, <Test as Config>::EpochNumber> =
			CapacityDetails {
				remaining: 10u64,
				total_tokens_staked: 10u64,
				total_available: 10u64,
				last_replenished_epoch: 1u32,
			};

		Capacity::set_capacity_for(target, capacity_details);

		let stored_capacity_details = Capacity::get_capacity_for(target).unwrap();

		assert_eq!(stored_capacity_details.remaining, 10);
		assert_eq!(stored_capacity_details.total_available, 10);
		assert_eq!(stored_capacity_details.last_replenished_epoch, 1);
	});
}

#[test]
fn it_configures_staking_minimum_greater_than_or_equal_to_existential_deposit() {
	new_test_ext().execute_with(|| {
		let minimum_staking_balance_config: BalanceOf<Test> =
			<Test as pallet_capacity::Config>::MinimumStakingAmount::get();
		assert!(
			minimum_staking_balance_config >=
				<Test as pallet_capacity::Config>::Currency::minimum_balance()
		)
	});
}

#[test]
fn calculate_capacity_reduction_determines_the_correct_capacity_reduction_amount() {
	let unstaking_amount = 10;
	let total_amount_staked = 100;
	let total_capacity = 200;

	let capacity_reduction = Capacity::calculate_capacity_reduction(
		unstaking_amount,
		total_amount_staked,
		total_capacity,
	);

	assert_eq!(capacity_reduction, 20);
}

#[test]
fn impl_balance_is_successful() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let remainging_amount = 10u32;
		let total_available_amount = 10u32;

		let _ = create_capacity_account_and_fund(
			target_msa_id,
			remainging_amount.into(),
			total_available_amount.into(),
			1u32,
		);

		assert_eq!(Capacity::balance(target_msa_id), BalanceOf::<Test>::from(10u32));
	});
}

#[test]
fn impl_balance_returns_zero_when_target_capacity_is_not_found() {
	new_test_ext().execute_with(|| {
		let msa_id = 1;
		assert_eq!(Capacity::balance(msa_id), BalanceOf::<Test>::zero());
	});
}

#[test]
fn impl_withdraw_is_successful() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let remaining_amount = BalanceOf::<Test>::from(10u32);
		let total_available_amount = BalanceOf::<Test>::from(10u32);
		let _ = create_capacity_account_and_fund(
			target_msa_id,
			remaining_amount,
			total_available_amount,
			1u32,
		);

		assert_ok!(Capacity::withdraw(target_msa_id, 5u32.into()));
		let events = staking_events();

		assert_eq!(
			events.last().unwrap(),
			&Event::CapacityWithdrawn { msa_id: target_msa_id, amount: 5u32.into() }
		);

		let mut capacity_details =
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber>::default();

		capacity_details.remaining = 5u32.into();
		capacity_details.total_tokens_staked = 10u32.into();
		capacity_details.total_available = 10u32.into();
		capacity_details.last_replenished_epoch = 1u32.into();

		assert_eq!(Capacity::get_capacity_for(target_msa_id).unwrap(), capacity_details);
	});
}

#[test]
fn impl_withdraw_errors_target_capacity_not_found() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let amount = BalanceOf::<Test>::from(10u32);
		assert_noop!(
			Capacity::withdraw(target_msa_id, amount),
			Error::<Test>::TargetCapacityNotFound
		);
	});
}

#[test]
fn impl_withdraw_errors_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let remaining_amount = BalanceOf::<Test>::from(10u32);
		let total_available_amount = BalanceOf::<Test>::from(10u32);
		let _ = create_capacity_account_and_fund(
			target_msa_id,
			remaining_amount,
			total_available_amount,
			1u32,
		);

		assert_noop!(
			Capacity::withdraw(target_msa_id, 20u32.into()),
			Error::<Test>::InsufficientBalance
		);

		let mut capacity_details =
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber>::default();

		capacity_details.remaining = 10u32.into();
		capacity_details.total_tokens_staked = 10u32.into();
		capacity_details.total_available = 10u32.into();
		capacity_details.last_replenished_epoch = 1u32.into();

		assert_eq!(Capacity::get_capacity_for(target_msa_id).unwrap(), capacity_details);
	});
}
#[test]
fn impl_deposit_is_successful() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let remaining_amount = BalanceOf::<Test>::from(5u32);
		let total_available_amount = BalanceOf::<Test>::from(10u32);
		let _ = create_capacity_account_and_fund(
			target_msa_id,
			remaining_amount,
			total_available_amount,
			1u32,
		);

		assert_ok!(Capacity::deposit(target_msa_id, 5u32.into()),);
	});
}

#[test]
fn impl_deposit_errors_target_capacity_not_found() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let amount = BalanceOf::<Test>::from(10u32);
		assert_noop!(
			Capacity::deposit(target_msa_id, amount),
			Error::<Test>::TargetCapacityNotFound
		);
	});
}

#[test]
fn impl_deposit_errors_increase_exceeds_available_when_trying_deposit_a_larger_amount_than_the_available_capacity(
) {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let remaining_amount = BalanceOf::<Test>::from(5u32);
		let total_available_amount = BalanceOf::<Test>::from(10u32);
		let _ = create_capacity_account_and_fund(
			target_msa_id,
			remaining_amount,
			total_available_amount,
			1u32,
		);

		assert_noop!(
			Capacity::deposit(target_msa_id, 10u32.into()),
			Error::<Test>::IncreaseExceedsAvailable
		);

		let mut capacity_details =
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber>::default();

		capacity_details.remaining = 5u32.into();
		capacity_details.total_tokens_staked = 10u32.into();
		capacity_details.total_available = 10u32.into();
		capacity_details.last_replenished_epoch = 1u32.into();

		assert_eq!(Capacity::get_capacity_for(target_msa_id).unwrap(), capacity_details);
	});
}

#[test]
fn impl_replenish_all_for_account_is_successful() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let remaining_amount = BalanceOf::<Test>::from(3u32);
		let total_available_amount = BalanceOf::<Test>::from(10u32);
		let _ = create_capacity_account_and_fund(
			target_msa_id,
			remaining_amount,
			total_available_amount,
			1u32,
		);

		assert_ok!(Capacity::replenish_all_for(target_msa_id));

		let mut capacity_details =
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber>::default();

		capacity_details.remaining = 10u32.into();
		capacity_details.total_tokens_staked = 10u32.into();
		capacity_details.total_available = 10u32.into();
		capacity_details.last_replenished_epoch = 1u32.into();

		assert_eq!(Capacity::get_capacity_for(target_msa_id).unwrap(), capacity_details);
	});
}

#[test]
fn impl_replenish_all_for_account_errors_target_capacity_not_found() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		assert_noop!(
			Capacity::replenish_all_for(target_msa_id),
			Error::<Test>::TargetCapacityNotFound
		);
	});
}

#[test]
fn impl_can_replenish_is_false_when_last_replenished_at_is_greater_or_equal_current_epoch() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let remaining_amount = BalanceOf::<Test>::from(3u32);
		let total_available_amount = BalanceOf::<Test>::from(10u32);
		let last_replenished_at = 1u32;

		let _ = create_capacity_account_and_fund(
			target_msa_id,
			remaining_amount,
			total_available_amount,
			last_replenished_at,
		);

		assert_eq!(Capacity::get_current_epoch(), 0);

		assert_eq!(Capacity::can_replenish(target_msa_id), false);

		CurrentEpoch::<Test>::set(1u32.into());

		assert_eq!(Capacity::can_replenish(target_msa_id), false);
	});
}

#[test]
fn impl_can_replenish_is_true_when_last_replenished_at_is_less_than_current_epoch() {
	new_test_ext().execute_with(|| {
		let target_msa_id = 1;
		let remaining_amount = BalanceOf::<Test>::from(3u32);
		let total_available_amount = BalanceOf::<Test>::from(10u32);
		let last_replenished_at = 2u32;

		let _ = create_capacity_account_and_fund(
			target_msa_id,
			remaining_amount,
			total_available_amount,
			last_replenished_at,
		);

		CurrentEpoch::<Test>::set(3u32.into());

		assert_eq!(Capacity::can_replenish(target_msa_id), true);
	});
}

#[test]
fn get_epoch_length_should_return_max_epoch_length_when_unset() {
	new_test_ext().execute_with(|| {
		let epoch_length: <Test as frame_system::Config>::BlockNumber =
			Capacity::get_epoch_length();
		let max_epoch_length: u64 = <Test as pallet_capacity::Config>::MaxEpochLength::get();

		assert_eq!(epoch_length, max_epoch_length);
	});
}
#[test]
fn get_epoch_length_should_return_storage_epoch_length() {
	new_test_ext().execute_with(|| {
		EpochLength::<Test>::set(101u64);
		let epoch_length: <Test as frame_system::Config>::BlockNumber =
			Capacity::get_epoch_length();

		assert_eq!(epoch_length, 101u64);
	});
}
