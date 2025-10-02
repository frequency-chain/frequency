#[cfg(test)]
use cid::multibase::Base;
use cid::{multibase, Cid};
#[cfg(test)]
use frame_support::assert_ok;
use frame_support::ensure;
use sp_io::hashing::sha2_256;
use sp_runtime::Vec;

/// Multihash type for wrapping digests (support up to 64-byte digests)
type Multihash = cid::multihash::Multihash<64>;

/// SHA2-256 multihash code
const SHA2_256: u64 = 0x12;
/// BLAKE3 multihash code
const BLAKE3: u64 = 0x1e;

/// List of hash algorithms supported by DSNP
const DSNP_HASH_ALGORITHMS: &[u64] = &[SHA2_256, BLAKE3];

/// Raw codec for CIDv1 (0x55)
const RAW: u64 = 0x55;

/// Error enum for CID validation
#[derive(Debug, PartialEq)]
pub enum CidError {
	/// Unsupported CID version
	UnsupportedCidVersion,
	/// Unsupported CID hash algorithm
	UnsupportedCidMultihash,
	/// Multibase decoding error
	MultibaseDecodeError,
	/// UTF-8 decoding error
	Utf8DecodeError,
	/// CID parsing error
	InvalidCid,
}

/// Computes a CIDv1 (RAW + SHA2-256 multihash)
pub fn compute_cid_v1(bytes: &[u8]) -> Option<Vec<u8>> {
	let digest = sha2_256(bytes);
	let mh = Multihash::wrap(SHA2_256, &digest).ok()?;
	let cid = Cid::new_v1(RAW, mh);
	Some(cid.to_bytes())
}

/// Validates a CID to conform to IPFS CIDv1 (or higher) formatting and allowed multihashes (does not validate decoded CID fields)
pub fn validate_cid(in_cid: &[u8]) -> Result<Vec<u8>, CidError> {
	// Decode SCALE encoded CID into string slice
	let cid_str: &str = core::str::from_utf8(in_cid).map_err(|_| CidError::Utf8DecodeError)?;
	ensure!(cid_str.len() > 2, CidError::InvalidCid);
	// starts_with handles Unicode multibyte characters safely
	ensure!(!cid_str.starts_with("Qm"), CidError::UnsupportedCidVersion);

	// Assume it's a multibase-encoded string. Decode it to a byte array so we can parse the CID.
	let cid_b = multibase::decode(cid_str).map_err(|_| CidError::MultibaseDecodeError)?.1;
	let cid = Cid::read_bytes(&cid_b[..]).map_err(|_| CidError::InvalidCid)?;
	ensure!(DSNP_HASH_ALGORITHMS.contains(&cid.hash().code()), CidError::UnsupportedCidMultihash);

	Ok(cid_b)
}

#[cfg(test)]
const DUMMY_CID_SHA512: &str = "bafkrgqb76pscorjihsk77zpyst3p364zlti6aojlu4nga34vhp7t5orzwbwwytvp7ej44r5yhjzneanqwb5arcnvuvfwo2d4qgzyx5hymvto4";
#[cfg(test)]
const DUMMY_CID_SHA256: &str = "bagaaierasords4njcts6vs7qvdjfcvgnume4hqohf65zsfguprqphs3icwea";
#[cfg(test)]
const DUMMY_CID_BLAKE3: &str = "bafkr4ihn4xalcdzoyslzy2nvf5q6il7vwqjvdhhatpqpctijrxh6l5xzru";

#[test]
fn validate_cid_invalid_utf8_errors() {
	let bad_cid = vec![0xfc, 0xa1, 0xa1, 0xa1, 0xa1, 0xa1];
	assert_eq!(
		validate_cid(&bad_cid).expect_err("Expected Utf8DecodeError"),
		CidError::Utf8DecodeError
	);
}

#[test]
fn validate_cid_too_short_errors() {
	let bad_cid = "a".as_bytes().to_vec();
	assert_eq!(validate_cid(&bad_cid).expect_err("Expected InvalidCid"), CidError::InvalidCid);
}

#[test]
fn validate_cid_v0_errors() {
	let bad_cid = "Qmxxx".as_bytes().to_vec();
	assert_eq!(
		validate_cid(&bad_cid).expect_err("Expected UnsupportedCidVersion"),
		CidError::UnsupportedCidVersion
	);
}

#[test]
fn validate_cid_invalid_multibase_errors() {
	let bad_cid = "aaaa".as_bytes().to_vec();
	assert_eq!(
		validate_cid(&bad_cid).expect_err("Expected MultibaseDecodeError"),
		CidError::MultibaseDecodeError
	);
}

#[test]
fn validate_cid_invalid_cid_errors() {
	let bad_cid = multibase::encode(Base::Base32Lower, "foo").as_bytes().to_vec();
	assert_eq!(validate_cid(&bad_cid).expect_err("Expected InvalidCid"), CidError::InvalidCid);
}

#[test]
fn validate_cid_valid_cid_sha2_256_succeeds() {
	let cid = DUMMY_CID_SHA256.as_bytes().to_vec();
	assert_ok!(validate_cid(&cid));
}

#[test]
fn validate_cid_valid_cid_blake3_succeeds() {
	let cid = DUMMY_CID_BLAKE3.as_bytes().to_vec();
	assert_ok!(validate_cid(&cid));
}

#[test]
fn validate_cid_invalid_hash_function_errors() {
	let bad_cid = DUMMY_CID_SHA512.as_bytes().to_vec();
	assert_eq!(
		validate_cid(&bad_cid).expect_err("Expected UnsupportedCidMultihash"),
		CidError::UnsupportedCidMultihash
	);
}
#[test]
fn validate_cid_not_valid_multibase() {
	// This should not panic, but should return an error.
	let bad_cid = vec![55, 197, 136, 0, 0, 0, 0, 0, 0, 0, 0];
	assert_eq!(
		validate_cid(&bad_cid).expect_err("Expected MultibaseDecodeError"),
		CidError::MultibaseDecodeError
	);
}

#[test]
fn validate_cid_not_correct_format_errors() {
	// This should not panic, but should return an error.
	let bad_cid = vec![0, 1, 0, 1, 203, 155, 0, 0, 0, 5, 67];
	assert_eq!(validate_cid(&bad_cid).expect_err("Expected InvalidCid"), CidError::InvalidCid);

	// This should not panic, but should return an error.
	let another_bad_cid = vec![241, 0, 0, 0, 0, 0, 128, 132, 132, 132, 58];
	assert_eq!(
		validate_cid(&another_bad_cid).expect_err("Expected Utf8DecodeError"),
		CidError::Utf8DecodeError
	);
}

#[test]
fn validate_cid_unwrap_errors() {
	// This should not panic, but should return an error.
	let bad_cid = vec![102, 70, 70, 70, 70, 70, 70, 70, 70, 48, 48, 48, 54, 53, 53, 48, 48];
	assert_eq!(validate_cid(&bad_cid).expect_err("Expected InvalidCid"), CidError::InvalidCid);
}
