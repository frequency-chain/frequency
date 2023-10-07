use super::{mock::*, testing_utils::*};
use crate as pallet_capacity;
use crate::{CapacityDetails, StakingAccountDetails, StakingTargetDetails, UnlockChunk};
use common_primitives::{
	capacity::{StakingType, StakingType::MaximumCapacity},
	msa::MessageSourceId,
};
use frame_support::{assert_noop, assert_ok, traits::Get};
use pallet_capacity::{BalanceOf, Config, Error, Event};
use sp_core::bounded::BoundedVec;

#[test]
fn unstake_happy_path() {
	new_test_ext().execute_with(|| {
		// TODO: ProviderBoost after unstake affects reward pool info
		let token_account = 200;
		let target: MessageSourceId = 1;
		let staking_amount = 100;
		let unstaking_amount = 40;

		register_provider(target, String::from("Test Target"));

		assert_ok!(Capacity::stake(
			RuntimeOrigin::signed(token_account),
			target,
			staking_amount,
			MaximumCapacity
		));
		assert_ok!(Capacity::unstake(
			RuntimeOrigin::signed(token_account),
			target,
			unstaking_amount
		));

		// Assert that staking account detail values are decremented correctly after unstaking
		let staking_account_details = Capacity::get_staking_account_for(token_account).unwrap();

		assert_eq!(staking_account_details.unlocking.len(), 1);
		let expected_unlocking_chunks: BoundedVec<
			UnlockChunk<BalanceOf<Test>, <Test as Config>::EpochNumber>,
			<Test as Config>::MaxUnlockingChunks,
		> = BoundedVec::try_from(vec![UnlockChunk { value: unstaking_amount, thaw_at: 2u32 }])
			.unwrap();

		assert_eq!(
			StakingAccountDetails::<Test> {
				active: BalanceOf::<Test>::from(60u64),
				total: BalanceOf::<Test>::from(staking_amount),
				unlocking: expected_unlocking_chunks,
				last_rewards_claimed_at: None,
				stake_change_unlocking: BoundedVec::default(),
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
				staking_type: StakingType::MaximumCapacity,
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

		assert_ok!(Capacity::stake(
			RuntimeOrigin::signed(token_account),
			target,
			staking_amount,
			MaximumCapacity
		));
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

		assert_ok!(Capacity::stake(
			RuntimeOrigin::signed(token_account),
			target,
			staking_amount,
			MaximumCapacity
		));

		for _n in 0..<Test as pallet_capacity::Config>::MaxUnlockingChunks::get() {
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

		assert_ok!(Capacity::stake(
			RuntimeOrigin::signed(token_account),
			target,
			staking_amount,
			MaximumCapacity
		));
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
