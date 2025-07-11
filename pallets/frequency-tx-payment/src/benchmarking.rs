use super::*;

use frame_benchmarking::v2::*;
use frame_system::{Call as SystemCall, RawOrigin};
use sp_runtime::traits::{AsTransactionAuthorizedOrigin, DispatchTransaction};

#[benchmarks(where
	T::RuntimeOrigin: AsTransactionAuthorizedOrigin,
	<T as frame_system::Config>::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo> + IsSubType<Call<T>>,
BalanceOf<T>: Send
		+ Sync
		+ FixedPointOperand
		+ From<u64>
		+ IsType<ChargeCapacityBalanceOf<T>>
		+ IsType<CapacityBalanceOf<T>>,
	<T as frame_system::Config>::RuntimeOrigin: AsSystemOriginSigner<T::AccountId> + Clone,
)]
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

	#[benchmark]
	fn charge_tx_payment_free() {
		let caller: T::AccountId = whitelisted_caller();
		<<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<
			T,
		>>::endow_account(&caller, 100_000u64.into());
		let ext: ChargeFrqTransactionPayment<T> = ChargeFrqTransactionPayment::from(0u64.into());
		let inner = frame_system::Call::remark { remark: alloc::vec![] };
		let call = <T as frame_system::Config>::RuntimeCall::from(inner);
		let info = DispatchInfo {
			call_weight: Weight::zero(),
			extension_weight: Weight::zero(),
			class: DispatchClass::Normal,
			pays_fee: Pays::No,
		};
		let post_info = PostDispatchInfo { actual_weight: None, pays_fee: Pays::No };
		#[block]
		{
			assert!(ext
				.test_run(RawOrigin::Signed(caller).into(), &call, &info, 0, 0, |_| Ok(post_info))
				.unwrap()
				.is_ok());
		}
	}

	#[benchmark]
	fn charge_tx_payment_token_based() {
		let caller: T::AccountId = whitelisted_caller();
		<<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<
			T,
		>>::endow_account(&caller, 1_000_000_000u32.into());
		let ext: ChargeFrqTransactionPayment<T> = ChargeFrqTransactionPayment::from(0u64.into());
		let inner = frame_system::Call::remark { remark: alloc::vec![] };
		let call = <T as frame_system::Config>::RuntimeCall::from(inner);
		let info = DispatchInfo {
			call_weight: Weight::from_parts(1, 0),
			extension_weight: Weight::zero(),
			class: DispatchClass::Normal,
			pays_fee: Pays::Yes,
		};
		let post_info =
			PostDispatchInfo { actual_weight: Some(Weight::from_parts(1, 0)), pays_fee: Pays::Yes };
		#[block]
		{
			let res = ext
				.test_run(RawOrigin::Signed(caller).into(), &call, &info, 0, 0, |_| Ok(post_info));
			log::info!("test_run result: {:?}", res);
			assert!(res.unwrap().is_ok());
		}
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::tests::mock::ExtBuilder::default().build(),
		crate::tests::mock::Test
	);
}
