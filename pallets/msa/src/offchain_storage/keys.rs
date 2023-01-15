use codec::{Decode, Encode};

/// Generic prefix for MSA index storage
pub const MSA_INDEX_KEY: &[u8] = b"frequency::msa::";

/// Derive storage key for MSA index
#[deny(clippy::clone_double_ref)]
pub(crate) fn derive_storage_key<K>(msa_id: K) -> Vec<u8>
where
	K: Encode + Clone + Ord + Decode,
{
	[MSA_INDEX_KEY, msa_id.encode().as_slice()].concat()
}
