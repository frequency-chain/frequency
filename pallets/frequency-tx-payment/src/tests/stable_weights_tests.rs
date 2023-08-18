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
				137185437,
				14946,
			),
			(
				"add_public_key_to_msa",
				SubstrateWeight::<Test>::add_public_key_to_msa(),
				188662000,
				18396,
			),
			("grant_delegation", SubstrateWeight::<Test>::grant_delegation(100), 123841076, 14946),
			(
				"grant_schema_permissions",
				SubstrateWeight::<Test>::grant_schema_permissions(100),
				33071573,
				0,
			),
			(
				"add_onchain_message",
				SubstrateWeight::<Test>::add_onchain_message(100),
				179298022,
				59148,
			),
			("add_ipfs_message", SubstrateWeight::<Test>::add_ipfs_message(), 174485000, 48664),
			(
				"apply_item_actions",
				SubstrateWeight::<Test>::apply_item_actions(100),
				106034371,
				45745,
			),
			("upsert_page", SubstrateWeight::<Test>::upsert_page(100), 32739701, 12791),
			("delete_page", SubstrateWeight::<Test>::delete_page(), 39471000, 13950),
			(
				"apply_item_actions_with_signature",
				SubstrateWeight::<Test>::apply_item_actions_with_signature(100),
				171407170,
				45752,
			),
			(
				"upsert_page_with_signature",
				SubstrateWeight::<Test>::upsert_page_with_signature(100),
				91859357,
				12724,
			),
			(
				"delete_page_with_signature",
				SubstrateWeight::<Test>::delete_page_with_signature(),
				92238000,
				13883,
			),
			("claim_handle", SubstrateWeight::<Test>::claim_handle(100), 96146074, 12434),
			("change_handle", SubstrateWeight::<Test>::change_handle(100), 115570724, 12434),
		];
		for t in table {
			assert_eq!(t.1, Weight::from_parts(t.2, t.3), "{}", t.0);
		}
	});
}
