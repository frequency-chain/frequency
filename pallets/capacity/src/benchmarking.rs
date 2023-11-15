use super::*;
use crate::Pallet as Capacity;

use frame_benchmarking::{account, benchmarks, whitelist_account};
use frame_support::{assert_ok, traits::Currency};
use frame_system::RawOrigin;
use parity_scale_codec::alloc::vec::Vec;

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[allow(clippy::expect_used)]
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

// In the benchmarks we expect a new epoch to always start so as to test worst case scenario.
pub fn set_up_epoch<T: Config>(current_block: BlockNumberFor<T>, current_epoch: T::EpochNumber) {
	CurrentEpoch::<T>::set(current_epoch);
	let epoch_start = current_block.saturating_sub(Capacity::<T>::get_epoch_length());
	CurrentEpochInfo::<T>::set(EpochInfo { epoch_start });
}

benchmarks! {
	stake {
		let caller: T::AccountId = create_funded_account::<T>("account", SEED, 105u32);
		let amount: BalanceOf<T> = T::MinimumStakingAmount::get();
		let capacity: BalanceOf<T> = Capacity::<T>::capacity_generated(amount);
		let target = 1;

		register_provider::<T>(target, "Foo");

	}: _ (RawOrigin::Signed(caller.clone()), target, amount)
	verify {
		assert!(StakingAccountLedger::<T>::contains_key(&caller));
		assert!(StakingTargetLedger::<T>::contains_key(&caller, target));
		assert!(CapacityLedger::<T>::contains_key(target));
		assert_last_event::<T>(Event::<T>::Staked {account: caller, amount, target, capacity}.into());
	}

	withdraw_unstaked {
		let caller: T::AccountId = create_funded_account::<T>("account", SEED, 5u32);
		let mut unlocking: UnlockChunks<T> = UnlockChunks::default();
		for _i in 0..T::MaxUnlockingChunks::get() {
			assert_ok!(unlocking.add(1u32.into(), 3u32.into()));
		}
		UnstakeUnlocks::<T>::set(&caller, Some(unlocking));

		CurrentEpoch::<T>::set(T::EpochNumber::from(5u32));

	}: _ (RawOrigin::Signed(caller.clone()))
	verify {
		let total = T::MaxUnlockingChunks::get();
		assert_last_event::<T>(Event::<T>::StakeWithdrawn {account: caller, amount: total.into() }.into());
	}

	on_initialize {
		let current_block: BlockNumberFor<T> = 100_000u32.into();
		let current_epoch: T::EpochNumber = 10_000u32.into();
		set_up_epoch::<T>(current_block, current_epoch);
	}: {
		Capacity::<T>::on_initialize(current_block);
	} verify {
		assert_eq!(current_epoch.saturating_add(1u32.into()), Capacity::<T>::get_current_epoch());
		assert_eq!(current_block, CurrentEpochInfo::<T>::get().epoch_start);
	}
	unstake {
		let caller: T::AccountId = create_funded_account::<T>("account", SEED, 5u32);
		let staking_amount: BalanceOf<T> = T::MinimumStakingAmount::get().saturating_add(20u32.into());
		let unstaking_amount = 9u32;
		let capacity_amount: BalanceOf<T> = Capacity::<T>::capacity_generated(staking_amount);
		let target = 1;
		let block_number = 4u32;

		let mut staking_account = StakingDetails::<T>::default();
		let mut target_details = StakingTargetDetails::<BalanceOf<T>>::default();
		let mut capacity_details = CapacityDetails::<BalanceOf<T>, <T as Config>::EpochNumber>::default();

		staking_account.deposit(staking_amount);
		target_details.deposit(staking_amount, capacity_amount);
		capacity_details.deposit(&staking_amount, &capacity_amount);

		let _ = Capacity::<T>::set_staking_account(&caller.clone(), &staking_account);
		Capacity::<T>::set_target_details_for(&caller.clone(), target, target_details);
		Capacity::<T>::set_capacity_for(target, capacity_details);

		// fill up unlock chunks to max bound - 1
		let mut unlocking: UnlockChunks<T> = UnlockChunks::default();
		let count = T::MaxUnlockingChunks::get()-1;
		for _i in 0..count {
			assert_ok!(unlocking.add(1u32.into(), 3u32.into()));
		}
		UnstakeUnlocks::<T>::set(&caller, Some(unlocking));


	}: _ (RawOrigin::Signed(caller.clone()), target, unstaking_amount.into())
	verify {
		assert_last_event::<T>(Event::<T>::UnStaked {account: caller, target: target, amount: unstaking_amount.into(), capacity: Capacity::<T>::calculate_capacity_reduction(unstaking_amount.into(), staking_amount, capacity_amount) }.into());
	}

	set_epoch_length {
		let epoch_length: BlockNumberFor<T> = 9u32.into();
	}: _ (RawOrigin::Root, epoch_length)
	verify {
		assert_eq!(Capacity::<T>::get_epoch_length(), 9u32.into());
		assert_last_event::<T>(Event::<T>::EpochLengthUpdated {blocks: epoch_length}.into());
	}

	impl_benchmark_test_suite!(Capacity,
		crate::tests::mock::new_test_ext(),
		crate::tests::mock::Test);
}
