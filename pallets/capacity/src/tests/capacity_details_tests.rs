use super::mock::*;
use crate::*;

#[test]
fn impl_staking_capacity_details_deposit() {
	let mut capacity_details =
		CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber>::default();

	assert_eq!(capacity_details.deposit(&10), Some(()));

	assert_eq!(
		capacity_details,
		CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
			remaining_capacity: BalanceOf::<Test>::from(10u64),
			total_tokens_staked: BalanceOf::<Test>::from(10u64),
			total_capacity_issued: BalanceOf::<Test>::from(10u64),
			last_replenished_epoch: <Test as Config>::EpochNumber::from(0u32)
		}
	)
}
