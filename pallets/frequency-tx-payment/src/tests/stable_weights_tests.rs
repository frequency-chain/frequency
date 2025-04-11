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
				1130383617,
				14946,
			),
			(
				"add_public_key_to_msa",
				SubstrateWeight::<Test>::add_public_key_to_msa(),
				1076501000,
				18396,
			),
			("grant_delegation", SubstrateWeight::<Test>::grant_delegation(100), 810962313, 14946),
			(
				"add_onchain_message",
				SubstrateWeight::<Test>::add_onchain_message(100),
				437712930,
				59148,
			),
			("add_ipfs_message", SubstrateWeight::<Test>::add_ipfs_message(), 375458000, 48664),
			(
				"apply_item_actions",
				SubstrateWeight::<Test>::apply_item_actions(100),
				368365590,
				45745,
			),
			("upsert_page", SubstrateWeight::<Test>::upsert_page(100), 295852581, 12791),
			("delete_page", SubstrateWeight::<Test>::delete_page(), 301288000, 13950),
			(
				"apply_item_actions_with_signature_v2_add",
				SubstrateWeight::<Test>::apply_item_actions_with_signature_v2_add(100),
				333516190,
				5770,
			),
			(
				"apply_item_actions_with_signature_v2_delete",
				SubstrateWeight::<Test>::apply_item_actions_with_signature_v2_delete(100),
				354334155,
				16148,
			),
			(
				"upsert_page_with_signature_v2",
				SubstrateWeight::<Test>::upsert_page_with_signature_v2(100),
				337144903,
				6857,
			),
			(
				"delete_page_with_signature_v2",
				SubstrateWeight::<Test>::delete_page_with_signature_v2(),
				333742000,
				6855,
			),
			("claim_handle", SubstrateWeight::<Test>::claim_handle(100), 461175676, 12434),
			("change_handle", SubstrateWeight::<Test>::change_handle(100), 556303864, 12434),
		];
		for t in table {
			assert_eq!(t.1, Weight::from_parts(t.2, t.3), "{}", t.0);
		}
	});
}
