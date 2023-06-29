use super::{mock::*, testing_utils::*};
use crate::{BalanceOf, CapacityDetails, Config, Error, StakingTargetDetails};
use common_primitives::{
	capacity::StakingType::{MaximumCapacity, ProviderBoost},
	msa::MessageSourceId,
};
use frame_support::{assert_noop, assert_ok, traits::Get};

fn setup_provider(staker: u64, target: MessageSourceId, amount: u64) {
	let provider_name = String::from("Cst-") + target.to_string().as_str();
	register_provider(target, provider_name);
	if amount > 0 {
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker), target, amount, ProviderBoost));
	}
}

type TestCapacityDetails = CapacityDetails<BalanceOf<Test>, u32>;
type TestTargetDetails = StakingTargetDetails<BalanceOf<Test>>;

#[test]
fn set_capacity_for_deletes_staking_target_details_if_amount_is_zero() {
	new_test_ext().execute_with(|| {
		let token_account = 200;
		let target: MessageSourceId = 1;
		let target2: MessageSourceId = 2;
		// minimum amount + 1 so it doesn't zero out the balance when we unstake
		let staking_amount = 11;

		register_provider(target, String::from("Test Target"));
		register_provider(target2, String::from("Test Target2"));

		// stake to both of them so we still have a staking balance overall
		assert_ok!(Capacity::stake(
			RuntimeOrigin::signed(token_account),
			target,
			staking_amount,
			MaximumCapacity
		));
		assert_ok!(Capacity::stake(
			RuntimeOrigin::signed(token_account),
			target2,
			staking_amount,
			MaximumCapacity
		));
		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(token_account), target, staking_amount));
		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(token_account), target, 1),
			Error::<Test>::StakerTargetRelationshipNotFound
		);
	})
}

#[test]
fn do_retarget_happy_path() {
	// reduces from_msa capacity, amount by provided amount/expected capacity
	// adds capacity/amount to to_msa
	new_test_ext().execute_with(|| {
		let staker = 200u64;
		let from_msa: MessageSourceId = 1;
		let from_amount = 20u64;
		let to_amount = 10u64;
		let to_msa: MessageSourceId = 2;
		setup_provider(staker, from_msa, from_amount);
		setup_provider(staker, to_msa, to_amount);

		// retarget half the stake to to_msa
		Capacity::do_retarget(&staker, &from_msa, &to_msa, &to_amount);

		let expected_from_details: TestCapacityDetails = CapacityDetails {
			remaining_capacity: 1,
			total_tokens_staked: 10,
			total_capacity_issued: 1,
			last_replenished_epoch: 0,
		};
		let from_capacity_details: TestCapacityDetails =
			Capacity::get_capacity_for(from_msa).unwrap();
		assert_eq!(from_capacity_details, expected_from_details);

		let expected_to_details: TestCapacityDetails = CapacityDetails {
			remaining_capacity: 2,
			total_tokens_staked: 20,
			total_capacity_issued: 2,
			last_replenished_epoch: 0,
		};
		let to_capacity_details = Capacity::get_capacity_for(to_msa).unwrap();
		assert_eq!(to_capacity_details, expected_to_details);

		let expected_from_target_details: TestTargetDetails =
			StakingTargetDetails { amount: 10, capacity: 1 };
		let from_target_details = Capacity::get_target_for(staker, from_msa).unwrap();
		assert_eq!(from_target_details, expected_from_target_details);

		let expected_to_target_details: TestTargetDetails =
			StakingTargetDetails { amount: 20, capacity: 2 };
		let to_target_details = Capacity::get_target_for(staker, to_msa).unwrap();
		assert_eq!(to_target_details, expected_to_target_details);
	})
}

#[test]
fn do_retarget_deletes_staking_account_if_zero_balance() {
	new_test_ext().execute_with(|| {
		let staker = 200u64;
		let from_msa: MessageSourceId = 1;
		let to_msa: MessageSourceId = 2;
		let amount = 10u64;
		setup_provider(staker, from_msa, amount);
		setup_provider(staker, to_msa, amount);

		// stake additional to provider from another Msa, doesn't matter which type.
		// total staked to from_msa is now 22u64.
		assert_ok!(Capacity::stake(
			RuntimeOrigin::signed(300u64),
			from_msa,
			12u64,
			MaximumCapacity
		));

		Capacity::do_retarget(&staker, &from_msa, &to_msa, &amount);

		let expected_from_details: TestCapacityDetails = CapacityDetails {
			remaining_capacity: 1,
			total_tokens_staked: 12,
			total_capacity_issued: 1,
			last_replenished_epoch: 0,
		};
		let from_capacity_details: TestCapacityDetails =
			Capacity::get_capacity_for(from_msa).unwrap();
		assert_eq!(from_capacity_details, expected_from_details);

		let expected_to_details: TestCapacityDetails = CapacityDetails {
			remaining_capacity: 2,
			total_tokens_staked: 2 * amount,
			total_capacity_issued: 2,
			last_replenished_epoch: 0,
		};

		let to_capacity_details = Capacity::get_capacity_for(to_msa).unwrap();
		assert_eq!(to_capacity_details, expected_to_details);

		assert!(Capacity::get_target_for(staker, from_msa).is_none());

		let expected_to_target_details: TestTargetDetails =
			StakingTargetDetails { amount: 2 * amount, capacity: 2 };
		let to_target_details = Capacity::get_target_for(staker, to_msa).unwrap();
		assert_eq!(to_target_details, expected_to_target_details);
	})
}

#[test]
fn change_staking_target_errors_if_amount_too_low() {
	new_test_ext().execute_with(|| {
		let staker = 200u64;
		let from_msa: MessageSourceId = 1;
		let to_msa: MessageSourceId = 2;

		let max_chunks: u32 = <Test as Config>::MaxUnlockingChunks::get();
		let staking_amount = (max_chunks + 2u32) * 10u32;
		setup_provider(staker, from_msa, staking_amount.into());
		setup_provider(staker, to_msa, 10);

		let retarget_amount = 10u64;
		for _i in 0..(max_chunks) {
			assert_ok!(Capacity::change_staking_target(
				RuntimeOrigin::signed(staker),
				from_msa,
				to_msa,
				retarget_amount
			));
		}

		struct TestCase {
			retarget_amount: u64,
			expected_err: Error<Test>,
		}
		let test_cases: Vec<TestCase> = vec![
			TestCase { retarget_amount: 0, expected_err: Error::<Test>::StakingAmountBelowMinimum },
			TestCase { retarget_amount: 1, expected_err: Error::<Test>::StakingAmountBelowMinimum },
			TestCase {
				retarget_amount: 999,
				expected_err: Error::<Test>::InsufficientStakingBalance,
			},
			// TestCase { retarget_amount, expected_err: Error::<Test>::MaxUnlockingChunksExceeded },
		];
		for tc in test_cases {
			assert_noop!(
				Capacity::change_staking_target(
					RuntimeOrigin::signed(staker),
					from_msa,
					to_msa,
					tc.retarget_amount
				),
				tc.expected_err
			);
		}
	});
}

#[test]
fn change_staking_target_parametric_validity() {
	new_test_ext().execute_with(|| {
		let account = 200u64;
		let from_target: MessageSourceId = 1;
		let from_amount = 10u64;
		setup_provider(account, from_target, 0);

		let to_target: MessageSourceId = 2;
		assert_noop!(
			Capacity::change_staking_target(
				RuntimeOrigin::signed(account),
				from_target,
				to_target,
				0
			),
			Error::<Test>::StakerTargetRelationshipNotFound
		);

		assert_ok!(Capacity::stake(
			RuntimeOrigin::signed(account),
			from_target,
			from_amount,
			ProviderBoost
		));

		assert_noop!(
			Capacity::change_staking_target(
				RuntimeOrigin::signed(account),
				from_target,
				to_target,
				0
			),
			Error::<Test>::StakingAmountBelowMinimum
		);

		assert_noop!(
			Capacity::change_staking_target(
				RuntimeOrigin::signed(account),
				from_target,
				to_target,
				12
			),
			Error::<Test>::InvalidTarget
		);
		setup_provider(account, to_target, 0);

		assert_ok!(Capacity::change_staking_target(
			RuntimeOrigin::signed(account),
			from_target,
			to_target,
			from_amount
		));
	});
}
