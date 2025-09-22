use common_primitives::msa::{ApplicationContext, ProviderId, ProviderRegistryEntry};
use frame_support::{assert_noop, assert_ok, traits::ChangeMembers, BoundedBTreeMap, BoundedVec};

use pallet_collective::ProposalOf;
use sp_core::{Encode, Pair};
use sp_weights::Weight;

use pretty_assertions::assert_eq;

use crate::{tests::mock::*, Error, Event, ProviderToApplicationRegistry, ProviderToRegistryEntry};

#[test]
fn update_provider_via_governance_happy_path() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, key_pair) = create_provider_with_name("OriginalProvider");
		let provider_account = key_pair;
		let old_provider_entry =
			ProviderToRegistryEntry::<Test>::get(ProviderId(provider_msa_id)).unwrap();
		assert_eq!(old_provider_entry.default_name, b"OriginalProvider".to_vec());
		assert_eq!(old_provider_entry.localized_names.len(), 1);
		assert_eq!(old_provider_entry.default_logo_250_100_png_cid.is_empty(), false);
		assert_eq!(old_provider_entry.localized_logo_250_100_png_cids.len(), 1);
		// Create updated provider entry
		let mut updated_entry = ProviderRegistryEntry::default();
		updated_entry.default_name = BoundedVec::try_from(b"UpdatedProvider".to_vec())
			.expect("Provider name should fit in bounds");
		let new_cid = "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku"
			.as_bytes()
			.to_vec();
		updated_entry.default_logo_250_100_png_cid =
			BoundedVec::try_from(new_cid.clone()).expect("Logo CID should fit in bounds");

		// Update provider via governance should succeed with overwrite
		let result = Msa::update_provider_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
			provider_account.into(),
			updated_entry.clone(),
		);

		// Assert that the call succeeded
		assert!(result.is_ok());

		// Extract and verify the weight information
		let dispatch_info = result.unwrap();

		// Assert that we got some weight refund information
		if let Some(actual_weight) = dispatch_info.actual_weight {
			// Assert that actual weight consumed is positive
			assert!(actual_weight.ref_time() > 0, "Expected positive weight consumption");
			assert!(actual_weight.proof_size() > 0, "Expected non-negative proof size");
			println!("Actual weight consumed: {:?}", actual_weight);
		}
		let stored_entry =
			ProviderToRegistryEntry::<Test>::get(ProviderId(provider_msa_id)).unwrap();
		assert_eq!(stored_entry.default_name, b"UpdatedProvider".to_vec());
		System::assert_last_event(
			Event::ProviderUpdated { provider_id: ProviderId(provider_msa_id) }.into(),
		);
		let updated_provider_entry =
			ProviderToRegistryEntry::<Test>::get(ProviderId(provider_msa_id)).unwrap();
		assert_eq!(updated_provider_entry.default_name, b"UpdatedProvider".to_vec());
		assert_eq!(updated_provider_entry.localized_names.len(), 0);
		assert_eq!(
			updated_provider_entry.default_logo_250_100_png_cid.as_slice(),
			new_cid.as_slice()
		);
		assert_eq!(updated_provider_entry.localized_logo_250_100_png_cids.len(), 0);

		// Update again with different data to ensure overwrite
		let mut second_update_entry = ProviderRegistryEntry::default();
		second_update_entry.default_name = BoundedVec::try_from(b"SecondUpdate".to_vec())
			.expect("Provider name should fit in bounds");
		let second_cid = "zb2rhojSkWwLpTH7Sc9UFA3gFySTS8tx1vVu9SXhHTBcMabfF".as_bytes().to_vec();
		second_update_entry.default_logo_250_100_png_cid =
			BoundedVec::try_from(second_cid.clone()).expect("Logo CID should fit in bounds");
		assert_ok!(Msa::update_provider_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
			provider_account.into(),
			second_update_entry.clone()
		));
		let final_stored_entry =
			ProviderToRegistryEntry::<Test>::get(ProviderId(provider_msa_id)).unwrap();
		assert_eq!(final_stored_entry.default_name, b"SecondUpdate".to_vec());
		System::assert_last_event(
			Event::ProviderUpdated { provider_id: ProviderId(provider_msa_id) }.into(),
		);
		let final_provider_entry =
			ProviderToRegistryEntry::<Test>::get(ProviderId(provider_msa_id)).unwrap();
		assert_eq!(final_provider_entry.default_name, b"SecondUpdate".to_vec());
		assert_eq!(final_provider_entry.localized_names.len(), 0);
		assert_eq!(
			final_provider_entry.default_logo_250_100_png_cid.as_slice(),
			second_cid.as_slice()
		);
		assert_eq!(final_provider_entry.localized_logo_250_100_png_cids.len(), 0);
	})
}

#[test]
fn update_provider_via_governance_fails_for_invalid_cid() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, key_pair) = create_provider_with_name("TestProvider");
		let provider_account = key_pair;

		// Create updated provider entry with invalid CID
		let mut updated_entry = ProviderRegistryEntry::default();
		updated_entry.default_name = BoundedVec::try_from(b"UpdatedProvider".to_vec())
			.expect("Provider name should fit in bounds");
		let invalid_cid = "invalid-cid".as_bytes().to_vec();
		updated_entry.default_logo_250_100_png_cid =
			BoundedVec::try_from(invalid_cid).expect("Logo CID should fit in bounds");

		// Update should fail due to invalid CID
		assert_noop!(
			Msa::update_provider_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
				provider_account.into(),
				updated_entry
			),
			Error::<Test>::InvalidCid
		);

		// Verify the provider was not updated
		let stored_entry = ProviderToRegistryEntry::<Test>::get(ProviderId(provider_msa_id));
		assert!(stored_entry.is_some());
		let entry = stored_entry.unwrap();
		assert_eq!(entry.default_name, b"TestProvider".to_vec());
	})
}

#[test]
fn update_provider_via_governance_fails_for_invalid_lang_code() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, key_pair) = create_provider_with_name("TestProvider");
		let provider_account = key_pair;

		// Create updated provider entry with invalid language code
		let mut updated_entry = ProviderRegistryEntry::default();
		updated_entry.default_name = BoundedVec::try_from(b"UpdatedProvider".to_vec())
			.expect("Provider name should fit in bounds");
		let mut localized_names = BoundedBTreeMap::new();
		localized_names
			.try_insert(
				BoundedVec::try_from("&en".as_bytes().to_vec()).expect("Locale too long"),
				BoundedVec::try_from(b"Foo".to_vec()).expect("Name too long"),
			)
			.expect("Map insertion should not exceed max size");
		updated_entry.localized_names = localized_names;

		// Update should fail due to invalid language code
		assert_noop!(
			Msa::update_provider_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
				provider_account.into(),
				updated_entry
			),
			Error::<Test>::InvalidBCP47LanguageCode
		);

		// Verify the provider was not updated
		let stored_entry = ProviderToRegistryEntry::<Test>::get(ProviderId(provider_msa_id));
		assert!(stored_entry.is_some());
		let entry = stored_entry.unwrap();
		assert_eq!(entry.default_name, b"TestProvider".to_vec());
	})
}

#[test]
fn update_provider_via_governance_fails_for_nonexistent_msa() {
	new_test_ext().execute_with(|| {
		let (_, key_pair) = sp_core::sr25519::Pair::generate();
		let provider_account = key_pair;

		let updated_entry = ProviderRegistryEntry::default();

		// Update should fail because the account doesn't have an MSA
		assert_noop!(
			Msa::update_provider_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
				provider_account.into(),
				updated_entry
			),
			Error::<Test>::NoKeyExists
		);
	})
}

#[test]
fn propose_to_update_provider_happy_path() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, key_pair) = create_provider_with_name("TestProvider");

		// Create updated provider entry
		let mut updated_entry = ProviderRegistryEntry::default();
		updated_entry.default_name = BoundedVec::try_from(b"ProposedProvider".to_vec())
			.expect("Provider name should fit in bounds");
		let new_cid = "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku"
			.as_bytes()
			.to_vec();
		updated_entry.default_logo_250_100_png_cid =
			BoundedVec::try_from(new_cid).expect("Logo CID should fit in bounds");

		// Propose to update provider
		assert_ok!(Msa::propose_to_update_provider(
			RuntimeOrigin::signed(key_pair.into()),
			updated_entry
		));

		// Find the Proposed event and get its hash and index
		let proposed_events: Vec<(u32, <Test as frame_system::Config>::Hash)> = System::events()
			.iter()
			.filter_map(|event| match event.event {
				RuntimeEvent::Council(pallet_collective::Event::Proposed {
					account: _,
					proposal_index,
					proposal_hash,
					threshold: _,
				}) => Some((proposal_index, proposal_hash)),
				_ => None,
			})
			.collect();

		assert_eq!(proposed_events.len(), 1);

		let proposal_index = proposed_events[0].0;
		let proposal_hash = proposed_events[0].1;
		let proposal = ProposalOf::<Test, CouncilCollective>::get(proposal_hash).unwrap();
		let proposal_len: u32 = proposal.encoded_size() as u32;

		// Set up the council members
		let council_member = test_public(1);
		let incoming = vec![];
		let outgoing = vec![];
		Council::change_members(&incoming, &outgoing, vec![council_member.clone()]);

		// Vote YES on the proposal
		assert_ok!(Council::vote(
			RuntimeOrigin::signed(council_member.clone()),
			proposal_hash,
			proposal_index,
			true
		));

		// Close the voting
		assert_ok!(Council::close(
			RuntimeOrigin::signed(test_public(5)),
			proposal_hash,
			proposal_index,
			Weight::MAX,
			proposal_len
		));

		// Verify the provider was updated after proposal executed
		let stored_entry =
			ProviderToRegistryEntry::<Test>::get(ProviderId(provider_msa_id)).unwrap();
		assert_eq!(stored_entry.default_name, b"ProposedProvider".to_vec());
	})
}

#[test]
fn propose_to_update_provider_requires_registered_provider() {
	new_test_ext().execute_with(|| {
		let (_, key_pair) = create_account();
		let provider_account = key_pair;

		let updated_entry = ProviderRegistryEntry::default();

		// Propose should fail because the account is not a registered provider
		assert_noop!(
			Msa::propose_to_update_provider(
				RuntimeOrigin::signed(provider_account.public().into()),
				updated_entry
			),
			Error::<Test>::ProviderNotRegistered
		);
	})
}

#[test]
fn propose_to_update_provider_fails_for_nonexistent_msa() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sp_core::sr25519::Pair::generate();
		let provider_account = key_pair;

		let updated_entry = ProviderRegistryEntry::default();

		// Propose should fail because the account doesn't have an MSA
		assert_noop!(
			Msa::propose_to_update_provider(
				RuntimeOrigin::signed(provider_account.public().into()),
				updated_entry
			),
			Error::<Test>::NoKeyExists
		);
	})
}

#[test]
fn update_application_via_governance_happy_path() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, key_pair) = create_provider_with_name("AppProvider");
		let provider_account = key_pair;

		// First create an application
		let app_entry = ApplicationContext::default();
		assert_ok!(Msa::create_application_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
			provider_account.into(),
			app_entry
		));

		let application_index = 0;

		// Create updated application entry
		let mut updated_entry = ApplicationContext::default();
		updated_entry.default_name =
			BoundedVec::try_from(b"UpdatedApp".to_vec()).expect("App name should fit in bounds");
		let new_cid = "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku"
			.as_bytes()
			.to_vec();
		updated_entry.default_logo_250_100_png_cid =
			BoundedVec::try_from(new_cid).expect("Logo CID should fit in bounds");

		// Update application via governance
		assert_ok!(Msa::update_application_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
			provider_account.into(),
			application_index,
			updated_entry.clone()
		));

		// Verify the application was updated
		let stored_entry = ProviderToApplicationRegistry::<Test>::get(
			ProviderId(provider_msa_id),
			application_index,
		);
		assert!(stored_entry.is_some());
		let entry = stored_entry.unwrap();
		assert_eq!(entry.default_name, b"UpdatedApp".to_vec());

		// Verify event was emitted
		System::assert_last_event(
			Event::ApplicationContextUpdated {
				provider_id: ProviderId(provider_msa_id),
				application_id: Some(application_index),
			}
			.into(),
		);
	})
}

#[test]
fn update_application_via_governance_fails_for_nonexistent_application() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, key_pair) = create_provider_with_name("AppProvider");
		let provider_account = key_pair;

		let updated_entry = ApplicationContext::default();
		let application_index = 999;

		// Update should fail because the application doesn't exist
		assert_noop!(
			Msa::update_application_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
				provider_account.into(),
				application_index,
				updated_entry
			),
			Error::<Test>::ApplicationNotFound
		);

		// Verify no application exists
		let stored_entry = ProviderToApplicationRegistry::<Test>::get(
			ProviderId(provider_msa_id),
			application_index,
		);
		assert!(stored_entry.is_none());
	})
}

#[test]
fn update_application_via_governance_fails_for_invalid_cid() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, key_pair) = create_provider_with_name("AppProvider");
		let provider_account = key_pair;

		// First create an application
		let mut app_entry = ApplicationContext::default();
		app_entry.default_name =
			BoundedVec::try_from(b"default".to_vec()).expect("App name should fit in bounds");
		assert_ok!(Msa::create_application_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
			provider_account.into(),
			app_entry
		));

		let application_index = 0;

		// Create updated application entry with invalid CID
		let mut updated_entry = ApplicationContext::default();
		updated_entry.default_name =
			BoundedVec::try_from(b"UpdatedApp".to_vec()).expect("App name should fit in bounds");
		let invalid_cid = "invalid-cid".as_bytes().to_vec();
		updated_entry.default_logo_250_100_png_cid =
			BoundedVec::try_from(invalid_cid).expect("Logo CID should fit in bounds");

		// Update should fail due to invalid CID
		assert_noop!(
			Msa::update_application_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
				provider_account.into(),
				application_index,
				updated_entry
			),
			Error::<Test>::InvalidCid
		);

		// Verify the application was not updated
		let stored_entry = ProviderToApplicationRegistry::<Test>::get(
			ProviderId(provider_msa_id),
			application_index,
		);
		assert!(stored_entry.is_some());
		let entry = stored_entry.unwrap();
		assert_eq!(entry.default_name, b"default".to_vec());
	})
}

#[test]
fn propose_to_update_application_happy_path() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, key_pair) = create_provider_with_name("AppProvider");

		// First create an application
		let app_entry = ApplicationContext::default();
		assert_ok!(Msa::create_application_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
			key_pair.into(),
			app_entry
		));

		let application_index = 0;

		// Create updated application entry
		let mut updated_entry = ApplicationContext::default();
		updated_entry.default_name =
			BoundedVec::try_from(b"ProposedApp".to_vec()).expect("App name should fit in bounds");
		let new_cid = "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku"
			.as_bytes()
			.to_vec();
		updated_entry.default_logo_250_100_png_cid =
			BoundedVec::try_from(new_cid).expect("Logo CID should fit in bounds");

		// Propose to update application
		assert_ok!(Msa::propose_to_update_application(
			RuntimeOrigin::signed(key_pair.into()),
			application_index,
			updated_entry
		));

		// Find the Proposed event and get its hash and index
		let proposed_events: Vec<(u32, <Test as frame_system::Config>::Hash)> = System::events()
			.iter()
			.filter_map(|event| match event.event {
				RuntimeEvent::Council(pallet_collective::Event::Proposed {
					account: _,
					proposal_index,
					proposal_hash,
					threshold: _,
				}) => Some((proposal_index, proposal_hash)),
				_ => None,
			})
			.collect();

		assert_eq!(proposed_events.len(), 1);

		let proposal_index = proposed_events[0].0;
		let proposal_hash = proposed_events[0].1;
		let proposal = ProposalOf::<Test, CouncilCollective>::get(proposal_hash).unwrap();
		let proposal_len: u32 = proposal.encoded_size() as u32;

		// Set up the council members
		let council_member = test_public(1);
		let incoming = vec![];
		let outgoing = vec![];
		Council::change_members(&incoming, &outgoing, vec![council_member.clone()]);

		// Vote YES on the proposal
		assert_ok!(Council::vote(
			RuntimeOrigin::signed(council_member.clone()),
			proposal_hash,
			proposal_index,
			true
		));

		// Close the voting
		assert_ok!(Council::close(
			RuntimeOrigin::signed(test_public(5)),
			proposal_hash,
			proposal_index,
			Weight::MAX,
			proposal_len
		));

		// Verify the application was updated
		let stored_entry = ProviderToApplicationRegistry::<Test>::get(
			ProviderId(provider_msa_id),
			application_index,
		);
		assert!(stored_entry.is_some());
		let entry = stored_entry.unwrap();
		assert_eq!(entry.default_name, b"ProposedApp".to_vec());
	})
}

#[test]
fn update_application_requires_registered_provider() {
	new_test_ext().execute_with(|| {
		let (_, key_pair) = create_account();
		let provider_account = key_pair;

		let updated_entry = ApplicationContext::default();
		let application_index = 0;

		// Propose should fail because the account is not a registered provider
		assert_noop!(
			Msa::update_application_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
				provider_account.public().into(),
				application_index,
				updated_entry
			),
			Error::<Test>::ProviderNotRegistered
		);
	})
}

#[test]
fn propose_to_update_application_requires_registered_provider() {
	new_test_ext().execute_with(|| {
		let (_, key_pair) = create_account();
		let provider_account = key_pair;

		let updated_entry = ApplicationContext::default();
		let application_index = 0;

		// Propose should fail because the account is not a registered provider
		assert_noop!(
			Msa::propose_to_update_application(
				RuntimeOrigin::signed(provider_account.public().into()),
				application_index,
				updated_entry
			),
			Error::<Test>::ProviderNotRegistered
		);
	})
}

#[test]
fn propose_to_update_application_fails_for_nonexistent_application() {
	new_test_ext().execute_with(|| {
		let (_, key_pair) = create_provider_with_name("AppProvider");

		let updated_entry = ApplicationContext::default();
		let application_index = 0;

		// Propose should fail because the application doesn't exist
		assert_noop!(
			Msa::propose_to_update_application(
				RuntimeOrigin::signed(key_pair.into()),
				application_index,
				updated_entry
			),
			Error::<Test>::ApplicationNotFound
		);
	})
}

#[test]
fn propose_to_update_application_fails_for_invalid_cid() {
	new_test_ext().execute_with(|| {
		let (_, key_pair) = create_provider_with_name("AppProvider");

		// First create an application
		let app_entry = ApplicationContext::default();
		assert_ok!(Msa::create_application_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
			key_pair.into(),
			app_entry
		));

		let application_index = 0;

		// Create updated application entry with invalid CID
		let mut updated_entry = ApplicationContext::default();
		updated_entry.default_name =
			BoundedVec::try_from(b"UpdatedApp".to_vec()).expect("App name should fit in bounds");
		let invalid_cid = "invalid-cid".as_bytes().to_vec();
		updated_entry.default_logo_250_100_png_cid =
			BoundedVec::try_from(invalid_cid).expect("Logo CID should fit in bounds");

		// Propose should fail due to invalid CID
		assert_noop!(
			Msa::propose_to_update_application(
				RuntimeOrigin::signed(key_pair.into()),
				application_index,
				updated_entry
			),
			Error::<Test>::InvalidCid
		);
	})
}

#[test]
fn propose_to_update_application_fails_for_nonexistent_msa() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sp_core::sr25519::Pair::generate();
		let provider_account = key_pair;

		let updated_entry = ApplicationContext::default();
		let application_index = 0;

		// Propose should fail because the account doesn't have an MSA
		assert_noop!(
			Msa::propose_to_update_application(
				RuntimeOrigin::signed(provider_account.public().into()),
				application_index,
				updated_entry
			),
			Error::<Test>::NoKeyExists
		);
	})
}

// ===== EDGE CASES AND COMPREHENSIVE TESTS =====

#[test]
fn update_provider_with_localized_names_and_logos() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, key_pair) = create_provider_with_name("TestProvider");
		let provider_account = key_pair;

		// Create updated provider entry with localized names and logos
		let mut updated_entry = ProviderRegistryEntry::default();
		updated_entry.default_name = BoundedVec::try_from(b"UpdatedProvider".to_vec())
			.expect("Provider name should fit in bounds");

		// Add localized names
		let mut localized_names = BoundedBTreeMap::new();
		localized_names
			.try_insert(
				BoundedVec::try_from("en-US".as_bytes().to_vec()).expect("Locale too long"),
				BoundedVec::try_from(b"Updated Provider".to_vec()).expect("Name too long"),
			)
			.expect("Map insertion should not exceed max size");
		localized_names
			.try_insert(
				BoundedVec::try_from("es-ES".as_bytes().to_vec()).expect("Locale too long"),
				BoundedVec::try_from(b"Prov Actual".to_vec()).expect("Name too long"),
			)
			.expect("Map insertion should not exceed max size");
		updated_entry.localized_names = localized_names;

		// Add localized logos
		let mut localized_logos = BoundedBTreeMap::new();
		let logo_cid_en = "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku"
			.as_bytes()
			.to_vec();
		let logo_cid_es = "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku"
			.as_bytes()
			.to_vec();
		localized_logos
			.try_insert(
				BoundedVec::try_from("en-US".as_bytes().to_vec()).expect("Locale too long"),
				BoundedVec::try_from(logo_cid_en).expect("Logo CID should fit in bounds"),
			)
			.expect("Map insertion should not exceed max size");
		localized_logos
			.try_insert(
				BoundedVec::try_from("es-ES".as_bytes().to_vec()).expect("Locale too long"),
				BoundedVec::try_from(logo_cid_es).expect("Logo CID should fit in bounds"),
			)
			.expect("Map insertion should not exceed max size");
		updated_entry.localized_logo_250_100_png_cids = localized_logos;

		// Update provider via governance should now succeed (overwrite)
		assert_ok!(Msa::update_provider_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
			provider_account.into(),
			updated_entry.clone()
		));

		// Ensure provider entry updated
		let stored_entry =
			ProviderToRegistryEntry::<Test>::get(ProviderId(provider_msa_id)).unwrap();
		assert_eq!(stored_entry.default_name, b"UpdatedProvider".to_vec());
	})
}

#[test]
fn update_application_with_complex_payload() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, key_pair) = create_provider_with_name("AppProvider");
		let provider_account = key_pair;

		// First create an application
		let app_entry = ApplicationContext::default();
		assert_ok!(Msa::create_application_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
			provider_account.into(),
			app_entry
		));

		let application_index = 0;

		// Create complex updated application entry
		let mut updated_entry = ApplicationContext::default();
		updated_entry.default_name =
			BoundedVec::try_from(b"ComplexApp".to_vec()).expect("App name should fit in bounds");

		// Add localized names
		let mut localized_names = BoundedBTreeMap::new();
		localized_names
			.try_insert(
				BoundedVec::try_from("en-US".as_bytes().to_vec()).expect("Locale too long"),
				BoundedVec::try_from(b"Complex App".to_vec()).expect("Name too long"),
			)
			.expect("Map insertion should not exceed max size");
		updated_entry.localized_names = localized_names;

		// Add localized logos
		let mut localized_logos = BoundedBTreeMap::new();
		let logo_cid = "bafkreihdwdcefgh4dqkjv67uzcmw7ojee6xedzdetojuzjevtenxquvyku"
			.as_bytes()
			.to_vec();
		localized_logos
			.try_insert(
				BoundedVec::try_from("en-US".as_bytes().to_vec()).expect("Locale too long"),
				BoundedVec::try_from(logo_cid).expect("Logo CID should fit in bounds"),
			)
			.expect("Map insertion should not exceed max size");
		updated_entry.localized_logo_250_100_png_cids = localized_logos;

		// Update application via governance
		assert_ok!(Msa::update_application_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
			provider_account.into(),
			application_index,
			updated_entry.clone()
		));

		// Verify the application was updated with complex content
		let stored_entry = ProviderToApplicationRegistry::<Test>::get(
			ProviderId(provider_msa_id),
			application_index,
		);
		assert!(stored_entry.is_some());
		let entry = stored_entry.unwrap();
		assert_eq!(entry.default_name, b"ComplexApp".to_vec());
		assert_eq!(entry.localized_names.len(), 1);
		assert_eq!(entry.localized_logo_250_100_png_cids.len(), 1);

		// Verify specific localized content
		let en_name = entry
			.localized_names
			.get(&BoundedVec::try_from("en-US".as_bytes().to_vec()).unwrap());
		assert!(en_name.is_some());
		assert_eq!(en_name.unwrap().as_slice(), b"Complex App");
	})
}
