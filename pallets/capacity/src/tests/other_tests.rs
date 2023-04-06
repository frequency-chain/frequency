use frame_support::traits::{Currency, Get};
use sp_runtime::traits::Zero;

use common_primitives::{capacity::Nontransferable, msa::MessageSourceId};

use crate::{
	BalanceOf, CapacityDetails, Config, CurrentEpoch, CurrentEpochInfo, EpochInfo,
	StakingAccountDetails, StakingTargetDetails,
};

use super::{mock::*, testing_utils::*};

struct TestCase<T: Config> {
	name: &'static str,
	starting_epoch: <T>::EpochNumber,
	epoch_start_block: <T>::BlockNumber,
	expected_epoch: <T>::EpochNumber,
	expected_epoch_start_block: <T>::BlockNumber,
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
				at_block: 301,
			},
			TestCase {
				//
				name: "epoch does not change",
				starting_epoch: 2,
				epoch_start_block: 201,
				expected_epoch: 2,
				expected_epoch_start_block: 201,
				at_block: 215,
			},
		];
		for tc in test_cases {
			CurrentEpoch::<Test>::set(tc.starting_epoch.clone());
			CurrentEpochInfo::<Test>::set(EpochInfo { epoch_start: tc.epoch_start_block });
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
		}
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
				remaining_capacity: 10u64,
				total_tokens_staked: 10u64,
				total_capacity_issued: 10u64,
				last_replenished_epoch: 1u32,
			};

		Capacity::set_capacity_for(target, capacity_details);

		let stored_capacity_details = Capacity::get_capacity_for(target).unwrap();

		assert_eq!(stored_capacity_details.remaining_capacity, 10);
		assert_eq!(stored_capacity_details.total_capacity_issued, 10);
		assert_eq!(stored_capacity_details.last_replenished_epoch, 1);
	});
}

#[test]
fn it_configures_staking_minimum_greater_than_or_equal_to_existential_deposit() {
	new_test_ext().execute_with(|| {
		let minimum_staking_balance_config: BalanceOf<Test> =
			<Test as Config>::MinimumStakingAmount::get();
		assert!(minimum_staking_balance_config >= <Test as Config>::Currency::minimum_balance())
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
		let remaining_amount = 10u32;
		let total_available_amount = 10u32;

		let _ = create_capacity_account_and_fund(
			target_msa_id,
			remaining_amount.into(),
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
