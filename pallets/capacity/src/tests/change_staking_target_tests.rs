use super::{mock::*, testing_utils::*};
use crate::{
	BalanceOf, CapacityDetails, Config, Error, Event, StakingAccountDetails, StakingTargetDetails,
};
use common_primitives::{
	capacity::{
		Nontransferable, StakingType,
		StakingType::{MaximumCapacity, ProviderBoost},
	},
	msa::MessageSourceId,
};
use frame_support::{assert_noop, assert_ok, traits::WithdrawReasons};
use sp_runtime::ArithmeticError;

fn setup_provider(staker: u64, target: MessageSourceId, amount: u64) {
	let provider_name = String::from("Cst-") + target.to_string().as_str();
	register_provider(target, provider_name);
	if amount > 0 {
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker), target, amount, ProviderBoost));
	}
}

#[test]
fn test_change_staking_target_parametric_validity() {
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
				Some(0)
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
				Some(0)
			),
			Error::<Test>::StakingAmountBelowMinimum
		);

		assert_noop!(
			Capacity::change_staking_target(
				RuntimeOrigin::signed(account),
				from_target,
				to_target,
				None
			),
			Error::<Test>::InvalidTarget
		);
		setup_provider(account, to_target, 0);

		assert_ok!(Capacity::change_staking_target(
			RuntimeOrigin::signed(account),
			from_target,
			to_target,
			None
		));
	});
}
