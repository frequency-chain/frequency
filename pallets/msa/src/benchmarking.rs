#![allow(clippy::unwrap_used)]
use super::*;

#[allow(unused)]
use crate::Pallet as Msa;
use crate::{
	types::{compute_cid, RecoveryCommitmentPayload},
	MsaIdToRecoveryCommitment,
};
use common_primitives::{
	msa::ProviderRegistryEntry,
	utils::{wrap_binary_data, XorRng},
};
use frame_benchmarking::{account, v2::*};
use frame_support::{
	assert_ok,
	traits::{fungible::Inspect, Get},
};
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

	let (msa_id, _) = Msa::<T>::create_account(account_id.clone()).unwrap();

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

// Pre-populate MSA and recovery provider storage for recover_account benchmark
fn prep_recovery_benchmark_storage<T: Config>(
) -> (T::AccountId, SignerId, MessageSourceId, T::AccountId, [u8; 32]) {
	let msa_id = 1u64;
	let provider_msa_id = 2u64;

	// Pre-create MSA account directly in storage
	let key_pair = SignerId::generate_pair(None);
	let msa_account = T::AccountId::decode(&mut &key_pair.encode()[..])
		.expect("Key pair should decode to AccountId");

	// Populate MSA storage directly
	PublicKeyToMsaId::<T>::insert(&msa_account, msa_id);
	PublicKeyCountForMsaId::<T>::insert(msa_id, 1u8);
	CurrentMsaIdentifierMaximum::<T>::put(msa_id);

	// Pre-create and approve recovery provider directly in storage
	let provider_account = create_account::<T>("recovery_provider", 0);

	// Populate provider storage directly
	PublicKeyToMsaId::<T>::insert(&provider_account, provider_msa_id);
	PublicKeyCountForMsaId::<T>::insert(provider_msa_id, 1u8);

	// Register as provider directly in storage
	use common_primitives::msa::{ProviderId, ProviderRegistryEntry};
	use frame_support::BoundedVec;
	let provider_name =
		BoundedVec::try_from(b"RecoveryPro".to_vec()).expect("Provider name should fit in bounds");
	let cid = "bafkreidgvpkjawlxz6sffxzwgooowe5yt7i6wsyg236mfoks77nywkptdq"
		.as_bytes()
		.to_vec();
	let entry = ProviderRegistryEntry {
		default_name: provider_name,
		localized_names: BoundedBTreeMap::new(),
		default_logo_250_100_png_cid: BoundedVec::try_from(cid)
			.expect("Logo CID should fit in bounds"),
		localized_logo_250_100_png_cids: BoundedBTreeMap::new(),
	};
	ProviderToRegistryEntry::<T>::insert(ProviderId(provider_msa_id), entry);

	// Pre-approve as recovery provider directly in storage
	RecoveryProviders::<T>::insert(ProviderId(provider_msa_id), true);

	// Pre-populate recovery commitment directly in storage
	let (intermediary_hash_a, intermediary_hash_b) = get_benchmark_recovery_hashes();
	let recovery_commitment =
		Msa::<T>::compute_recovery_commitment(intermediary_hash_a, intermediary_hash_b);
	MsaIdToRecoveryCommitment::<T>::insert(msa_id, recovery_commitment);

	(msa_account, key_pair, msa_id, provider_account, recovery_commitment)
}

// Cached hash computation for recovery benchmarks
fn get_benchmark_recovery_hashes() -> ([u8; 32], [u8; 32]) {
	use sp_core::keccak_256;

	// Pre-computed values to avoid hash operations during benchmarking
	const RECOVERY_SECRET_HEX: &str =
		"ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789";
	const CONTACT: &str = "user@example.com";

	let recovery_secret_bytes =
		hex::decode(RECOVERY_SECRET_HEX).expect("Recovery secret should be valid hex");

	let hash_a = keccak_256(&recovery_secret_bytes);

	let mut combined = Vec::new();
	combined.extend_from_slice(&recovery_secret_bytes);
	combined.extend_from_slice(CONTACT.as_bytes());
	let hash_b = keccak_256(&combined);

	(hash_a, hash_b)
}

// Helper function to create a language code of a given length
fn make_lang_code(mut i: usize, len: usize) -> Vec<u8> {
	let mut code = vec![b'a'; len];
	for j in (0..len).rev() {
		code[j] = b'a' + (i % 26) as u8;
		i /= 26;
	}
	code
}

fn generate_provider_registry_entry<T: Config>(
	names_len: usize,
	logos_len: usize,
	provider_name: Vec<u8>,
	seed: u64,
	add_to_db: bool,
) -> ProviderRegistryEntry<
	T::MaxProviderNameSize,
	T::MaxLanguageCodeSize,
	T::MaxLogoCidSize,
	T::MaxLocaleCount,
> {
	let mut rng = XorRng::new(seed);
	let mut localized_names = BoundedBTreeMap::new();
	let mut localized_cids = BoundedBTreeMap::new();

	let default_logo_bytes =
		(0..T::MaxLogoSize::get() as u8).map(|_| rng.gen_u8()).collect::<Vec<_>>();
	let default_cid: BoundedVec<_, _> =
		compute_cid(default_logo_bytes.as_slice()).try_into().unwrap();

	if add_to_db {
		// Insert default logo in approved logos
		ApprovedLogos::<T>::insert(
			&default_cid,
			BoundedVec::try_from(default_logo_bytes.clone()).unwrap(),
		);
	}

	// Set up localized names based on parameter n
	for i in 0..names_len {
		let lang: BoundedVec<_, _> =
			make_lang_code(i, T::MaxLanguageCodeSize::get() as usize).try_into().unwrap();
		let name = BoundedVec::try_from(provider_name.clone()).unwrap_or_default();
		localized_names.try_insert(lang.clone(), name).unwrap();
	}

	for i in 0..logos_len {
		let logo_bytes = (0..T::MaxLogoSize::get() as u8).map(|_| rng.gen_u8()).collect::<Vec<_>>();
		let bounded_cid = compute_cid(&logo_bytes).try_into().unwrap();

		if add_to_db {
			// Insert all CIDs in approved logo storage
			ApprovedLogos::<T>::insert(&bounded_cid, BoundedVec::try_from(logo_bytes).unwrap());
		}

		let lang = make_lang_code(i, T::MaxLanguageCodeSize::get() as usize).try_into().unwrap();
		localized_cids.try_insert(lang, bounded_cid).unwrap();
	}

	ProviderRegistryEntry {
		default_name: BoundedVec::try_from(provider_name.clone()).unwrap_or_default(),
		localized_names: localized_names.clone(),
		default_logo_250_100_png_cid: default_cid.clone(),
		localized_logo_250_100_png_cids: localized_cids.clone(),
	}
}

#[benchmarks(where
	T: Config + Send + Sync,
)]
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
		let entry = ProviderRegistryEntry {
			default_name: BoundedVec::truncate_from(Vec::from("Foo")),
			localized_names: BoundedBTreeMap::new(),
			default_logo_250_100_png_cid: BoundedVec::new(),
			localized_logo_250_100_png_cids: BoundedBTreeMap::new(),
		};

		assert_ok!(Msa::<T>::create_provider_via_governance_v2(
			RawOrigin::Root.into(),
			caller.clone(),
			entry
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
			Msa::<T>::create_account(provider_account).unwrap();

		let delegator_account = create_account::<T>("account", 1);
		let (delegator_msa_id, _) = Msa::<T>::create_account(delegator_account).unwrap();

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

		let (provider_msa_id, _) = Msa::<T>::create_account(provider_caller.clone()).unwrap();
		let entry = ProviderRegistryEntry::default();

		assert_ok!(Msa::<T>::create_provider_via_governance_v2(
			RawOrigin::Root.into(),
			provider_caller.clone(),
			entry
		));

		let (payload, signature, delegator_key) =
			create_payload_and_signature::<T>(schemas, provider_msa_id);
		let (delegator_msa_id, _) = Msa::<T>::create_account(delegator_key.clone()).unwrap();

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
		let (provider_msa_id, _) = Msa::<T>::create_account(provider_account).unwrap();

		let delegator_account = create_account::<T>("account", 1);
		let (delegator_msa_id, delegator_public_key) =
			Msa::<T>::create_account(delegator_account).unwrap();

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
		let (provider_msa_id, provider_public_key) = Msa::<T>::create_account(account).unwrap();

		#[extrinsic_call]
		_(RawOrigin::Signed(provider_public_key), provider_name);

		assert!(ProviderToRegistryEntry::<T>::get(ProviderId(provider_msa_id)).is_some());
		Ok(())
	}

	#[benchmark]
	fn create_provider_via_governance_v2(
		n: Linear<0, { T::MaxLocaleCount::get() }>,
		m: Linear<0, { T::MaxLocaleCount::get() }>,
	) -> Result<(), BenchmarkError> {
		let s = T::MaxProviderNameSize::get();
		let lang_size = T::MaxLanguageCodeSize::get();

		let provider_name = (1..s as u8).collect::<Vec<_>>();
		let cid = "bafkreidgvpkjawlxz6sffxzwgooowe5yt7i6wsyg236mfoks77nywkptdq"
			.as_bytes()
			.to_vec();
		let mut localized_names = BoundedBTreeMap::new();
		let mut localized_cids = BoundedBTreeMap::new();
		for i in 0..n {
			let lang_code = make_lang_code(i as usize, lang_size as usize);
			let lang = BoundedVec::try_from(lang_code).unwrap();
			let name = BoundedVec::try_from(provider_name.clone()).unwrap_or_default();
			localized_names.try_insert(lang.clone(), name).unwrap();
		}
		for i in 0..m {
			let lang_code = make_lang_code(i as usize, lang_size as usize);
			let lang = BoundedVec::try_from(lang_code).unwrap();
			let logo = BoundedVec::try_from(cid.clone()).unwrap();
			localized_cids.try_insert(lang, logo).unwrap();
		}
		let account = create_account::<T>("account", 0);
		let (provider_msa_id, provider_public_key) = Msa::<T>::create_account(account).unwrap();

		let entry = ProviderRegistryEntry {
			default_name: BoundedVec::try_from(provider_name).unwrap_or_default(),
			localized_names,
			default_logo_250_100_png_cid: BoundedVec::try_from(cid).unwrap(),
			localized_logo_250_100_png_cids: localized_cids,
		};
		#[extrinsic_call]
		_(RawOrigin::Root, provider_public_key, entry);

		assert!(Msa::<T>::is_registered_provider(provider_msa_id));
		Ok(())
	}

	#[benchmark]
	fn propose_to_be_provider_v2() -> Result<(), BenchmarkError> {
		let s = T::MaxProviderNameSize::get();
		let lang_size = T::MaxLanguageCodeSize::get();

		let provider_name = (1..s as u8).collect::<Vec<_>>();
		let cid = "bafkreidgvpkjawlxz6sffxzwgooowe5yt7i6wsyg236mfoks77nywkptdq"
			.as_bytes()
			.to_vec();
		let mut localized_names = BoundedBTreeMap::new();
		let mut localized_cids = BoundedBTreeMap::new();
		let (m, n) = (T::MaxLocaleCount::get(), T::MaxLocaleCount::get());
		for i in 0..n {
			let lang_code = make_lang_code(i as usize, lang_size as usize);
			let lang = BoundedVec::try_from(lang_code).unwrap();
			let name = BoundedVec::try_from(provider_name.clone()).unwrap_or_default();
			localized_names.try_insert(lang.clone(), name).unwrap();
		}
		for i in 0..m {
			let lang_code = make_lang_code(i as usize, lang_size as usize);
			let lang = BoundedVec::try_from(lang_code).unwrap();
			let logo = BoundedVec::try_from(cid.clone()).unwrap();
			localized_cids.try_insert(lang, logo).unwrap();
		}
		let account = create_account::<T>("account", 0);
		let (_, provider_public_key) = Msa::<T>::create_account(account).unwrap();

		let entry = ProviderRegistryEntry {
			default_name: BoundedVec::try_from(provider_name).unwrap_or_default(),
			localized_names,
			default_logo_250_100_png_cid: BoundedVec::try_from(cid).unwrap(),
			localized_logo_250_100_png_cids: localized_cids,
		};
		#[extrinsic_call]
		_(RawOrigin::Signed(provider_public_key), entry);

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
		let signature = MultiSignature::Sr25519(
			msa_key_pair.sign(&encoded_payload).expect("Signing should succeed").into(),
		);

		#[extrinsic_call]
		_(RawOrigin::Signed(provider_caller), msa_public_key.clone(), signature, payload);

		// Verify the commitment was stored
		assert!(MsaIdToRecoveryCommitment::<T>::get(msa_id).is_some());
		Ok(())
	}

	#[benchmark]
	fn remove_recovery_provider() -> Result<(), BenchmarkError> {
		let account = create_account::<T>("account", 0);
		let (provider_msa_id, _provider_public_key) =
			Msa::<T>::create_account(account.clone()).unwrap();
		let entry = ProviderRegistryEntry {
			default_name: BoundedVec::truncate_from(Vec::from("Foo")),
			localized_names: BoundedBTreeMap::new(),
			default_logo_250_100_png_cid: BoundedVec::new(),
			localized_logo_250_100_png_cids: BoundedBTreeMap::new(),
		};
		assert_ok!(Msa::<T>::upsert_provider_for(provider_msa_id, entry, false));

		#[extrinsic_call]
		_(RawOrigin::Root, ProviderId(provider_msa_id));

		assert!(RecoveryProviders::<T>::get(ProviderId(provider_msa_id)).is_none());

		Ok(())
	}

	#[benchmark]
	fn approve_recovery_provider() -> Result<(), BenchmarkError> {
		let account = create_account::<T>("account", 0);
		let (provider_msa_id, provider_public_key) =
			Msa::<T>::create_account(account.clone()).unwrap();
		let entry = ProviderRegistryEntry {
			default_name: BoundedVec::truncate_from(Vec::from("Foo")),
			localized_names: BoundedBTreeMap::new(),
			default_logo_250_100_png_cid: BoundedVec::new(),
			localized_logo_250_100_png_cids: BoundedBTreeMap::new(),
		};
		assert_ok!(Msa::<T>::upsert_provider_for(provider_msa_id, entry, false));

		assert!(ProviderToRegistryEntry::<T>::get(ProviderId(provider_msa_id)).is_some());

		#[extrinsic_call]
		_(RawOrigin::Root, provider_public_key);

		assert!(RecoveryProviders::<T>::get(ProviderId(provider_msa_id)).is_some());

		Ok(())
	}

	#[benchmark]
	fn check_free_extrinsic_use_revoke_delegation_by_provider() -> Result<(), BenchmarkError> {
		let provider_account = create_account::<T>("provider", 0);
		let (provider_msa_id, provider_public_key) =
			Msa::<T>::create_account(provider_account).unwrap();

		let delegator_account = create_account::<T>("delegator", 1);
		let (delegator_msa_id, _) = Msa::<T>::create_account(delegator_account).unwrap();

		assert_ok!(Msa::<T>::add_provider(
			ProviderId(provider_msa_id),
			DelegatorId(delegator_msa_id),
			vec![]
		));

		#[block]
		{
			let _ = crate::CheckFreeExtrinsicUse::<T>::validate_delegation_by_provider(
				&provider_public_key,
				&delegator_msa_id,
			);
		}
		Ok(())
	}

	#[benchmark]
	fn check_free_extrinsic_use_revoke_delegation_by_delegator() -> Result<(), BenchmarkError> {
		let provider_account = create_account::<T>("provider", 0);
		let (provider_msa_id, _) = Msa::<T>::create_account(provider_account).unwrap();

		let delegator_account = create_account::<T>("delegator", 1);
		let (delegator_msa_id, delegator_public_key) =
			Msa::<T>::create_account(delegator_account).unwrap();

		assert_ok!(Msa::<T>::add_provider(
			ProviderId(provider_msa_id),
			DelegatorId(delegator_msa_id),
			vec![]
		));

		#[block]
		{
			let _ = crate::CheckFreeExtrinsicUse::<T>::validate_delegation_by_delegator(
				&delegator_public_key,
				&provider_msa_id,
			);
		}
		Ok(())
	}

	#[benchmark]
	fn check_free_extrinsic_use_delete_msa_public_key() -> Result<(), BenchmarkError> {
		let (original_key, _original_key_pair, msa_id) = create_msa_account_and_keys::<T>();
		let new_keys = SignerId::generate_pair(None);
		let new_key: T::AccountId = T::AccountId::decode(&mut &new_keys.encode()[..]).unwrap();
		assert_ok!(Msa::<T>::add_key(msa_id, &new_key));

		#[block]
		{
			let _ = crate::CheckFreeExtrinsicUse::<T>::validate_key_delete(&new_key, &original_key);
		}
		Ok(())
	}

	#[benchmark]
	fn check_free_extrinsic_use_retire_msa() -> Result<(), BenchmarkError> {
		let (key, _key_pair, _msa_id) = create_msa_account_and_keys::<T>();
		// Set up state so MSA can be retired (no provider, one key, no balance, no delegations)
		#[block]
		{
			let _ = crate::CheckFreeExtrinsicUse::<T>::ensure_msa_can_retire(&key);
		}
		Ok(())
	}

	#[benchmark]
	fn check_free_extrinsic_use_withdraw_tokens() -> Result<(), BenchmarkError> {
		prep_signature_registry::<T>();
		let (msa_public_key, msa_key_pair, msa_id) = create_msa_account_and_keys::<T>();
		let (add_key_payload, owner_signature, new_account_id) =
			withdraw_tokens_payload_and_signature::<T>(msa_id, msa_key_pair);
		#[block]
		{
			let _ = crate::CheckFreeExtrinsicUse::<T>::validate_msa_token_withdrawal(
				&new_account_id,
				&msa_public_key,
				&owner_signature,
				&add_key_payload,
			);
		}
		Ok(())
	}

	#[benchmark]
	fn recover_account() -> Result<(), BenchmarkError> {
		frame_system::Pallet::<T>::set_block_number(1u32.into());
		prep_signature_registry::<T>();

		// Use pre-populated storage with existing recovery commitment
		let (_msa_account, _msa_key_pair, msa_id, provider_account, _recovery_commitment) =
			prep_recovery_benchmark_storage::<T>();

		// Verify the recovery commitment already exists
		assert!(MsaIdToRecoveryCommitment::<T>::get(msa_id).is_some());

		// Use pre-computed hash values for efficiency
		let (intermediary_hash_a, intermediary_hash_b) = get_benchmark_recovery_hashes();

		// Generate a new control key for recovery
		let new_control_key_pair = SignerId::generate_pair(None);
		let new_control_key = T::AccountId::decode(&mut &new_control_key_pair.encode()[..])
			.expect("New control key pair should decode to AccountId");

		let expiration = 10u32.into();
		// Create AddKeyData payload and sign it with the new control key
		let add_key_payload =
			AddKeyData::<T> { msa_id, expiration, new_public_key: new_control_key.clone() };

		let encoded_add_key_payload = wrap_binary_data(add_key_payload.encode());
		let new_control_key_proof = MultiSignature::Sr25519(
			new_control_key_pair
				.sign(&encoded_add_key_payload)
				.expect("Signing should succeed")
				.into(),
		);

		#[extrinsic_call]
		_(
			RawOrigin::Signed(provider_account),
			intermediary_hash_a,
			intermediary_hash_b,
			new_control_key_proof,
			add_key_payload,
		);

		// Verify the recovery was successful
		assert!(PublicKeyToMsaId::<T>::get(&new_control_key).is_some());
		assert!(MsaIdToRecoveryCommitment::<T>::get(msa_id).is_none());

		Ok(())
	}

	#[benchmark]
	fn propose_to_add_application(
		n: Linear<0, { T::MaxLocaleCount::get() }>,
		m: Linear<0, { T::MaxLocaleCount::get() }>,
	) -> Result<(), BenchmarkError> {
		let application_name = (1..T::MaxProviderNameSize::get() as u8).collect::<Vec<_>>();

		let provider_caller: T::AccountId = whitelisted_caller();
		let (_, provider_public_key) = Msa::<T>::create_account(provider_caller.clone()).unwrap();

		let entry = ProviderRegistryEntry::default();
		assert_ok!(Msa::<T>::create_provider_via_governance_v2(
			RawOrigin::Root.into(),
			provider_caller.clone(),
			entry
		));
		let application_payload = generate_provider_registry_entry::<T>(
			n as usize,
			m as usize,
			application_name.clone(),
			1001111u64,
			false,
		);

		#[extrinsic_call]
		_(RawOrigin::Signed(provider_public_key), application_payload);

		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
		Ok(())
	}

	#[benchmark]
	fn create_application_via_governance(
		n: Linear<0, { T::MaxLocaleCount::get() }>,
		m: Linear<0, { T::MaxLocaleCount::get() }>,
	) -> Result<(), BenchmarkError> {
		let s = T::MaxProviderNameSize::get();
		let lang_size = T::MaxLanguageCodeSize::get();

		let application_name = (1..s as u8).collect::<Vec<_>>();
		let cid = "bafkreidgvpkjawlxz6sffxzwgooowe5yt7i6wsyg236mfoks77nywkptdq"
			.as_bytes()
			.to_vec();

		let mut localized_names = BoundedBTreeMap::new();
		let mut localized_cids = BoundedBTreeMap::new();

		for i in 0..n {
			let lang_code = make_lang_code(i as usize, lang_size as usize);
			let lang = BoundedVec::try_from(lang_code).unwrap();
			let name = BoundedVec::try_from(application_name.clone()).unwrap_or_default();
			localized_names.try_insert(lang.clone(), name).unwrap();
		}
		for i in 0..m {
			let lang_code = make_lang_code(i as usize, lang_size as usize);
			let lang = BoundedVec::try_from(lang_code).unwrap();
			let logo = BoundedVec::try_from(cid.clone()).unwrap();
			localized_cids.try_insert(lang, logo).unwrap();
		}

		let provider_caller: T::AccountId = whitelisted_caller();
		let (provider_id, provider_public_key) =
			Msa::<T>::create_account(provider_caller.clone()).unwrap();
		let entry = ProviderRegistryEntry::default();

		assert_ok!(Msa::<T>::create_provider_via_governance_v2(
			RawOrigin::Root.into(),
			provider_caller.clone(),
			entry
		));
		let application_payload = ApplicationContext {
			default_name: BoundedVec::try_from(application_name).unwrap_or_default(),
			localized_names,
			default_logo_250_100_png_cid: BoundedVec::try_from(cid).unwrap(),
			localized_logo_250_100_png_cids: localized_cids,
		};

		#[extrinsic_call]
		_(RawOrigin::Root, provider_public_key, application_payload);

		assert_eq!(NextApplicationIndex::<T>::get(ProviderId(provider_id)), 1);
		assert!(ProviderToApplicationRegistry::<T>::get(ProviderId(provider_id), 0).is_some());
		Ok(())
	}

	#[benchmark]
	fn upload_logo() -> Result<(), BenchmarkError> {
		let max_logo_size = T::MaxLogoSize::get();
		let max_logo_bytes = vec![0u8; max_logo_size as usize];
		let logo_cid = compute_cid(&max_logo_bytes);
		let provider_caller: T::AccountId = whitelisted_caller();
		let (_, provider_public_key) = Msa::<T>::create_account(provider_caller.clone()).unwrap();
		let entry = ProviderRegistryEntry::default();

		assert_ok!(Msa::<T>::create_provider_via_governance_v2(
			RawOrigin::Root.into(),
			provider_caller.clone(),
			entry
		));

		let input_bounded_cid = BoundedVec::try_from(logo_cid).unwrap();
		let input_bounded_logo = BoundedVec::try_from(max_logo_bytes).unwrap();
		ApprovedLogos::<T>::insert(&input_bounded_cid, BoundedVec::new());

		#[extrinsic_call]
		_(
			RawOrigin::Signed(provider_public_key),
			input_bounded_cid.clone(),
			input_bounded_logo.clone(),
		);

		assert!(ApprovedLogos::<T>::get(input_bounded_cid.clone()).is_some());
		let stored_logo_bytes = ApprovedLogos::<T>::get(&input_bounded_cid).unwrap();
		assert_eq!(stored_logo_bytes, input_bounded_logo);
		Ok(())
	}

	#[benchmark]
	fn propose_to_update_provider(
		n: Linear<0, { T::MaxLocaleCount::get() }>,
		m: Linear<0, { T::MaxLocaleCount::get() }>,
	) -> Result<(), BenchmarkError> {
		let provider_name = (1..T::MaxProviderNameSize::get() as u8).collect::<Vec<_>>();

		let account = create_account::<T>("account", 0);
		let (_, provider_public_key) = Msa::<T>::create_account(account).unwrap();

		let entry = generate_provider_registry_entry::<T>(
			n as usize,
			m as usize,
			provider_name.clone(),
			1001111u64,
			false,
		);

		// Must be an already registered provider to propose update
		assert_ok!(Msa::<T>::create_provider_via_governance_v2(
			RawOrigin::Root.into(),
			provider_public_key.clone(),
			ProviderRegistryEntry::default()
		));

		#[extrinsic_call]
		_(RawOrigin::Signed(provider_public_key), entry);

		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
		Ok(())
	}

	#[benchmark]
	fn update_application_via_governance(
		n: Linear<0, { T::MaxLocaleCount::get() }>,
		m: Linear<0, { T::MaxLocaleCount::get() }>,
	) -> Result<(), BenchmarkError> {
		let application_name = (1..T::MaxProviderNameSize::get() as u8).collect::<Vec<_>>();

		let provider_caller: T::AccountId = whitelisted_caller();
		let (provider_id, provider_public_key) =
			Msa::<T>::create_account(provider_caller.clone()).unwrap();

		let provider_entry = ProviderRegistryEntry {
			default_name: BoundedVec::try_from(b"Test Provider".to_vec()).unwrap_or_default(),
			..Default::default()
		};

		assert_ok!(Msa::<T>::create_provider_via_governance_v2(
			RawOrigin::Root.into(),
			provider_caller.clone(),
			provider_entry
		));

		// Create initial application with maximum number of localized logos
		let initial_payload = generate_provider_registry_entry::<T>(
			T::MaxLocaleCount::get() as usize,
			T::MaxLocaleCount::get() as usize,
			b"init".to_vec(),
			10101111u64,
			true,
		);

		// Create an initial application to update with maximum logos for worst-case removal
		assert_ok!(Msa::<T>::create_application_via_governance(
			RawOrigin::Root.into(),
			provider_public_key.clone(),
			initial_payload.clone()
		));

		let stored_application =
			ProviderToApplicationRegistry::<T>::get(ProviderId(provider_id), 0).unwrap();
		assert_eq!(
			stored_application.default_logo_250_100_png_cid,
			initial_payload.default_logo_250_100_png_cid
		);
		assert_eq!(stored_application.localized_names.len() as u32, T::MaxLocaleCount::get());
		assert_eq!(
			stored_application.localized_logo_250_100_png_cids.len() as u32,
			T::MaxLocaleCount::get()
		);

		// Create new entry for update (could have different number of logos)
		let application_payload = generate_provider_registry_entry::<T>(
			n as usize,
			m as usize,
			application_name.clone(),
			1001111u64,
			true,
		);

		#[extrinsic_call]
		_(RawOrigin::Root, provider_public_key, 0u16, application_payload.clone());

		// Verify the update was successful
		assert!(ProviderToApplicationRegistry::<T>::get(ProviderId(provider_id), 0).is_some());
		assert_eq!(
			ProviderToApplicationRegistry::<T>::get(ProviderId(provider_id), 0)
				.unwrap()
				.localized_names,
			application_payload.localized_names
		);

		Ok(())
	}

	#[benchmark]
	fn propose_to_update_application(
		n: Linear<0, { T::MaxLocaleCount::get() }>,
		m: Linear<0, { T::MaxLocaleCount::get() }>,
	) -> Result<(), BenchmarkError> {
		let application_name = (1..T::MaxProviderNameSize::get() as u8).collect::<Vec<_>>();

		let provider_caller: T::AccountId = whitelisted_caller();
		let (_, provider_public_key) = Msa::<T>::create_account(provider_caller.clone()).unwrap();
		assert_ok!(Msa::<T>::create_provider_via_governance_v2(
			RawOrigin::Root.into(),
			provider_caller.clone(),
			ProviderRegistryEntry::default()
		));

		// create an initial application to update
		let initial_payload = ApplicationContext::default();
		assert_ok!(Msa::<T>::create_application_via_governance(
			RawOrigin::Root.into(),
			provider_public_key.clone(),
			initial_payload
		));

		let application_payload = generate_provider_registry_entry::<T>(
			n as usize,
			m as usize,
			application_name.clone(),
			1001111u64,
			false,
		);

		#[extrinsic_call]
		_(RawOrigin::Signed(provider_public_key), 0u16, application_payload);

		assert_eq!(frame_system::Pallet::<T>::events().len(), 1);
		Ok(())
	}

	#[benchmark]
	fn update_provider_via_governance(
		n: Linear<0, { T::MaxLocaleCount::get() }>,
		m: Linear<0, { T::MaxLocaleCount::get() }>,
	) -> Result<(), BenchmarkError> {
		let account = create_account::<T>("account_updated", 0);
		let (provider_msa_id, provider_public_key) = Msa::<T>::create_account(account).unwrap();

		let provider_name = (1..T::MaxProviderNameSize::get() as u8).collect::<Vec<_>>();
		let initial_provider_entry = generate_provider_registry_entry::<T>(
			T::MaxLocaleCount::get() as usize,
			T::MaxLocaleCount::get() as usize,
			provider_name.clone(),
			10101111u64,
			true,
		);

		// Register provider with maximum logos to simulate worst-case removal scenario
		assert_ok!(Msa::<T>::create_provider_via_governance_v2(
			RawOrigin::Root.into(),
			provider_public_key.clone(),
			initial_provider_entry.clone()
		));

		let stored_provider =
			ProviderToRegistryEntry::<T>::get(ProviderId(provider_msa_id)).unwrap();
		assert_eq!(
			stored_provider.default_logo_250_100_png_cid,
			initial_provider_entry.default_logo_250_100_png_cid
		);
		assert_eq!(stored_provider.localized_names.len() as u32, T::MaxLocaleCount::get());
		assert_eq!(
			stored_provider.localized_logo_250_100_png_cids.len() as u32,
			T::MaxLocaleCount::get()
		);

		// Create new entry for update (could have different number of logos)
		let entry = generate_provider_registry_entry::<T>(
			n as usize,
			m as usize,
			provider_name.clone(),
			1001111u64,
			true,
		);

		#[extrinsic_call]
		_(RawOrigin::Root, provider_public_key, entry.clone());

		// Verify the update was successful
		assert!(Msa::<T>::is_registered_provider(provider_msa_id));
		let provider_entry = ProviderToRegistryEntry::<T>::get(ProviderId(provider_msa_id))
			.expect("Provider must exist");
		assert_eq!(provider_entry.default_name, entry.default_name);
		assert_eq!(provider_entry.localized_names, entry.localized_names);
		assert_eq!(provider_entry.default_logo_250_100_png_cid, entry.default_logo_250_100_png_cid);
		assert_eq!(
			provider_entry.localized_logo_250_100_png_cids,
			entry.localized_logo_250_100_png_cids
		);

		Ok(())
	}

	impl_benchmark_test_suite!(
		Msa,
		crate::tests::mock::new_test_ext_keystore(),
		crate::tests::mock::Test
	);
}
