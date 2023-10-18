use super::{mock::*, testing_utils::*};
use crate::{BalanceOf, CapacityDetails, Error, Event, StakingAccountDetails};
use common_primitives::{
	capacity::{
		Nontransferable, StakingType,
		StakingType::{MaximumCapacity, ProviderBoost},
	},
	msa::MessageSourceId,
};
use frame_support::{assert_noop, assert_ok, traits::WithdrawReasons};

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

fn provider_boost_updates_staking_account_history() {
	new_test_ext().execute_with(|| {
		assert!(false);
	})
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
