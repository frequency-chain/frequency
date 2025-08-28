use super::mock::*;
use crate::{pallet::IntentGroups, Error, Event as AnnouncementEvent, SchemaNamePayload};
use common_primitives::{
	node::AccountId,
	schema::{IntentId, MappedEntityIdentifier, PayloadLocation},
};
use frame_support::{assert_noop, assert_ok, traits::ChangeMembers, weights::Weight, BoundedVec};
use pallet_collective::ProposalOf;
use parity_scale_codec::Encode;

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
fn create_intent_group_happy_path() {
	new_test_ext().execute_with(|| {
		// arrange
		let sender: AccountId = test_public(1);
		let name = "namespace.descriptor";
		let intent_group_name: SchemaNamePayload =
			BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");
		assert_ok!(SchemasPallet::create_intent(
			RuntimeOrigin::signed(sender.clone()),
			BoundedVec::try_from("namespace.schema".to_string().into_bytes())
				.expect("should convert"),
			PayloadLocation::Paginated,
			BoundedVec::default(),
		));

		// act
		assert_ok!(SchemasPallet::create_intent_group(
			RuntimeOrigin::signed(sender.clone()),
			intent_group_name.clone(),
			BoundedVec::try_from(vec![1]).unwrap(),
		));
		let res = SchemasPallet::get_intent_group_by_id(1);
		let parsed_name = intent_group_name.into_inner();
		let resolved_id = SchemasPallet::get_intent_or_group_ids_by_name(parsed_name.clone());

		// assert
		System::assert_last_event(
			AnnouncementEvent::IntentGroupCreated {
				key: sender,
				intent_group_id: 1,
				intent_group_name: parsed_name,
			}
			.into(),
		);
		assert!(res.as_ref().is_some());
		assert!(resolved_id.is_some());
		assert_eq!(resolved_id.unwrap()[0].entity_id, MappedEntityIdentifier::IntentGroup(1));
	})
}

#[test]
fn create_intent_group_via_governance_happy_path() {
	new_test_ext().execute_with(|| {
		// arrange
		let sender: AccountId = test_public(5);
		let name = "namespace.descriptor";
		let intent_group_name: SchemaNamePayload =
			BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");
		assert_ok!(SchemasPallet::create_intent(
			RuntimeOrigin::signed(sender.clone()),
			BoundedVec::try_from("namespace.schema".to_string().into_bytes())
				.expect("should convert"),
			PayloadLocation::Paginated,
			BoundedVec::default(),
		));

		// act
		assert_ok!(SchemasPallet::create_intent_group_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
			sender.clone(),
			intent_group_name.clone(),
			BoundedVec::try_from(vec![1]).unwrap(),
		));
		let res = SchemasPallet::get_intent_group_by_id(1);
		let parsed_name = intent_group_name.into_inner();
		let resolved_id = SchemasPallet::get_intent_or_group_ids_by_name(parsed_name.clone());

		// assert
		System::assert_last_event(
			AnnouncementEvent::IntentGroupCreated {
				key: sender,
				intent_group_id: 1,
				intent_group_name: parsed_name,
			}
			.into(),
		);
		assert!(res.as_ref().is_some());
		assert!(resolved_id.is_some());
		assert_eq!(resolved_id.unwrap()[0].entity_id, MappedEntityIdentifier::IntentGroup(1));
	})
}

#[test]
fn create_intent_group_via_governance_with_non_existent_intent_should_fail() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();

		// arrange
		let sender: AccountId = test_public(1);
		let name = "namespace.descriptor";
		let intent_group_name: SchemaNamePayload =
			BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");

		// act
		assert_noop!(
			SchemasPallet::create_intent_group_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
				sender.clone(),
				intent_group_name.clone(),
				BoundedVec::try_from(vec![1]).unwrap(),
			),
			Error::<Test>::InvalidIntentId
		);
	})
}

#[test]
fn create_intent_group_via_governance_with_pre_existing_name_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let sender: AccountId = test_public(1);
		let name = "namespace.descriptor";
		let intent_group_name: SchemaNamePayload =
			BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");
		assert_ok!(SchemasPallet::create_intent(
			RuntimeOrigin::signed(sender.clone()),
			intent_group_name.clone(),
			PayloadLocation::Paginated,
			BoundedVec::default(),
		));

		// act
		assert_noop!(
			SchemasPallet::create_intent_group_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
				sender.clone(),
				intent_group_name,
				BoundedVec::try_from(vec![1]).unwrap(),
			),
			Error::<Test>::NameAlreadyExists
		);
	})
}

#[test]
fn propose_to_create_intent_group_happy_path() {
	new_test_ext().execute_with(|| {
		sudo_set_max_schema_size();

		let intent_group_name =
			SchemaNamePayload::try_from("namespace.descriptor".to_string().into_bytes())
				.expect("should work");
		assert_ok!(SchemasPallet::create_intent(
			RuntimeOrigin::signed(test_public(5)),
			BoundedVec::try_from("namespace.schema".to_string().into_bytes())
				.expect("should convert"),
			PayloadLocation::Paginated,
			BoundedVec::default(),
		));

		// Propose a new schema
		_ = SchemasPallet::propose_to_create_intent_group(
			test_origin_signed(5),
			intent_group_name.clone(),
			BoundedVec::try_from(vec![1]).unwrap(),
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

		// Find the IntentGroupCreated event and check if it passed
		let intent_events: Vec<IntentId> = System::events()
			.iter()
			.filter_map(|event| match event.event {
				RuntimeEvent::SchemasPallet(AnnouncementEvent::IntentGroupCreated {
					key: _,
					intent_group_id,
					intent_group_name: _,
				}) => Some(intent_group_id),
				_ => None,
			})
			.collect();

		// Confirm that the IntentGroup was created
		assert_eq!(intent_events.len(), 1);

		let last_intent_group_id = intent_events[0];
		let created_intent_group = SchemasPallet::get_intent_group_by_id(last_intent_group_id);
		assert!(created_intent_group.as_ref().is_some());

		let resolved_intent_group_id =
			SchemasPallet::get_intent_or_group_ids_by_name(intent_group_name.clone().into_inner());
		assert!(resolved_intent_group_id.is_some());
		assert_eq!(
			resolved_intent_group_id.unwrap()[0].entity_id,
			MappedEntityIdentifier::IntentGroup(last_intent_group_id)
		);
	})
}

#[test]
fn update_intent_group_happy_path() {
	new_test_ext().execute_with(|| {
		// arrange
		let sender: AccountId = test_public(1);
		let name = "namespace.descriptor";
		let intent_group_name: SchemaNamePayload =
			BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");
		for i in 0..2 {
			let mut name = "namespace.intent".to_string();
			name.push_str(i.to_string().as_str());
			assert_ok!(SchemasPallet::create_intent(
				RuntimeOrigin::signed(sender.clone()),
				BoundedVec::try_from(name.into_bytes()).expect("should convert"),
				PayloadLocation::Paginated,
				BoundedVec::default(),
			));
		}
		assert_ok!(SchemasPallet::create_intent_group(
			RuntimeOrigin::signed(sender.clone()),
			intent_group_name.clone(),
			BoundedVec::try_from(vec![1]).unwrap(),
		));

		// act
		assert_ok!(SchemasPallet::update_intent_group(
			RuntimeOrigin::signed(sender.clone()),
			1,
			BoundedVec::try_from(vec![1, 2]).unwrap(),
		));

		// assert
		System::assert_last_event(
			AnnouncementEvent::IntentGroupUpdated { key: sender, intent_group_id: 1 }.into(),
		);

		let updated_intent = <IntentGroups<Test>>::get(1).expect("should exist");
		assert_eq!(updated_intent.intent_ids, vec![1, 2]);
	})
}

#[test]
fn update_intent_group_via_governance_happy_path() {
	new_test_ext().execute_with(|| {
		// arrange
		let sender: AccountId = test_public(1);
		let name = "namespace.descriptor";
		let intent_group_name: SchemaNamePayload =
			BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");
		for i in 0..2 {
			let mut name = "namespace.intent".to_string();
			name.push_str(i.to_string().as_str());
			assert_ok!(SchemasPallet::create_intent(
				RuntimeOrigin::signed(sender.clone()),
				BoundedVec::try_from(name.into_bytes()).expect("should convert"),
				PayloadLocation::Paginated,
				BoundedVec::default(),
			));
		}
		assert_ok!(SchemasPallet::create_intent_group(
			RuntimeOrigin::signed(sender.clone()),
			intent_group_name.clone(),
			BoundedVec::try_from(vec![1]).unwrap(),
		));

		// act
		assert_ok!(SchemasPallet::update_intent_group_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
			sender.clone(),
			1,
			BoundedVec::try_from(vec![1, 2]).unwrap(),
		));

		// assert
		System::assert_last_event(
			AnnouncementEvent::IntentGroupUpdated { key: sender, intent_group_id: 1 }.into(),
		);

		let updated_intent = <IntentGroups<Test>>::get(1).expect("should exist");
		assert_eq!(updated_intent.intent_ids, vec![1, 2]);
	})
}

#[test]
fn update_intent_group_via_governance_with_non_existent_intent_group_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let sender: AccountId = test_public(1);

		// act
		assert_noop!(
			SchemasPallet::update_intent_group_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
				sender.clone(),
				1,
				BoundedVec::try_from(vec![1, 2]).unwrap(),
			),
			Error::<Test>::InvalidIntentGroupId
		);
	})
}

#[test]
fn update_intent_group_via_governance_with_non_existent_intent_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let sender: AccountId = test_public(1);
		let name = "namespace.descriptor";
		let intent_group_name: SchemaNamePayload =
			BoundedVec::try_from(name.to_string().into_bytes()).expect("should convert");
		for i in 0..2 {
			let mut name = "namespace.intent".to_string();
			name.push_str(i.to_string().as_str());
			assert_ok!(SchemasPallet::create_intent(
				RuntimeOrigin::signed(sender.clone()),
				BoundedVec::try_from(name.into_bytes()).expect("should convert"),
				PayloadLocation::Paginated,
				BoundedVec::default(),
			));
		}
		assert_ok!(SchemasPallet::create_intent_group(
			RuntimeOrigin::signed(sender.clone()),
			intent_group_name.clone(),
			BoundedVec::try_from(vec![1]).unwrap(),
		));

		// act
		assert_noop!(
			SchemasPallet::update_intent_group_via_governance(
				RuntimeOrigin::from(pallet_collective::RawOrigin::Members(2, 3)),
				sender.clone(),
				1,
				BoundedVec::try_from(vec![1, 2, 3]).unwrap(),
			),
			Error::<Test>::InvalidIntentId
		);
	})
}
