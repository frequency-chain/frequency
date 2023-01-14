use common_primitives::msa::MessageSourceId;

/// Generic prefix for MSA index storage
pub const MSA_INDEX_KEY: &[u8] = b"frequency::msa::";

/// Derive storage key for MSA index
#[deny(clippy::clone_double_ref)]
pub(crate) fn derive_storage_key(msa_id: MessageSourceId) -> Vec<u8> {
	[MSA_INDEX_KEY, msa_id.to_string().as_bytes()].concat()
}
