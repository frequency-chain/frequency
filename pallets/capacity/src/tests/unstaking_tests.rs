use super::{mock::*, testing_utils::*};
use crate as pallet_capacity;
use crate::{
	CapacityDetails, CapacityLedger, FreezeReason, StakingAccountLedger, StakingDetails,
	StakingTargetDetails, StakingTargetLedger, StakingType, UnlockChunk, UnstakeUnlocks,
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
		let staking_account_details = StakingAccountLedger::<Test>::get(token_account).unwrap();

		let expected_unlocking_chunks: BoundedVec<
			UnlockChunk<BalanceOf<Test>, <Test as Config>::EpochNumber>,
			<Test as Config>::MaxUnlockingChunks,
		> = BoundedVec::try_from(vec![UnlockChunk { value: unstaking_amount, thaw_at: 2u32 }])
			.unwrap();

		let unlocking = UnstakeUnlocks::<Test>::get(token_account).unwrap();
		assert_eq!(unlocking, expected_unlocking_chunks);

		assert_eq!(
			StakingDetails::<Test> {
				active: BalanceOf::<Test>::from(60u64),
				staking_type: StakingType::MaximumCapacity,
			},
			staking_account_details,
		);

		// Assert that staking target detail values are decremented correctly after unstaking
		let staking_target_details =
			StakingTargetLedger::<Test>::get(token_account, target).unwrap();

		assert_eq!(
			staking_target_details,
			StakingTargetDetails::<BalanceOf<Test>> {
				amount: BalanceOf::<Test>::from(60u32),
				capacity: BalanceOf::<Test>::from(6u32),
			}
		);

		// Assert that the capacity detail values for the target are decremented properly after unstaking
		let capacity_details = CapacityLedger::<Test>::get(target).unwrap();

		assert_eq!(
			capacity_details,
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
				remaining_capacity: BalanceOf::<Test>::from(6u64),
				total_tokens_staked: BalanceOf::<Test>::from(60u64),
				total_capacity_issued: BalanceOf::<Test>::from(6u64),
				last_replenished_epoch: <Test as Config>::EpochNumber::from(0u32),
			}
		);

		let events = capacity_events();
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

// This test checks that when two accounts stake to a target, and one
// account unstakes everything, that all the capacity generated is removed AND that
// the remaining capacity is correct
#[test]
fn unstaking_all_by_one_staker_reaps_target() {
	new_test_ext().execute_with(|| {
		let token_account = 200;
		let token_account2 = 400;
		let staking_amount1 = 100;
		let staking_amount2 = 101;
		let target: MessageSourceId = 1;

		register_provider(target, String::from("Test Target"));

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(token_account), target, staking_amount1));
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(token_account2), target, staking_amount2));

		let mut capacity_details = CapacityLedger::<Test>::get(target).unwrap();
		assert_eq!(
			capacity_details,
			CapacityDetails {
				remaining_capacity: 20,
				total_tokens_staked: 201,
				total_capacity_issued: 20,
				last_replenished_epoch: 0,
			}
		);

		assert_ok!(Capacity::unstake(
			RuntimeOrigin::signed(token_account),
			target,
			staking_amount1
		));

		// Assert that the staking details is reaped
		assert!(StakingAccountLedger::<Test>::get(token_account).is_none());

		// Assert target details is reaped
		assert!(StakingTargetLedger::<Test>::get(token_account, target).is_none());

		// Assert that capacity account is adjusted correctly
		capacity_details = CapacityLedger::<Test>::get(target).unwrap();
		assert_eq!(
			capacity_details,
			CapacityDetails {
				remaining_capacity: 10,
				total_tokens_staked: 101,
				total_capacity_issued: 10,
				last_replenished_epoch: 0,
			}
		);
	})
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

fn fill_unstake_unlock_chunks(token_account: u64, target: MessageSourceId, unstaking_amount: u64) {
	for _n in 0..<Test as Config>::MaxUnlockingChunks::get() {
		assert_ok!(Capacity::unstake(
			RuntimeOrigin::signed(token_account),
			target,
			unstaking_amount
		));
	}
}
#[test]
fn unstake_errors_max_unlocking_chunks_exceeded_stake() {
	new_test_ext().execute_with(|| {
		let token_account = 200;
		let target: MessageSourceId = 1;
		let staking_amount = 60;
		let unstaking_amount = 10;

		register_provider(target, String::from("Test Target"));

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(token_account), target, staking_amount));

		fill_unstake_unlock_chunks(token_account, target, unstaking_amount);

		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(token_account), target, unstaking_amount),
			Error::<Test>::MaxUnlockingChunksExceeded
		);
	});
}
#[test]
fn unstake_errors_max_unlocking_chunks_exceeded_provider_boost() {
	new_test_ext().execute_with(|| {
		let token_account = 200;
		let target: MessageSourceId = 1;
		let staking_amount = 60;
		let unstaking_amount = 10;

		register_provider(target, String::from("Test Target"));

		assert_ok!(Capacity::provider_boost(
			RuntimeOrigin::signed(token_account),
			target,
			staking_amount
		));

		fill_unstake_unlock_chunks(token_account, target, unstaking_amount);

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
		let booster = 600;
		let target = 1;
		let amount = 20;
		assert_ok!(Capacity::set_epoch_length(RuntimeOrigin::root(), 10));

		register_provider(target, String::from("WithdrawUnst"));
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker), target, amount));
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(booster), target, amount));

		run_to_block(1);
		// unstake everything
		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(staker), target, 20));
		assert_eq!(20u64, Balances::balance_frozen(&FreezeReason::CapacityStaking.into(), &staker));
		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(booster), target, 20));
		assert_eq!(20u64, Balances::balance_frozen(&FreezeReason::CapacityStaking.into(), &staker));

		// it should reap the staking account right away
		assert!(StakingAccountLedger::<Test>::get(&staker).is_none());
		assert!(StakingAccountLedger::<Test>::get(&booster).is_none());
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

#[test]
fn unstake_provider_boosted_target_in_same_era_adjusts_reward_pool_total() {
	new_test_ext().execute_with(|| {
		// two accounts staking to the same target
		let account1 = 600;
		let target: MessageSourceId = 1;
		let amount1 = 500;
		let unstake_amount = 200;
		register_provider(target, String::from("Foo"));
		run_to_block(5); // ensures Capacity::on_initialize is run

		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account1), target, amount1));
		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(account1), target, unstake_amount));

		assert_eq!(CurrentEraProviderBoostTotal::<Test>::get(), 300u64);
	});
}

#[test]
fn unstake_maximum_does_not_change_reward_pool() {
	new_test_ext().execute_with(|| {
		// two accounts staking to the same target
		let account1 = 600;
		let a_booster = 10_000;
		let target: MessageSourceId = 1;
		let amount1 = 500;
		let unstake_amount = 200;

		register_provider(target, String::from("Foo"));
		run_to_block(5); // ensures Capacity::on_initialize is run

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account1), target, amount1));
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(a_booster), target, amount1));

		assert_eq!(CurrentEraProviderBoostTotal::<Test>::get(), amount1);

		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(account1), target, unstake_amount));
		assert_eq!(CurrentEraProviderBoostTotal::<Test>::get(), amount1);
	});
}

#[test]
fn unstake_fills_up_common_unlock_for_any_target() {
	new_test_ext().execute_with(|| {
		let staker = 10_000;

		let target1 = 1;
		let target2 = 2;
		register_provider(target1, String::from("Test Target"));
		register_provider(target2, String::from("Test Target"));

		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(staker), target1, 1_000));
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(staker), target2, 2_000));

		// max unlock chunks in mock is 4
		for _i in 0..2 {
			assert_ok!(Capacity::unstake(RuntimeOrigin::signed(staker), target1, 50));
			assert_ok!(Capacity::unstake(RuntimeOrigin::signed(staker), target2, 50));
		}
		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(staker), target1, 50),
			Error::<Test>::MaxUnlockingChunksExceeded
		);
	})
}

#[test]
fn unstake_by_a_booster_updates_provider_boost_history_with_correct_amount() {
	new_test_ext().execute_with(|| {
		let staker = 10_000;
		let target1 = 1;
		register_provider(target1, String::from("Test Target"));

		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(staker), target1, 1_000));
		let pbh = Capacity::get_staking_history_for(staker).unwrap();
		assert_eq!(pbh.count(), 1);

		// If unstaking in the next era, this should add a new staking history entry.
		system_run_to_block(9);
		run_to_block(51);
		assert_ok!(Capacity::claim_staking_rewards(RuntimeOrigin::signed(staker)));
		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(staker), target1, 400u64));

		assert_eq!(get_balance::<Test>(&staker), 10_016u64);
		assert_transferable::<Test>(&staker, 16u64);
	})
}

#[test]
fn unstake_all_by_booster_reaps_boost_history() {
	new_test_ext().execute_with(|| {
		let staker = 10_000;
		let target1 = 1;
		register_provider(target1, String::from("Test Target"));

		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(staker), target1, 1_000));
		let pbh = Capacity::get_staking_history_for(staker).unwrap();
		assert_eq!(pbh.count(), 1);

		// If unstaking in the next era, this should add a new staking history entry.
		system_run_to_block(9);
		run_to_block(51);
		assert_ok!(Capacity::claim_staking_rewards(RuntimeOrigin::signed(staker)));
		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(staker), target1, 1_000));
		assert!(Capacity::get_staking_history_for(staker).is_none());
		assert_eq!(get_balance::<Test>(&staker), 10_016u64);
		assert_transferable::<Test>(&staker, 16u64);
	})
}

#[test]
fn unstake_maximum_does_not_change_provider_boost_history() {
	new_test_ext().execute_with(|| {
		let staker = 10_000;
		let target1 = 1;
		register_provider(target1, String::from("Test Target"));

		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker), target1, 1_000));
		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(staker), target1, 500));
		assert!(Capacity::get_staking_history_for(staker).is_none());
	})
}

// Simulate a series of stake/unstake events over 10 eras then check for
// correct staking values, including for eras that do not have an explicit entry.
#[test]
fn get_amount_staked_for_era_works() {
	let mut staking_history: ProviderBoostHistory<Test> = ProviderBoostHistory::new();

	for i in 10u32..=13u32 {
		staking_history.add_era_balance(&i.into(), &5u64);
	}
	assert_eq!(staking_history.get_amount_staked_for_era(&10u32), 5u64);
	assert_eq!(staking_history.get_amount_staked_for_era(&13u32), 20u64);

	staking_history.subtract_era_balance(&14u32, &7u64);
	assert_eq!(staking_history.get_amount_staked_for_era(&14u32), 13u64);
	assert_eq!(staking_history.get_amount_staked_for_era(&15u32), 13u64);

	staking_history.add_era_balance(&15u32, &10u64);

	let expected_balance = 23u64;
	assert_eq!(staking_history.get_amount_staked_for_era(&15u32), expected_balance);

	// unstake everything
	staking_history.subtract_era_balance(&20u32, &expected_balance);

	assert_eq!(staking_history.get_amount_staked_for_era(&16u32), expected_balance);
	assert_eq!(staking_history.get_amount_staked_for_era(&17u32), expected_balance);
	assert_eq!(staking_history.get_amount_staked_for_era(&18u32), expected_balance);
	assert_eq!(staking_history.get_amount_staked_for_era(&19u32), expected_balance);

	// from 20 onward, should return 0.
	assert_eq!(staking_history.get_amount_staked_for_era(&20u32), 0u64);
	assert_eq!(staking_history.get_amount_staked_for_era(&31u32), 0u64);

	// ensure reporting from earlier is still correct.
	assert_eq!(staking_history.get_amount_staked_for_era(&14u32), 13u64);

	// querying for an era that has been cleared due to the hitting the bound
	// (ProviderBoostHistoryLimit = 5 in mock) returns zero.
	assert_eq!(staking_history.get_amount_staked_for_era(&9u32), 0u64);
}

#[test]
fn unstake_fails_if_provider_boosted_and_have_unclaimed_rewards() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let target: MessageSourceId = 10;
		let amount = 1_000u64;

		// staking 1k as of block 1, era 1
		setup_provider(&account, &target, &amount, ProviderBoost);

		// staking 2k as of block 11, era 2
		run_to_block(11);
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account), target, amount));

		//  staking 3k as of era 4, block 31
		run_to_block(31);

		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(account), target, amount),
			Error::<Test>::MustFirstClaimRewards
		);
	})
}

#[test]
fn unstake_all_if_no_unclaimed_rewards_removes_provider_boost_history() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let target: MessageSourceId = 10;
		let amount = 1_000u64;

		// staking 1k as of block 1, era 1
		setup_provider(&account, &target, &amount, ProviderBoost);
		assert!(Capacity::get_staking_history_for(account).is_some());
		run_to_block(10);
		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(account), target, amount));
		assert!(Capacity::get_staking_history_for(account).is_none());
	});
}
