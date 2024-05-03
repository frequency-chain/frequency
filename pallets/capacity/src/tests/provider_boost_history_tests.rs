use sp_runtime::BoundedBTreeMap;
use crate::{ ProviderBoostHistory, Config};
use crate::StakingType::{MaximumCapacity, ProviderBoost};
use crate::tests::mock::{Capacity, new_test_ext, Test};
use crate::tests::testing_utils::setup_provider;
use sp_runtime::traits::Get;

#[test]
fn provider_boost_adds_to_staking_history() {
    new_test_ext().execute_with(|| {
        let staker = 10_000u64;
        let target = 1u64;
        
        setup_provider(&staker, &target, &1_000u64, ProviderBoost);
        let history = Capacity::get_staking_history_for(staker);
        assert!(history.is_some());
    })
}

#[test]
fn stake_does_not_add_to_staking_history() {
    new_test_ext().execute_with(|| {
        let staker = 10_000u64;
        let target = 1u64;

        setup_provider(&staker, &target, &1_000u64, MaximumCapacity);
        let history = Capacity::get_staking_history_for(staker);
        assert!(history.is_none());
    })    
}

#[test]
fn provider_boost_History_add_era_balance_can_add_new_entries() {
    let mut pbh = ProviderBoostHistory::<Test>::new();
    let bound = <Test as Config>::StakingRewardsPastErasMax::get();
    
    for era in 1..bound {
        let add_amount: u64 = 1_000 * era as u64;
        pbh.add_era_balance(&era, &add_amount);
        assert_eq!(pbh.count(), era as usize);
        assert_eq!(pbh.get_staking_amount_for_era(&era), Some(&add_amount));
    }
    assert_eq!(pbh.count(), bound);
    
    let new_era: <Test as Config>::RewardEra = 99;
    pbh.add_era_balance(&new_era, &99.into());
    assert_eq!(pbh.count(), bound);
    
    let first_era: <Test as Config>::RewardEra = 1;
    assert!(pbh.get_staking_amount_for_era(&first_era).is_none());
    assert_eq!(pbh.get_staking_amount_for_era(&new_era), Some(&99u64));
}

#[test]
fn provider_boost_History_add_era_balance_can_update_entries() {
    
}

#[test]
fn provider_boost_History_add_era_balance_removes_old_entries_if_full() {}
