use cid::Cid;
use multihash::Multihash;
use sp_io::hashing::sha2_256;

#[derive(thiserror::Error, Debug)]
pub enum CidError {
	#[error("Multihash creation failed")]
	MultihashCreationFailed(#[from] multihash::Error),
}

/// SHA2-256 multihash code for wrapping digests
const SHA2_256: u64 = 0x12;

/// Raw codec for CIDv1 (0x55)
const RAW: u64 = 0x55;

/// Computes a CIDv1 for the given byte slice using a combination of multihash and CID crate.
///
/// This function:
/// 1. Computes the SHA2-256 digest of the input using `sp_io::hashing::sha2_256`.
/// 2. Wraps the digest in a `Multihash` (properly encoding the hash code and length).
/// 3. Constructs a CIDv1 using the `cid` crate with the RAW codec (0x55).
/// 4. Returns the CID as a vector of bytes that can be stored or transmitted.
///
/// # Arguments
/// * `bytes` - The input data to hash and wrap in a CID.
///
/// # Returns
/// A `Vec<u8>` containing the CIDv1 bytes.
///
/// # Example
/// ```rust
/// let data = b"hello world";
/// let cid_bytes = compute_cid_v1(data);
/// println!("CIDv1 bytes: {:?}", cid_bytes);
/// ```
pub fn compute_cid_v1(bytes: &[u8]) -> Result<Vec<u8>, CidError> {
	// Compute the SHA2-256 digest using Substrate's no_std hash
	let digest = sha2_256(bytes);

	// Wrap the digest in a multihash (code + length + digest)
	let mh = Multihash::<64>::wrap(SHA2_256, &digest).map_err(CidError::MultihashCreationFailed)?;

	// Construct CIDv1 with RAW codec
	let cid = Cid::new_v1(RAW, mh);

	// Return CID as bytes
	Ok(cid.to_bytes())
}
