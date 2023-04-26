use sp_std::collections::btree_map::BTreeMap;

pub fn blocked_calls() -> BTreeMap<&'static str, Vec<&'static str>> {
	let mut blocked_calls = BTreeMap::new();
	blocked_calls.insert(
		"Msa",
		vec![
			"create_provider",
			"revoke_delegation_by_delegator",
			"revoke_delegation_by_provider",
			"delete_msa_public_key",
			"retire_msa",
		],
	);
	blocked_calls.insert("Handles", vec!["retire_handle"]);
	blocked_calls
}
