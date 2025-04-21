use super::mock::*;
use crate::*;
#[test]
fn staking_account_details_withdraw_reduces_active_staking_balance() {
	let mut staking_account_details =
		StakingDetails::<Test> { active: 15u64, staking_type: StakingType::MaximumCapacity };
	assert_eq!(Ok(3u64), staking_account_details.withdraw(3));

	assert_eq!(
		staking_account_details,
		StakingDetails::<Test> { active: 12u64, staking_type: StakingType::MaximumCapacity }
	)
}

#[test]
fn staking_account_details_withdraw_goes_to_zero_when_result_below_minimum() {
	let mut staking_account_details =
		StakingDetails::<Test> { active: 10u64, staking_type: StakingType::MaximumCapacity };
	assert_eq!(Ok(10u64), staking_account_details.withdraw(6));
	assert_eq!(0u64, staking_account_details.active);

	staking_account_details.deposit(10);
	assert_eq!(Ok(10u64), staking_account_details.withdraw(9));
	assert_eq!(0u64, staking_account_details.active);

	staking_account_details.deposit(10);
	assert_eq!(Ok(10u64), staking_account_details.withdraw(11));
	assert_eq!(0u64, staking_account_details.active);
}

#[test]
fn impl_staking_account_details_increase_by() {
	let mut staking_account = StakingDetails::<Test>::default();
	assert_eq!(staking_account.deposit(10), Some(()));

	assert_eq!(
		staking_account,
		StakingDetails::<Test> { active: 10u64, staking_type: StakingType::MaximumCapacity }
	)
}

#[test]
fn impl_staking_account_details_default() {
	assert_eq!(
		StakingDetails::<Test>::default(),
		StakingDetails::<Test> {
			active: BalanceOf::<Test>::zero(),
			staking_type: StakingType::MaximumCapacity,
		},
	);
}
