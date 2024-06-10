use super::{mock::*, testing_utils::*};
use crate::{
	Config, CurrentEraProviderBoostTotal, Error, Event, FreezeReason, StakingDetails, StakingType,
};
use common_primitives::msa::MessageSourceId;
use frame_support::{assert_noop, assert_ok, traits::fungible::InspectFreeze};

#[test]
fn provider_boost_works() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 200;
		let capacity = 10; // Maximized stake (10% of staked amount) * 50% (in trait impl)
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account), target, amount));

		// Check that StakingAccountLedger is updated.
		let boost_account: StakingDetails<Test> =
			Capacity::get_staking_account_for(account).unwrap();

		// Check that the staking account has the correct staking type.
		assert_eq!(boost_account.active, 200);
		assert_eq!(boost_account.staking_type, StakingType::ProviderBoost);

		// Check that the capacity generated is correct. (5% of amount staked, since 10% is what's in the mock)
		let capacity_details = Capacity::get_capacity_for(target).unwrap();
		assert_eq!(capacity_details.total_capacity_issued, capacity);

		let events = staking_events();
		assert_eq!(
			events.first().unwrap(),
			&Event::ProviderBoosted { account, target, amount, capacity }
		);

		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::CapacityStaking.into(),
				&account
			),
			200u64
		);

		let target_details = Capacity::get_target_for(account, target).unwrap();
		assert_eq!(target_details.amount, amount);
	});
}

#[test]
fn provider_boost_updates_reward_pool_history() {
	// two accounts staking to the same target
	new_test_ext().execute_with(|| {
		let account1 = 600;
		let account2 = 500;
		let target: MessageSourceId = 1;
		let amount1 = 500;
		let amount2 = 200;
		register_provider(target, String::from("Foo"));

		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account1), target, amount1));
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account2), target, amount2));
		assert_eq!(CurrentEraProviderBoostTotal::<Test>::get(), 700u64);
	});
}

#[test]
fn provider_boost_updates_staking_details() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 500;
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account), target, amount));
		let boost_details: StakingDetails<Test> =
			Capacity::get_staking_account_for(account).unwrap();
		assert_eq!(boost_details.active, 500);
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
