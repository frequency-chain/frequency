use crate::msa::MessageSourceId;
use numtoa::NumToA;
use sp_std::{vec, vec::Vec};

/// Lock expiration timeout in in milli-seconds for initial data import msa pallet
pub const MSA_INITIAL_LOCK_TIMEOUT_EXPIRATION: u64 = 2000;
/// Lock expiration block for initial data import msa pallet
pub const MSA_INITIAL_LOCK_BLOCK_EXPIRATION: u32 = 20;
/// Lock name for initial data import msa pallet
pub const MSA_INITIAL_LOCK_NAME: &[u8; 29] = b"Msa::ofw::initial-import-lock";

/// Lock expiration timeout in in milli-seconds for msa pallet per msa account
pub const MSA_ACCOUNT_LOCK_TIMEOUT_EXPIRATION: u64 = 5;
/// Lock name prefix for msa account
pub const MSA_ACCOUNT_LOCK_NAME_PREFIX: &[u8; 16] = b"Msa::ofw::lock::";
/// Offchain storage prefix for msa account
pub const MSA_ACCOUNT_STORAGE_NAME_PREFIX: &[u8; 16] = b"Msa::ofw::keys::";
/// msa account lock name
pub fn get_msa_account_lock_name(msa_id: MessageSourceId) -> Vec<u8> {
	let mut buff = [0u8; 30];
	vec![MSA_ACCOUNT_LOCK_NAME_PREFIX, msa_id.numtoa(10, &mut buff)].concat()
}
/// msa account storage key name
pub fn get_msa_account_storage_key_name(msa_id: MessageSourceId) -> Vec<u8> {
	let mut buff = [0u8; 30];
	vec![MSA_ACCOUNT_STORAGE_NAME_PREFIX, msa_id.numtoa(10, &mut buff)].concat()
}
