#![cfg(feature = "runtime-benchmarks")]

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
	let schemas: Vec<SchemaId> = vec![1, 2];
	T::SchemaValidator::set_schema_count(schemas.len().try_into().unwrap());
	let expiration = 10u32;
	let add_provider_payload = AddProvider::new(1u64, Some(schemas), expiration);
	let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

	let signature = delegator_account.sign(&encode_add_provider_data).unwrap();
	let acc = T::AccountId::decode(&mut &delegator_account.encode()[..]).unwrap();
	(add_provider_payload, MultiSignature::Sr25519(signature.into()), acc.into())
}

fn add_key_payload_and_signature<T: Config>() -> (AddKeyData, MultiSignature, T::AccountId) {
	let account = SignerId::generate_pair(None);
	let add_key_payload = AddKeyData { msa_id: 1u64.into(), nonce: 0, expiration: 10 };
	let encode_add_provider_data = wrap_binary_data(add_key_payload.encode());

	let signature = account.sign(&encode_add_provider_data).unwrap();
	let acc = T::AccountId::decode(&mut &account.encode()[..]).unwrap();
	(add_key_payload, MultiSignature::Sr25519(signature.into()), acc.into())
}

fn create_account_with_msa_id<T: Config>(n: u32) -> (T::AccountId, MessageSourceId) {
	let provider = create_account::<T>("account", n);

	assert_ok!(Msa::<T>::create(RawOrigin::Signed(provider.clone()).into()));

	let msa_id = Msa::<T>::try_get_msa_from_account_id(&provider).unwrap();

	(provider.clone(), msa_id)
}

fn add_delegation<T: Config>(delegator: Delegator, provider: Provider) {
	let schemas: Vec<SchemaId> = vec![1, 2];
	T::SchemaValidator::set_schema_count(schemas.len().try_into().unwrap());
	assert_ok!(Msa::<T>::add_provider(provider, delegator, schemas));
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
		assert_ok!(Msa::<T>::register_provider(RawOrigin::Signed(caller.clone()).into(),Vec::from("Foo")));

		let (payload, signature, key) = create_payload_and_signature::<T>();

	}: _ (RawOrigin::Signed(caller), key, signature, payload)

	revoke_delegation_by_provider {
		let s in 5 .. 1005;

		let (provider, provider_msa_id) = create_account_with_msa_id::<T>(0);
		let (delegator, delegator_msa_id) = create_account_with_msa_id::<T>(1);
		add_delegation::<T>(Delegator(delegator_msa_id), Provider(provider_msa_id.clone()));

		for j in 2 .. s {
			let (other, other_msa_id) = create_account_with_msa_id::<T>(j);
			add_delegation::<T>(Delegator(other_msa_id), Provider(provider_msa_id.clone()));
		}
	}: _ (RawOrigin::Signed(provider), delegator_msa_id)

	add_key_to_msa {
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(Msa::<T>::create(RawOrigin::Signed(caller.clone()).into()));
		let (add_provider_payload, signature, key) = add_key_payload_and_signature::<T>();

	}: _ (RawOrigin::Signed(caller), key, signature, add_provider_payload)

	delete_msa_key {
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(Msa::<T>::create(RawOrigin::Signed(caller.clone()).into()));

		let (add_provider_payload, signature, key) = add_key_payload_and_signature::<T>();
		assert_ok!(Msa::<T>::add_key_to_msa(RawOrigin::Signed(caller.clone()).into(), key.clone(), signature, add_provider_payload));

	}: _(RawOrigin::Signed(caller), key)

	retire_msa {
		let caller: T::AccountId = whitelisted_caller();

		// Create a MSA account
		assert_ok!(Msa::<T>::create(RawOrigin::Signed(caller.clone()).into()));
		let msa_id = Msa::<T>::try_get_msa_from_account_id(&caller).unwrap();

		assert_eq!(Msa::<T>::is_registered_provider(msa_id),false);

	}: _(RawOrigin::Signed(caller))

	add_provider_to_msa {
		let caller: T::AccountId = whitelisted_caller();
		let (payload, signature, key) = create_payload_and_signature::<T>();

		assert_ok!(Msa::<T>::create(RawOrigin::Signed(caller.clone()).into()));
		assert_ok!(Msa::<T>::register_provider(RawOrigin::Signed(caller.clone()).into(),Vec::from("Foo")));
		assert_ok!(Msa::<T>::create(RawOrigin::Signed(key.clone()).into()));

	}: _ (RawOrigin::Signed(caller), key, signature, payload)

	revoke_msa_delegation_by_delegator {
		let (provider, provider_msa_id) = create_account_with_msa_id::<T>(0);
		let (delegator, delegator_msa_id) = create_account_with_msa_id::<T>(1);
		add_delegation::<T>(Delegator(delegator_msa_id), Provider(provider_msa_id.clone()));


	}: _ (RawOrigin::Signed(delegator), provider_msa_id)

	register_provider {
		let (provider, _provider_msa_id) = create_account_with_msa_id::<T>(1);
	}: _ (RawOrigin::Signed(provider), Vec::from("Foo"))

	impl_benchmark_test_suite!(Msa, crate::mock::new_test_ext_keystore(), crate::mock::Test);
}
