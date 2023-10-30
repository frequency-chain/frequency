use super::{mock::*, testing_utils::*};
use crate::{
	BoostingAccountDetails, Error, Event, RewardPoolInfo, StakingAccountDetails, StakingHistory,
};
use common_primitives::{capacity::StakingType::ProviderBoost, msa::MessageSourceId};
use frame_support::{
	assert_noop, assert_ok,
	traits::{Len, WithdrawReasons},
};

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
		let boost_account: BoostingAccountDetails<Test> =
			Capacity::get_boost_details_for(account).unwrap();

		assert_eq!(boost_account.staking_details.total, 200);
		assert_eq!(boost_account.staking_details.active, 200);
		// assert_eq!(boost_account.unlocking.len(), 0);
		// assert_eq!(staking_account.last_rewards_claimed_at, None);
		// assert_eq!(staking_account.stake_change_unlocking.len(), 0);

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
fn provider_boost_updates_boost_account_details() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 500;
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account), target, amount));
		let boost_details: BoostingAccountDetails<Test> =
			Capacity::get_boost_details_for(account).unwrap();
		assert_eq!(boost_details.staking_details.active, 500);
		assert_eq!(boost_details.staking_details.total, 500);
		assert!(boost_details.last_rewards_claimed_at.is_none());
		assert_eq!(boost_details.boost_history.len(), 1);

		let expected_history = StakingHistory { reward_era: 0, total_staked: 500 };
		let actual_history = boost_details.boost_history.get(0).unwrap();
		assert_eq!(actual_history, &expected_history);
	})
}

#[test]
fn provider_boost_adjusts_reward_pool_total() {
	new_test_ext().execute_with(|| {
		// TODO: when work resumes on reward pool branch
		// let account = 600;
		// let target: MessageSourceId = 1;
		// let amount = 500;
		// register_provider(target, String::from("Foo"));
		// assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account), target, amount));
		//
		// let reward_pool_info = Capacity::get_reward_pool_for_era(0).unwrap();
		// assert_eq!(reward_pool_info, RewardPoolInfo {
		// 	total_staked_token: 500,
		// 	total_reward_pool: 50,
		// 	unclaimed_balance: 50,
		// });
		assert!(true);
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
