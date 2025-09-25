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
				7554030569,
				6531,
			),
			(
				"add_public_key_to_msa",
				SubstrateWeight::<Test>::add_public_key_to_msa(),
				6313813000,
				9981,
			),
			("grant_delegation", SubstrateWeight::<Test>::grant_delegation(100), 5872858357, 6531),
			(
				"add_recovery_commitment",
				SubstrateWeight::<Test>::add_recovery_commitment(),
				3228654000,
				5733,
			),
			("recover_account", SubstrateWeight::<Test>::recover_account(), 6111801000, 6531),
			(
				"add_onchain_message",
				SubstrateWeight::<Test>::add_onchain_message(100),
				2123280221,
				4177,
			),
			("add_ipfs_message", SubstrateWeight::<Test>::add_ipfs_message(), 1460893000, 4008),
			(
				"apply_item_actions",
				SubstrateWeight::<Test>::apply_item_actions(100),
				2771625824,
				6077,
			),
			("upsert_page", SubstrateWeight::<Test>::upsert_page(100), 2776718227, 7235),
			("delete_page", SubstrateWeight::<Test>::delete_page(), 2775473000, 7233),
			(
				"apply_item_actions_with_signature",
				SubstrateWeight::<Test>::apply_item_actions_with_signature(100),
				2205639813,
				6084,
			),
			(
				"upsert_page_with_signature",
				SubstrateWeight::<Test>::upsert_page_with_signature(100),
				2208495907,
				7168,
			),
			(
				"delete_page_with_signature",
				SubstrateWeight::<Test>::delete_page_with_signature(),
				2203757000,
				7166,
			),
			("claim_handle", SubstrateWeight::<Test>::claim_handle(100), 2448389826, 4019),
			("change_handle", SubstrateWeight::<Test>::change_handle(100), 2588878542, 4019),
		];
		for t in table {
			assert_eq!(t.1, Weight::from_parts(t.2, t.3), "{}", t.0);
		}
	});
}
