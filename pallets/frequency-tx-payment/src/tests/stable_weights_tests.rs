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
				1913551100,
				7722,
			),
			(
				"add_public_key_to_msa",
				SubstrateWeight::<Test>::add_public_key_to_msa(),
				1809066000,
				9981,
			),
			("grant_delegation", SubstrateWeight::<Test>::grant_delegation(100), 1364482284, 7722),
			(
				"add_recovery_commitment",
				SubstrateWeight::<Test>::add_recovery_commitment(),
				971601000,
				5733,
			),
			("recover_account", SubstrateWeight::<Test>::recover_account(), 1603916000, 6531),
			(
				"add_onchain_message",
				SubstrateWeight::<Test>::add_onchain_message(100),
				432456221,
				4177,
			),
			("add_ipfs_message", SubstrateWeight::<Test>::add_ipfs_message(), 333677000, 4008),
			(
				"apply_item_actions",
				SubstrateWeight::<Test>::apply_item_actions(100),
				3428845824,
				6077,
			),
			("upsert_page", SubstrateWeight::<Test>::upsert_page(100), 3430685600, 7130),
			("delete_page", SubstrateWeight::<Test>::delete_page(), 3429011000, 7128),
			(
				"apply_item_actions_with_signature",
				SubstrateWeight::<Test>::apply_item_actions_with_signature(100),
				3426467813,
				6084,
			),
			(
				"upsert_page_with_signature",
				SubstrateWeight::<Test>::upsert_page_with_signature(100),
				3415766276,
				7063,
			),
			(
				"delete_page_with_signature",
				SubstrateWeight::<Test>::delete_page_with_signature(),
				3415818000,
				7061,
			),
			("claim_handle", SubstrateWeight::<Test>::claim_handle(100), 757565826, 4019),
			("change_handle", SubstrateWeight::<Test>::change_handle(100), 898054542, 4019),
		];
		for t in table {
			assert_eq!(t.1, Weight::from_parts(t.2, t.3), "{}", t.0);
		}
	});
}
