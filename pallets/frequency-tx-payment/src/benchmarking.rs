use super::*;

use frame_benchmarking::v2::*;
use frame_system::{Call as SystemCall, RawOrigin};

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn pay_with_capacity() -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let call: Box<<T as Config>::RuntimeCall> =
			Box::new(SystemCall::remark { remark: vec![] }.into());

		#[extrinsic_call]
		_(RawOrigin::Signed(sender), call);

		Ok(())
	}

	#[benchmark]
	fn pay_with_capacity_batch_all(
		n: Linear<0, { T::MaximumCapacityBatchLength::get() as u32 }>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();

		let mut batched_calls: Vec<<T as Config>::RuntimeCall> = vec![];

		for _ in 0..n {
			let call: <T as Config>::RuntimeCall = SystemCall::remark { remark: vec![] }.into();
			batched_calls.push(call);
		}

		#[extrinsic_call]
		_(RawOrigin::Signed(sender), batched_calls);

		Ok(())
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::tests::mock::ExtBuilder::default().build(),
		crate::tests::mock::Test
	);
}
