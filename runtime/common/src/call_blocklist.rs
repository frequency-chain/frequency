use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

pub fn blocked_calls() -> BTreeMap<&'static str, Vec<&'static str>> {
	let mut blocked_calls = BTreeMap::new();
	// following msa calls are blocked for Msa pallet to be included in Utility batches
	let mut block_list_msa = Vec::new();
	block_list_msa.push("create_provider");
	block_list_msa.push("revoke_delegation_by_delegator");
	block_list_msa.push("revoke_delegation_by_provider");
	block_list_msa.push("delete_msa_public_key");
	block_list_msa.push("retire_msa");

	// following handles calls are blocked for Msa pallet to be included in Utility batches
	let mut block_list_handles = Vec::new();
	block_list_handles.push("retire_handle");

	blocked_calls.insert("Msa", block_list_msa);
	blocked_calls.insert("Handles", block_list_handles);
	blocked_calls
}
