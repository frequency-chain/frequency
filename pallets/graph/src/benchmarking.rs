use super::*;
#[allow(unused)]
use crate::Pallet as GraphPallet;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::{assert_ok, dispatch::DispatchResult};
use frame_system::RawOrigin;

const SEED: u32 = 0;
const NODES: u32 = 1000;
const FOLLOWS: u32 = 500;
const PAGE_SIZE: u32 = 500;
const PAGES: u32 = 20;

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	account(name, index, SEED)
}

fn node_addition<T: Config>(n: u32) -> DispatchResult {
	let other = create_account::<T>("account", n);
	GraphPallet::<T>::add_node(RawOrigin::Signed(other.clone()).into(), n.into())
}

fn do_follow_adj<T: Config>(from: u32, to: u32) -> DispatchResult {
	let caller: T::AccountId = whitelisted_caller();
	GraphPallet::<T>::follow_adj(RawOrigin::Signed(caller).into(), from.into(), to.into())
}

fn do_follow_map<T: Config>(from: u32, to: u32) -> DispatchResult {
	let caller: T::AccountId = whitelisted_caller();
	GraphPallet::<T>::follow_map(RawOrigin::Signed(caller).into(), from.into(), to.into())
}

fn do_follow_child<T: Config>(
	from: u32,
	to: u32,
	permission: Permission,
	page: u16,
) -> DispatchResult {
	let caller: T::AccountId = whitelisted_caller();
	GraphPallet::<T>::follow_child_public(
		RawOrigin::Signed(caller).into(),
		from.into(),
		to.into(),
		permission,
		page,
	)
}

benchmarks! {
	add_node {
		let n in 1..NODES;
		let caller: T::AccountId = whitelisted_caller();
		for j in 0..n {
			assert_ok!(node_addition::<T>(j));
		}
	}: _(RawOrigin::Signed(caller), n.into())

   follow_adj {
		let n in 2..NODES;
		let caller: T::AccountId = whitelisted_caller();

		for i in 0..=NODES {
			assert_ok!(node_addition::<T>(i));
		}

		for i in 0..=NODES {
			for j in 2..=FOLLOWS {
				if i != j {
					assert_ok!(do_follow_adj::<T>(i,j));
				}
			}
		}

	}: _(RawOrigin::Signed(caller), n.into(), 1u64.into())

	 unfollow_adj {
		let n in 2..NODES;
		let caller: T::AccountId = whitelisted_caller();

		for i in 0..=NODES {
			assert_ok!(node_addition::<T>(i));
		}

		for i in 0..=NODES {
			for j in 1..=FOLLOWS {
				if i != j {
					assert_ok!(do_follow_adj::<T>(i,j));
				}
			}
		}
	}: _(RawOrigin::Signed(caller), n.into(), 1u64.into())

   follow_map {
		let n in 2..NODES;
		let caller: T::AccountId = whitelisted_caller();

		for i in 0..=NODES {
			assert_ok!(node_addition::<T>(i));
		}

		for i in 0..=NODES {
			for j in 2..=FOLLOWS {
				if i != j {
					assert_ok!(do_follow_map::<T>(i,j));
				}
			}
		}

	}: _(RawOrigin::Signed(caller), n.into(), 1u64.into())

	 unfollow_map {
		let n in 2..NODES;
		let caller: T::AccountId = whitelisted_caller();

		for i in 0..=NODES {
			assert_ok!(node_addition::<T>(i));
		}

		for i in 0..=NODES {
			for j in 1..=FOLLOWS {
				if i != j {
					assert_ok!(do_follow_map::<T>(i,j));
				}
			}
		}
	}: _(RawOrigin::Signed(caller), n.into(), 1u64.into())

	follow_child_public {
		let n in 0..NODES;
		let p in 0..PAGES;
		let caller: T::AccountId = whitelisted_caller();

		for i in 0..=NODES {
			assert_ok!(node_addition::<T>(i));
		}
		let permission = Permission { data: 1000u16};

		for i in 0..=NODES {
			for k in 0..=PAGES {
				for j in 2..=PAGE_SIZE {
					if i != j {
						assert_ok!(do_follow_child::<T>(i,j, permission.clone(), k as u16));
					}
				}
			}
		}

	}: _(RawOrigin::Signed(caller), n.into(), 1u64.into(), permission, p.try_into().unwrap())

	unfollow_child_public {
		let n in 0..NODES;
		let p in 0..PAGES;
		let caller: T::AccountId = whitelisted_caller();

		for i in 0..=NODES {
			assert_ok!(node_addition::<T>(i));
		}
		let permission = Permission { data: 1000u16};

		for i in 0..=NODES {
			for k in 0..=PAGES {
				for j in 2..=PAGE_SIZE {
					if i != j {
						assert_ok!(do_follow_child::<T>(i,j, permission.clone(), k as u16));
					}
				}
			}
		}
	}: _(RawOrigin::Signed(caller), n.into(), 2u64.into(), permission, p.try_into().unwrap())

	impl_benchmark_test_suite!(GraphPallet, crate::mock::new_test_ext(), crate::mock::Test);
}
