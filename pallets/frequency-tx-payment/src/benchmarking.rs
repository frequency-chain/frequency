use super::*;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::{Call as SystemCall, RawOrigin};

benchmarks! {
	pay_with_capacity {
		let sender: T::AccountId = whitelisted_caller();
		let call: Box<<T as Config>::RuntimeCall> = Box::new(SystemCall::remark { remark: vec![] }.into());
	}: _ (RawOrigin::Signed(sender), call)

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::ExtBuilder::default().build(),
		crate::mock::Test
	);
}
