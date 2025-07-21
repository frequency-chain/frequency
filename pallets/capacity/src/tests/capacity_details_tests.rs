use super::mock::*;
use crate::*;

#[test]
fn impl_staking_capacity_details_deposit() {
	let mut capacity_details =
		CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber>::default();

	assert_eq!(capacity_details.deposit(&10, &1), Some(()));

	assert_eq!(
		capacity_details,
		CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
			total_tokens_staked: 10u64,
			remaining_capacity: 1u64,
			total_capacity_issued: 1u64,
			last_replenished_epoch: 0u32
		}
	)
}
