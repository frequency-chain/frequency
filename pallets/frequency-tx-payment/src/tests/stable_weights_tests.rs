use crate::tests::mock::*;
use frame_support::weights::Weight;

use crate::capacity_stable_weights::{SubstrateWeight, WeightInfo};

#[test]
fn test_weights_are_stable() {
	ExtBuilder::default().build().execute_with(|| {
		let table = vec![
			(SubstrateWeight::<Test>::create_sponsored_account_with_delegation(100), 112601200),
			(SubstrateWeight::<Test>::add_public_key_to_msa(), 147786000),
			(SubstrateWeight::<Test>::grant_delegation(100), 107267145),
			(SubstrateWeight::<Test>::grant_schema_permissions(100), 33071573),
			(SubstrateWeight::<Test>::add_onchain_message(100), 139576386),
			(SubstrateWeight::<Test>::add_ipfs_message(), 131669000),
			(SubstrateWeight::<Test>::apply_item_actions(100), 66240801),
			(SubstrateWeight::<Test>::upsert_page(100), 23063086),
			(SubstrateWeight::<Test>::delete_page(), 26000000),
			(SubstrateWeight::<Test>::apply_item_actions_with_signature(100), 106536191),
			(SubstrateWeight::<Test>::upsert_page_with_signature(100), 61765307),
			(SubstrateWeight::<Test>::delete_page_with_signature(), 65000000),
			(SubstrateWeight::<Test>::claim_handle(100), 100989953),
		];
		for t in table {
			assert_eq!(t.0, Weight::from_ref_time(t.1));
		}
	});
}
