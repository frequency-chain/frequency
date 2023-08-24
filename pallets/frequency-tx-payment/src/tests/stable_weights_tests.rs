use crate::tests::mock::*;
use frame_support::weights::Weight;

use crate::capacity_stable_weights::{SubstrateWeight, WeightInfo};

#[test]
fn test_weights_are_stable() {
	ExtBuilder::default().build().execute_with(|| {
		let table = vec![
			(
				"create_sponsored_account_with_delegation",
				SubstrateWeight::<Test>::create_sponsored_account_with_delegation(100),
				136951617,
				14946,
			),
			(
				"add_public_key_to_msa",
				SubstrateWeight::<Test>::add_public_key_to_msa(),
				177629000,
				18396,
			),
			("grant_delegation", SubstrateWeight::<Test>::grant_delegation(100), 135218313, 14946),
			(
				"add_onchain_message",
				SubstrateWeight::<Test>::add_onchain_message(100),
				174216930,
				59148,
			),
			("add_ipfs_message", SubstrateWeight::<Test>::add_ipfs_message(), 159242000, 48664),
			(
				"apply_item_actions",
				SubstrateWeight::<Test>::apply_item_actions(100),
				104869590,
				45745,
			),
			("upsert_page", SubstrateWeight::<Test>::upsert_page(100), 32356581, 12791),
			("delete_page", SubstrateWeight::<Test>::delete_page(), 37792000, 13950),
			(
				"apply_item_actions_with_signature",
				SubstrateWeight::<Test>::apply_item_actions_with_signature(100),
				160314012,
				45752,
			),
			(
				"upsert_page_with_signature",
				SubstrateWeight::<Test>::upsert_page_with_signature(100),
				88433059,
				12724,
			),
			(
				"delete_page_with_signature",
				SubstrateWeight::<Test>::delete_page_with_signature(),
				90631000,
				13883,
			),
			("claim_handle", SubstrateWeight::<Test>::claim_handle(100), 471207676, 12434),
			("change_handle", SubstrateWeight::<Test>::change_handle(100), 591959864, 12434),
		];
		for t in table {
			assert_eq!(t.1, Weight::from_parts(t.2, t.3), "{}", t.0);
		}
	});
}
