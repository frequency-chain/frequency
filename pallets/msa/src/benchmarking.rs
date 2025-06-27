#![allow(clippy::unwrap_used)]
use super::*;

#[allow(unused)]
use crate::Pallet as Msa;
use crate::{
	types::{RecoveryCommitmentPayload, EMPTY_FUNCTION},
	MsaIdToRecoveryCommitment,
};
use common_primitives::utils::wrap_binary_data;
use frame_benchmarking::{account, v2::*};
use frame_support::{assert_ok, traits::fungible::Inspect};
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
	(add_provider_payload, MultiSignature::Sr25519(signature.into()), acc)
}

fn add_key_payload_and_signature<T: Config>(
	msa_id: u64,
) -> (AddKeyData<T>, MultiSignature, T::AccountId) {
	let new_keys = SignerId::generate_pair(None);
	let public_key = T::AccountId::decode(&mut &new_keys.encode()[..]).unwrap();
	let add_key_payload =
		AddKeyData::<T> { msa_id, expiration: 10u32.into(), new_public_key: public_key };

	let encoded_add_key_payload = wrap_binary_data(add_key_payload.encode());

	let signature = new_keys.sign(&encoded_add_key_payload).unwrap();
	let acc = T::AccountId::decode(&mut &new_keys.encode()[..]).unwrap();
	(add_key_payload, MultiSignature::Sr25519(signature.into()), acc)
}

fn withdraw_tokens_payload_and_signature<T: Config>(
	msa_id: u64,
	msa_key_pair: SignerId,
) -> (AuthorizedKeyData<T>, MultiSignature, T::AccountId) {
	let new_keys = SignerId::generate_pair(None);
	let public_key = T::AccountId::decode(&mut &new_keys.encode()[..]).unwrap();
	let withdraw_tokens_payload = AuthorizedKeyData::<T> {
		discriminant: PayloadTypeDiscriminator::AuthorizedKeyData,
		msa_id,
		expiration: 10u32.into(),
		authorized_public_key: public_key,
	};

	let encoded_withdraw_tokens_payload = wrap_binary_data(withdraw_tokens_payload.encode());

	let signature = msa_key_pair.sign(&encoded_withdraw_tokens_payload).unwrap();
	let acc = T::AccountId::decode(&mut &new_keys.encode()[..]).unwrap();
	(withdraw_tokens_payload, MultiSignature::Sr25519(signature.into()), acc)
}

fn create_msa_account_and_keys<T: Config>() -> (T::AccountId, SignerId, MessageSourceId) {
	let key_pair = SignerId::generate_pair(None);
	let account_id = T::AccountId::decode(&mut &key_pair.encode()[..]).unwrap();

	let (msa_id, _) = Msa::<T>::create_account(account_id.clone(), EMPTY_FUNCTION).unwrap();

	(account_id, key_pair, msa_id)
}

fn generate_fake_signature(i: u8) -> MultiSignature {
	let sig = [i; 64];
	MultiSignature::Sr25519(sp_core::sr25519::Signature::from_raw(sig))
}

fn prep_signature_registry<T: Config>() {
	// Add it with an 0 block expiration
	let signatures: Vec<MultiSignature> = (1..=50u8).map(generate_fake_signature).collect();
	let signature_expires_at: BlockNumberFor<T> = 0u32.into();
	let len = signatures.len();
	for (i, sig) in signatures.iter().enumerate() {
		if i < (len - 1) {
			<PayloadSignatureRegistryList<T>>::insert(
				sig,
				(signature_expires_at, signatures[i + 1].clone()),
			);
		}
	}
	PayloadSignatureRegistryPointer::<T>::put(SignatureRegistryPointer {
		// The count doesn't change if list is full, so fake the count
		count: T::MaxSignaturesStored::get().unwrap_or(3),
		newest: signatures.last().unwrap().clone(),
		newest_expires_at: signature_expires_at,
		oldest: signatures.first().unwrap().clone(),
	});
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()));

		assert!(PublicKeyToMsaId::<T>::get(caller).is_some());
		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
		Ok(())
	}

	#[benchmark]
	fn create_sponsored_account_with_delegation(
		s: Linear<0, { T::MaxSchemaGrantsPerDelegation::get() }>,
	) -> Result<(), BenchmarkError> {
		prep_signature_registry::<T>();

		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(Msa::<T>::create(RawOrigin::Signed(caller.clone()).into()));
		assert_ok!(Msa::<T>::create_provider(
			RawOrigin::Signed(caller.clone()).into(),
			Vec::from("Foo")
		));

		let schemas: Vec<SchemaId> = (0..s as u16).collect();
		T::SchemaValidator::set_schema_count(schemas.len().try_into().unwrap());
		let (payload, signature, key) = create_payload_and_signature::<T>(schemas, 1u64);

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), key.clone(), signature, payload);

		assert!(PublicKeyToMsaId::<T>::get(key).is_some());
		Ok(())
	}

	#[benchmark]
	fn revoke_delegation_by_provider() -> Result<(), BenchmarkError> {
		let provider_account = create_account::<T>("account", 0);
		let (provider_msa_id, provider_public_key) =
			Msa::<T>::create_account(provider_account, EMPTY_FUNCTION).unwrap();

		let delegator_account = create_account::<T>("account", 1);
		let (delegator_msa_id, _) =
			Msa::<T>::create_account(delegator_account, EMPTY_FUNCTION).unwrap();

		assert_ok!(Msa::<T>::add_provider(
			ProviderId(provider_msa_id),
			DelegatorId(delegator_msa_id),
			vec![]
		));

		#[extrinsic_call]
		_(RawOrigin::Signed(provider_public_key), delegator_msa_id);

		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
		assert_eq!(
			DelegatorAndProviderToDelegation::<T>::get(
				DelegatorId(delegator_msa_id),
				ProviderId(provider_msa_id)
			)
			.unwrap()
			.revoked_at,
			BlockNumberFor::<T>::from(1u32)
		);
		Ok(())
	}

	#[benchmark]
	fn add_public_key_to_msa() -> Result<(), BenchmarkError> {
		prep_signature_registry::<T>();

		let (provider_public_key, _, _) = create_msa_account_and_keys::<T>();
		let (delegator_public_key, delegator_key_pair, delegator_msa_id) =
			create_msa_account_and_keys::<T>();

		let (add_key_payload, new_public_key_signature, new_public_key) =
			add_key_payload_and_signature::<T>(delegator_msa_id);

		let encoded_add_key_payload = wrap_binary_data(add_key_payload.encode());
		let owner_signature = MultiSignature::Sr25519(
			delegator_key_pair.sign(&encoded_add_key_payload).unwrap().into(),
		);

		#[extrinsic_call]
		_(
			RawOrigin::Signed(provider_public_key.clone()),
			delegator_public_key.clone(),
			owner_signature,
			new_public_key_signature,
			add_key_payload,
		);

		assert!(PublicKeyToMsaId::<T>::get(new_public_key).is_some());
		Ok(())
	}

	#[benchmark]
	fn delete_msa_public_key() -> Result<(), BenchmarkError> {
		frame_system::Pallet::<T>::set_block_number(1u32.into());
		prep_signature_registry::<T>();

		let (provider_public_key, _, _) = create_msa_account_and_keys::<T>();
		let (caller_and_delegator_public_key, delegator_key_pair, delegator_msa_id) =
			create_msa_account_and_keys::<T>();

		let (add_key_payload, new_public_key_signature, new_public_key) =
			add_key_payload_and_signature::<T>(delegator_msa_id);

		let encoded_add_key_payload = wrap_binary_data(add_key_payload.encode());
		let owner_signature = MultiSignature::Sr25519(
			delegator_key_pair.sign(&encoded_add_key_payload).unwrap().into(),
		);

		assert_ok!(Msa::<T>::add_public_key_to_msa(
			RawOrigin::Signed(provider_public_key).into(),
			caller_and_delegator_public_key.clone(),
			owner_signature,
			new_public_key_signature,
			add_key_payload
		));

		#[extrinsic_call]
		_(RawOrigin::Signed(caller_and_delegator_public_key), new_public_key.clone());

		assert!(PublicKeyToMsaId::<T>::get(new_public_key).is_none());
		Ok(())
	}

	#[benchmark]
	fn retire_msa() -> Result<(), BenchmarkError> {
		let (delegator_account, _, _) = create_msa_account_and_keys::<T>();

		#[block]
		{
			assert_ok!(Msa::<T>::retire_msa(RawOrigin::Signed(delegator_account.clone()).into()));
		}
		// Assert that the MSA has no accounts
		let key_count = PublicKeyCountForMsaId::<T>::get(1);
		assert_eq!(key_count, 0);
		Ok(())
	}

	#[benchmark]
	fn grant_delegation(
		s: Linear<0, { T::MaxSchemaGrantsPerDelegation::get() }>,
	) -> Result<(), BenchmarkError> {
		prep_signature_registry::<T>();

		let provider_caller: T::AccountId = whitelisted_caller();

		let schemas: Vec<SchemaId> = (0..s as u16).collect();
		T::SchemaValidator::set_schema_count(schemas.len().try_into().unwrap());

		let (provider_msa_id, _) =
			Msa::<T>::create_account(provider_caller.clone(), EMPTY_FUNCTION).unwrap();
		assert_ok!(Msa::<T>::create_provider(
			RawOrigin::Signed(provider_caller.clone()).into(),
			Vec::from("Foo")
		));

		let (payload, signature, delegator_key) =
			create_payload_and_signature::<T>(schemas, provider_msa_id);
		let (delegator_msa_id, _) =
			Msa::<T>::create_account(delegator_key.clone(), EMPTY_FUNCTION).unwrap();

		#[extrinsic_call]
		_(RawOrigin::Signed(provider_caller), delegator_key, signature, payload);

		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
		assert!(DelegatorAndProviderToDelegation::<T>::get(
			DelegatorId(delegator_msa_id),
			ProviderId(provider_msa_id)
		)
		.is_some());
		Ok(())
	}

	#[benchmark]
	fn revoke_delegation_by_delegator() -> Result<(), BenchmarkError> {
		let provider_account = create_account::<T>("account", 0);
		let (provider_msa_id, _) =
			Msa::<T>::create_account(provider_account, EMPTY_FUNCTION).unwrap();

		let delegator_account = create_account::<T>("account", 1);
		let (delegator_msa_id, delegator_public_key) =
			Msa::<T>::create_account(delegator_account, EMPTY_FUNCTION).unwrap();

		assert_ok!(Msa::<T>::add_provider(
			ProviderId(provider_msa_id),
			DelegatorId(delegator_msa_id),
			vec![]
		));

		#[extrinsic_call]
		_(RawOrigin::Signed(delegator_public_key), provider_msa_id);

		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
		assert_eq!(
			DelegatorAndProviderToDelegation::<T>::get(
				DelegatorId(delegator_msa_id),
				ProviderId(provider_msa_id)
			)
			.unwrap()
			.revoked_at,
			BlockNumberFor::<T>::from(1u32)
		);
		Ok(())
	}

	#[benchmark]
	fn create_provider() -> Result<(), BenchmarkError> {
		let s = T::MaxProviderNameSize::get();

		let provider_name = (1..s as u8).collect::<Vec<_>>();
		let account = create_account::<T>("account", 0);
		let (provider_msa_id, provider_public_key) =
			Msa::<T>::create_account(account, EMPTY_FUNCTION).unwrap();

		#[extrinsic_call]
		_(RawOrigin::Signed(provider_public_key), provider_name);

		assert!(ProviderToRegistryEntry::<T>::get(ProviderId(provider_msa_id)).is_some());
		Ok(())
	}

	#[benchmark]
	fn create_provider_via_governance() -> Result<(), BenchmarkError> {
		let s = T::MaxProviderNameSize::get();

		let provider_name = (1..s as u8).collect::<Vec<_>>();
		let account = create_account::<T>("account", 0);
		let (provider_msa_id, provider_public_key) =
			Msa::<T>::create_account(account, EMPTY_FUNCTION).unwrap();

		#[extrinsic_call]
		_(RawOrigin::Root, provider_public_key, provider_name);

		assert!(Msa::<T>::is_registered_provider(provider_msa_id));
		Ok(())
	}

	#[benchmark]
	fn propose_to_be_provider() -> Result<(), BenchmarkError> {
		let s = T::MaxProviderNameSize::get();

		let provider_name = (1..s as u8).collect::<Vec<_>>();
		let account = create_account::<T>("account", 0);
		let (_, provider_public_key) = Msa::<T>::create_account(account, EMPTY_FUNCTION).unwrap();

		#[extrinsic_call]
		_(RawOrigin::Signed(provider_public_key), provider_name);

		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
		Ok(())
	}

	#[benchmark]
	fn reindex_offchain() -> Result<(), BenchmarkError> {
		let key = create_account::<T>("account", 0);
		let caller = whitelisted_caller();
		let msa_id = 1u64;
		let event = OffchainReplayEvent::MsaPallet(MsaOffchainReplayEvent::KeyReIndex {
			msa_id,
			index_key: Some(key),
		});

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), event);

		Ok(())
	}

	#[benchmark]
	fn withdraw_tokens() -> Result<(), BenchmarkError> {
		prep_signature_registry::<T>();

		let (msa_public_key, msa_key_pair, msa_id) = create_msa_account_and_keys::<T>();

		let eth_account_id: H160 = Msa::<T>::msa_id_to_eth_address(msa_id);
		let mut bytes = &EthereumAddressMapper::to_bytes32(&eth_account_id.0)[..];
		let msa_account_id = <T as frame_system::Config>::AccountId::decode(&mut bytes).unwrap();

		// Fund MSA
		// let balance = <<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance.from(10_000_000u128);
		let balance = <T as Config>::Currency::minimum_balance();
		T::Currency::set_balance(&msa_account_id, balance);
		assert_eq!(T::Currency::balance(&msa_account_id), balance);

		let (add_key_payload, owner_signature, new_account_id) =
			withdraw_tokens_payload_and_signature::<T>(msa_id, msa_key_pair);

		#[extrinsic_call]
		_(
			RawOrigin::Signed(new_account_id.clone()),
			msa_public_key.clone(),
			owner_signature,
			add_key_payload,
		);

		assert_eq!(T::Currency::balance(&msa_account_id), Zero::zero());
		Ok(())
	}

	#[benchmark]
	fn add_recovery_commitment() -> Result<(), BenchmarkError> {
		prep_signature_registry::<T>();

		let (msa_public_key, msa_key_pair, msa_id) = create_msa_account_and_keys::<T>();
		let provider_caller: T::AccountId = whitelisted_caller();

		// Create a recovery commitment
		let recovery_commitment = [1u8; 32];
		let expiration = 10u32.into();

		// Create the payload
		let payload = RecoveryCommitmentPayload::<T> {
			discriminant: PayloadTypeDiscriminator::RecoveryCommitmentPayload,
			recovery_commitment,
			expiration,
		};
		// Sign the payload with the MSA owner key
		let encoded_payload = wrap_binary_data(payload.encode());
		let signature =
			MultiSignature::Sr25519(msa_key_pair.sign(&encoded_payload).unwrap().into());

		#[extrinsic_call]
		_(RawOrigin::Signed(provider_caller), msa_public_key.clone(), signature, payload);

		// Verify the commitment was stored
		assert!(MsaIdToRecoveryCommitment::<T>::get(msa_id).is_some());
		Ok(())
	}

	impl_benchmark_test_suite!(
		Msa,
		crate::tests::mock::new_test_ext_keystore(),
		crate::tests::mock::Test
	);
}
