use sp_core::{crypto::AccountId32, sr25519, Encode, Pair};
use sp_runtime::MultiSignature;

use frame_support::{assert_noop, assert_ok, dispatch::GetDispatchInfo};

use sp_weights::Weight;

use crate::{
	ensure, tests::mock::*, types::AddProvider, CurrentMsaIdentifierMaximum, DispatchResult, Error,
	Event, PublicKeyToMsaId,
};

use sp_std::{cmp::*, collections::btree_map::BTreeMap};

use common_primitives::{
	msa::{DelegatorId, MessageSourceId, ProviderId},
	node::BlockNumber,
	utils::wrap_binary_data,
};

#[test]
pub fn iteration_test() {
	new_test_ext().execute_with(|| {

		let b = b"Msa::ofw::keys::100";
		for i in 0..1000 {
			let _ = create_account();
		}

		let mut map: BTreeMap<u64, Vec<AccountId32>> = BTreeMap::new();
		let mut count = 0u32;
		for (account_id, msa_id) in PublicKeyToMsaId::<Test>::iter() {
			map.entry(msa_id)
				.and_modify(|v| v.push(account_id.clone()))
				.or_insert(vec![account_id.clone()]);
			println!("{}", account_id);
		}

		println!("");

		for (account_id, msa_id) in PublicKeyToMsaId::<Test>::iter() {
			println!("{}", account_id);
			count += 1;

			if count % 10 == 0 {
				let _ = create_account();
			}
		}

		assert_eq!(map.len(), 100);
	});
}

#[test]
pub fn create_sponsored_account_with_delegation_with_valid_input_should_succeed() {
	new_test_ext().execute_with(|| {
		// arrange
		let (provider_msa, provider_key_pair) = create_account();
		let provider_account = provider_key_pair.public();
		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

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
			Msa::get_msa_by_public_key(&AccountId32::new(delegator_account.0)).unwrap();

		let provider_info = Msa::get_delegation(DelegatorId(2), ProviderId(1));
		assert_eq!(provider_info.is_some(), true);

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

		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

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

		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

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

		assert!(dispatch_info.weight.ref_time() > Weight::from_parts(10_000 as u64, 0).ref_time());
	});
}

#[test]
fn it_creates_an_msa_account() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::create(test_origin_signed(1)));

		assert_eq!(Msa::get_msa_by_public_key(test_public(1)), Some(1 as MessageSourceId));

		assert_eq!(Msa::get_current_msa_identifier_maximum(), 1);

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

		assert_eq!(Msa::get_current_msa_identifier_maximum(), 1);
	});
}
