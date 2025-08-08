use sp_core::{crypto::AccountId32, sr25519, Encode, Pair};
use sp_runtime::MultiSignature;

use frame_support::{
	assert_noop, assert_ok, dispatch::GetDispatchInfo, BoundedBTreeMap, BoundedVec,
};

use sp_weights::Weight;

use crate::{
	ensure, tests::mock::*, types::AddProvider, CurrentMsaIdentifierMaximum,
	DelegatorAndProviderToDelegation, DispatchResult, Error, Event, PublicKeyToMsaId,
};

use common_primitives::{
	msa::{DelegatorId, MessageSourceId, ProviderId, ProviderRegistryEntry},
	node::BlockNumber,
	utils::wrap_binary_data,
};

#[test]
pub fn create_sponsored_account_with_delegation_with_valid_input_should_succeed() {
	new_test_ext().execute_with(|| {
		// arrange
		let (provider_msa, provider_key_pair) = create_account();
		let provider_account = provider_key_pair.public();
		let entry = ProviderRegistryEntry::default();
		// Register provider
		assert_ok!(Msa::create_provider(RuntimeOrigin::signed(provider_account.into()), entry));

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_account = key_pair_delegator.public();

		let expiration: BlockNumber = 10;

		let add_provider_payload = AddProvider::new(provider_msa, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		// act
		assert_ok!(Msa::create_sponsored_account_with_delegation(
			RuntimeOrigin::signed(provider_account.into()),
			delegator_account.into(),
			signature,
			add_provider_payload
		));

		// assert
		let delegator_msa =
			PublicKeyToMsaId::<Test>::get(AccountId32::new(delegator_account.0)).unwrap();

		let provider_info =
			DelegatorAndProviderToDelegation::<Test>::get(DelegatorId(2), ProviderId(1));
		assert!(provider_info.is_some());

		let events_occured = System::events();
		let created_event = &events_occured.as_slice()[1];
		let provider_event = &events_occured.as_slice()[2];
		assert_eq!(
			created_event.event,
			Event::MsaCreated { msa_id: delegator_msa, key: delegator_account.into() }.into()
		);
		assert_eq!(
			provider_event.event,
			Event::DelegationGranted {
				provider_id: provider_msa.into(),
				delegator_id: delegator_msa.into()
			}
			.into()
		);
	});
}

#[test]
fn create_sponsored_account_with_delegation_with_invalid_signature_should_fail() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_account = key_pair_delegator.public();

		let (signer_pair, _) = sr25519::Pair::generate();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = signer_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));

		// act
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				RuntimeOrigin::signed(provider_account.into()),
				delegator_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::InvalidSignature
		);
	});
}

#[test]
pub fn create_sponsored_account_with_delegation_with_invalid_add_provider_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_account = key_pair_delegator.public();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));
		assert_ok!(Msa::create(RuntimeOrigin::signed(delegator_account.into())));
		let entry = ProviderRegistryEntry::default();
		// Register provider
		assert_ok!(Msa::create_provider(RuntimeOrigin::signed(provider_account.into()), entry));

		// act
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				RuntimeOrigin::signed(provider_account.into()),
				delegator_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::KeyAlreadyRegistered
		);
	});
}

#[test]
pub fn create_sponsored_account_with_delegation_with_different_authorized_msa_id_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_account = key_pair_delegator.public();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(3u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));

		// act
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				RuntimeOrigin::signed(provider_account.into()),
				delegator_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::UnauthorizedProvider
		);
	});
}

#[test]
pub fn create_sponsored_account_with_delegation_expired() {
	new_test_ext().execute_with(|| {
		// arrange
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_account = key_pair_delegator.public();

		let expiration: BlockNumber = 0;

		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));
		let entry = ProviderRegistryEntry::default();
		// Register provider
		assert_ok!(Msa::create_provider(RuntimeOrigin::signed(provider_account.into()), entry));

		// act
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				RuntimeOrigin::signed(provider_account.into()),
				delegator_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::ProofHasExpired
		);
	});
}

#[test]
pub fn create_account_with_panic_in_on_success_should_revert_everything() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1u64;
		let key = test_public(msa_id as u8);
		let next_msa_id = Msa::get_next_msa_id().unwrap();

		// act
		assert_noop!(
			Msa::create_account(key, |new_msa_id| -> DispatchResult {
				ensure!(new_msa_id != msa_id, Error::<Test>::InvalidSelfRemoval);
				Ok(())
			}),
			Error::<Test>::InvalidSelfRemoval
		);

		// assert
		assert_eq!(next_msa_id, Msa::get_next_msa_id().unwrap());
	});
}

#[test]
fn it_create_has_weight() {
	new_test_ext().execute_with(|| {
		let call = MsaCall::<Test>::create {};
		let dispatch_info = call.get_dispatch_info();

		assert!(
			dispatch_info.call_weight.ref_time() > Weight::from_parts(10_000_u64, 0).ref_time()
		);
	});
}

#[test]
fn it_creates_an_msa_account() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::create(test_origin_signed(1)));

		assert_eq!(PublicKeyToMsaId::<Test>::get(test_public(1)), Some(1 as MessageSourceId));

		assert_eq!(CurrentMsaIdentifierMaximum::<Test>::get(), 1);

		System::assert_last_event(Event::MsaCreated { msa_id: 1, key: test_public(1) }.into());
	});
}

#[test]
fn it_throws_msa_identifier_overflow() {
	new_test_ext().execute_with(|| {
		CurrentMsaIdentifierMaximum::<Test>::set(u64::MAX);

		assert_noop!(Msa::create(test_origin_signed(1)), Error::<Test>::MsaIdOverflow);
	});
}

#[test]
#[allow(unused_must_use)]
fn it_does_not_allow_duplicate_keys() {
	new_test_ext().execute_with(|| {
		Msa::create(test_origin_signed(1));

		assert_noop!(Msa::create(test_origin_signed(1)), Error::<Test>::KeyAlreadyRegistered);

		assert_eq!(CurrentMsaIdentifierMaximum::<Test>::get(), 1);
	});
}

#[test]
fn verify_signature_with_wrapped_bytes() {
	new_test_ext().execute_with(|| {
		let provider_msa = 1;
		let (key_pair_delegator, _) = sr25519::Pair::generate();

		let expiration: BlockNumber = 10;

		let add_provider_payload = AddProvider::new(provider_msa, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		assert!(Msa::verify_signature(
			&signature,
			&key_pair_delegator.public().into(),
			&add_provider_payload
		));
	});
}

#[test]
fn verify_signature_without_wrapped_bytes() {
	new_test_ext().execute_with(|| {
		let provider_msa = 1;
		let (key_pair_delegator, _) = sr25519::Pair::generate();

		let expiration: BlockNumber = 10;

		let add_provider_payload = AddProvider::new(provider_msa, None, expiration);

		let signature: MultiSignature =
			key_pair_delegator.sign(&add_provider_payload.encode()).into();

		assert!(Msa::verify_signature(
			&signature,
			&key_pair_delegator.public().into(),
			&add_provider_payload
		));
	});
}

#[test]
pub fn create_provider_fails_with_invalid_cid_logo() {
	new_test_ext().execute_with(|| {
		// arrange
		let (_, provider_key_pair) = create_account();
		let provider_account = provider_key_pair.public();
		let cid = "invalid-cid".as_bytes().to_vec();
		let mut entry = ProviderRegistryEntry::default();
		entry.default_logo_250_100_png_cid =
			BoundedVec::try_from(cid).expect("Logo CID should fit in bounds");
		// Fail to register provider with invalid CID
		assert_noop!(
			Msa::create_provider(RuntimeOrigin::signed(provider_account.into()), entry),
			Error::<Test>::InvalidCid
		);
	});
}

#[test]
pub fn create_provider_fails_with_invalid_cid_localized_logo() {
	new_test_ext().execute_with(|| {
		// arrange
		let (_, provider_key_pair) = create_account();
		let provider_account = provider_key_pair.public();
		let mut localized_logo_png = BoundedBTreeMap::new();
		localized_logo_png
			.try_insert(
				BoundedVec::try_from("en".as_bytes().to_vec()).expect("Locale too long"),
				BoundedVec::try_from("invalid-cid".as_bytes().to_vec()).expect("CID too long"),
			)
			.expect("Map insertion should not exceed max size");

		let mut entry = ProviderRegistryEntry::default();
		entry.localized_logo_250_100_png_cids = localized_logo_png;
		// Fail to register provider with invalid CID
		assert_noop!(
			Msa::create_provider(RuntimeOrigin::signed(provider_account.into()), entry),
			Error::<Test>::InvalidCid
		);
	});
}

#[test]
pub fn create_provider_fails_with_invalid_logo_locale() {
	new_test_ext().execute_with(|| {
		// arrange
		let (_, provider_key_pair) = create_account();
		let provider_account = provider_key_pair.public();
		let cid = "bafkreidgvpkjawlxz6sffxzwgooowe5yt7i6wsyg236mfoks77nywkptdq"
			.as_bytes()
			.to_vec();
		let mut localized_logo_png = BoundedBTreeMap::new();
		localized_logo_png
			.try_insert(
				BoundedVec::try_from("&en".as_bytes().to_vec()).expect("Locale too long"),
				BoundedVec::try_from(cid.clone()).expect("CID too long"),
			)
			.expect("Map insertion should not exceed max size");
		let mut localized_names = BoundedBTreeMap::new();
		localized_names
			.try_insert(
				BoundedVec::try_from("en".as_bytes().to_vec()).expect("Locale too long"),
				BoundedVec::try_from(b"Foo".to_vec()).expect("Name too long"),
			)
			.expect("Map insertion should not exceed max size");
		let mut entry = ProviderRegistryEntry::default();
		entry.default_name =
			BoundedVec::try_from(b"Foo".to_vec()).expect("Provider name should fit in bounds");
		entry.localized_names = localized_names;
		entry.default_logo_250_100_png_cid =
			BoundedVec::try_from(cid).expect("Logo CID should fit in bounds");
		entry.localized_logo_250_100_png_cids = localized_logo_png;
		// Fail to register provider with invalid CID
		assert_noop!(
			Msa::create_provider(RuntimeOrigin::signed(provider_account.into()), entry),
			Error::<Test>::InvalidBCP47LanguageCode
		);
	});
}

#[test]
pub fn create_provider_fails_with_invalid_name_locale() {
	new_test_ext().execute_with(|| {
		// arrange
		let (_, provider_key_pair) = create_account();
		let provider_account = provider_key_pair.public();
		let mut localized_names = BoundedBTreeMap::new();
		localized_names
			.try_insert(
				BoundedVec::try_from("&en".as_bytes().to_vec()).expect("Locale too long"),
				BoundedVec::try_from(b"Foo".to_vec()).expect("Name too long"),
			)
			.expect("Map insertion should not exceed max size");
		let mut entry = ProviderRegistryEntry::default();
		entry.localized_names = localized_names;
		// Fail to register provider with invalid CID
		assert_noop!(
			Msa::create_provider(RuntimeOrigin::signed(provider_account.into()), entry),
			Error::<Test>::InvalidBCP47LanguageCode
		);
	});
}
