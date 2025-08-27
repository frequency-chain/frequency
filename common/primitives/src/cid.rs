use cid::Cid;
use sp_io::hashing::sha2_256;
use sp_runtime::Vec;

/// Multihash type for wrapping digests (support up to 64-byte digests)
pub type Multihash = cid::multihash::Multihash<64>;

/// SHA2-256 multihash code
const SHA2_256: u64 = 0x12;

/// Raw codec for CIDv1 (0x55)
const RAW: u64 = 0x55;

/// Computes a CIDv1 (RAW + SHA2-256 multihash)
pub fn compute_cid_v1(bytes: &[u8]) -> Option<Vec<u8>> {
	let digest = sha2_256(bytes);
	let mh = Multihash::wrap(SHA2_256, &digest).ok()?;
	let cid = Cid::new_v1(RAW, mh);
	Some(cid.to_bytes())
}
