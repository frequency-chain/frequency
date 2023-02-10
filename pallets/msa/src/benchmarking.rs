use super::*;

use crate::types::EMPTY_FUNCTION;
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

fn create_payload_and_signature<T: Config>(
	schemas: Vec<SchemaId>,
	authorized_msa_id: MessageSourceId,
) -> (AddProvider, MultiSignature, T::AccountId) {
	let delegator_account = SignerId::generate_pair(None);
	let expiration = 10u32;
	let add_provider_payload = AddProvider::new(authorized_msa_id, Some(schemas), expiration);
	let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

	let signature = delegator_account.sign(&encode_add_provider_data).unwrap();
	let acc = T::AccountId::decode(&mut &delegator_account.encode()[..]).unwrap();
	(add_provider_payload, MultiSignature::Sr25519(signature.into()), acc.into())
}

fn add_key_payload_and_signature<T: Config>(
	msa_id: u64,
) -> (AddKeyData<T>, MultiSignature, T::AccountId) {
	let new_keys = SignerId::generate_pair(None);
	let public_key = T::AccountId::decode(&mut &new_keys.encode()[..]).unwrap();
	let add_key_payload = AddKeyData::<T> {
		msa_id: msa_id.into(),
		expiration: 10u32.into(),
		new_public_key: public_key.into(),
	};

	let encoded_add_key_payload = wrap_binary_data(add_key_payload.encode());

	let signature = new_keys.sign(&encoded_add_key_payload).unwrap();
	let acc = T::AccountId::decode(&mut &new_keys.encode()[..]).unwrap();
	(add_key_payload, MultiSignature::Sr25519(signature.into()), acc.into())
}

fn create_msa_account_and_keys<T: Config>() -> (T::AccountId, SignerId, MessageSourceId) {
	let key_pair = SignerId::generate_pair(None);
	let account_id = T::AccountId::decode(&mut &key_pair.encode()[..]).unwrap();

	let (msa_id, _) = Msa::<T>::create_account(account_id.clone(), EMPTY_FUNCTION).unwrap();

	(account_id, key_pair, msa_id)
}

benchmarks! {
	create {
		let caller: T::AccountId = whitelisted_caller();

	}: _ (RawOrigin::Signed(caller.clone()))
	verify {
		assert!(Msa::<T>::get_msa_by_public_key(caller).is_some());
		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
	}

	create_sponsored_account_with_delegation {
		let s in 0 .. T::MaxSchemaGrantsPerDelegation::get();

		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(Msa::<T>::create(RawOrigin::Signed(caller.clone()).into()));
		assert_ok!(Msa::<T>::create_provider(RawOrigin::Signed(caller.clone()).into(),Vec::from("Foo")));

		let schemas: Vec<SchemaId> = (0 .. s as u16).collect();
		T::SchemaValidator::set_schema_count(schemas.len().try_into().unwrap());
		let (payload, signature, key) = create_payload_and_signature::<T>(schemas, 1u64.into());
	}: _ (RawOrigin::Signed(caller), key, signature, payload)
	verify {
		assert_eq!(frame_system::Pallet::<T>::events().len(), 2);
	}

	revoke_delegation_by_provider {
		let provider_account = create_account::<T>("account", 0);
		let (provider_msa_id, provider_public_key) = Msa::<T>::create_account(provider_account.into(), EMPTY_FUNCTION).unwrap();

		let delegator_account = create_account::<T>("account", 1);
		let (delegator_msa_id, _) = Msa::<T>::create_account(delegator_account.into(), EMPTY_FUNCTION).unwrap();

		assert_ok!(Msa::<T>::add_provider(ProviderId(provider_msa_id), DelegatorId(delegator_msa_id), vec![]));
	}: _ (RawOrigin::Signed(provider_public_key), delegator_msa_id)
	verify {
		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
		assert_eq!(Msa::<T>::get_delegation(DelegatorId(delegator_msa_id), ProviderId(provider_msa_id)).unwrap().revoked_at, T::BlockNumber::from(1u32));
	}

	add_public_key_to_msa {
		let (provider_public_key, provider_key_pair, _) = create_msa_account_and_keys::<T>();
		let (delegator_public_key, delegator_key_pair, delegator_msa_id) = create_msa_account_and_keys::<T>();

		let (add_key_payload, new_public_key_signature, _) = add_key_payload_and_signature::<T>(delegator_msa_id);

		let encoded_add_key_payload = wrap_binary_data(add_key_payload.encode());
		let owner_signature = MultiSignature::Sr25519(delegator_key_pair.sign(&encoded_add_key_payload).unwrap().into());
	}: _ (RawOrigin::Signed(provider_public_key.clone()), delegator_public_key.clone(), owner_signature, new_public_key_signature, add_key_payload)
	verify {
		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
	}

	delete_msa_public_key {
		let (provider_public_key, provider_key_pair, _) = create_msa_account_and_keys::<T>();
		let (caller_and_delegator_public_key, delegator_key_pair, delegator_msa_id) = create_msa_account_and_keys::<T>();

		let (add_key_payload, new_public_key_signature, new_public_key) = add_key_payload_and_signature::<T>(delegator_msa_id);

		let encoded_add_key_payload = wrap_binary_data(add_key_payload.encode());
		let owner_signature = MultiSignature::Sr25519(delegator_key_pair.sign(&encoded_add_key_payload).unwrap().into());

		assert_ok!(Msa::<T>::add_public_key_to_msa(RawOrigin::Signed(provider_public_key).into(), caller_and_delegator_public_key.clone(), owner_signature,  new_public_key_signature, add_key_payload));

	}: _(RawOrigin::Signed(caller_and_delegator_public_key), new_public_key)
	verify {
		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
	}

	retire_msa {
		let s in 5 .. EXPECTED_MAX_NUMBER_OF_PROVIDERS_PER_DELEGATOR;
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(Msa::<T>::add_key(ProviderId(1).into(), &caller.clone(), EMPTY_FUNCTION));
		T::SchemaValidator::set_schema_count(2);
		for j in 2 .. s  {
			assert_ok!(Msa::<T>::add_provider(ProviderId(j.into()), DelegatorId(1), vec![1, 2]));
		}
	}: {
		assert_ok!(Msa::<T>::retire_msa(RawOrigin::Signed(caller.clone()).into()));
	}
	verify {
		// Assert that the MSA has no accounts
		let key_count = Msa::<T>::get_public_key_count_by_msa_id(1);
		assert_eq!(key_count, 0);
	}

	grant_delegation {
		let s in 0 .. T::MaxSchemaGrantsPerDelegation::get();
		let provider_caller: T::AccountId = whitelisted_caller();

		let schemas: Vec<SchemaId> = (0 .. s as u16).collect();
		T::SchemaValidator::set_schema_count(schemas.len().try_into().unwrap());

		let (provider_msa_id, _ ) = Msa::<T>::create_account(provider_caller.clone().into(), EMPTY_FUNCTION).unwrap();
		assert_ok!(Msa::<T>::create_provider(RawOrigin::Signed(provider_caller.clone()).into(), Vec::from("Foo")));

		let (payload, signature, delegator_key) = create_payload_and_signature::<T>(schemas, provider_msa_id);
		let (delegator_msa_id, delegator_public_key) = Msa::<T>::create_account(delegator_key.clone(), EMPTY_FUNCTION).unwrap();
	}: _ (RawOrigin::Signed(provider_caller), delegator_key, signature, payload)
	verify {
		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
		assert!(Msa::<T>::get_delegation(DelegatorId(delegator_msa_id), ProviderId(provider_msa_id)).is_some());
	}

	revoke_delegation_by_delegator {
		let provider_account = create_account::<T>("account", 0);
		let (provider_msa_id, provider) = Msa::<T>::create_account(provider_account.into(), EMPTY_FUNCTION).unwrap();

		let delegator_account = create_account::<T>("account", 1);
		let (delegator_msa_id, delegator_public_key) = Msa::<T>::create_account(delegator_account.into(), EMPTY_FUNCTION).unwrap();

		assert_ok!(Msa::<T>::add_provider(ProviderId(provider_msa_id), DelegatorId(delegator_msa_id), vec![]));
	}: _ (RawOrigin::Signed(delegator_public_key), provider_msa_id)
	verify {
		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
		assert_eq!(Msa::<T>::get_delegation(DelegatorId(delegator_msa_id), ProviderId(provider_msa_id)).unwrap().revoked_at, T::BlockNumber::from(1u32));
	}

	create_provider {
		let s in 0 .. T::MaxProviderNameSize::get();

		let provider_name = (1 .. s as u8).collect::<Vec<_>>();
		let account = create_account::<T>("account", 0);
		let (provider_msa_id, provider_public_key) = Msa::<T>::create_account(account.into(), EMPTY_FUNCTION).unwrap();
	}: _ (RawOrigin::Signed(provider_public_key), provider_name)
	verify {
		assert!(Msa::<T>::get_provider_registry_entry(ProviderId(provider_msa_id)).is_some());
	}

	create_provider_via_governance {
		let s = T::MaxProviderNameSize::get();

		let provider_name = (1 .. s as u8).collect::<Vec<_>>();
		let account = create_account::<T>("account", 0);
		let (provider_msa_id, provider_public_key) = Msa::<T>::create_account(account.into(), EMPTY_FUNCTION).unwrap();

	}:	_ (RawOrigin::Root, provider_public_key, provider_name)
	verify {
		assert!(Msa::<T>::is_registered_provider(provider_msa_id));
	}

	propose_to_be_provider {
		let s = T::MaxProviderNameSize::get();

		let provider_name = (1 .. s as u8).collect::<Vec<_>>();
		let account = create_account::<T>("account", 0);
		let (provider_msa_id, provider_public_key) = Msa::<T>::create_account(account.into(), EMPTY_FUNCTION).unwrap();

	}:	_ (RawOrigin::Signed(provider_public_key), provider_name)
	verify {
		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
	}

	on_initialize {
		// we should not need to max out storage for this benchmark, see:
		// https://substrate.stackexchange.com/a/4430/2060
		let m in 1 .. T::MaxSignaturesPerBucket::get();
		for j in 0 .. m {
			let mortality_block = 49 as u32;
			let mut data = [0u8; 64];
			data[0..8].copy_from_slice(&(m, j).encode());
			let multi_sig = MultiSignature::Sr25519(sp_core::sr25519::Signature::from_raw(data));
			assert_ok!(Msa::<T>::register_signature(&multi_sig, T::BlockNumber::from(mortality_block)));
		}

		let bucket_zero_iter = PayloadSignatureRegistry::<T>::iter_prefix(T::BlockNumber::from(0u32));
		assert_eq!(bucket_zero_iter.count(), m as usize);
	}: {
		Msa::<T>::on_initialize(100u32.into());
	} verify {
		let bucket_zero_iter = PayloadSignatureRegistry::<T>::iter_prefix(T::BlockNumber::from(0u32));
		assert_eq!(bucket_zero_iter.count(), 0 as usize);
	}

	grant_schema_permissions {
		let s in 0 .. T::MaxSchemaGrantsPerDelegation::get();

		let provider_account = create_account::<T>("account", 0);
		let (provider_msa_id, provider) = Msa::<T>::create_account(provider_account.into(), EMPTY_FUNCTION).unwrap();

		let delegator_account = create_account::<T>("account", 1);
		let (delegator_msa_id, delegator_public_key) = Msa::<T>::create_account(delegator_account.into(), EMPTY_FUNCTION).unwrap();

		let schema_ids: Vec<SchemaId> = (1..s as u16).collect::<Vec<_>>();
		T::SchemaValidator::set_schema_count(schema_ids.len().try_into().unwrap());

		assert_ok!(Msa::<T>::add_provider(ProviderId(provider_msa_id), DelegatorId(delegator_msa_id), vec![]));
	}: _ (RawOrigin::Signed(delegator_public_key), provider_msa_id, schema_ids.clone())
	verify {
		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
		assert_eq!(Msa::<T>::get_delegation(DelegatorId(delegator_msa_id), ProviderId(provider_msa_id)).unwrap().schema_permissions.len(), schema_ids.len() as usize);
	}

	revoke_schema_permissions {
		let s in 0 .. T::MaxSchemaGrantsPerDelegation::get();

		let provider_account = create_account::<T>("account", 0);
		let (provider_msa_id, _) = Msa::<T>::create_account(provider_account.into(), EMPTY_FUNCTION).unwrap();

		let delegator_account = create_account::<T>("account", 1);
		let (delegator_msa_id, delegator_public_key) = Msa::<T>::create_account(delegator_account.into(), EMPTY_FUNCTION).unwrap();

		let schema_ids: Vec<SchemaId> = (1..s as u16).collect::<Vec<_>>();
		T::SchemaValidator::set_schema_count(schema_ids.len().try_into().unwrap());

		assert_ok!(Msa::<T>::add_provider(ProviderId(provider_msa_id), DelegatorId(delegator_msa_id), schema_ids.clone()));
	}: _ (RawOrigin::Signed(delegator_public_key), provider_msa_id, schema_ids.clone())
	verify {
		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
	}

	impl_benchmark_test_suite!(Msa,
		crate::mock::new_test_ext_keystore(),
		crate::mock::Test);
}
