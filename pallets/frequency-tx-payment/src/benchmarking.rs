use super::*;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::{Call as SystemCall, RawOrigin};

benchmarks! {
	pay_with_capacity {
		let sender: T::AccountId = whitelisted_caller();
		let call: Box<<T as Config>::RuntimeCall> = Box::new(SystemCall::remark { remark: vec![] }.into());
	}: _ (RawOrigin::Signed(sender), call)

	pay_with_capacity_batch_all {
		let n in 0 .. (T::MaximumCapacityBatchLength::get() as u32);

		let sender: T::AccountId = whitelisted_caller();

		let mut batched_calls: Vec<<T as Config>::RuntimeCall> = vec![];

		for i in 0 .. n {
			let call: <T as Config>::RuntimeCall = SystemCall::remark { remark: vec![] }.into();
			batched_calls.push(call);
		}
	}: _ (RawOrigin::Signed(sender), batched_calls)

	impl_benchmark_test_suite!(
		Pallet,
		crate::tests::mock::ExtBuilder::default().build(),
		crate::tests::mock::Test
	);
}
