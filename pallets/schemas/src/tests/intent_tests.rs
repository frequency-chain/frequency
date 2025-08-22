use crate::{Error, Event as AnnouncementEvent, SchemaNamePayload};
use common_primitives::{
	node::AccountId,
	schema::{IntentId, MappedEntityIdentifier, PayloadLocation, SchemaSetting},
};
use frame_support::{assert_noop, assert_ok, traits::ChangeMembers, weights::Weight, BoundedVec};
use pallet_collective::ProposalOf;
use parity_scale_codec::Encode;

use super::mock::*;

#[test]
fn get_non_existing_intent_by_id_should_return_none() {
	new_test_ext().execute_with(|| {
		// act
		let res = SchemasPallet::get_intent_by_id(1, false);

		// assert
		assert!(res.as_ref().is_none());
	})
}

#[test]
fn create_intent_happy_path() {
	new_test_ext().execute_with(|| {
		// arrange
		let sender: AccountId = test_public(1);
		let name = "namespace.descriptor";
		let intent_name: SchemaNamePayload =
			BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");

		// act
		assert_ok!(SchemasPallet::create_intent(
			RuntimeOrigin::signed(sender.clone()),
			intent_name.clone(),
			PayloadLocation::OnChain,
			BoundedVec::default(),
		));
		let res = SchemasPallet::get_intent_by_id(1, false);
		let parsed_name = intent_name.into_inner();

		// assert
		System::assert_last_event(
			AnnouncementEvent::IntentCreated {
				key: sender,
				intent_id: 1,
				intent_name: parsed_name,
			}
			.into(),
		);
		assert!(res.as_ref().is_some());
	})
}

#[test]
fn create_intent_via_governance_happy_path() {
	new_test_ext().execute_with(|| {
		// arrange
		let settings = vec![SchemaSetting::AppendOnly];
		let sender: AccountId = test_public(5);
		let name = "namespace.descriptor";
		let intent_name: SchemaNamePayload =
			BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");

		// act
		assert_ok!(SchemasPallet::create_intent_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
			sender.clone(),
			PayloadLocation::Itemized,
			BoundedVec::try_from(settings.clone()).unwrap(),
			intent_name.clone(),
		));

		// assert
		let res = SchemasPallet::get_intent_by_id(1, false);
		let parsed_name = intent_name.into_inner();

		// assert
		System::assert_last_event(
			AnnouncementEvent::IntentCreated {
				key: sender,
				intent_id: 1,
				intent_name: parsed_name,
			}
			.into(),
		);
		assert!(res.as_ref().is_some());
	})
}

#[test]
fn create_intent_via_governance_with_append_only_setting_and_non_itemized_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let settings = vec![SchemaSetting::AppendOnly];
		let sender: AccountId = test_public(1);
		let name = "namespace.descriptor";
		let intent_name: SchemaNamePayload =
			BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");

		for location in
			[PayloadLocation::OnChain, PayloadLocation::IPFS, PayloadLocation::Paginated]
		{
			// act and assert
			assert_noop!(
				SchemasPallet::create_intent_via_governance(
					RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
					sender.clone(),
					location,
					BoundedVec::try_from(settings.clone()).unwrap(),
					intent_name.clone(),
				),
				Error::<Test>::InvalidSetting
			);
		}
	})
}
#[test]
fn create_intent_via_governance_with_signature_required_setting_and_wrong_location_should_fail() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();

		// arrange
		let settings = vec![SchemaSetting::SignatureRequired];
		let sender: AccountId = test_public(1);
		let name = "namespace.descriptor";
		let intent_name: SchemaNamePayload =
			BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");

		for location in [PayloadLocation::OnChain, PayloadLocation::IPFS] {
			// act and assert
			assert_noop!(
				SchemasPallet::create_intent_via_governance(
					RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
					sender.clone(),
					location,
					BoundedVec::try_from(settings.clone()).unwrap(),
					intent_name.clone(),
				),
				Error::<Test>::InvalidSetting
			);
		}
	})
}

#[test]
fn propose_to_create_intent_happy_path() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();

		let intent_name =
			SchemaNamePayload::try_from("namespace.descriptor".to_string().into_bytes())
				.expect("should work");
		// Propose a new schema
		_ = SchemasPallet::propose_to_create_intent(
			test_origin_signed(5),
			PayloadLocation::OnChain,
			BoundedVec::default(),
			intent_name.clone(),
		);

		// Find the Proposed event and get its hash and index so it can be voted on
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
		let council_member_1 = test_public(1); // Use ALICE as a council member
		let council_member_2 = test_public(2); // Use BOB as a council member
		let council_member_3 = test_public(3); // Use CHARLIE as a council member

		let incoming = vec![];
		let outgoing = vec![];
		Council::change_members(
			&incoming,
			&outgoing,
			vec![council_member_1.clone(), council_member_2.clone(), council_member_3.clone()],
		);

		// Council member #1 votes AYE on the proposal
		assert_ok!(Council::vote(
			RuntimeOrigin::signed(council_member_1.clone()),
			proposal_hash,
			proposal_index,
			true
		));
		// Council member #2 votes AYE on the proposal
		assert_ok!(Council::vote(
			RuntimeOrigin::signed(council_member_2.clone()),
			proposal_hash,
			proposal_index,
			true
		));
		// Council member #3 votes NAY on the proposal
		assert_ok!(Council::vote(
			RuntimeOrigin::signed(council_member_3.clone()),
			proposal_hash,
			proposal_index,
			false
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

		assert_eq!(voted_events.len(), 3);
		assert_eq!(voted_events[1].1, 2); // There should be two AYE (out of three) votes to pass

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
		assert_eq!(closed_events[0].0, 2); // There should be two YES votes to pass

		// Find the IntentCreated event and check if it passed
		let intent_events: Vec<IntentId> = System::events()
			.iter()
			.filter_map(|event| match event.event {
				RuntimeEvent::SchemasPallet(AnnouncementEvent::IntentCreated {
					key: _,
					intent_id,
					intent_name: _,
				}) => Some(intent_id),
				_ => None,
			})
			.collect();

		// Confirm that the schema was created
		assert_eq!(intent_events.len(), 1);

		let last_schema_id = intent_events[0];
		let created_intent = SchemasPallet::get_intent_by_id(last_schema_id, false);
		assert!(created_intent.as_ref().is_some());

		let resolved_intent_id =
			SchemasPallet::get_intent_or_group_ids_by_name(intent_name.clone().into_inner());
		assert!(resolved_intent_id.is_some());
		assert_eq!(
			resolved_intent_id.unwrap()[0].entity_id,
			MappedEntityIdentifier::Intent(last_schema_id)
		);
	})
}

#[test]
fn get_intent_or_group_ids_by_name_should_return_none_for_non_existing_name() {}
