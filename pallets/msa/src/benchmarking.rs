use super::*;

#[allow(unused)]
use crate::Pallet as Msa;
use frame_benchmarking::{account, benchmarks, benchmarks_instance_pallet, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_core::{sr25519, Encode, Pair};
use sp_runtime::traits::AccountIdConversion;
use sp_runtime::traits::IdentifyAccount;

const SEED: u32 = 0;

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	account(name, index, SEED)
}

fn create_msa<T: Config>(n: u32) -> DispatchResult {
	let acc = create_account::<T>("account", n);
	Msa::<T>::create(RawOrigin::Signed(acc.clone()).into())
}

fn create_payload_and_signature<T: Config>() -> (AddDelegate, MultiSignature, T::AccountId)
{
	let (key_pair_delegator, _) = sr25519::Pair::generate();
	let delegator_account = key_pair_delegator.public();

	let add_delegate_payload = AddDelegate { authorized_msa_id: 1u64.into(), permission: 0 };
	let encode_add_delegate_data = wrap_binary_data(add_delegate_payload.encode());

	let signature: MultiSignature =
		key_pair_delegator.sign(&encode_add_delegate_data).into();
	// let acc: T::AccountId = Decode::decode( &mut delegator_account.0.clone()).unwrap();
	let acc = AccountId32::new(delegator_account.into());
	(add_delegate_payload, signature, acc.into())
}

benchmarks! {
	create {
		let s in 1 .. 1000;
		let caller: T::AccountId = whitelisted_caller();

		for j in 0 .. s {
			assert_ok!(create_msa::<T>(j));
		}
	}: _ (RawOrigin::Signed(caller))

	create_sponsored_account_with_delegation {
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(Msa::<T>::create(RawOrigin::Signed(caller.clone()).into()));
		let (payload, signature, key) = create_payload_and_signature::<T>();

	}: _ (RawOrigin::Signed(caller), key, signature, payload)

	impl_benchmark_test_suite!(Msa, crate::mock::new_test_ext(), crate::mock::Test);
}
