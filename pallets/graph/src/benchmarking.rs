use super::*;
#[allow(unused)]
use crate::Pallet as GraphPallet;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::{assert_ok, dispatch::DispatchResult};
use frame_system::RawOrigin;

const SEED: u32 = 0;
const NODES: u32 = 1_000;
const FOLLOWS: u32 = 500;

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	account(name, index, SEED)
}

fn node_addition<T: Config>(n: u32) -> DispatchResult {
	let other = create_account::<T>("account", n);
	GraphPallet::<T>::add_node(RawOrigin::Signed(other.clone()).into(), n.into())
}

fn do_follow<T: Config>(from: u32, to: u32) -> DispatchResult {
	let caller: T::AccountId = whitelisted_caller();
	GraphPallet::<T>::follow(RawOrigin::Signed(caller).into(), from.into(), to.into())
}

fn do_follow2<T: Config>(from: u32, to: u32) -> DispatchResult {
	let caller: T::AccountId = whitelisted_caller();
	GraphPallet::<T>::follow2(RawOrigin::Signed(caller).into(), from.into(), to.into())
}

benchmarks! {
	add_node {
		let n in 1..NODES;
		let caller: T::AccountId = whitelisted_caller();
		for j in 0..n {
			assert_ok!(node_addition::<T>(j));
		}
	}: _(RawOrigin::Signed(caller), n.into())

   follow {
		let n in 2..NODES;
		let caller: T::AccountId = whitelisted_caller();

		for i in 0..=NODES {
			assert_ok!(node_addition::<T>(i));
		}

		for i in 0..=NODES {
			for j in 2..=FOLLOWS {
				if i != j {
					assert_ok!(do_follow::<T>(i,j));
				}
			}
		}

	}: _(RawOrigin::Signed(caller), n.into(), 1u64.into())

	 unfollow {
		let n in 2..NODES;
		let caller: T::AccountId = whitelisted_caller();

		for i in 0..=NODES {
			assert_ok!(node_addition::<T>(i));
		}

		for i in 0..=NODES {
			for j in 1..=FOLLOWS {
				if i != j {
					assert_ok!(do_follow::<T>(i,j));
				}
			}
		}
	}: _(RawOrigin::Signed(caller), n.into(), 1u64.into())

   follow2 {
		let n in 2..NODES;
		let caller: T::AccountId = whitelisted_caller();

		for i in 0..=NODES {
			assert_ok!(node_addition::<T>(i));
		}

		for i in 0..=NODES {
			for j in 2..=FOLLOWS {
				if i != j {
					assert_ok!(do_follow2::<T>(i,j));
				}
			}
		}

	}: _(RawOrigin::Signed(caller), n.into(), 1u64.into())

	 unfollow2 {
		let n in 2..NODES;
		let caller: T::AccountId = whitelisted_caller();

		for i in 0..=NODES {
			assert_ok!(node_addition::<T>(i));
		}

		for i in 0..=NODES {
			for j in 1..=FOLLOWS {
				if i != j {
					assert_ok!(do_follow2::<T>(i,j));
				}
			}
		}
	}: _(RawOrigin::Signed(caller), n.into(), 1u64.into())

	impl_benchmark_test_suite!(GraphPallet, crate::mock::new_test_ext(), crate::mock::Test);
}
