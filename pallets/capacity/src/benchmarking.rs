use super::*;
use crate::Pallet as Capacity;

use frame_benchmarking::{account, benchmarks, whitelist_account, Vec};
use frame_support::{assert_ok, traits::Currency};
use frame_system::RawOrigin;

const SEED: u32 = 0;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

#[allow(clippy::expect_used)]
pub fn register_provider<T: Config>(target_id: MessageSourceId, name: &'static str) {
	let name = Vec::from(name).try_into().expect("error");
	assert_ok!(T::BenchmarkHelper::create(target_id, name));
}

#[allow(clippy::expect_used)]
pub fn setup_provider_stake<T: Config>(
	staker: &T::AccountId,
	target_id: &MessageSourceId,
	amount: u32,
) {
	let staking_amount: BalanceOf<T> = T::MinimumStakingAmount::get().saturating_add(amount.into());
	let capacity_amount: BalanceOf<T> = Capacity::<T>::capacity_generated(staking_amount);

	// will add the amount to the account if it exists.
	let mut staking_account: StakingAccountDetails<T> =
		Capacity::get_staking_account_for(staker).unwrap_or_default();

	let mut target_details = StakingTargetDetails::<T>::default();
	let mut capacity_details =
		CapacityDetails::<BalanceOf<T>, <T as Config>::EpochNumber>::default();

	staking_account.deposit(staking_amount);
	target_details.deposit(staking_amount, capacity_amount);
	capacity_details.deposit(&staking_amount, &capacity_amount);

	Capacity::<T>::set_staking_account(staker, &staking_account);
	Capacity::<T>::set_target_details_for(staker, target_id, &target_details);
	Capacity::<T>::set_capacity_for(target_id, &capacity_details);
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
pub fn set_up_epoch<T: Config>(current_block: T::BlockNumber, current_epoch: T::EpochNumber) {
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
		let staking_type = StakingType::MaximumCapacity;

		register_provider::<T>(target, "Foo");

	}: _ (RawOrigin::Signed(caller.clone()), target, amount, staking_type)
	verify {
		assert!(StakingAccountLedger::<T>::contains_key(&caller));
		assert!(StakingTargetLedger::<T>::contains_key(&caller, target));
		assert!(CapacityLedger::<T>::contains_key(target));
		assert_last_event::<T>(Event::<T>::Staked {account: caller, amount, target, capacity}.into());
	}

	withdraw_unstaked {
		let caller: T::AccountId = create_funded_account::<T>("account", SEED, 5u32);
		let amount: BalanceOf<T> = T::MinimumStakingAmount::get();

		let mut staking_account = StakingAccountDetails::<T>::default();
		staking_account.deposit(500u32.into());

		// set new unlock chunks using tuples of (value, thaw_at)
		let new_unlocks: Vec<(u32, u32)> = Vec::from([(50u32, 3u32), (50u32, 5u32)]);
		assert_eq!(true, staking_account.set_unlock_chunks(&new_unlocks));

		Capacity::<T>::set_staking_account(&caller.clone(), &staking_account);
		CurrentEpoch::<T>::set(T::EpochNumber::from(5u32));

	}: _ (RawOrigin::Signed(caller.clone()))
	verify {
		assert_last_event::<T>(Event::<T>::StakeWithdrawn {account: caller, amount: 100u32.into() }.into());
	}

	on_initialize {
		let current_block: T::BlockNumber = 100_000u32.into();
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

		let mut staking_account = StakingAccountDetails::<T>::default();
		let mut target_details = StakingTargetDetails::<T>::default();
		let mut capacity_details = CapacityDetails::<BalanceOf<T>, <T as Config>::EpochNumber>::default();

		staking_account.deposit(staking_amount);
		target_details.deposit(staking_amount, capacity_amount);
		capacity_details.deposit(&staking_amount, &capacity_amount);

		Capacity::<T>::set_staking_account(&caller.clone(), &staking_account);
		Capacity::<T>::set_target_details_for(&caller.clone(), &target, &target_details);
		Capacity::<T>::set_capacity_for(&target, &capacity_details);

	}: _ (RawOrigin::Signed(caller.clone()), target, unstaking_amount.into())
	verify {
		assert_last_event::<T>(Event::<T>::UnStaked {account: caller, target: target, amount: unstaking_amount.into(), capacity: Capacity::<T>::calculate_capacity_reduction(unstaking_amount.into(), staking_amount, capacity_amount) }.into());
	}

	set_epoch_length {
		let epoch_length: T::BlockNumber = 9u32.into();
	}: _ (RawOrigin::Root, epoch_length)
	verify {
		assert_eq!(Capacity::<T>::get_epoch_length(), 9u32.into());
		assert_last_event::<T>(Event::<T>::EpochLengthUpdated {blocks: epoch_length}.into());
	}

	change_staking_target {
		let caller: T::AccountId = create_funded_account::<T>("account", SEED, 5u32);
		let from_msa = 33;
		let to_msa = 34;
		// amount in addition to minimum
		let from_msa_amount = 32u32;
		let to_msa_amount = 1u32;

		register_provider::<T>(from_msa, "frommsa");
		register_provider::<T>(to_msa, "tomsa");
		setup_provider_stake::<T>(&caller, &from_msa, from_msa_amount);
		setup_provider_stake::<T>(&caller, &to_msa, to_msa_amount);
		let restake_amount = 11u32;

	}: _ (RawOrigin::Signed(caller.clone(), ), from_msa, to_msa, restake_amount.into())
	verify {
		assert_last_event::<T>(Event::<T>::StakingTargetChanged {
			account: caller,
			from_msa,
			to_msa,
			amount: restake_amount.into()
		}.into());
	}

	impl_benchmark_test_suite!(Capacity,
		crate::tests::mock::new_test_ext(),
		crate::tests::mock::Test);
}
