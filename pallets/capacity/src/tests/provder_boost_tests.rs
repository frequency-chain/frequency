use super::{mock::*, testing_utils::*};
use crate::{Error, Event, RewardPoolInfo, StakingAccountDetails, StakingHistory, StakingRewardPool};
use common_primitives::{
	capacity::{StakingType::{ProviderBoost}, },
	msa::MessageSourceId,
};
use frame_support::{assert_noop, assert_ok, traits::WithdrawReasons};
use frame_support::traits::Len;

#[test]
fn provider_boost_works() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 200;
		let capacity = 1;
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account), target, amount));

		// Check that StakingAccountLedger is updated.
		let staking_account: StakingAccountDetails<Test> =
			Capacity::get_staking_account_for(account).unwrap();

		assert_eq!(staking_account.total, 200);
		assert_eq!(staking_account.active, 200);
		assert_eq!(staking_account.unlocking.len(), 0);
		assert_eq!(staking_account.last_rewards_claimed_at, None);
		assert_eq!(staking_account.stake_change_unlocking.len(), 0);

		let events = staking_events();
		assert_eq!(
			events.first().unwrap(),
			&Event::ProviderBoosted { account, target, amount, capacity }
		);

		assert_eq!(Balances::locks(&account)[0].amount, amount);
		assert_eq!(Balances::locks(&account)[0].reasons, WithdrawReasons::all().into());

		let target_details = Capacity::get_target_for(account, target).unwrap();
		assert_eq!(target_details.amount, amount);
		assert_eq!(target_details.staking_type, ProviderBoost);
	});
}

#[test]
fn provider_boost_updates_staking_account_details() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 500;
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account), target, amount));
		// TODO: BoostAccountDetails
		let staking_account_details: StakingAccountDetails<Test> = Capacity::get_staking_account_for(account).unwrap();
		assert_eq!(staking_account_details.active, 500);
		assert_eq!(staking_account_details.total, 500);
		assert!(staking_account_details.unlocking.is_empty());
		assert!(staking_account_details.last_rewards_claimed_at.is_none());
		assert_eq!(staking_account_details.boost_history.len(), 1);

		let expected_history = StakingHistory { reward_era: 0,total_staked: 500 };
		let actual_history = staking_account_details.boost_history.get(1).unwrap();
		assert_eq!(actual_history, &expected_history);
	})
}

#[test]
fn provider_boost_adjusts_reward_pool_total() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 500;
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account), target, amount));

		let reward_pool_info = Capacity::get_reward_pool_for_era(0).unwrap();
		assert_eq!(reward_pool_info, RewardPoolInfo {
			total_staked_token: 500,
			total_reward_pool: 50,
			unclaimed_balance: 50,
		});
	});
}

#[test]
fn account_can_stake_different_types_for_different_targets() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target1: MessageSourceId = 1;
		let target2: MessageSourceId = 2;
		let amount1 = 200;
		let amount2 = 100;
		let reward_era = 0;
		register_provider(target1, String::from("Foo"));
		register_provider(target2, String::from("Bar"));

		StakingRewardPool::<Test>::insert(reward_era, RewardPoolInfo::default());

		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account), target1, amount1));
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account), target2, amount2));

		// let reward_pool_info = Capacity::get_reward_pool_for_era(1).unwrap();
		// assert_eq!(reward_pool_info, RewardPoolInfo {
		// 	total_staked_token: 200,
		// 	total_reward_pool: 20,
		// 	unclaimed_balance: 20,
		// });

		// TODO: BoostAccountDetails
		let staking_account_details: StakingAccountDetails<Test> = Capacity::get_staking_account_for(account).unwrap();
		// assert_eq!(staking_account_details.active, 100);
		assert_eq!(staking_account_details.active, 200);
		assert_eq!(staking_account_details.total, 300);
		assert!(staking_account_details.unlocking.is_empty());
		assert!(staking_account_details.last_rewards_claimed_at.is_none());
		assert_eq!(staking_account_details.boost_history.len(), 1);

		let expected_history = StakingHistory { reward_era, total_staked: 200 };
		let actual_history = staking_account_details.boost_history.get(0).unwrap();
		assert_eq!(actual_history, &expected_history);
	});
}

#[test]
fn calling_stake_on_provider_boost_target_errors() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 200;
		register_provider(target, String::from("Bear"));
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account), target, amount));
		assert_noop!(
			Capacity::stake(RuntimeOrigin::signed(account), target, 50),
			Error::<Test>::CannotChangeStakingType
		);
	})
}
#[test]
fn calling_provider_boost_on_staked_target_errors() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 200;
		register_provider(target, String::from("Foobear"));
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(account), target, amount));
		assert_noop!(
			Capacity::provider_boost(RuntimeOrigin::signed(account), target, 50),
			Error::<Test>::CannotChangeStakingType
		);
	})
}
