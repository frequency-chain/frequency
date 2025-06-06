use crate::{
	tests::{
		mock::{new_test_ext, Capacity, RuntimeOrigin, Test},
		testing_utils::{capacity_events, register_provider},
	},
	*,
};
use common_primitives::{capacity::StakingType, msa::MessageSourceId};
use frame_support::{assert_noop, assert_ok};

#[test]
fn committed_boost_works() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 200;
		let capacity = 10; // Maximized stake (10% of staked amount) * 50% (in trait impl)
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::committed_boost(RuntimeOrigin::signed(account), target, amount));

		// Check that StakingAccountLedger is updated.
		let boost_account: StakingDetails<Test> =
			StakingAccountLedger::<Test>::get(account).unwrap();

		// Check that the staking account has the correct staking type.
		assert_eq!(boost_account.active, 200);
		assert_eq!(boost_account.staking_type, StakingType::CommittedBoost);

		// Check that the capacity generated is correct. (5% of amount staked, since 10% is what's in the mock)
		let capacity_details = CapacityLedger::<Test>::get(target).unwrap();
		assert_eq!(capacity_details.total_capacity_issued, capacity);

		let events = capacity_events();
		assert_eq!(
			events.first().unwrap(),
			&Event::StakedV2 {
				account,
				target,
				amount,
				capacity,
				staking_type: StakingType::CommittedBoost
			}
		);

		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::CapacityStaking.into(),
				&account
			),
			200u64
		);

		let target_details = StakingTargetLedger::<Test>::get(account, target).unwrap();
		assert_eq!(target_details.amount, amount);
	});
}

#[test]
fn committed_boost_unstake_should_fail_before_pte() {
	new_test_ext().execute_with(|| {
		let account = 600;
		let target: MessageSourceId = 1;
		let amount = 200;
		register_provider(target, String::from("Foo"));
		assert_ok!(Capacity::committed_boost(RuntimeOrigin::signed(account), target, amount));

		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(account), target, amount),
			Error::<Test>::InsufficientUnfrozenStakingBalance
		);
	});
}
