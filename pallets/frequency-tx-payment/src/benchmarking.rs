use super::*;
use common_primitives::msa::MessageSourceId;
use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_runtime::traits::{AsTransactionAuthorizedOrigin, DispatchTransaction};

pub fn register_provider<T: Config>(target_id: MessageSourceId, name: &'static str) {
	#[allow(clippy::useless_conversion)]
	let name = Vec::from(name).try_into().expect("error");
	assert_ok!(T::ProviderBenchmarkHelper::create(target_id, name));
}

pub fn register_specified_msa<T: Config>(target_id: MessageSourceId, account: T::AccountId) {
	assert_ok!(T::MsaBenchmarkHelper::create_msa(target_id, account.clone()));
}

pub fn fund_msa_capacity<T: Config>(
	target_id: MessageSourceId,
	account: T::AccountId,
	amount: u32,
) {
	assert_ok!(T::CapacityBenchmarkHelper::stake_benchmark(account.clone(), target_id, amount));
}

#[benchmarks(where
	T::RuntimeOrigin: AsTransactionAuthorizedOrigin,
	<T as frame_system::Config>::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo> + IsSubType<Call<T>> + From<crate::Call<T>> + From<frame_system::Call<T>>,
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
			Box::new(frame_system::Call::<T>::remark { remark: vec![] }.into());

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
			let call: <T as Config>::RuntimeCall =
				frame_system::Call::<T>::remark { remark: vec![] }.into();
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
		let inner = frame_system::Call::<T>::remark { remark: alloc::vec![] };
		let call: <T as frame_system::Config>::RuntimeCall = inner.into();
		let info = DispatchInfo {
			call_weight: Weight::zero(),
			extension_weight: Weight::zero(),
			class: DispatchClass::Normal,
			pays_fee: Pays::No,
		};
		let post_info = PostDispatchInfo { actual_weight: None, pays_fee: Pays::No };
		#[block]
		{
			let res = ext
				.test_run(RawOrigin::Signed(caller).into(), &call, &info, 0, 0, |_| Ok(post_info));
			assert_ok!(res);
			assert!(res.expect("should be ok").is_ok());
		}
	}

	#[benchmark]
	fn charge_tx_payment_token_based() {
		let caller: T::AccountId = whitelisted_caller();
		<<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<
			T,
		>>::endow_account(&caller, 1_000_000_000u32.into());
		let ext: ChargeFrqTransactionPayment<T> = ChargeFrqTransactionPayment::from(0u64.into());
		let inner = frame_system::Call::<T>::remark { remark: alloc::vec![] };
		let call: <T as frame_system::Config>::RuntimeCall = inner.into();
		let extension_weight = ext.weight(&call);
		let info = DispatchInfo {
			call_weight: Weight::from_parts(100, 0),
			extension_weight,
			class: DispatchClass::Operational,
			pays_fee: Pays::Yes,
		};
		let post_info = PostDispatchInfo {
			actual_weight: Some(Weight::from_parts(10, 0)),
			pays_fee: Pays::Yes,
		};
		#[block]
		{
			let res = ext
				.test_run(RawOrigin::Signed(caller).into(), &call, &info, 0, 0, |_| Ok(post_info));
			assert_ok!(res);
			assert!(res.expect("should be ok").is_ok());
		}
	}

	#[benchmark]
	fn charge_tx_payment_capacity_based() {
		let caller: T::AccountId = whitelisted_caller();
		<<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<
			T,
		>>::endow_account(&caller, 8054550000u64.into());
		let msa_id = MessageSourceId::from(1001u32);
		register_specified_msa::<T>(msa_id, caller.clone());
		register_provider::<T>(msa_id, "provider1");
		fund_msa_capacity::<T>(msa_id, caller.clone(), 2054550000u32);
		let inner_call: <T as Config>::RuntimeCall =
			frame_system::Call::<T>::remark { remark: vec![] }.into();

		let pay_with_capacity_call =
			crate::Call::<T>::pay_with_capacity { call: Box::new(inner_call) };
		let runtime_call: <T as frame_system::Config>::RuntimeCall = pay_with_capacity_call.into();

		let ext: ChargeFrqTransactionPayment<T> = ChargeFrqTransactionPayment::from(0u64.into());

		let info = DispatchInfo {
			call_weight: Weight::from_parts(100, 0),
			extension_weight: Weight::zero(),
			class: DispatchClass::Normal,
			pays_fee: Pays::Yes,
		};
		let post_info = PostDispatchInfo {
			actual_weight: Some(Weight::from_parts(10, 0)),
			pays_fee: Pays::Yes,
		};

		#[block]
		{
			let res = ext.test_run(
				RawOrigin::Signed(caller.clone()).into(),
				&runtime_call,
				&info,
				0,
				0,
				|_| Ok(post_info),
			);
			assert_ok!(res);
			assert!(res.expect("should be ok").is_ok());
		}
	}
	impl_benchmark_test_suite!(
		Pallet,
		crate::tests::mock::ExtBuilder::default().build(),
		crate::tests::mock::Test
	);
}
