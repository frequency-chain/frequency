use super::*;
use crate::Pallet as Capacity;
use frame_system::Pallet as System;

use frame_benchmarking::{account, benchmarks, whitelist_account, Vec};
use frame_support::{assert_ok, traits::Currency};
use frame_system::RawOrigin;

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

pub fn register_provider<T: Config>(target_id: MessageSourceId, name: &'static str) {
	let name = Vec::from(name).try_into().expect("error");
	assert_ok!(T::BenchmarkHelper::create(target_id, name));
}
pub fn create_funded_account<T: Config>(
	string: &'static str,
	n: u32,
	balance_factor: u32,
) -> T::AccountId {
	let user = account(string, n, SEED);
	whitelist_account!(user);
	let balance = T::Currency::minimum_balance() * balance_factor.into();
	let _ = T::Currency::make_free_balance_be(&user, balance);
	assert_eq!(T::Currency::free_balance(&user), balance.into());
	user
}

benchmarks! {
	stake {
		let caller: T::AccountId = create_funded_account::<T>("account", SEED, 5u32);
		let amount: BalanceOf<T> = T::MinimumStakingAmount::get();
		let target = 1;

		register_provider::<T>(target, "Foo");

	}: _ (RawOrigin::Signed(caller.clone()), target, amount)
	verify {
		assert!(StakingAccountLedger::<T>::contains_key(&caller));
		assert!(StakingTargetLedger::<T>::contains_key(&caller, target));
		assert!(CapacityLedger::<T>::contains_key(target));
		assert_last_event::<T>(Event::<T>::Staked {account: caller, amount, target }.into());
	}

	withdraw_unstaked {
		let caller: T::AccountId = create_funded_account::<T>("account", SEED, 5u32);
		let amount: BalanceOf<T> = T::MinimumStakingAmount::get();
		let block_number = 4u32;

		let mut staking_account = StakingAccountDetails::<T>::default();
		staking_account.increase_by(10u32.into());

		// set new unlock chunks using tuples of (value, thaw_at)
		let new_unlocks: Vec<(u32, u32)> = Vec::from([(50u32, 3u32), (50u32, block_number)]);
		assert_eq!(true, staking_account.set_unlock_chunks(&new_unlocks));

		Capacity::<T>::set_staking_account(&caller.clone(), &staking_account);
		System::<T>::set_block_number(block_number.into());

	}: _ (RawOrigin::Signed(caller.clone()))
	verify {
		assert_last_event::<T>(Event::<T>::StakeWithdrawn {account: caller, amount: 100u32.into() }.into());
	}

	impl_benchmark_test_suite!(Capacity,
		crate::mock::new_test_ext(),
		crate::mock::Test);
}
