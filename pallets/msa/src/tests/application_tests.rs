use common_primitives::msa::{ApplicationContext, ProviderId};
use frame_support::{assert_noop, assert_ok, traits::ChangeMembers, BoundedBTreeMap, BoundedVec};

use pallet_collective::ProposalOf;
use sp_core::{Encode, Pair};
use sp_weights::Weight;

use pretty_assertions::assert_eq;

use crate::{
	tests::mock::*, types::compute_cid, ApprovedLogos, Error, NextApplicationIndex,
	ProviderToApplicationRegistry,
};

#[test]
fn create_application_via_governance_happy_path() {
	new_test_ext().execute_with(|| {
		let (new_msa_id, key_pair) = create_provider_with_name("AppProvider");
		let entry = ApplicationContext::default();
		// Create the application based on 1 yes vote by the council
		assert_ok!(Msa::create_application_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
			key_pair.into(),
			entry
		));
		// Confirm that the MSA is now a provider
		assert!(Msa::is_registered_provider(new_msa_id));
		assert_eq!(NextApplicationIndex::<Test>::get(ProviderId(new_msa_id)), 1);
		assert!(ProviderToApplicationRegistry::<Test>::get(ProviderId(new_msa_id), 0).is_some());
	})
}

#[test]
fn create_application_via_governance_fails_for_invalid_cid() {
	new_test_ext().execute_with(|| {
		let (new_msa_id, key_pair) = create_provider_with_name("AppProvider");
		let mut entry = ApplicationContext::default();
		let cid = "invalid-cid".as_bytes().to_vec();
		entry.default_logo_250_100_png_cid =
			BoundedVec::try_from(cid).expect("Logo CID should fit in bounds");
		// Create the application based on 1 yes vote by the council
		assert_noop!(
			Msa::create_application_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
				key_pair.into(),
				entry
			),
			Error::<Test>::InvalidCid
		);
		// Confirm that the MSA is now a provider
		assert!(Msa::is_registered_provider(new_msa_id));
		// Confirm that no application is added due to invalid payload
		assert_eq!(NextApplicationIndex::<Test>::get(ProviderId(new_msa_id)), 0);
		assert!(ProviderToApplicationRegistry::<Test>::get(ProviderId(new_msa_id), 0).is_none());
	})
}

#[test]
fn create_application_via_governance_fails_for_invalid_lang_code() {
	new_test_ext().execute_with(|| {
		let (new_msa_id, key_pair) = create_provider_with_name("AppProvider");
		let mut entry = ApplicationContext::default();
		let mut localized_names = BoundedBTreeMap::new();
		localized_names
			.try_insert(
				BoundedVec::try_from("&en".as_bytes().to_vec()).expect("Locale too long"),
				BoundedVec::try_from(b"Foo".to_vec()).expect("Name too long"),
			)
			.expect("Map insertion should not exceed max size");
		entry.localized_names = localized_names;
		// Create the application based on 1 yes vote by the council
		assert_noop!(
			Msa::create_application_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
				key_pair.into(),
				entry
			),
			Error::<Test>::InvalidBCP47LanguageCode
		);
		// Confirm that the MSA is now a provider
		assert!(Msa::is_registered_provider(new_msa_id));
		// Confirm that no application is added due to invalid payload
		assert_eq!(NextApplicationIndex::<Test>::get(ProviderId(new_msa_id)), 0);
		assert!(ProviderToApplicationRegistry::<Test>::get(ProviderId(new_msa_id), 0).is_none());
	})
}

#[test]
fn propose_to_add_application_happy_path() {
	new_test_ext().execute_with(|| {
		// Create a new provider account
		let (new_msa_id, key_pair) = create_provider_with_name("AppProvider");

		let entry = ApplicationContext::default();
		_ = Msa::propose_to_add_application(RuntimeOrigin::signed(key_pair.into()), entry);

		// Find the Proposed event and get it's hash and index so it can be voted on
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
		let council_member = test_public(1); // Use ALICE as the council member

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

		// Find the Voted event and check if it passed
		let voted_events: Vec<(bool, u32, u32)> = System::events()
			.iter()
			.filter_map(|event| match event.event {
				RuntimeEvent::Council(pallet_collective::Event::Voted {
					account: _,
					proposal_hash: _,
					voted,
					yes,
					no,
				}) => Some((voted, yes, no)),
				_ => None,
			})
			.collect();

		assert_eq!(voted_events.len(), 1);
		assert_eq!(voted_events[0].0, true); // Was it voted on?
		assert_eq!(voted_events[0].1, 1); // There should be one YES vote to pass

		// Close the voting
		assert_ok!(Council::close(
			RuntimeOrigin::signed(test_public(5)),
			proposal_hash,
			proposal_index,
			Weight::MAX,
			proposal_len
		));

		// Find the Closed event and check if it passed
		let closed_events: Vec<(u32, u32)> = System::events()
			.iter()
			.filter_map(|event| match event.event {
				RuntimeEvent::Council(pallet_collective::Event::Closed {
					proposal_hash: _,
					yes,
					no,
				}) => Some((yes, no)),
				_ => None,
			})
			.collect();

		assert_eq!(closed_events.len(), 1);
		assert_eq!(closed_events[0].0, 1); // There should be one YES vote to pass

		// Confirm that the MSA is now a provider
		assert!(Msa::is_registered_provider(new_msa_id));
		assert!(ProviderToApplicationRegistry::<Test>::get(ProviderId(new_msa_id), 0).is_some());
		assert!(NextApplicationIndex::<Test>::get(ProviderId(new_msa_id)) > 0);
		let old_app_id = NextApplicationIndex::<Test>::get(ProviderId(new_msa_id)) - 1;
		let application_context =
			Msa::get_provider_application_context(ProviderId(new_msa_id), Some(old_app_id), None);
		assert!(application_context.is_some());
		let app_context = application_context.unwrap();
		assert_eq!(app_context.application_id, Some(old_app_id));
		assert_eq!(app_context.provider_id, ProviderId(new_msa_id));
		assert_eq!(app_context.localized_name, None);
	})
}

#[test]
fn propose_to_add_application_requires_registered_provider() {
	new_test_ext().execute_with(|| {
		// Create a new provider account
		let (_, key_pair) = create_account();
		let provider_account = key_pair.public();

		let entry = ApplicationContext::default();
		assert_noop!(
			Msa::propose_to_add_application(RuntimeOrigin::signed(provider_account.into()), entry),
			Error::<Test>::ProviderNotRegistered
		);
	});
}

#[test]
fn create_application_via_governance_requires_registered_provider() {
	new_test_ext().execute_with(|| {
		// Create a new provider account
		let (_, key_pair) = create_account();
		let provider_account = key_pair.public();

		let entry = ApplicationContext::default();
		assert_noop!(
			Msa::create_application_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
				provider_account.into(),
				entry
			),
			Error::<Test>::ProviderNotRegistered
		);
	});
}

#[test]
fn create_application_via_governance_fails_for_duplicate_application() {
	new_test_ext().execute_with(|| {
		let (new_msa_id, key_pair) = create_provider_with_name("AppProvider");
		let entry = ApplicationContext::default();
		// Add the entry directly to `ProviderToApplicationRegistry`
		ProviderToApplicationRegistry::<Test>::insert(ProviderId(new_msa_id), 0, entry.clone());
		// Create the application based on 1 yes vote by the council
		assert_noop!(
			Msa::create_application_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
				key_pair.into(),
				entry
			),
			Error::<Test>::DuplicateApplicationRegistryEntry
		);
	})
}

#[test]
fn upload_logo_happy_path() {
	new_test_ext().execute_with(|| {
		let (_, key_pair) = create_provider_with_name("LogoProvider");
		let logo_cid = compute_cid(b"test_logo_data");
		let input_bounded_cid = BoundedVec::try_from(logo_cid).unwrap();

		// assuming logo was approved via governance
		ApprovedLogos::<Test>::insert(&input_bounded_cid, BoundedVec::new());

		assert_ok!(Msa::upload_logo(
			RuntimeOrigin::signed(key_pair.into()),
			BoundedVec::try_from(input_bounded_cid.clone()).expect("Logo CID should fit in bounds"),
			BoundedVec::try_from(b"test_logo_data".to_vec())
				.expect("Logo data should fit in bounds")
		));

		let stored_logo_bytes = ApprovedLogos::<Test>::get(&input_bounded_cid);
		let expect_logo_bytes = BoundedVec::try_from(b"test_logo_data".to_vec())
			.expect("Logo data should fit in bounds");
		assert_eq!(stored_logo_bytes, Some(expect_logo_bytes));
	});
}

#[test]
fn upload_logo_fails_for_unapproved_logo() {
	new_test_ext().execute_with(|| {
		let (_, key_pair) = create_provider_with_name("LogoProvider");
		let logo_cid = compute_cid(b"test_logo_data");
		let input_bounded_cid = BoundedVec::try_from(logo_cid).unwrap();
		assert_noop!(
			Msa::upload_logo(
				RuntimeOrigin::signed(key_pair.into()),
				BoundedVec::try_from(input_bounded_cid.clone())
					.expect("Logo CID should fit in bounds"),
				BoundedVec::try_from(b"test_logo_data".to_vec())
					.expect("Logo data should fit in bounds")
			),
			Error::<Test>::LogoCidNotApproved
		);
	});
}

#[test]
fn upload_logo_fails_for_non_provider() {
	new_test_ext().execute_with(|| {
		let (_, key_pair) = create_account();
		let logo_cid = compute_cid(b"test_logo_data");
		let input_bounded_cid = BoundedVec::try_from(logo_cid).unwrap();
		assert_noop!(
			Msa::upload_logo(
				RuntimeOrigin::signed(key_pair.public().into()),
				BoundedVec::try_from(input_bounded_cid.clone())
					.expect("Logo CID should fit in bounds"),
				BoundedVec::try_from(b"test_logo_data".to_vec())
					.expect("Logo data should fit in bounds")
			),
			Error::<Test>::ProviderNotRegistered
		);
	});
}

#[test]
fn upload_logo_fails_non_msa() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sp_core::sr25519::Pair::generate();
		let logo_cid = compute_cid(b"test_logo_data");
		let input_bounded_cid = BoundedVec::try_from(logo_cid).unwrap();
		assert_noop!(
			Msa::upload_logo(
				RuntimeOrigin::signed(key_pair.public().into()),
				BoundedVec::try_from(input_bounded_cid.clone())
					.expect("Logo CID should fit in bounds"),
				BoundedVec::try_from(b"test_logo_data".to_vec())
					.expect("Logo data should fit in bounds")
			),
			Error::<Test>::NoKeyExists
		);
	});
}

#[test]
fn upload_logo_fails_mismatch_logo_data() {
	new_test_ext().execute_with(|| {
		let (_, key_pair) = create_provider_with_name("LogoProvider");
		let logo_cid = compute_cid(b"test_logo_data");
		let input_bounded_cid = BoundedVec::try_from(logo_cid).unwrap();

		// assuming logo was approved via governance
		ApprovedLogos::<Test>::insert(&input_bounded_cid, BoundedVec::new());

		assert_noop!(
			Msa::upload_logo(
				RuntimeOrigin::signed(key_pair.into()),
				BoundedVec::try_from(input_bounded_cid.clone())
					.expect("Logo CID should fit in bounds"),
				BoundedVec::try_from(b"wrong_logo_data".to_vec())
					.expect("Logo data should fit in bounds")
			),
			Error::<Test>::InvalidLogoBytes
		);
	});
}

#[test]
fn compute_cid_v1_test() {
	new_test_ext().execute_with(|| {
		// read frequency.png from assets
		let logo_data = include_bytes!("../../../../e2e/msa/frequency.png");
		let cid = common_primitives::cid::compute_cid_v1(logo_data).expect("Failed to compute CID");
		let encoded = multibase::encode(multibase::Base::Base58Btc, cid);
		assert_eq!(encoded, "zb2rhojSkWwLpTH7Sc9UFA3gFySTS8tx1vVu9SXhHTBcMabfF");
	});
}

#[test]
fn create_application_via_governance_with_no_logos_and_no_localized_names() {
	new_test_ext().execute_with(|| {
		let (new_msa_id, key_pair) = create_provider_with_name("BareProvider");

		// ApplicationContext::default() has no logos and no localized names
		let entry = ApplicationContext::default();

		// Approve application via council governance
		assert_ok!(Msa::create_application_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
			key_pair.into(),
			entry.clone(),
		));

		// Ensure provider is registered
		assert!(Msa::is_registered_provider(new_msa_id));

		// Application index should increment
		assert_eq!(NextApplicationIndex::<Test>::get(ProviderId(new_msa_id)), 1);

		// Application entry should be stored
		let stored_entry = ProviderToApplicationRegistry::<Test>::get(ProviderId(new_msa_id), 0)
			.expect("Application should be stored");

		// The stored entry should match our default (no logos, no localized names)
		assert_eq!(stored_entry.default_logo_250_100_png_cid.len(), 0);
		assert!(stored_entry.localized_names.is_empty());
		assert!(stored_entry.localized_logo_250_100_png_cids.is_empty());

		// The public-facing application context should also reflect empties
		let app_context =
			Msa::get_provider_application_context(ProviderId(new_msa_id), Some(0), None)
				.expect("App context should exist");
		assert_eq!(app_context.default_name, b"default".to_vec());
		assert_eq!(app_context.application_id, Some(0));
		assert_eq!(app_context.provider_id, ProviderId(new_msa_id));
		assert_eq!(app_context.localized_name, None);
		assert_eq!(app_context.localized_logo_250_100_png_bytes, None);
	});
}
