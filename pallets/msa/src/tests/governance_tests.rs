use frame_support::{
	assert_noop, assert_ok,
	traits::{ChangeMembers, Hash},
};

use sp_weights::Weight;

use pretty_assertions::assert_eq;
use sp_core::{Encode, Pair};

use crate::{tests::mock::*, Error};

#[test]
fn create_provider_via_governance_happy_path() {
	new_test_ext().execute_with(|| {
		let (_new_msa_id, key_pair) = create_account();

		// Create the provider based on 1 yes vote by the council
		assert_ok!(Msa::create_provider_via_governance(
			RuntimeOrigin::from(pallet_collective::RawOrigin::Members(1, 1)),
			key_pair.public().into(),
			Vec::from("ACME Widgets")
		));
		// Confirm that the MSA is now a provider
		assert!(Msa::is_registered_provider(_new_msa_id));
	})
}

/// Test that a request to be a provider, makes the MSA a provider after the council approves it.
#[test]
fn propose_to_be_provider_happy_path() {
	new_test_ext().execute_with(|| {
		// Create a new MSA account and request that it become a provider
		let (_new_msa_id, key_pair) = create_account();
		_ = Msa::propose_to_be_provider(
			RuntimeOrigin::signed(key_pair.public().into()),
			Vec::from("ACME Widgets"),
		);

		// Find the Proposed event and get it's hash and index so it can be voted on
		let proposed_events: Vec<(u32, Hash)> = System::events()
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
		let proposal = Council::proposal_of(proposal_hash).unwrap();
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
		assert!(Msa::is_registered_provider(_new_msa_id));
	})
}

#[test]
fn propose_to_be_provider_long_name_should_fail() {
	new_test_ext().execute_with(|| {
		// Create a new MSA account and request that it become a provider
		let (_new_msa_id, key_pair) = create_account();
		let proposal_res = Msa::propose_to_be_provider(
			RuntimeOrigin::signed(key_pair.public().into()),
			Vec::from("this_is_a_really_long_name_that_should_fail"),
		);

		assert_noop!(proposal_res, Error::<Test>::ExceedsMaxProviderNameSize);
	})
}
