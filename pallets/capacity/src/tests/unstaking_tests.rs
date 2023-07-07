use super::{mock::*, testing_utils::*};
use crate as pallet_capacity;
use crate::{
	CapacityDetails, FreezeReason, StakingDetails, StakingTargetDetails, StakingType, UnlockChunk,
};
use common_primitives::msa::MessageSourceId;
use frame_support::{
	assert_noop, assert_ok,
	traits::{fungible::InspectFreeze, Get},
};
use pallet_capacity::{BalanceOf, Config, Error, Event};
use sp_core::bounded::BoundedVec;

#[test]
fn unstake_happy_path() {
	new_test_ext().execute_with(|| {
		let token_account = 200;
		let target: MessageSourceId = 1;
		let staking_amount = 100;
		let unstaking_amount = 40;

		register_provider(target, String::from("Test Target"));

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(token_account), target, staking_amount));
		assert_ok!(Capacity::unstake(
			RuntimeOrigin::signed(token_account),
			target,
			unstaking_amount
		));

		// Assert that staking account detail values are decremented correctly after unstaking
		let staking_account_details = Capacity::get_staking_account_for(token_account).unwrap();

		let expected_unlocking_chunks: BoundedVec<
			UnlockChunk<BalanceOf<Test>, <Test as Config>::EpochNumber>,
			<Test as Config>::MaxUnlockingChunks,
		> = BoundedVec::try_from(vec![UnlockChunk { value: unstaking_amount, thaw_at: 2u32 }])
			.unwrap();

		let unlocking = Capacity::get_unstake_unlocking_for(token_account).unwrap();
		assert_eq!(unlocking, expected_unlocking_chunks);

		assert_eq!(
			StakingDetails::<Test> {
				active: BalanceOf::<Test>::from(60u64),
				staking_type: StakingType::MaximumCapacity,
			},
			staking_account_details,
		);

		// Assert that staking target detail values are decremented correctly after unstaking
		let staking_target_details = Capacity::get_target_for(token_account, target).unwrap();

		assert_eq!(
			staking_target_details,
			StakingTargetDetails::<Test> {
				amount: BalanceOf::<Test>::from(60u64),
				capacity: BalanceOf::<Test>::from(6u64),
			}
		);

		// Assert that the capacity detail values for the target are decremented properly after unstaking
		let capacity_details = Capacity::get_capacity_for(target).unwrap();

		assert_eq!(
			capacity_details,
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
				remaining_capacity: BalanceOf::<Test>::from(6u64),
				total_tokens_staked: BalanceOf::<Test>::from(60u64),
				total_capacity_issued: BalanceOf::<Test>::from(6u64),
				last_replenished_epoch: <Test as Config>::EpochNumber::from(0u32),
			}
		);

		let events = staking_events();
		assert_eq!(
			events.last().unwrap(),
			&Event::UnStaked {
				account: token_account,
				target,
				amount: unstaking_amount,
				capacity: BalanceOf::<Test>::from(4u64)
			}
		);
	});
}

#[test]
fn unstake_errors_unstaking_amount_is_zero() {
	new_test_ext().execute_with(|| {
		let token_account = 200;
		let target: MessageSourceId = 1;
		let staking_amount = 10;
		let unstaking_amount = 0;

		register_provider(target, String::from("Test Target"));

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(token_account), target, staking_amount));
		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(token_account), target, unstaking_amount),
			Error::<Test>::UnstakedAmountIsZero
		);
	});
}

#[test]
fn unstake_errors_max_unlocking_chunks_exceeded() {
	new_test_ext().execute_with(|| {
		let token_account = 200;
		let target: MessageSourceId = 1;
		let staking_amount = 60;
		let unstaking_amount = 10;

		register_provider(target, String::from("Test Target"));

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(token_account), target, staking_amount));

		for _n in 0..<Test as Config>::MaxUnlockingChunks::get() {
			assert_ok!(Capacity::unstake(
				RuntimeOrigin::signed(token_account),
				target,
				unstaking_amount
			));
		}

		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(token_account), target, unstaking_amount),
			Error::<Test>::MaxUnlockingChunksExceeded
		);
	});
}

#[test]
fn unstake_errors_amount_to_unstake_exceeds_amount_staked() {
	new_test_ext().execute_with(|| {
		let token_account = 200;
		let target: MessageSourceId = 1;
		let staking_amount = 10;
		let unstaking_amount = 11;

		register_provider(target, String::from("Test Target"));

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(token_account), target, staking_amount));
		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(token_account), target, unstaking_amount),
			Error::<Test>::InsufficientStakingBalance
		);
	});
}

#[test]
fn unstake_errors_not_a_staking_account() {
	new_test_ext().execute_with(|| {
		let token_account = 200;
		let target: MessageSourceId = 1;

		let unstaking_amount = 11;

		register_provider(target, String::from("Test Target"));

		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(token_account), target, unstaking_amount),
			Error::<Test>::NotAStakingAccount
		);
	});
}

#[test]
fn unstaking_everything_reaps_staking_account() {
	new_test_ext().execute_with(|| {
		let staker = 500;
		let target = 1;
		let amount = 20;
		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));

		register_provider(target, String::from("WithdrawUnst"));
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker), target, amount));

		run_to_block(1);
		// unstake everything
		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(staker), target, 20));
		assert_eq!(20u64, Balances::balance_frozen(&FreezeReason::CapacityStaking.into(), &staker));

		// it should reap the staking account right away
		assert!(Capacity::get_staking_account_for(&staker).is_none());
	})
}

#[test]
fn unstake_when_not_staking_to_target_errors() {
	new_test_ext().execute_with(|| {
		let staker = 500;
		let target = 1;
		let amount = 20;
		register_provider(target, String::from("WithdrawUnst"));

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker), target, amount));
		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(staker), 2, 20),
			Error::<Test>::StakerTargetRelationshipNotFound
		);
	})
}
