use super::*;
use crate::Pallet as Capacity;

use crate::StakingType::*;
use frame_benchmarking::{account, benchmarks, whitelist_account};
use frame_support::{assert_ok, BoundedVec};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use parity_scale_codec::alloc::vec::Vec;

const SEED: u32 = 0;
const REWARD_POOL_TOTAL: u32 = 2_000_000;

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
	T::Currency::set_balance(&user, balance);
	assert_eq!(T::Currency::balance(&user), balance.into());
	user
}

// In the benchmarks we expect a new epoch to always start so as to test worst case scenario.
pub fn set_up_epoch<T: Config>(current_block: BlockNumberFor<T>, current_epoch: T::EpochNumber) {
	CurrentEpoch::<T>::set(current_epoch);
	let epoch_start = current_block.saturating_sub(EpochLength::<T>::get());
	CurrentEpochInfo::<T>::set(EpochInfo { epoch_start });
}

pub fn set_era_and_reward_pool_at_block<T: Config>(
	era_index: RewardEra,
	started_at: BlockNumberFor<T>,
	total_staked_token: BalanceOf<T>,
) {
	let era_info: RewardEraInfo<RewardEra, BlockNumberFor<T>> =
		RewardEraInfo { era_index, started_at };
	CurrentEraInfo::<T>::set(era_info);
	CurrentEraProviderBoostTotal::<T>::set(total_staked_token)
}

// caller stakes the given amount to the given target
pub fn setup_provider_stake<T: Config>(
	caller: &T::AccountId,
	target: &MessageSourceId,
	staking_amount: BalanceOf<T>,
	is_provider_boost: bool,
) {
	let capacity_amount: BalanceOf<T> = Capacity::<T>::capacity_generated(staking_amount);

	let mut staking_account = StakingDetails::<T>::default();
	if is_provider_boost {
		staking_account.staking_type = ProviderBoost;
	}
	let mut target_details = StakingTargetDetails::<BalanceOf<T>>::default();
	let mut capacity_details =
		CapacityDetails::<BalanceOf<T>, <T as Config>::EpochNumber>::default();

	staking_account.deposit(staking_amount);
	target_details.deposit(staking_amount, capacity_amount);
	capacity_details.deposit(&staking_amount, &capacity_amount);

	Capacity::<T>::set_staking_account_and_lock(caller, &staking_account)
		.expect("Failed to set staking account");
	Capacity::<T>::set_target_details_for(caller, *target, target_details);
	Capacity::<T>::set_capacity_for(*target, capacity_details);
}

// fill up unlock chunks to max bound - 1
fn fill_unlock_chunks<T: Config>(caller: &T::AccountId, count: u32) {
	let mut unlocking: UnlockChunkList<T> = BoundedVec::default();
	for _i in 0..count {
		let unlock_chunk: UnlockChunk<BalanceOf<T>, T::EpochNumber> =
			UnlockChunk { value: 1u32.into(), thaw_at: 3u32.into() };
		assert_ok!(unlocking.try_push(unlock_chunk));
	}
	UnstakeUnlocks::<T>::set(caller, Some(unlocking));
}

fn fill_reward_pool_chunks<T: Config>(current_era: RewardEra) {
	let history_limit: RewardEra = <T as Config>::ProviderBoostHistoryLimit::get();
	let starting_era: RewardEra = current_era - history_limit - 1u32;
	for era in starting_era..current_era {
		Capacity::<T>::update_provider_boost_reward_pool(era, REWARD_POOL_TOTAL.into());
	}
}

fn fill_boost_history<T: Config>(
	caller: &T::AccountId,
	amount: BalanceOf<T>,
	current_era: RewardEra,
) {
	let max_history: RewardEra = <T as Config>::ProviderBoostHistoryLimit::get().into();
	let starting_era = current_era - max_history - 1u32;
	for i in starting_era..current_era {
		assert_ok!(Capacity::<T>::upsert_boost_history(caller.into(), i, amount, true));
	}
}

fn unclaimed_rewards_total<T: Config>(caller: &T::AccountId) -> BalanceOf<T> {
	let zero_balance: BalanceOf<T> = 0u32.into();
	let rewards: Vec<UnclaimedRewardInfo<BalanceOf<T>, BlockNumberFor<T>>> =
		Capacity::<T>::list_unclaimed_rewards(caller).unwrap_or_default().to_vec();
	rewards
		.iter()
		.fold(zero_balance, |acc, reward_info| acc.saturating_add(reward_info.earned_amount))
		.into()
}

benchmarks! {
	stake {
		let caller: T::AccountId = create_funded_account::<T>("account", SEED, 105u32);
		let amount: BalanceOf<T> = T::MinimumStakingAmount::get();
		let capacity: BalanceOf<T> = Capacity::<T>::capacity_generated(amount);
		let target = 1;
		let staking_type = MaximumCapacity;

		set_era_and_reward_pool_at_block::<T>(1u32.into(), 1u32.into(), 1_000u32.into());
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
		fill_unlock_chunks::<T>(&caller, T::MaxUnlockingChunks::get());

		CurrentEpoch::<T>::set(T::EpochNumber::from(5u32));

	}: _ (RawOrigin::Signed(caller.clone()))
	verify {
		let total = T::MaxUnlockingChunks::get();
		assert_last_event::<T>(Event::<T>::StakeWithdrawn {account: caller, amount: total.into() }.into());
	}

	start_new_epoch_if_needed {
		let current_block: BlockNumberFor<T> = 100_000u32.into();
		let current_epoch: T::EpochNumber = 10_000u32.into();
		set_up_epoch::<T>(current_block, current_epoch);
	}:  {
		Capacity::<T>::start_new_epoch_if_needed(current_block)
	} verify {
		assert_eq!(current_epoch.saturating_add(1u32.into()), CurrentEpoch::<T>::get());
		assert_eq!(current_block, CurrentEpochInfo::<T>::get().epoch_start);
	}

	start_new_reward_era_if_needed {
		let current_block: BlockNumberFor<T> = 1_209_600u32.into();
		let history_limit: u32 = <T as Config>::ProviderBoostHistoryLimit::get();
		let total_reward_pool: BalanceOf<T> = <T as Config>::RewardPoolEachEra::get();
		let unclaimed_balance: BalanceOf<T> = 5_000u32.into();
		let total_staked_token: BalanceOf<T> = 5_000u32.into();
		let started_at: BlockNumberFor<T> = current_block.saturating_sub(<T as Config>::EraLength::get().into());

		let current_era: RewardEra = (history_limit + 1u32).into();
		CurrentEraInfo::<T>::set(RewardEraInfo{ era_index: current_era, started_at });
		fill_reward_pool_chunks::<T>(current_era);
	}: {
		Capacity::<T>::start_new_reward_era_if_needed(current_block);
	} verify {
		let new_era_info = CurrentEraInfo::<T>::get();
		assert_eq!(current_era.saturating_add(1u32.into()), new_era_info.era_index);
		assert_eq!(current_block, new_era_info.started_at);
	}
	unstake {
		let caller: T::AccountId = create_funded_account::<T>("account", SEED, 5u32);
		let staking_amount: BalanceOf<T> = T::MinimumStakingAmount::get().saturating_add(20u32.into());
		let unstaking_amount = 9u32;
		let capacity_amount: BalanceOf<T> = Capacity::<T>::capacity_generated(staking_amount);
		let target = 1;
		let block_number = 4u32;

		// Adds a boost history entry for this era only so unstake succeeds and there is an update
		// to provider boost history.
		let mut pbh: ProviderBoostHistory<T> = ProviderBoostHistory::new();
		pbh.add_era_balance(&1u32.into(), &staking_amount);
		ProviderBoostHistories::<T>::set(caller.clone(), Some(pbh));
		set_era_and_reward_pool_at_block::<T>(1u32.into(), 1u32.into(), 1_000u32.into());

		setup_provider_stake::<T>(&caller, &target, staking_amount, true);

		fill_unlock_chunks::<T>(&caller, T::MaxUnlockingChunks::get() - 1);
	}: _ (RawOrigin::Signed(caller.clone()), target, unstaking_amount.into())
	verify {
		assert_last_event::<T>(Event::<T>::UnStaked {
			account: caller.clone(),
			target,
			amount: unstaking_amount.into(),
			capacity: Capacity::<T>::calculate_capacity_reduction(unstaking_amount.into(), staking_amount,capacity_amount)
		}.into());
	}

	set_epoch_length {
		let epoch_length: BlockNumberFor<T> = 9u32.into();
	}: _ (RawOrigin::Root, epoch_length)
	verify {
		assert_eq!(EpochLength::<T>::get(), 9u32.into());
		assert_last_event::<T>(Event::<T>::EpochLengthUpdated {blocks: epoch_length}.into());
	}

	change_staking_target {
		let caller: T::AccountId = create_funded_account::<T>("account", SEED, 5u32);
		let from_msa = 33;
		let to_msa = 34;
		// amount in addition to minimum
		let from_msa_amount: BalanceOf<T> = T::MinimumStakingAmount::get().saturating_add(31u32.into());
		let to_msa_amount: BalanceOf<T> = T::MinimumStakingAmount::get().saturating_add(1u32.into());

		set_era_and_reward_pool_at_block::<T>(1u32.into(), 1u32.into(), 1_000u32.into());
		register_provider::<T>(from_msa, "frommsa");
		register_provider::<T>(to_msa, "tomsa");
		setup_provider_stake::<T>(&caller, &from_msa, from_msa_amount, false);
		setup_provider_stake::<T>(&caller, &to_msa, to_msa_amount, false);
		let restake_amount: BalanceOf<T> = from_msa_amount.saturating_sub(10u32.into());

	}: _ (RawOrigin::Signed(caller.clone(), ), from_msa, to_msa, restake_amount)
	verify {
		assert_last_event::<T>(Event::<T>::StakingTargetChanged {
			account: caller,
			from_msa,
			to_msa,
			amount: restake_amount.into()
		}.into());
	}

	provider_boost {
		let caller: T::AccountId = create_funded_account::<T>("boostaccount", SEED, 260u32);
		let boost_amount: BalanceOf<T> = T::MinimumStakingAmount::get().saturating_add(1u32.into());
		let capacity: BalanceOf<T> = Capacity::<T>::capacity_generated(<T>::RewardsProvider::capacity_boost(boost_amount));
		let target = 1;

		set_era_and_reward_pool_at_block::<T>(1u32.into(), 1u32.into(), 1_000u32.into());
		register_provider::<T>(target, "Foo");

	}: _ (RawOrigin::Signed(caller.clone()), target, boost_amount)
	verify {
		assert_last_event::<T>(Event::<T>::ProviderBoosted {account: caller, amount: boost_amount, target, capacity}.into());
	}

	// TODO: vary the boost_history to get better weight estimates.
	claim_staking_rewards {
		let caller: T::AccountId = create_funded_account::<T>("account", SEED, 5u32);
		let from_msa = 33;
		let boost_amount: BalanceOf<T> = T::MinimumStakingAmount::get();
		setup_provider_stake::<T>(&caller, &from_msa, boost_amount, false);
		frame_system::Pallet::<T>::set_block_number(1002u32.into());
		let current_era: RewardEra = 100;
		set_era_and_reward_pool_at_block::<T>(current_era, 1001u32.into(), REWARD_POOL_TOTAL.into());
		fill_reward_pool_chunks::<T>(current_era);
		fill_boost_history::<T>(&caller, 100u32.into(), current_era);
		let unclaimed_rewards = unclaimed_rewards_total::<T>(&caller);
	}: _ (RawOrigin::Signed(caller.clone()))
	verify {
		assert_last_event::<T>(Event::<T>::ProviderBoostRewardClaimed {account: caller.clone(), reward_amount: unclaimed_rewards}.into());
	}

	impl_benchmark_test_suite!(Capacity,
		tests::mock::new_test_ext(),
		tests::mock::Test);
}
