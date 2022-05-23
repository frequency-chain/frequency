use super::*;

#[allow(unused)]
use crate::Pallet as Msa;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_core::{crypto::KeyTypeId, Encode};
use sp_runtime::RuntimeAppPublic;

pub const TEST_KEY_TYPE_ID: KeyTypeId = KeyTypeId(*b"test");

mod app_sr25519 {
	use super::TEST_KEY_TYPE_ID;
	use sp_core::sr25519;
	use sp_runtime::app_crypto::app_crypto;
	app_crypto!(sr25519, TEST_KEY_TYPE_ID);
}

type SignerId = app_sr25519::Public;

const SEED: u32 = 0;

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	account(name, index, SEED)
}

fn create_msa<T: Config>(n: u32) -> DispatchResult {
	let acc = create_account::<T>("account", n);
	Msa::<T>::create(RawOrigin::Signed(acc.clone()).into())
}

fn create_payload_and_signature<T: Config>() -> (AddProvider, MultiSignature, T::AccountId) {
	let delegator_account = SignerId::generate_pair(None);
	let add_provider_payload = AddProvider { authorized_msa_id: 1u64.into(), permission: 0 };
	let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

	let signature = delegator_account.sign(&encode_add_provider_data).unwrap();
	let acc = T::AccountId::decode(&mut &delegator_account.encode()[..]).unwrap();
	(add_provider_payload, MultiSignature::Sr25519(signature.into()), acc.into())
}

fn create_account_with_msa_id<T: Config>(n: u32) -> (T::AccountId, MessageSenderId) {
	let provider = create_account::<T>("account", n);

	assert_ok!(Msa::<T>::create(RawOrigin::Signed(provider.clone()).into()));

	let key_info = Msa::<T>::try_get_key_info(&provider).unwrap();

	(provider.clone(), key_info.msa_id)
}

// fn add_delegation<T: Config>(delegator: Delegator, provider: Delegate) {
// 	assert_ok!(Msa::<T>::add_delegate(provider, delegator));
// }

benchmarks! {
	create {
		let s in 1 .. 1000;
		let caller: T::AccountId = whitelisted_caller();

		for j in 0 .. s {
			assert_ok!(create_msa::<T>(j));
		}
	}: _ (RawOrigin::Signed(caller))

	// create_sponsored_account_with_delegation {
	// 	let caller: T::AccountId = whitelisted_caller();
	// 	assert_ok!(Msa::<T>::create(RawOrigin::Signed(caller.clone()).into()));
	// 	let (payload, signature, key) = create_payload_and_signature::<T>();
	//
	// }: _ (RawOrigin::Signed(caller), key, signature, payload)
	//
	// remove_msa_delegation_by_provider {
	// 	let s in 5 .. 1005;
	//
	// 	let (provider, provider_msa_id) = create_account_with_msa_id::<T>(0);
	// 	let (delegator, delegator_msa_id) = create_account_with_msa_id::<T>(1);
	// 	add_delegation::<T>(Delegator(delegator_msa_id), Delegate(provider_msa_id.clone()));
	//
	// 	for j in 2 .. s {
	// 		let (other, other_msa_id) = create_account_with_msa_id::<T>(j);
	// 		add_delegation::<T>(Delegator(other_msa_id), Delegate(provider_msa_id.clone()));
	// 	}
	// }: _ (RawOrigin::Signed(provider), delegator_msa_id)

	impl_benchmark_test_suite!(Msa, crate::mock::new_test_ext_keystore(), crate::mock::Test);
}
