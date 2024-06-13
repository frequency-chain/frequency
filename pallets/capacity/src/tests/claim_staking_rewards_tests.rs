use frame_support::assert_noop;
use crate::*;
use crate::tests::mock::{Capacity, new_test_ext, RuntimeOrigin, Test};
use crate::tests::testing_utils::{run_to_block, setup_provider};

#[test]
fn claim_staking_rewards_leaves_one_history_item_for_current_era() {
    new_test_ext().execute_with(|| {
        let account = 10_000u64;
        let target: MessageSourceId = 10;
        let amount = 1_000u64;

        setup_provider(&account, &target, &amount, ProviderBoost);
        run_to_block(31);
        assert_eq!(Capacity::get_current_era().era_index, 4u32);

        let current_history = Capacity::get_staking_history_for(account).unwrap();
        assert_eq!(current_history.count(), 1usize);
        let history_item = current_history.get_entry_for_era(&1u32).unwrap();
        assert_eq!(*history_item, amount);

        // assert_ok!(Capacity::claim_staking_rewards(::signed(account)));
    })
}

#[test]
fn claim_staking_rewards_allows_unstake() {
    todo!();
}

#[test]
fn claim_staking_rewards_mints_and_transfers_expected_total() {
    new_test_ext().execute_with(|| {
        // it mints the expected total
        // it transfers the expected total
        // the reward value is unlocked
        // the staking amount remains locked
    })
}

#[test]
fn claim_staking_rewards_has_expected_total_when_other_stakers() {
    todo!();
}

#[test]
fn claim_staking_rewards_has_expected_total_if_amount_should_be_capped() {
    todo!();
}

#[test]
fn claim_staking_rewards_has_expected_total_for_multiple_stake_events() {
    todo!();
}

#[test]
fn claim_staking_rewards_fails_if_no_available_rewards() {
    new_test_ext().execute_with(|| {
        todo!();
    })
}

#[test]
fn claim_staking_rewards_fails_if_stake_maximized() {
    new_test_ext().execute_with(|| {
        let account = 10_000u64;
        let target: MessageSourceId = 10;
        let amount = 1_000u64;

        assert_noop!(
            Capacity::claim_staking_rewards(RuntimeOrigin::signed(account)),
            Error::<Test>::NotAProviderBoostAccount
        );

        setup_provider(&account, &target, &amount, MaximumCapacity);

        assert_noop!(
            Capacity::claim_staking_rewards(RuntimeOrigin::signed(account)),
            Error::<Test>::NotAProviderBoostAccount
        );
    })
}