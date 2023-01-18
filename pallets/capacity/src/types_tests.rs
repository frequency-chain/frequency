use super::*;
use crate::mock::*;

#[test]
fn staking_account_details_reap_thawed_happy_path() {
	let mut staking_account = StakingAccountDetails::<Test>::default();
	staking_account.increase_by(10);

	let new_unlocks: Vec<(u32, u32)> = vec![(1u32, 2u32), (2u32, 3u32), (3u32, 4u32)];
	assert_eq!(true, staking_account.set_unlock_chunks(&new_unlocks));
	assert_eq!(10, staking_account.total);
	assert_eq!(3, staking_account.unlocking.len());

	assert_eq!(1u64, staking_account.reap_thawed(2));
	assert_eq!(2, staking_account.unlocking.len());
	assert_eq!(9, staking_account.total);

	assert_eq!(5u64, staking_account.reap_thawed(5));
	assert_eq!(0, staking_account.unlocking.len());
	assert_eq!(4, staking_account.total);
}
