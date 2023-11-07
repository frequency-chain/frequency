use super::mock::*;
use crate::*;
#[test]
fn staking_account_details_withdraw_reduces_active_staking_balance() {
	let mut staking_account_details = StakingAccountDetailsV2::<Test> {
		active: BalanceOf::<Test>::from(15u64),
		staking_type: StakingType::MaximumCapacity,
	};
	assert_eq!(Ok(3u64), staking_account_details.withdraw(3));

	assert_eq!(
		staking_account_details,
		StakingAccountDetailsV2::<Test> {
			active: BalanceOf::<Test>::from(12u64),
			staking_type: StakingType::MaximumCapacity,
		}
	)
}

#[test]
fn staking_account_details_withdraw_goes_to_zero_when_result_below_minimum() {
	let mut staking_account_details = StakingAccountDetailsV2::<Test> {
		active: BalanceOf::<Test>::from(10u64),
		staking_type: StakingType::MaximumCapacity,
	};
	assert_eq!(Ok(10u64), staking_account_details.withdraw(6));
	assert_eq!(0u64, staking_account_details.active);
	assert_eq!(10u64, staking_account_details.total);

	staking_account_details.deposit(10);
	assert_eq!(Ok(10u64), staking_account_details.withdraw(9, 3));
	assert_eq!(0u64, staking_account_details.active);

	staking_account_details.deposit(10);
	assert_eq!(Ok(10u64), staking_account_details.withdraw(11, 3));
	assert_eq!(0u64, staking_account_details.active);
}

#[test]
fn impl_staking_account_details_increase_by() {
	let mut staking_account = StakingAccountDetailsV2::<Test>::default();
	assert_eq!(staking_account.deposit(10), Some(()));

	assert_eq!(
		staking_account,
		StakingAccountDetailsV2::<Test> {
			active: BalanceOf::<Test>::from(10u64),
			staking_type: StakingType::MaximumCapacity,
		}
	)
}

#[test]
fn impl_staking_account_details_default() {
	assert_eq!(
		StakingAccountDetailsV2::<Test>::default(),
		StakingAccountDetailsV2::<Test> {
			active: BalanceOf::<Test>::zero(),
			staking_type: StakingType::MaximumCapacity,
		},
	);
}

